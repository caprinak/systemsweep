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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    fn create_file_with_size(name: &str, size_bytes: u64) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique_name = format!(
            "large_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique_name);
        
        let file = File::create(&path).unwrap();
        file.set_len(size_bytes).unwrap();
        path
    }

    #[test]
    fn test_large_file_finder_limits_and_sorts() {
        let size_1mb = 1 * 1024 * 1024;
        let size_2mb = 2 * 1024 * 1024;
        let size_5mb = 5 * 1024 * 1024;

        let path_1 = create_file_with_size("1mb.txt", size_1mb);
        let path_2 = create_file_with_size("2mb.txt", size_2mb);
        let path_5 = create_file_with_size("5mb.txt", size_5mb);

        // Find all files min_size 2MB, no limit
        let finder = LargeFileFinder::new(2, None);
        let paths = vec![path_1.clone(), path_2.clone(), path_5.clone()];
        let result = finder.find(&paths, None).unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.files[0].path, path_5, "First file should be the largest (5MB)");
        assert_eq!(result.files[0].size, size_5mb);
        assert_eq!(result.files[1].path, path_2, "Second file should be 2MB");
        assert_eq!(result.files[1].size, size_2mb);

        // Find all files min_size 2MB, top_n = 1
        let finder_top1 = LargeFileFinder::new(2, Some(1));
        let result_top1 = finder_top1.find(&paths, None).unwrap();
        assert_eq!(result_top1.total_count, 1);
        assert_eq!(result_top1.files[0].path, path_5);

        let _ = std::fs::remove_file(path_1);
        let _ = std::fs::remove_file(path_2);
        let _ = std::fs::remove_file(path_5);
    }
}

