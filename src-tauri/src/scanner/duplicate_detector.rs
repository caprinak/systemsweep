// src-tauri/src/scanner/duplicate_detector.rs
use super::*;
use crate::error::{CleanerError, Result};
use crate::state::AppState;
use blake3::Hasher as Blake3Hasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<PathBuf>,
    pub total_wasted_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateScanResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_duplicate_files: u64,
    pub total_wasted_space: u64,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DuplicateDetectorOptions {
    pub min_size: u64,
    pub max_size: Option<u64>,
    pub quick_hash_size: usize,
    pub use_full_hash: bool,
}

impl Default for DuplicateDetectorOptions {
    fn default() -> Self {
        Self {
            min_size: 1024,
            max_size: None,
            quick_hash_size: 4096,
            use_full_hash: true,
        }
    }
}

pub struct DuplicateDetector {
    options: DuplicateDetectorOptions,
}

impl DuplicateDetector {
    pub fn new(options: DuplicateDetectorOptions) -> Self {
        Self { options }
    }

    pub fn find_duplicates(
        &self,
        paths: &[PathBuf],
        state: Option<Arc<AppState>>,
    ) -> Result<DuplicateScanResult> {
        let start = std::time::Instant::now();

        if let Some(ref s) = state {
            s.update_progress(crate::state::ScanProgress {
                current_path: "Grouping files by size...".to_string(),
                files_scanned: 0,
                bytes_scanned: 0,
                files_found: 0,
                bytes_found: 0,
                phase: "size_grouping".to_string(),
                percentage: 10.0,
            });
        }

        let scanner = FileScanner::new(ScanOptions {
            min_size: Some(self.options.min_size),
            max_size: self.options.max_size,
            ..Default::default()
        });

        let scan_result = scanner.scan(paths, state.clone())?;
        
        let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        for file in &scan_result.files {
            if file.file_type == FileType::File {
                size_groups
                    .entry(file.size)
                    .or_default()
                    .push(file.path.clone());
            }
        }

        let size_groups: Vec<(u64, Vec<PathBuf>)> = size_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        if let Some(ref s) = state {
            if s.is_cancelled() {
                return Err(CleanerError::Cancelled);
            }
        }

        if let Some(ref s) = state {
            s.update_progress(crate::state::ScanProgress {
                current_path: "Computing quick hashes...".to_string(),
                files_scanned: 0,
                bytes_scanned: 0,
                files_found: 0,
                bytes_found: 0,
                phase: "quick_hash".to_string(),
                percentage: 30.0,
            });
        }

        let mut quick_hash_groups: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
        
        for (size, files) in &size_groups {
            for path in files {
                if let Some(ref s) = state {
                    if s.is_cancelled() {
                        return Err(CleanerError::Cancelled);
                    }
                }

                if let Ok(hash) = self.compute_quick_hash(path) {
                    quick_hash_groups
                        .entry(hash)
                        .or_default()
                        .push((path.clone(), *size));
                }
            }
        }

        let potential_duplicates: Vec<(String, Vec<(PathBuf, u64)>)> = quick_hash_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        let mut duplicate_groups: Vec<DuplicateGroup> = Vec::new();

        if self.options.use_full_hash {
            if let Some(ref s) = state {
                s.update_progress(crate::state::ScanProgress {
                    current_path: "Computing full hashes...".to_string(),
                    files_scanned: 0,
                    bytes_scanned: 0,
                    files_found: 0,
                    bytes_found: 0,
                    phase: "full_hash".to_string(),
                    percentage: 60.0,
                });
            }

            let mut full_hash_groups: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();

            for (_, files) in potential_duplicates {
                for (path, size) in files {
                    if let Some(ref s) = state {
                        if s.is_cancelled() {
                            return Err(CleanerError::Cancelled);
                        }
                    }

                    if let Ok(hash) = self.compute_full_hash(&path) {
                        full_hash_groups
                            .entry(hash)
                            .or_default()
                            .push((path, size));
                    }
                }
            }

            for (hash, files) in full_hash_groups {
                if files.len() > 1 {
                    let size = files[0].1;
                    let file_paths: Vec<PathBuf> = files.into_iter().map(|(p, _)| p).collect();
                    let wasted = size * (file_paths.len() as u64 - 1);
                    
                    duplicate_groups.push(DuplicateGroup {
                        hash,
                        size,
                        files: file_paths,
                        total_wasted_space: wasted,
                    });
                }
            }
        } else {
            for (hash, files) in potential_duplicates {
                let size = files[0].1;
                let file_paths: Vec<PathBuf> = files.into_iter().map(|(p, _)| p).collect();
                let wasted = size * (file_paths.len() as u64 - 1);
                
                duplicate_groups.push(DuplicateGroup {
                    hash,
                    size,
                    files: file_paths,
                    total_wasted_space: wasted,
                });
            }
        }

        duplicate_groups.sort_by(|a, b| b.total_wasted_space.cmp(&a.total_wasted_space));

        let total_duplicate_files: u64 = duplicate_groups
            .iter()
            .map(|g| g.files.len() as u64)
            .sum();
        let total_wasted_space: u64 = duplicate_groups
            .iter()
            .map(|g| g.total_wasted_space)
            .sum();

        Ok(DuplicateScanResult {
            groups: duplicate_groups,
            total_duplicate_files,
            total_wasted_space,
            scan_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn compute_quick_hash(&self, path: &PathBuf) -> Result<String> {
        let file = File::open(path).map_err(CleanerError::Io)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; self.options.quick_hash_size];
        
        let bytes_read = reader.read(&mut buffer).map_err(CleanerError::Io)?;
        buffer.truncate(bytes_read);

        let hash = blake3::hash(&buffer);
        Ok(hash.to_hex().to_string())
    }

    fn compute_full_hash(&self, path: &PathBuf) -> Result<String> {
        let file = File::open(path).map_err(CleanerError::Io)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Blake3Hasher::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(CleanerError::Io)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }
}
