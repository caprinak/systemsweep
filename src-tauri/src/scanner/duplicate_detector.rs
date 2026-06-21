// src-tauri/src/scanner/duplicate_detector.rs
use super::*;
use crate::error::{CleanerError, Result};
use crate::state::AppState;
use blake3::Hasher as Blake3Hasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

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

        let quick_hash_groups: Arc<Mutex<HashMap<String, Vec<(PathBuf, u64)>>>> = Arc::new(Mutex::new(HashMap::new()));
        
        let file_tasks: Vec<_> = size_groups.iter().flat_map(|(size, files)| {
            files.iter().map(move |path| (path.clone(), *size))
        }).collect();

        file_tasks.par_iter().for_each(|(path, size)| {
            if let Some(ref s) = state {
                if s.is_cancelled() {
                    return;
                }
            }

            if let Ok(hash) = self.compute_quick_hash(path) {
                let mut map = quick_hash_groups.lock().unwrap();
                map.entry(hash).or_default().push((path.clone(), *size));
            }
        });
        
        let quick_hash_groups = Arc::try_unwrap(quick_hash_groups).unwrap().into_inner().unwrap();

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

            let full_hash_groups: Arc<Mutex<HashMap<String, Vec<(PathBuf, u64)>>>> = Arc::new(Mutex::new(HashMap::new()));

            let full_tasks: Vec<_> = potential_duplicates.into_iter().flat_map(|(_, files)| files).collect();

            full_tasks.par_iter().for_each(|(path, size)| {
                if let Some(ref s) = state {
                    if s.is_cancelled() {
                        return;
                    }
                }

                if let Ok(hash) = self.compute_full_hash(path) {
                    let mut map = full_hash_groups.lock().unwrap();
                    map.entry(hash).or_default().push((path.clone(), *size));
                }
            });
            
            let full_hash_groups = Arc::try_unwrap(full_hash_groups).unwrap().into_inner().unwrap();

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
        let mut file = File::open(path).map_err(CleanerError::Io)?;
        let metadata = file.metadata().map_err(CleanerError::Io)?;
        let size = metadata.len();
        
        let chunk_size = self.options.quick_hash_size as u64;
        let mut buffer = Vec::new();
        
        if size <= chunk_size * 3 {
            file.read_to_end(&mut buffer).map_err(CleanerError::Io)?;
        } else {
            let mut chunk = vec![0u8; chunk_size as usize];
            
            let bytes_read = file.read(&mut chunk).map_err(CleanerError::Io)?;
            buffer.extend_from_slice(&chunk[..bytes_read]);
            
            file.seek(SeekFrom::Start(size / 2 - chunk_size / 2)).map_err(CleanerError::Io)?;
            let bytes_read = file.read(&mut chunk).map_err(CleanerError::Io)?;
            buffer.extend_from_slice(&chunk[..bytes_read]);
            
            file.seek(SeekFrom::End(-(chunk_size as i64))).map_err(CleanerError::Io)?;
            let bytes_read = file.read(&mut chunk).map_err(CleanerError::Io)?;
            buffer.extend_from_slice(&chunk[..bytes_read]);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn get_temp_file(name: &str, content: &[u8]) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique_name = format!(
            "dup_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique_name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_compute_quick_hash_small_file() {
        let detector = DuplicateDetector::new(DuplicateDetectorOptions::default());
        let content = b"hello world small file";
        let path = get_temp_file("small.txt", content);
        
        let hash = detector.compute_quick_hash(&path).unwrap();
        assert!(!hash.is_empty());
        
        let hash2 = detector.compute_quick_hash(&path).unwrap();
        assert_eq!(hash, hash2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_compute_quick_hash_large_file() {
        let detector = DuplicateDetector::new(DuplicateDetectorOptions {
            quick_hash_size: 10,
            ..Default::default()
        });
        
        // Create a file large enough to trigger the 3-part sampling (size > 3 * 10 = 30 bytes)
        // We will make it 40 bytes.
        // It samples:
        // - First 10 bytes: index 0..10
        // - Middle 10 bytes: index 15..25 (middle is 40/2 - 10/2 = 20 - 5 = 15)
        // - Last 10 bytes: index 30..40 (end - 10)
        let mut content1 = vec![0u8; 40];
        let mut content2 = vec![0u8; 40];
        
        // Start difference
        content1[0] = 1; content2[0] = 2;
        let path1 = get_temp_file("large1.txt", &content1);
        let path2 = get_temp_file("large2.txt", &content2);
        let h1 = detector.compute_quick_hash(&path1).unwrap();
        let h2 = detector.compute_quick_hash(&path2).unwrap();
        assert_ne!(h1, h2, "Should differ in start chunk");
        let _ = std::fs::remove_file(&path1);
        let _ = std::fs::remove_file(&path2);

        // Middle difference
        content1[0] = 0; content2[0] = 0;
        content1[20] = 1; content2[20] = 2;
        let path1 = get_temp_file("large1.txt", &content1);
        let path2 = get_temp_file("large2.txt", &content2);
        let h1 = detector.compute_quick_hash(&path1).unwrap();
        let h2 = detector.compute_quick_hash(&path2).unwrap();
        assert_ne!(h1, h2, "Should differ in middle chunk");
        let _ = std::fs::remove_file(&path1);
        let _ = std::fs::remove_file(&path2);

        // End difference
        content1[20] = 0; content2[20] = 0;
        content1[39] = 1; content2[39] = 2;
        let path1 = get_temp_file("large1.txt", &content1);
        let path2 = get_temp_file("large2.txt", &content2);
        let h1 = detector.compute_quick_hash(&path1).unwrap();
        let h2 = detector.compute_quick_hash(&path2).unwrap();
        assert_ne!(h1, h2, "Should differ in end chunk");
        let _ = std::fs::remove_file(&path1);
        let _ = std::fs::remove_file(&path2);

        // Non-sampled region difference (e.g. index 12, which is not in 0..10, 15..25, 30..40)
        content1[39] = 0; content2[39] = 0;
        content1[12] = 1; content2[12] = 2;
        let path1 = get_temp_file("large1.txt", &content1);
        let path2 = get_temp_file("large2.txt", &content2);
        let h1 = detector.compute_quick_hash(&path1).unwrap();
        let h2 = detector.compute_quick_hash(&path2).unwrap();
        assert_eq!(h1, h2, "Should NOT differ when modification is outside sampled chunks");
        let _ = std::fs::remove_file(&path1);
        let _ = std::fs::remove_file(&path2);
    }

    #[test]
    fn test_find_duplicates() {
        let detector = DuplicateDetector::new(DuplicateDetectorOptions {
            min_size: 5,
            ..Default::default()
        });
        
        let c_dup = b"duplicate content hello";
        let c_diff = b"different content world";
        
        let path_a = get_temp_file("dup_a.txt", c_dup);
        let path_b = get_temp_file("dup_b.txt", c_dup);
        let path_c = get_temp_file("diff.txt", c_diff);
        let path_d = get_temp_file("small.txt", b"tiny"); // under min_size (5 bytes)
        
        let result = detector.find_duplicates(&[path_a.clone(), path_b.clone(), path_c.clone(), path_d.clone()], None).unwrap();
        
        assert_eq!(result.groups.len(), 1, "Should find exactly 1 group of duplicates");
        let group = &result.groups[0];
        assert_eq!(group.files.len(), 2, "Duplicate group should contain 2 files");
        assert!(group.files.contains(&path_a));
        assert!(group.files.contains(&path_b));
        assert_eq!(group.size, c_dup.len() as u64);
        assert_eq!(group.total_wasted_space, c_dup.len() as u64);

        let _ = std::fs::remove_file(path_a);
        let _ = std::fs::remove_file(path_b);
        let _ = std::fs::remove_file(path_c);
        let _ = std::fs::remove_file(path_d);
    }
}

