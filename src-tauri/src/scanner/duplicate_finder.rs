use super::*;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;

pub struct DuplicateFinder {
    min_file_size: u64,
    max_file_size: Option<u64>,
    excluded_patterns: Vec<String>,
}

impl DuplicateFinder {
    pub fn new() -> Self {
        Self {
            min_file_size: 1024, // 1KB minimum
            max_file_size: None,
            excluded_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                ".svn".to_string(),
            ],
        }
    }

    pub fn with_min_size(mut self, size: u64) -> Self {
        self.min_file_size = size;
        self
    }

    pub fn with_max_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    pub async fn find_in_paths(&self, paths: &[String]) -> Result<Vec<DuplicateGroup>> {
        let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

        // Phase 1: Group files by size (very fast)
        for path_str in paths {
            self.collect_files_by_size(Path::new(path_str), &mut size_groups)?;
        }

        // Filter out unique sizes
        let potential_duplicates: Vec<_> = size_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        let mut duplicates = Vec::new();

        // Phase 2: Compare file hashes
        for (size, files) in potential_duplicates {
            let groups = self.find_duplicates_by_hash(&files, size).await?;
            duplicates.extend(groups);
        }

        duplicates.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));
        Ok(duplicates)
    }

    fn collect_files_by_size(
        &self,
        root: &Path,
        size_groups: &mut HashMap<u64, Vec<PathBuf>>,
    ) -> Result<()> {
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e.path()))
            .filter_map(|e| e.ok())
        {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let size = metadata.len();

                    // Apply size filters
                    if size < self.min_file_size {
                        continue;
                    }
                    if let Some(max) = self.max_file_size {
                        if size > max {
                            continue;
                        }
                    }

                    size_groups
                        .entry(size)
                        .or_default()
                        .push(entry.path().to_path_buf());
                }
            }
        }

        Ok(())
    }

    async fn find_duplicates_by_hash(
        &self,
        files: &[PathBuf],
        size: u64,
    ) -> Result<Vec<DuplicateGroup>> {
        let mut hash_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // For small files, hash entirely
        // For large files, use partial hashing strategy
        let use_partial = size > 10 * 1024 * 1024; // 10MB

        for file in files {
            let hash = if use_partial {
                self.partial_hash(file).await?
            } else {
                self.full_hash(file).await?
            };

            hash_groups.entry(hash).or_default().push(file.clone());
        }

        // For partial hash matches, verify with full hash
        let mut result = Vec::new();

        for (hash, paths) in hash_groups.into_iter().filter(|(_, v)| v.len() > 1) {
            let verified = if use_partial {
                self.verify_duplicates(&paths).await?
            } else {
                vec![paths]
            };

            for group in verified {
                if group.len() > 1 {
                    let file_infos = self.paths_to_file_infos(&group).await?;
                    result.push(DuplicateGroup {
                        hash: hash.clone(),
                        size,
                        potential_savings: size * (group.len() as u64 - 1),
                        files: file_infos,
                    });
                }
            }
        }

        Ok(result)
    }

    async fn full_hash(&self, path: &Path) -> Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    async fn partial_hash(&self, path: &Path) -> Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024];

        // Hash first 64KB
        let n = file.read(&mut buffer).await?;
        hasher.update(&buffer[..n]);

        // Hash middle 64KB
        if size > 128 * 1024 {
            use tokio::io::AsyncSeekExt;
            file.seek(std::io::SeekFrom::Start(size / 2)).await?;
            let n = file.read(&mut buffer).await?;
            hasher.update(&buffer[..n]);
        }

        // Hash last 64KB
        if size > 64 * 1024 {
            use tokio::io::AsyncSeekExt;
            file.seek(std::io::SeekFrom::End(-64 * 1024)).await?;
            let n = file.read(&mut buffer).await?;
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    async fn verify_duplicates(&self, paths: &[PathBuf]) -> Result<Vec<Vec<PathBuf>>> {
        let mut hash_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for path in paths {
            let hash = self.full_hash(path).await?;
            hash_groups.entry(hash).or_default().push(path.clone());
        }

        Ok(hash_groups.into_values().filter(|v| v.len() > 1).collect())
    }

    async fn paths_to_file_infos(&self, paths: &[PathBuf]) -> Result<Vec<FileInfo>> {
        let mut infos = Vec::new();

        for path in paths {
            if let Ok(metadata) = tokio::fs::metadata(path).await {
                let modified = metadata
                    .modified()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let created = metadata
                    .created()
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

                infos.push(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    size: metadata.len(),
                    modified,
                    created,
                    is_directory: false,
                    extension: path.extension().map(|e| e.to_string_lossy().to_string()),
                    file_type: FileType::Duplicate,
                });
            }
        }

        Ok(infos)
    }

    fn should_exclude(&self, path: &Path) -> bool {
        for pattern in &self.excluded_patterns {
            if path.to_string_lossy().contains(pattern) {
                return true;
            }
        }
        false
    }
}
