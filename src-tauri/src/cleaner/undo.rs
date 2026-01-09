use super::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub files_count: usize,
    pub bytes_freed: u64,
    pub status: UndoStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoStatus {
    Available,
    Restored,
    Expired,
}

pub struct UndoManager {
    db: Mutex<Connection>,
    backup_dir: PathBuf,
}

impl UndoManager {
    pub async fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("desktop-cleaner");

        fs::create_dir_all(&data_dir).await?;

        let backup_dir = data_dir.join("backups");
        fs::create_dir_all(&backup_dir).await?;

        let db_path = data_dir.join("undo.db");
        let conn = Connection::open(&db_path)?;

        // Initialize database
        conn.execute(
            "CREATE TABLE IF NOT EXISTS undo_entries (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                files_count INTEGER NOT NULL,
                bytes_freed INTEGER NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS undo_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_id TEXT NOT NULL,
                original_path TEXT NOT NULL,
                backup_path TEXT NOT NULL,
                size INTEGER NOT NULL,
                FOREIGN KEY (entry_id) REFERENCES undo_entries(id)
            )",
            [],
        )?;

        Ok(Self {
            db: Mutex::new(conn),
            backup_dir,
        })
    }

    pub async fn backup_file(&self, entry_id: &str, path: &Path) -> Result<()> {
        // Create backup directory for this entry
        let entry_backup_dir = self.backup_dir.join(entry_id);
        fs::create_dir_all(&entry_backup_dir).await?;

        // Generate backup filename (preserve original structure somewhat)
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let backup_name = format!("{}_{}", uuid::Uuid::new_v4(), file_name);
        let backup_path = entry_backup_dir.join(&backup_name);

        // Copy file to backup location
        fs::copy(path, &backup_path).await?;

        // Get file size
        let metadata = fs::metadata(path).await?;
        let size = metadata.len();

        // Record in database
        let db = self.db.lock().await;
        db.execute(
            "INSERT INTO undo_files (entry_id, original_path, backup_path, size) VALUES (?1, ?2, ?3, ?4)",
            params![
                entry_id,
                path.to_string_lossy().to_string(),
                backup_path.to_string_lossy().to_string(),
                size as i64
            ],
        )?;

        Ok(())
    }

    pub async fn finalize_entry(
        &self,
        entry_id: &str,
        files_count: usize,
        bytes_freed: u64,
    ) -> Result<()> {
        let db = self.db.lock().await;
        db.execute(
            "INSERT INTO undo_entries (id, created_at, files_count, bytes_freed, status) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry_id,
                Utc::now().to_rfc3339(),
                files_count as i64,
                bytes_freed as i64,
                "Available"
            ],
        )?;

        info!("Created undo entry: {} ({} files, {} bytes)", entry_id, files_count, bytes_freed);
        Ok(())
    }

    pub async fn undo_last(&mut self) -> Result<CleanupResult> {
        let db = self.db.lock().await;

        // Get the last available entry
        let entry: Option<(String, i64)> = db.query_row(
            "SELECT id, files_count FROM undo_entries 
             WHERE status = 'Available' 
             ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        drop(db);

        let (entry_id, _files_count) = entry.ok_or_else(|| anyhow::anyhow!("No undo entries available"))?;

        self.restore_entry(&entry_id).await
    }

    async fn restore_entry(&self, entry_id: &str) -> Result<CleanupResult> {
        let db = self.db.lock().await;

        // Get all files for this entry
        let mut stmt = db.prepare(
            "SELECT original_path, backup_path, size FROM undo_files WHERE entry_id = ?1"
        )?;

        let files: Vec<(String, String, i64)> = stmt
            .query_map(params![entry_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        drop(stmt);
        drop(db);

        let mut restored = 0usize;
        let mut bytes_restored = 0u64;
        let mut errors = Vec::new();

        for (original_path, backup_path, size) in files {
            match fs::copy(&backup_path, &original_path).await {
                Ok(_) => {
                    restored += 1;
                    bytes_restored += size as u64;
                    info!("Restored: {}", original_path);

                    // Clean up backup
                    let _ = fs::remove_file(&backup_path).await;
                }
                Err(e) => {
                    errors.push(CleanupError {
                        path: original_path,
                        error: e.to_string(),
                    });
                }
            }
        }

        // Update entry status
        let db = self.db.lock().await;
        db.execute(
            "UPDATE undo_entries SET status = 'Restored' WHERE id = ?1",
            params![entry_id],
        )?;

        // Clean up backup directory
        let entry_backup_dir = self.backup_dir.join(entry_id);
        let _ = fs::remove_dir_all(entry_backup_dir).await;

        Ok(CleanupResult {
            success: errors.is_empty(),
            files_deleted: restored,
            bytes_freed: bytes_restored,
            errors,
            undo_id: None,
        })
    }

    pub async fn get_history(&self) -> Result<Vec<UndoEntry>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT id, created_at, files_count, bytes_freed, status 
             FROM undo_entries 
             ORDER BY created_at DESC 
             LIMIT 50"
        )?;

        let entries = stmt
            .query_map([], |row| {
                let status_str: String = row.get(4)?;
                let status = match status_str.as_str() {
                    "Available" => UndoStatus::Available,
                    "Restored" => UndoStatus::Restored,
                    _ => UndoStatus::Expired,
                };

                Ok(UndoEntry {
                    id: row.get(0)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    files_count: row.get::<_, i64>(2)? as usize,
                    bytes_freed: row.get::<_, i64>(3)? as u64,
                    status,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub async fn cleanup_old_entries(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        
        let db = self.db.lock().await;

        // Get old entry IDs
        let mut stmt = db.prepare(
            "SELECT id FROM undo_entries WHERE created_at < ?1"
        )?;

        let old_entries: Vec<String> = stmt
            .query_map(params![cutoff.to_rfc3339()], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        drop(stmt);

        let count = old_entries.len();

        for entry_id in old_entries {
            // Delete backup files
            let entry_backup_dir = self.backup_dir.join(&entry_id);
            let _ = tokio::fs::remove_dir_all(entry_backup_dir).await;

            // Delete database entries
            db.execute("DELETE FROM undo_files WHERE entry_id = ?1", params![entry_id])?;
            db.execute("DELETE FROM undo_entries WHERE id = ?1", params![entry_id])?;
        }

        Ok(count)
    }
}
