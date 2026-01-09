// src-tauri/src/cleanup/restore.rs
use crate::error::{CleanerError, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub id: i64,
    pub timestamp: String,
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub file_size: u64,
    pub restored: bool,
}

pub fn get_restore_points(conn: &Connection) -> Result<Vec<RestorePoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, original_path, backup_path, file_size, restored 
         FROM restore_points 
         WHERE restored = 0 
         ORDER BY timestamp DESC"
    )?;

    let points = stmt.query_map([], |row| {
        Ok(RestorePoint {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            original_path: PathBuf::from(row.get::<_, String>(2)?),
            backup_path: PathBuf::from(row.get::<_, String>(3)?),
            file_size: row.get::<_, i64>(4)? as u64,
            restored: row.get::<_, i64>(5)? != 0,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(points)
}

pub fn restore_file(conn: &Connection, restore_point_id: i64) -> Result<PathBuf> {
    let mut stmt = conn.prepare(
        "SELECT original_path, backup_path FROM restore_points WHERE id = ?"
    )?;

    let (original_path, backup_path): (String, String) = stmt
        .query_row([restore_point_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let original = PathBuf::from(&original_path);
    let backup = PathBuf::from(&backup_path);

    if !backup.exists() {
        return Err(CleanerError::FileNotFound(backup_path));
    }

    if let Some(parent) = original.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&backup, &original)?;

    conn.execute(
        "UPDATE restore_points SET restored = 1 WHERE id = ?",
        [restore_point_id],
    )?;

    let _ = fs::remove_file(&backup);

    Ok(original)
}
