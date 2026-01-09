// src-tauri/src/core/cleaner.rs

// Safe file cleanup with dry-run mode and undo support

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::scanner::ScannedFile;
use super::undo::{UndoManager, UndoOperation, UndoOperationType};

#[derive(Error, Debug)]
pub enum CleanError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Trash error: {0}")]
    TrashError(String),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Protected file: {0}")]
    ProtectedFile(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub operation_id: String,
    pub files_processed: u64,
    pub bytes_freed: u64,
    pub files_failed: u64,
    pub errors: Vec<CleanupError>,
    pub dry_run: bool,
    pub duration_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanupError {
    pub path: PathBuf,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanupOptions {
    pub dry_run: bool,
    pub use_trash: bool,
    pub secure_delete: bool,
    pub skip_in_use: bool,
    pub create_backup: bool,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            dry_run: true,
            use_trash: true,
            secure_delete: false,
            skip_in_use: true,
            create_backup: false,
        }
    }
}

pub struct Cleaner {
    options: CleanupOptions,
    undo_manager: Arc<RwLock<UndoManager>>,
    protected_paths: Vec<PathBuf>,
}

impl Cleaner {
    pub fn new(undo_manager: Arc<RwLock<UndoManager>>) -> Self {
        Self {
            options: CleanupOptions::default(),
            undo_manager,
            protected_paths: Self::get_protected_paths(),
        }
    }
    
    pub fn with_options(mut self, options: CleanupOptions) -> Self {
        self.options = options;
        self
    }
    
    fn get_protected_paths() -> Vec<PathBuf> {
        let mut protected = Vec::new();
        
        #[cfg(windows)]
        {
            if let Ok(windows) = std::env::var("SYSTEMROOT") {
                protected.push(PathBuf::from(&windows));
            }
            if let Ok(program_files) = std::env::var("PROGRAMFILES") {
                protected.push(PathBuf::from(&program_files));
            }
        }
        
        #[cfg(unix)]
        {
            protected.extend([
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/usr"),
                PathBuf::from("/lib"),
                PathBuf::from("/etc"),
                PathBuf::from("/var/lib"),
            ]);
        }
        
        #[cfg(target_os = "macos")]
        {
            protected.extend([
                PathBuf::from("/System"),
                PathBuf::from("/Library"),
                PathBuf::from("/Applications"),
            ]);
        }
        
        protected
    }
    
    pub async fn clean_files(&self, files: &[ScannedFile]) -> Result<CleanupResult, CleanError> {
        let start_time = std::time::Instant::now();
        let operation_id = uuid::Uuid::new_v4().to_string();
        
        info!(
            "Starting cleanup operation {} ({} files, dry_run={})",
            operation_id,
            files.len(),
            self.options.dry_run
        );
        
        let mut bytes_freed: u64 = 0;
        let mut files_processed: u64 = 0;
        let mut files_failed: u64 = 0;
        let mut errors: Vec<CleanupError> = Vec::new();
        let mut undo_operations: Vec<(PathBuf, u64)> = Vec::new();
        
        for file in files {
            if !file.can_delete {
                continue;
            }
            
            match self.process_file(&file.path, file.size).await {
                Ok(freed) => {
                    bytes_freed += freed;
                    files_processed += 1;
                    
                    if !self.options.dry_run {
                        undo_operations.push((file.path.clone(), file.size));
                    }
                }
                Err(e) => {
                    files_failed += 1;
                    errors.push(CleanupError {
                        path: file.path.clone(),
                        error: e.to_string(),
                    });
                    warn!("Failed to clean {:?}: {}", file.path, e);
                }
            }
        }
        
        // Record undo operation if not dry run
        if !self.options.dry_run && !undo_operations.is_empty() {
            let undo_op = UndoOperation {
                id: operation_id.clone(),
                operation_type: if self.options.use_trash {
                    UndoOperationType::MoveToTrash
                } else {
                    UndoOperationType::Delete
                },
                files: undo_operations,
                timestamp: chrono::Utc::now(),
                can_undo: self.options.use_trash,
            };
            
            self.undo_manager.write().await.record_operation(undo_op);
        }
        
        let duration = start_time.elapsed().as_secs_f64();
        
        info!(
            "Cleanup {} complete: {} files, {} bytes freed, {} failed in {:.2}s",
            operation_id,
            files_processed,
            bytes_freed,
            files_failed,
            duration
        );
        
        Ok(CleanupResult {
            operation_id,
            files_processed,
            bytes_freed,
            files_failed,
            errors,
            dry_run: self.options.dry_run,
            duration_seconds: duration,
        })
    }
    
