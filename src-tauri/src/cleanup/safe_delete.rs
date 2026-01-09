// src-tauri/src/cleanup/safe_delete.rs
use crate::database;
use crate::error::{Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOptions {
    pub dry_run: bool,
    pub use_trash: bool,
    pub create_restore_point: bool,
    pub secure_delete: bool,
}

impl Default for DeleteOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            use_trash: true,
            create_restore_point: true,
            secure_delete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub deleted_files: Vec<PathBuf>,
    pub failed_files: Vec<(PathBuf, String)>,
    pub bytes_freed: u64,
    pub restore_point_id: Option<i64>,
    pub was_dry_run: bool,
}

pub struct SafeDeleter {
    options: DeleteOptions,
    backup_dir: PathBuf,
}

impl SafeDeleter {
    pub fn new(options: DeleteOptions, app_data_dir: &Path) -> Self {
        let backup_dir = app_data_dir.join("backups");
        Self { options, backup_dir }
    }

    pub fn delete_files(
        &self,
        files: &[PathBuf],
        db_conn: &Connection,
    ) -> Result<DeleteResult> {
        let mut deleted = Vec::new();
        let mut failed = Vec::new();
        let mut bytes_freed = 0u64;
        let mut restore_point_id = None;

        if self.options.create_restore_point && !self.options.dry_run {
            fs::create_dir_all(&self.backup_dir)?;
        }

        for path in files {
            if !path.exists() {
                failed.push((path.clone(), "File not found".to_string()));
                continue;
            }

            let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

            if self.options.dry_run {
                deleted.push(path.clone());
                bytes_freed += file_size;
                continue;
            }

            if self.options.create_restore_point {
                match self.backup_file(path, db_conn) {
                    Ok(id) => restore_point_id = Some(id),
                    Err(e) => {
                        tracing::warn!("Failed to create backup for {:?}: {}", path, e);
                    }
                }
            }

            let delete_result = if self.options.use_trash {
                trash::delete(path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            } else if self.options.secure_delete {
                super::secure_delete::secure_delete(path)
            } else {
                if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                }
            };

            match delete_result {
                Ok(_) => {
                    deleted.push(path.clone());
                    bytes_freed += file_size;
                }
                Err(e) => {
                    failed.push((path.clone(), e.to_string()));
                }
            }
        }

        if !self.options.dry_run && !deleted.is_empty() {
            let details = serde_json::to_string(&deleted).ok();
            database::add_cleanup_history(
                db_conn,
                "delete",
                deleted.len() as i64,
                bytes_freed as i64,
                details.as_deref(),
            )?;
        }

        Ok(DeleteResult {
            deleted_files: deleted,
            failed_files: failed,
            bytes_freed,
            restore_point_id,
            was_dry_run: self.options.dry_run,
        })
    }

    fn backup_file(&self, path: &Path, db_conn: &Connection) -> Result<i64> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let backup_name = format!("{}_{}", timestamp, file_name);
        let backup_path = self.backup_dir.join(&backup_name);

        fs::copy(path, &backup_path)?;

        let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        
        let id = database::add_restore_point(
            db_conn,
            &path.to_string_lossy(),
            &backup_path.to_string_lossy(),
            None,
            file_size as i64,
        )?;

        Ok(id)
    }
}
