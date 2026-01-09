// src-tauri/src/core/scanner.rs

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{UNIX_EPOCH};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

use super::rules::{RuleEngine};

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Scan cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
    pub accessed: u64,
    pub created: u64,
    pub is_dir: bool,
    pub category: FileCategory,
    pub cleanup_reason: Option<String>,
    pub can_delete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileCategory {
    TempFile,
    CacheFile,
    LogFile,
    Duplicate,
    LargeFile,
    OldFile,
    BrowserCache,
    SystemJunk,
    UserDownloads,
    Thumbnail,
    RecycleBin,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanProgress {
    pub scan_id: String,
    pub files_scanned: u64,
    pub total_size: u64,
    pub current_path: String,
    pub files_found: u64,
    pub space_recoverable: u64,
    pub elapsed_seconds: f64,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub files: Vec<ScannedFile>,
    pub total_files: u64,
    pub total_size: u64,
    pub recoverable_space: u64,
    pub duration_seconds: f64,
    pub categories: std::collections::HashMap<FileCategory, CategoryStats>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CategoryStats {
    pub count: u64,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct ScanConfig {
    pub paths: Vec<PathBuf>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub include_hidden: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            max_depth: Some(20),
            follow_symlinks: false,
            include_hidden: true,
        }
    }
}

pub struct FileScanner {
    config: ScanConfig,
    rule_engine: RuleEngine,
    cancel_flag: Arc<AtomicBool>,
    files_scanned: Arc<AtomicU64>,
    bytes_scanned: Arc<AtomicU64>,
    progress_tx: Option<broadcast::Sender<ScanProgress>>,
}

impl FileScanner {
    pub fn new(config: ScanConfig, rule_engine: RuleEngine) -> Self {
        Self {
            config,
            rule_engine,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            files_scanned: Arc::new(AtomicU64::new(0)),
            bytes_scanned: Arc::new(AtomicU64::new(0)),
            progress_tx: None,
        }
    }
    
    pub fn with_progress_channel(mut self, tx: broadcast::Sender<ScanProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }
    
    pub fn with_cancel_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = flag;
        self
    }
    
    pub async fn scan(&self, scan_id: &str) -> Result<ScanResult, ScanError> {
        let start_time = std::time::Instant::now();
        info!("Starting scan {} with {} paths", scan_id, self.config.paths.len());
        
        let mut all_files: Vec<ScannedFile> = Vec::new();
        
        for base_path in &self.config.paths {
            if self.cancel_flag.load(Ordering::SeqCst) {
                return Err(ScanError::Cancelled);
            }
            
            if !base_path.exists() {
                warn!("Path does not exist: {:?}", base_path);
                continue;
            }
            
            let files = self.scan_directory(base_path, scan_id)?;
            all_files.extend(files);
        }
        
        let total_size: u64 = all_files.iter().map(|f| f.size).sum();
        let recoverable_space: u64 = all_files.iter()
            .filter(|f| f.can_delete)
            .map(|f| f.size)
            .sum();
        
        let mut categories: std::collections::HashMap<FileCategory, CategoryStats> = 
            std::collections::HashMap::new();
        
        for file in &all_files {
            let stats = categories.entry(file.category.clone()).or_default();
            stats.count += 1;
            stats.size += file.size;
        }
        
        let duration = start_time.elapsed().as_secs_f64();
        
        info!("Scan complete: {} files in {:.2}s", all_files.len(), duration);
        
        Ok(ScanResult {
            scan_id: scan_id.to_string(),
            total_files: all_files.len() as u64,
            files: all_files,
            total_size,
            recoverable_space,
            duration_seconds: duration,
            categories,
        })
    }
    
    fn scan_directory(&self, path: &Path, scan_id: &str) -> Result<Vec<ScannedFile>, ScanError> {
        let walker = WalkDir::new(path)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_entry(|e| self.should_scan_entry(e));
        
        let entries: Vec<_> = walker
            .filter_map(|e| e.ok())
            .collect();
        
        let files: Vec<ScannedFile> = entries
            .par_iter()
            .filter_map(|entry| {
                if self.cancel_flag.load(Ordering::SeqCst) {
                    return None;
                }
                self.process_entry(entry, scan_id)
            })
            .collect();
        
        Ok(files)
    }
    
    fn should_scan_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        
        if !self.config.include_hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    return false;
                }
            }
        }
        
        let protected_dirs = ["Windows", "System32", "Program Files", "usr", "bin", "sbin", "lib"];
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if protected_dirs.iter().any(|&d| name_str.eq_ignore_ascii_case(d)) {
                return false;
            }
        }
        
        true
    }
    
    fn process_entry(&self, entry: &DirEntry, scan_id: &str) -> Option<ScannedFile> {
        let path = entry.path();
        let metadata = entry.metadata().ok()?;
        
        self.files_scanned.fetch_add(1, Ordering::SeqCst);
        self.bytes_scanned.fetch_add(metadata.len(), Ordering::SeqCst);
        
        if self.files_scanned.load(Ordering::SeqCst) % 1000 == 0 {
            if let Some(ref tx) = self.progress_tx {
                let _ = tx.send(ScanProgress {
                    scan_id: scan_id.to_string(),
                    files_scanned: self.files_scanned.load(Ordering::SeqCst),
                    total_size: self.bytes_scanned.load(Ordering::SeqCst),
                    current_path: path.display().to_string(),
                    files_found: self.files_scanned.load(Ordering::SeqCst),
                    space_recoverable: 0,
                    elapsed_seconds: 0.0,
                    status: "scanning".to_string(),
                });
            }
        }
        
        let (category, cleanup_reason) = self.rule_engine.categorize_file(path, &metadata);
        let can_delete = cleanup_reason.is_some();
        
        if !can_delete && category == FileCategory::Unknown {
            return None;
        }
        
        let modified = metadata.modified().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()).unwrap_or(0);
        let accessed = metadata.accessed().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()).unwrap_or(0);
        let created = metadata.created().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()).unwrap_or(0);
        
        Some(ScannedFile {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            accessed,
            created,
            is_dir: metadata.is_dir(),
            category,
            cleanup_reason,
            can_delete,
        })
    }
}