    async fn process_file(&self, path: &Path, size: u64) -> Result<u64, CleanError> {
        // Check if file is protected
        if self.is_protected(path) {
            return Err(CleanError::ProtectedFile(path.to_path_buf()));
        }
        
        // Check if file exists
        if !path.exists() {
            return Err(CleanError::FileNotFound(path.to_path_buf()));
        }
        
        // Dry run - just return the size
        if self.options.dry_run {
            return Ok(size);
        }
        
        // Check if file is in use (Windows)
        #[cfg(windows)]
        if self.options.skip_in_use && self.is_file_in_use(path) {
            return Err(CleanError::PermissionDenied(path.to_path_buf()));
        }
        
        // Perform deletion
        if self.options.use_trash {
            trash::delete(path)
                .map_err(|e| CleanError::TrashError(e.to_string()))?;
        } else if self.options.secure_delete {
            self.secure_delete_file(path)?;
        } else {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        
        debug!("Deleted: {:?} ({} bytes)", path, size);
        Ok(size)
    }
    
    fn is_protected(&self, path: &Path) -> bool {
        for protected in &self.protected_paths {
            if path.starts_with(protected) {
                // Allow cleaning temp/cache subdirectories
                let relative = path.strip_prefix(protected).unwrap_or(path);
                let relative_str = relative.to_string_lossy().to_lowercase();
                
                if !relative_str.contains("temp") 
                    && !relative_str.contains("cache")
                    && !relative_str.contains("tmp") 
                {
                    return true;
                }
            }
        }
        false
    }
    
    #[cfg(windows)]
    fn is_file_in_use(&self, path: &Path) -> bool {
        use std::fs::OpenOptions;
        
        OpenOptions::new()
            .write(true)
            .open(path)
            .is_err()
    }
    
    fn secure_delete_file(&self, path: &Path) -> Result<(), CleanError> {
        use std::io::Write;
        
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        
        // Overwrite with zeros
        let file = fs::OpenOptions::new()
            .write(true)
            .open(path)?;
        
        let zeros = vec![0u8; 65536];
        let mut written = 0u64;
        let mut writer = std::io::BufWriter::new(file);
        
        while written < size {
            let to_write = std::cmp::min(65536, (size - written) as usize);
            writer.write_all(&zeros[..to_write])?;
            written += to_write as u64;
        }
        
        writer.flush()?;
        drop(writer);
        
        // Delete the file
        fs::remove_file(path)?;
        
        Ok(())
    }
}

/// Preview what would be cleaned without actually cleaning
pub struct CleanupPreview {
    pub files: Vec<ScannedFile>,
    pub total_size: u64,
    pub total_count: u64,
    pub by_category: std::collections::HashMap<String, CategoryPreview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryPreview {
    pub name: String,
    pub count: u64,
    pub size: u64,
    pub files: Vec<PathBuf>,
}

impl Cleaner {
    pub async fn preview(&self, files: &[ScannedFile]) -> CleanupPreview {
        use super::scanner::FileCategory;
        use std::collections::HashMap;
        
        let mut by_category: HashMap<String, CategoryPreview> = HashMap::new();
        let mut total_size = 0u64;
        let mut total_count = 0u64;
        
        for file in files {
            if !file.can_delete {
                continue;
            }
            
            total_size += file.size;
            total_count += 1;
            
            let category_name = format!("{:?}", file.category);
            let preview = by_category.entry(category_name.clone()).or_insert_with(|| {
                CategoryPreview {
                    name: category_name,
                    count: 0,
                    size: 0,
                    files: Vec::new(),
                }
            });
            
            preview.count += 1;
            preview.size += file.size;
            preview.files.push(file.path.clone());
        }
        
        CleanupPreview {
            files: files.iter().filter(|f| f.can_delete).cloned().collect(),
            total_size,
            total_count,
            by_category,
        }
    }
}
