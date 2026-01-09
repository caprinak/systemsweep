use super::*;
use crate::CleanupOptions;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct SafeDeleter {
    secure_passes: usize,
}

impl SafeDeleter {
    pub fn new() -> Self {
        Self { secure_passes: 3 }
    }

    pub async fn cleanup(
        &self,
        files: Vec<String>,
        options: CleanupOptions,
        undo_manager: &mut UndoManager,
    ) -> Result<CleanupResult> {
        let undo_id = if options.create_restore_point && !options.dry_run {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };

        let mut files_deleted = 0usize;
        let mut bytes_freed = 0u64;
        let mut errors = Vec::new();

        for file_path in &files {
            let path = Path::new(file_path);

            if !path.exists() {
                warn!("File does not exist: {}", file_path);
                continue;
            }

            // Get file size before deletion
            let size = match fs::metadata(path).await {
                Ok(m) => m.len(),
                Err(e) => {
                    errors.push(CleanupError {
                        path: file_path.clone(),
                        error: format!("Failed to get metadata: {}", e),
                    });
                    continue;
                }
            };

            if options.dry_run {
                info!("[DRY RUN] Would delete: {} ({} bytes)", file_path, size);
                files_deleted += 1;
                bytes_freed += size;
                continue;
            }

            // Create restore point if enabled
            if let Some(ref id) = undo_id {
                if let Err(e) = undo_manager.backup_file(id, path).await {
                    warn!("Failed to backup file for undo: {}", e);
                }
            }

            // Perform deletion
            let result = if options.use_trash {
                self.move_to_trash(path).await
            } else if options.secure_delete {
                self.secure_delete(path).await
            } else {
                self.delete_file(path).await
            };

            match result {
                Ok(_) => {
                    info!("Deleted: {} ({} bytes)", file_path, size);
                    files_deleted += 1;
                    bytes_freed += size;
                }
                Err(e) => {
                    errors.push(CleanupError {
                        path: file_path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Finalize undo entry
        if let Some(ref id) = undo_id {
            undo_manager
                .finalize_entry(id, files_deleted, bytes_freed)
                .await?;
        }

        Ok(CleanupResult {
            success: errors.is_empty(),
            files_deleted,
            bytes_freed,
            errors,
            undo_id,
        })
    }

    async fn delete_file(&self, path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)
                .await
                .context("Failed to remove directory")?;
        } else {
            fs::remove_file(path)
                .await
                .context("Failed to remove file")?;
        }
        Ok(())
    }

    async fn move_to_trash(&self, path: &Path) -> Result<()> {
        trash::delete(path).context("Failed to move to trash")?;
        Ok(())
    }

    async fn secure_delete(&self, path: &Path) -> Result<()> {
        if path.is_file() {
            let metadata = fs::metadata(path).await?;
            let size = metadata.len();

            // Overwrite with random data multiple times
            for pass in 0..self.secure_passes {
                self.overwrite_file(path, size, pass).await?;
            }

            // Final overwrite with zeros
            self.overwrite_file_with_zeros(path, size).await?;
        }

        // Delete the file
        self.delete_file(path).await
    }

    async fn overwrite_file(&self, path: &Path, size: u64, pass: usize) -> Result<()> {
        use rand::Rng;
        use tokio::io::AsyncWriteExt;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .await?;

        let mut rng = rand::thread_rng();
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut buffer = vec![0u8; chunk_size];
        let mut written = 0u64;

        while written < size {
            rng.fill(&mut buffer[..]);
            let to_write = std::cmp::min(chunk_size as u64, size - written) as usize;
            file.write_all(&buffer[..to_write]).await?;
            written += to_write as u64;
        }

        file.sync_all().await?;
        debug!("Secure delete pass {} complete for {:?}", pass + 1, path);

        Ok(())
    }

    async fn overwrite_file_with_zeros(&self, path: &Path, size: u64) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .await?;

        let chunk_size = 64 * 1024;
        let buffer = vec![0u8; chunk_size];
        let mut written = 0u64;

        while written < size {
            let to_write = std::cmp::min(chunk_size as u64, size - written) as usize;
            file.write_all(&buffer[..to_write]).await?;
            written += to_write as u64;
        }

        file.sync_all().await?;
        Ok(())
    }
}
