// src-tauri/src/core/duplicates.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use blake3::Hasher;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info};

#[derive(Error, Debug)]
pub enum DuplicateError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Operation cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateFile {
    pub path: PathBuf,
    pub size: u64,
    pub hash: String,
    pub modified: u64,
    pub is_original: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub total_wasted_space: u64,
    pub files: Vec<DuplicateFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_duplicates: u64,
    pub total_wasted_space: u64,
    pub duration_seconds: f64,
}

pub struct DuplicateFinder {
    cancel_flag: Arc<AtomicBool>,
    files_processed: Arc<AtomicU64>,
}

impl DuplicateFinder {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            files_processed: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn with_cancel_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = flag;
        self
    }
    
    pub async fn find_duplicates(&self, files: &[PathBuf]) -> Result<DuplicateResult, DuplicateError> {
        let start_time = std::time::Instant::now();
        
        let size_groups = self.group_by_size(files)?;
        let quick_hash_groups = self.quick_hash_groups(&size_groups)?;
        let duplicate_groups = self.full_hash_groups(&quick_hash_groups)?;
        
        let total_duplicates: u64 = duplicate_groups.iter()
            .map(|g| g.files.len() as u64 - 1).sum();
        let total_wasted_space: u64 = duplicate_groups.iter()
            .map(|g| g.total_wasted_space).sum();
        
        let duration = start_time.elapsed().as_secs_f64();
        info!("Duplicate scan complete: {} wasted in {:.2}s", total_wasted_space, duration);
        
        Ok(DuplicateResult {
            groups: duplicate_groups,
            total_duplicates,
            total_wasted_space,
            duration_seconds: duration,
        })
    }
    
    fn group_by_size(&self, files: &[PathBuf]) -> Result<HashMap<u64, Vec<PathBuf>>, DuplicateError> {
        let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        for path in files {
            if self.cancel_flag.load(Ordering::SeqCst) { return Err(DuplicateError::Cancelled); }
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                if size > 1024 { // Min 1KB
                    size_map.entry(size).or_default().push(path.clone());
                }
            }
        }
        size_map.retain(|_, v| v.len() > 1);
        Ok(size_map)
    }
    
    fn quick_hash_groups(&self, size_groups: &HashMap<u64, Vec<PathBuf>>) -> Result<HashMap<String, Vec<(PathBuf, u64)>>, DuplicateError> {
        let mut hash_map: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
        for (size, paths) in size_groups {
            if self.cancel_flag.load(Ordering::SeqCst) { return Err(DuplicateError::Cancelled); }
            let hashes: Vec<_> = paths.par_iter().filter_map(|path| {
                self.quick_hash(path).ok().map(|h| (path.clone(), h))
            }).collect();
            for (path, hash) in hashes {
                let key = format!("{}_{}", size, hash);
                hash_map.entry(key).or_default().push((path, *size));
            }
        }
        hash_map.retain(|_, v| v.len() > 1);
        Ok(hash_map)
    }
    
    fn full_hash_groups(&self, quick_hash_groups: &HashMap<String, Vec<(PathBuf, u64)>>) -> Result<Vec<DuplicateGroup>, DuplicateError> {
        let mut duplicate_groups = Vec::new();
        for (_, candidates) in quick_hash_groups {
            if self.cancel_flag.load(Ordering::SeqCst) { return Err(DuplicateError::Cancelled); }
            let mut full_hash_map: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
            for (path, size) in candidates {
                if let Ok(hash) = self.full_hash(path) {
                    full_hash_map.entry(hash).or_default().push((path.clone(), *size));
                }
            }
            for (hash, files) in full_hash_map {
                if files.len() > 1 {
                    let size = files[0].1;
                    let wasted = size * (files.len() as u64 - 1);
                    let duplicate_files: Vec<DuplicateFile> = files.iter().enumerate().map(|(i, (path, size))| {
                        DuplicateFile { path: path.clone(), size: *size, hash: hash.clone(), modified: 0, is_original: i == 0 }
                    }).collect();
                    duplicate_groups.push(DuplicateGroup { hash: hash.clone(), size, total_wasted_space: wasted, files: duplicate_files });
                }
            }
        }
        duplicate_groups.sort_by(|a, b| b.total_wasted_space.cmp(&a.total_wasted_space));
        Ok(duplicate_groups)
    }
    
    fn quick_hash(&self, path: &Path) -> Result<String, DuplicateError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 4096];
        let bytes_read = reader.read(&mut buffer)?;
        let mut hasher = Hasher::new();
        hasher.update(&buffer[..bytes_read]);
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    fn full_hash(&self, path: &Path) -> Result<String, DuplicateError> {
        let file = File::open(path)?;
        let mut reader = BufReader::with_capacity(1024 * 1024, file);
        let mut hasher = Hasher::new();
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 { break; }
            hasher.update(&buffer[..bytes_read]);
            self.files_processed.fetch_add(bytes_read as u64, Ordering::SeqCst);
        }
        Ok(hasher.finalize().to_hex().to_string())
    }
}
