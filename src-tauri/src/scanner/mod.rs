// src-tauri/src/scanner/mod.rs
pub mod file_scanner;
pub mod duplicate_detector;
pub mod large_file_finder;
pub mod cache_scanner;
pub mod rules;

pub use file_scanner::*;
pub use duplicate_detector::*;
pub use large_file_finder::*;
pub use cache_scanner::*;
pub use rules::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub file_type: FileType,
    pub category: FileCategory,
    pub hash: Option<String>,
    pub is_hidden: bool,
    pub is_system: bool,
    pub risk_level: RiskLevel,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileCategory {
    Cache,
    Temporary,
    Log,
    Thumbnail,
    Download,
    Document,
    Media,
    Archive,
    Executable,
    Config,
    System,
    BrowserCache,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub min_age_days: Option<u32>,
    pub max_age_days: Option<u32>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub categories: Vec<FileCategory>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            follow_symlinks: false,
            max_depth: None,
            min_size: None,
            max_size: None,
            min_age_days: None,
            max_age_days: None,
            include_patterns: vec![],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
            ],
            categories: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
    pub total_size: u64,
    pub total_count: u64,
    pub scan_duration_ms: u64,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub path: PathBuf,
    pub error: String,
}
