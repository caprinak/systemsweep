// src-tauri/src/commands/mod.rs
use crate::cleanup::{DeleteOptions, SafeDeleter, DeleteResult, RestorePoint, restore, restore_file};
use crate::config::AppConfig;
use crate::error::Result;
use crate::scanner::*;
use crate::state::{AppState, ScanProgress};
use crate::startup;
use crate::system;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

// ============ Scanner Commands ============

#[tauri::command]
pub async fn scan_system(state: State<'_, AppState>) -> Result<ScanResult> {
    let paths = get_default_scan_paths();
    let scanner = FileScanner::new(ScanOptions {
        categories: vec![
            FileCategory::Cache,
            FileCategory::Temporary,
            FileCategory::Log,
            FileCategory::Thumbnail,
        ],
        ..Default::default()
    });
    
    state.reset_scan();
    scanner.scan(&paths, Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn scan_directory(
    path: String,
    options: Option<ScanOptions>,
    state: State<'_, AppState>,
) -> Result<ScanResult> {
    let paths = vec![PathBuf::from(path)];
    let scanner = FileScanner::new(options.unwrap_or_default());
    
    state.reset_scan();
    scanner.scan(&paths, Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn scan_duplicates(
    paths: Vec<String>,
    min_size_mb: Option<u64>,
    state: State<'_, AppState>,
) -> Result<DuplicateScanResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let detector = DuplicateDetector::new(DuplicateDetectorOptions {
        min_size: min_size_mb.unwrap_or(1) * 1024 * 1024,
        ..Default::default()
    });
    
    state.reset_scan();
    detector.find_duplicates(&paths, Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn scan_large_files(
    paths: Vec<String>,
    min_size_mb: u64,
    top_n: Option<usize>,
    state: State<'_, AppState>,
) -> Result<LargeFileResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let finder = LargeFileFinder::new(min_size_mb, top_n);
    
    state.reset_scan();
    finder.find(&paths, Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn scan_old_files(
    paths: Vec<String>,
    min_age_days: u32,
    state: State<'_, AppState>,
) -> Result<ScanResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let scanner = FileScanner::new(ScanOptions {
        min_age_days: Some(min_age_days),
        include_hidden: true,
        ..Default::default()
    });
    
    state.reset_scan();
    scanner.scan(&paths, Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn scan_cache(state: State<'_, AppState>) -> Result<CacheScanResult> {
    state.reset_scan();
    CacheScanner::scan(Some(Arc::new(state.inner().clone())))
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, AppState>) -> Result<()> {
    state.cancel_scan();
    Ok(())
}

#[tauri::command]
pub async fn get_scan_progress(state: State<'_, AppState>) -> Result<ScanProgress> {
    Ok(state.get_progress())
}

// ============ Cleanup Commands ============

#[tauri::command]
pub async fn delete_files(
    paths: Vec<String>,
    use_trash: bool,
    create_restore_point: bool,
    state: State<'_, AppState>,
) -> Result<DeleteResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let conn = Connection::open(&state.db_path)?;
    
    let deleter = SafeDeleter::new(
        DeleteOptions {
            dry_run: false,
            use_trash,
            create_restore_point,
            secure_delete: false,
        },
        state.db_path.parent().unwrap(),
    );
    
    deleter.delete_files(&paths, &conn)
}

#[tauri::command]
pub async fn delete_files_dry_run(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<DeleteResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let conn = Connection::open(&state.db_path)?;
    
    let deleter = SafeDeleter::new(
        DeleteOptions {
            dry_run: true,
            ..Default::default()
        },
        state.db_path.parent().unwrap(),
    );
    
    deleter.delete_files(&paths, &conn)
}

#[tauri::command]
pub async fn secure_delete(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<DeleteResult> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let conn = Connection::open(&state.db_path)?;
    
    let deleter = SafeDeleter::new(
        DeleteOptions {
            dry_run: false,
            use_trash: false,
            create_restore_point: false,
            secure_delete: true,
        },
        state.db_path.parent().unwrap(),
    );
    
    deleter.delete_files(&paths, &conn)
}

#[tauri::command]
pub async fn move_to_trash(paths: Vec<String>) -> Result<Vec<String>> {
    let mut moved = Vec::new();
    for path in paths {
        if trash::delete(&path).is_ok() {
            moved.push(path);
        }
    }
    Ok(moved)
}

#[tauri::command]
pub async fn restore_files(
    restore_point_id: i64,
    state: State<'_, AppState>,
) -> Result<PathBuf> {
    let conn = Connection::open(&state.db_path)?;
    restore_file(&conn, restore_point_id)
}

#[tauri::command]
pub async fn get_restore_points(
    state: State<'_, AppState>,
) -> Result<Vec<RestorePoint>> {
    let conn = Connection::open(&state.db_path)?;
    restore::get_restore_points(&conn)
}

// ============ Startup Commands ============

#[tauri::command]
pub async fn get_startup_items() -> Result<Vec<startup::StartupItem>> {
    startup::get_startup_items().await
}

#[tauri::command]
pub async fn toggle_startup_item(name: String, enabled: bool) -> Result<bool> {
    startup::toggle_startup_item(&name, enabled).await
}

#[tauri::command]
pub async fn add_startup_item(
    name: String,
    command: String,
    description: Option<String>,
) -> Result<PathBuf> {
    startup::add_startup_item(&name, &command, description.as_deref()).await
}

#[tauri::command]
pub async fn remove_startup_item(name: String) -> Result<bool> {
    startup::remove_startup_item(&name).await
}

// ============ System Info Commands ============

#[tauri::command]
pub async fn get_system_info() -> Result<system::SystemInfo> {
    Ok(system::get_system_info())
}

#[tauri::command]
pub async fn get_disk_usage() -> Result<Vec<system::DiskInfo>> {
    Ok(system::get_disk_usage())
}

#[tauri::command]
pub async fn get_memory_usage() -> Result<system::MemoryInfo> {
    Ok(system::get_memory_usage())
}

#[tauri::command]
pub async fn get_running_processes() -> Result<Vec<system::ProcessInfo>> {
    Ok(system::get_running_processes())
}

// ============ Scheduler Commands ============

#[tauri::command]
pub async fn schedule_cleanup(
    name: String,
    schedule_type: String,
    schedule_value: String,
    task_config: String,
    state: State<'_, AppState>,
) -> Result<i64> {
    let conn = Connection::open(&state.db_path)?;
    conn.execute(
        "INSERT INTO scheduled_tasks (name, schedule_type, schedule_value, task_config) VALUES (?, ?, ?, ?)",
        rusqlite::params![name, schedule_type, schedule_value, task_config],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn get_scheduled_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>> {
    let conn = Connection::open(&state.db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, schedule_type, schedule_value, task_config, enabled, last_run, next_run FROM scheduled_tasks"
    )?;
    
    let tasks: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "schedule_type": row.get::<_, String>(2)?,
            "schedule_value": row.get::<_, String>(3)?,
            "task_config": row.get::<_, String>(4)?,
            "enabled": row.get::<_, i64>(5)? != 0,
            "last_run": row.get::<_, Option<String>>(6)?,
            "next_run": row.get::<_, Option<String>>(7)?
        }))
    })?.filter_map(|r| r.ok()).collect();
    
    Ok(tasks)
}

#[tauri::command]
pub async fn remove_scheduled_task(
    task_id: i64,
    state: State<'_, AppState>,
) -> Result<bool> {
    let conn = Connection::open(&state.db_path)?;
    let affected = conn.execute("DELETE FROM scheduled_tasks WHERE id = ?", [task_id])?;
    Ok(affected > 0)
}

// ============ Configuration Commands ============

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig> {
    // In a real app we'd load from disk
    Ok(AppConfig::default())
}

#[tauri::command]
pub async fn save_config(
    _config: AppConfig,
    _state: State<'_, AppState>,
) -> Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn reset_config(_state: State<'_, AppState>) -> Result<AppConfig> {
    Ok(AppConfig::default())
}

// ============ History Commands ============

#[tauri::command]
pub async fn get_cleanup_history(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>> {
    let conn = Connection::open(&state.db_path)?;
    let limit = limit.unwrap_or(100);
    
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, operation_type, files_count, bytes_cleaned, details 
         FROM cleanup_history 
         ORDER BY timestamp DESC 
         LIMIT ?"
    )?;
    
    let history: Vec<serde_json::Value> = stmt.query_map([limit], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, String>(1)?,
            "operation_type": row.get::<_, String>(2)?,
            "files_count": row.get::<_, i64>(3)?,
            "bytes_cleaned": row.get::<_, i64>(4)?,
            "details": row.get::<_, Option<String>>(5)?
        }))
    })?.filter_map(|r| r.ok()).collect();
    
    Ok(history)
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<()> {
    let conn = Connection::open(&state.db_path)?;
    conn.execute("DELETE FROM cleanup_history", [])?;
    Ok(())
}

// ============ Utility Functions ============

fn get_default_scan_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".cache"));
            paths.push(home.join(".local/share/Trash"));
        }
        paths.push(PathBuf::from("/tmp"));
        paths.push(PathBuf::from("/var/tmp"));
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = dirs::data_local_dir() {
            paths.push(local.join("Temp"));
        }
        paths.push(PathBuf::from("C:\\Windows\\Temp"));
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Caches"));
            paths.push(home.join(".Trash"));
        }
    }
    
    paths.into_iter().filter(|p| p.exists()).collect()
}
