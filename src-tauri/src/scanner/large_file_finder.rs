// src-tauri/src/scanner/large_file_finder.rs
use super::*;
use crate::error::Result;
use crate::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileResult {
    pub files: Vec<ScannedFile>,
    pub total_size: u64,
    pub total_count: u64,
    pub scan_duration_ms: u64,
}

pub struct LargeFileFinder {
    min_size: u64,
    top_n: Option<usize>,
}

impl LargeFileFinder {
    pub fn new(min_size_mb: u64, top_n: Option<usize>) -> Self {
        Self {
            min_size: min_size_mb * 1024 * 1024,
            top_n,
        }
    }

    pub fn find(
        &self,
        paths: &[PathBuf],
        state: Option<Arc<AppState>>,
    ) -> Result<LargeFileResult> {
        let start = std::time::Instant::now();
        
        let scanner = FileScanner::new(ScanOptions {
            min_size: Some(self.min_size),
            include_hidden: true,
            ..Default::default()
        });

        let mut result = scanner.scan(paths, state)?;
        
        // Sort by size descending
        result.files.sort_by(|a, b| b.size.cmp(&a.size));
        
        // Take top N if specified
        if let Some(n) = self.top_n {
            result.files.truncate(n);
        }

        let total_size: u64 = result.files.iter().map(|f| f.size).sum();

        Ok(LargeFileResult {
            total_count: result.files.len() as u64,
            files: result.files,
            total_size,
            scan_duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
