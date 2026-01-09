// src-tauri/src/scanner/file_scanner.rs
use super::*;
use crate::error::{CleanerError, Result};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use glob::Pattern;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

pub struct FileScanner {
    options: ScanOptions,
    exclude_patterns: Vec<Pattern>,
    include_patterns: Vec<Pattern>,
    rule_engine: RuleEngine,
}

impl FileScanner {
    pub fn new(options: ScanOptions) -> Self {
        let exclude_patterns: Vec<Pattern> = options
            .exclude_patterns
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();
            
        let include_patterns: Vec<Pattern> = options
            .include_patterns
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        // Initialize RuleEngine with default rules
        let rule_engine = RuleEngine::new(CleanupRule::default_rules());

        Self {
            options,
            exclude_patterns,
            include_patterns,
            rule_engine,
        }
    }

    pub fn scan(&self, paths: &[PathBuf], state: Option<Arc<AppState>>) -> Result<ScanResult> {
        let start = Instant::now();
        let files_scanned = AtomicU64::new(0);
        let bytes_scanned = AtomicU64::new(0);
        let mut all_files = Vec::new();
        let mut errors = Vec::new();

        for base_path in paths {
            if let Some(ref s) = state {
                if s.is_cancelled() {
                    return Err(CleanerError::Cancelled);
                }
            }

            let walker = WalkDir::new(base_path)
                .follow_links(self.options.follow_symlinks)
                .max_depth(self.options.max_depth.unwrap_or(usize::MAX))
                .into_iter();

            for entry_result in walker {
                if let Some(ref s) = state {
                    if s.is_cancelled() {
                        return Err(CleanerError::Cancelled);
                    }
                }

                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();
                        
                        if self.should_exclude(path) {
                            continue;
                        }

                        if !self.include_patterns.is_empty() && !self.should_include(path) {
                            continue;
                        }

                        if let Some(scanned) = self.scan_file(path) {
                            if self.passes_filters(&scanned) {
                                files_scanned.fetch_add(1, Ordering::Relaxed);
                                bytes_scanned.fetch_add(scanned.size, Ordering::Relaxed);
                                
                                if let Some(ref s) = state {
                                    s.update_progress(crate::state::ScanProgress {
                                        current_path: path.display().to_string(),
                                        files_scanned: files_scanned.load(Ordering::Relaxed),
                                        bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
                                        files_found: all_files.len() as u64 + 1,
                                        bytes_found: all_files.iter().map(|f: &ScannedFile| f.size).sum::<u64>() + scanned.size,
                                        phase: "scanning".to_string(),
                                        percentage: 0.0,
                                    });
                                }
                                
                                all_files.push(scanned);
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(ScanError {
                            path: PathBuf::new(),
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        let total_size: u64 = all_files.iter().map(|f| f.size).sum();
        let duration = start.elapsed();

        Ok(ScanResult {
            files: all_files,
            total_size,
            total_count: files_scanned.load(Ordering::Relaxed),
            scan_duration_ms: duration.as_millis() as u64,
            errors,
        })
    }

    fn scan_file(&self, path: &Path) -> Option<ScannedFile> {
        let metadata = fs::metadata(path).ok()?;
        
        let modified = metadata
            .modified()
            .ok()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(Utc::now);
            
        let created = metadata.created().ok().map(|t| DateTime::<Utc>::from(t));
        let accessed = metadata.accessed().ok().map(|t| DateTime::<Utc>::from(t));

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_symlink() {
            FileType::Symlink
        } else {
            FileType::File
        };

        // Use RuleEngine for categorization and risk assessment
        let (category, risk_level, description) = self.rule_engine.categorize(path, &metadata);
        
        // Fallback categorization if RuleEngine returns Unknown (to keep user's original logic as safety net)
        let final_category = if category == FileCategory::Unknown {
            categorize_file_fallback(path)
        } else {
            category
        };

        let is_hidden = is_hidden_file(path);
        
        #[cfg(windows)]
        let is_system = {
            use std::os::windows::fs::MetadataExt;
            (metadata.file_attributes() & 0x4) != 0
        };
        
        #[cfg(not(windows))]
        let is_system = false;

        Some(ScannedFile {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            created,
            accessed,
            file_type,
            category: final_category,
            hash: None,
            is_hidden,
            is_system,
            risk_level,
            description,
        })
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns.iter().any(|p| p.matches(&path_str))
    }

    fn should_include(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.include_patterns.iter().any(|p| p.matches(&path_str))
    }

    fn passes_filters(&self, file: &ScannedFile) -> bool {
        if file.is_hidden && !self.options.include_hidden {
            return false;
        }

        if let Some(min) = self.options.min_size {
            if file.size < min {
                return false;
            }
        }
        if let Some(max) = self.options.max_size {
            if file.size > max {
                return false;
            }
        }

        let now = Utc::now();
        let age_days = (now - file.modified).num_days() as u32;
        
        if let Some(min) = self.options.min_age_days {
            if age_days < min {
                return false;
            }
        }
        if let Some(max) = self.options.max_age_days {
            if age_days > max {
                return false;
            }
        }

        if !self.options.categories.is_empty() {
            if !self.options.categories.contains(&file.category) {
                return false;
            }
        }

        true
    }
}

pub fn categorize_file_fallback(path: &Path) -> FileCategory {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.contains("cache") || path_str.contains(".cache") {
        return FileCategory::Cache;
    }
    if path_str.contains("temp") || path_str.contains("tmp") {
        return FileCategory::Temporary;
    }
    if path_str.contains("thumbnail") || path_str.contains("thumbs") {
        return FileCategory::Thumbnail;
    }
    if path_str.contains("download") {
        return FileCategory::Download;
    }
    if path_str.contains("log") || path_str.contains("logs") {
        return FileCategory::Log;
    }

    match extension.as_deref() {
        Some("log" | "logs") => FileCategory::Log,
        Some("tmp" | "temp" | "bak" | "swp" | "swo") => FileCategory::Temporary,
        Some("cache") => FileCategory::Cache,
        Some("pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xls" | "xlsx" | "ppt" | "pptx") => {
            FileCategory::Document
        }
        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "mp3" | "mp4" | "avi" | 
             "mkv" | "mov" | "wav" | "flac" | "ogg" | "wmv") => FileCategory::Media,
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz") => FileCategory::Archive,
        Some("exe" | "msi" | "dmg" | "app" | "deb" | "rpm" | "sh" | "bat" | "cmd") => {
            FileCategory::Executable
        }
        Some("json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "config") => {
            FileCategory::Config
        }
        _ => FileCategory::Unknown,
    }
}

pub fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}
