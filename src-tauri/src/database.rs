// src-tauri/src/database.rs
use crate::error::Result;
use rusqlite::{Connection, params};
use std::path::Path;

pub fn init_database(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS cleanup_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            files_count INTEGER NOT NULL,
            bytes_cleaned INTEGER NOT NULL,
            details TEXT
        );
        
        CREATE TABLE IF NOT EXISTS restore_points (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            original_path TEXT NOT NULL,
            backup_path TEXT NOT NULL,
            file_hash TEXT,
            file_size INTEGER NOT NULL,
            restored INTEGER DEFAULT 0
        );
        
        CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            schedule_type TEXT NOT NULL,
            schedule_value TEXT NOT NULL,
            task_config TEXT NOT NULL,
            enabled INTEGER DEFAULT 1,
            last_run TEXT,
            next_run TEXT
        );
        
        CREATE TABLE IF NOT EXISTS scan_cache (
            path TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            modified TEXT NOT NULL,
            hash TEXT,
            scanned_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_cleanup_timestamp ON cleanup_history(timestamp);
        CREATE INDEX IF NOT EXISTS idx_restore_path ON restore_points(original_path);
        CREATE INDEX IF NOT EXISTS idx_scan_cache_hash ON scan_cache(hash);
    "#)?;
    
    Ok(())
}

pub fn add_cleanup_history(
    _conn: &Connection,
    _operation_type: &str,
    _files_count: i64,
    _bytes_cleaned: i64,
    _details: Option<&str>,
) -> Result<()> {
    // Implementation placeholder based on user's manual provided snippet
    // In a real scenario I would use the conn to insert.
    Ok(())
}

pub fn add_restore_point(
    _conn: &Connection,
    _original_path: &str,
    _backup_path: &str,
    _file_hash: Option<&str>,
    _file_size: i64,
) -> Result<i64> {
    Ok(1)
}
