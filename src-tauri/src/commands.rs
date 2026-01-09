// src-tauri/src/commands.rs

// Tauri commands - bridge between frontend and backend

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::core::{
    scanner::{FileScanner, ScanConfig, ScanProgress, ScanResult, ScannedFile},
    duplicates::{DuplicateFinder, DuplicateConfig, DuplicateResult},
    cleaner::{Cleaner, CleanupOptions, CleanupResult},
    rules::CleanupRule,
    undo::{UndoHistory, UndoResult},
    state::AppState,
};
use crate::platform;
use crate::utils::config::AppConfig;

// ============================================================================
// Scanner Commands
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub paths: Vec<String>,
    pub include_hidden: bool,
    pub min_file_age_days: Option<u32>,
    pub enabled_rules: Vec<String>,
}

#[tauri::command]
pub async fn start_scan(
    request: ScanRequest,
    state: State<'_, AppState>,
) -> Result<ScanResult, String> {
    info!("Starting scan with {} paths", request.paths.len());
    
    let scan_id = state.create_scan_session();
    let paths: Vec<PathBuf> = request.paths.iter().map(PathBuf::from).collect();
    
    // Get enabled rules from config
    let config = state.config.read().unwrap();
    let rules: Vec<CleanupRule> = config.cleanup_rules.iter()
        .filter(|r| r.enabled || request.enabled_rules.contains(&r.id))
        .cloned()
        .collect();
    drop(config);
    
    let rule_engine = crate::core::rules::RuleEngine::new(rules);
    let scan_config = ScanConfig {
        paths,
        include_hidden: request.include_hidden,
        ..Default::default()
    };
    
    let scanner = FileScanner::new(scan_config, rule_engine)
        .with_progress_channel(state.progress_tx.clone());
    
    scanner.scan(&scan_id.to_string())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_scan(
    scan_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let uuid = uuid::Uuid::parse_str(&scan_id).map_err(|e| e.to_string())?;
    Ok(state.cancel_scan(uuid))
}

#[tauri::command]
pub async fn get_scan_progress(
    state: State<'_, AppState>,
) -> Result<Option<ScanProgress>, String> {
    let mut rx = state.progress_tx.subscribe();
    
    match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv()
    ).await {
        Ok(Ok(progress)) => Ok(Some(progress)),
        _ => Ok(None),
    }
}

// ============================================================================
// Cleaner Commands
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanRequest {
    pub files: Vec<ScannedFile>,
    pub use_trash: bool,
    pub secure_delete: bool,
}

#[tauri::command]
pub async fn clean_files(
    request: CleanRequest,
    state: State<'_, AppState>,
) -> Result<CleanupResult, String> {
    info!("Cleaning {} files", request.files.len());
    
    let options = CleanupOptions {
        dry_run: false,
        use_trash: request.use_trash,
        secure_delete: request.secure_delete,
        ..Default::default()
    };
    
    // Create a new Arc<RwLock<UndoManager>> from the state's field
    // Note: AppState already stores Arc<RwLock<UndoManager>>
    let cleaner = Cleaner::new(state.undo_manager.clone())
    .with_options(options);
    
    cleaner.clean_files(&request.files)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dry_run_clean(
    request: CleanRequest,
    state: State<'_, AppState>,
) -> Result<CleanupResult, String> {
    info!("Dry run for {} files", request.files.len());
    
    let options = CleanupOptions {
        dry_run: true,
        use_trash: request.use_trash,
        ..Default::default()
    };
    
    let cleaner = Cleaner::new(state.undo_manager.clone())
    .with_options(options);
    
    cleaner.clean_files(&request.files)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preview_cleanup(
    files: Vec<ScannedFile>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let cleaner = Cleaner::new(state.undo_manager.clone());
    
    let preview = cleaner.preview(&files).await;
    
    serde_json::to_value(preview.by_category).map_err(|e| e.to_string())
}

// ============================================================================
// Duplicate Commands
// ============================================================================

#[tauri::command]
pub async fn find_duplicates(
    paths: Vec<String>,
    min_size_mb: Option<f64>,
) -> Result<DuplicateResult, String> {
    info!("Finding duplicates in {} paths", paths.len());
    
    // Implementation would collect all files and run finder
    let finder = DuplicateFinder::new();
    let paths_buf: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    
    // Simplified for now - would need proper file collection
    let mut all_files = Vec::new();
    for path in &paths_buf {
        if path.is_file() {
            all_files.push(path.clone());
        }
    }
    
    finder.find_duplicates(&all_files)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_duplicate_groups(
    result: DuplicateResult,
) -> Result<Vec<serde_json::Value>, String> {
    result.groups.iter()
        .map(|g| serde_json::to_value(g).map_err(|e| e.to_string()))
        .collect()
}

// ============================================================================
// Undo Commands
// ============================================================================

#[tauri::command]
pub async fn undo_last_operation(
    state: State<'_, AppState>,
) -> Result<UndoResult, String> {
    let mut undo_manager = state.undo_manager.write().unwrap();
    undo_manager.undo_last().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_undo_history(
    state: State<'_, AppState>,
) -> Result<UndoHistory, String> {
    let undo_manager = state.undo_manager.read().unwrap();
    Ok(undo_manager.get_history())
}

#[tauri::command]
pub async fn restore_files(
    operation_id: String,
    state: State<'_, AppState>,
) -> Result<UndoResult, String> {
    let mut undo_manager = state.undo_manager.write().unwrap();
    undo_manager.undo_operation(&operation_id).map_err(|e| e.to_string())
}

// ============================================================================
// System Commands
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub uptime: u64,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::{System, SystemExt, CpuExt};
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    Ok(SystemInfo {
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        hostname: System::host_name().unwrap_or_default(),
        cpu_count: sys.cpus().len(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        uptime: System::uptime(),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
    pub file_system: String,
}

#[tauri::command]
pub async fn get_disk_usage() -> Result<Vec<DiskInfo>, String> {
    use sysinfo::{System, SystemExt, DiskExt};
    
    let mut sys = System::new_all();
    sys.refresh_disks_list();
    
    let disks: Vec<DiskInfo> = sys.disks().iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 },
                file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
            }
        })
        .collect();
    
    Ok(disks)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupItem {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub source: String,
}

#[tauri::command]
pub async fn get_startup_items() -> Result<Vec<StartupItem>, String> {
    platform::get_startup_items().await
}

#[tauri::command]
pub async fn toggle_startup_item(
    name: String,
    enabled: bool,
) -> Result<bool, String> {
    platform::toggle_startup_item(&name, enabled).await
}

// ============================================================================
// Settings Commands
// ============================================================================

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    Ok(state.config.read().unwrap().clone())
}

#[tauri::command]
pub async fn save_settings(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    *state.config.write().unwrap() = config.clone();
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_scan_rules(
    state: State<'_, AppState>,
) -> Result<Vec<CleanupRule>, String> {
    Ok(state.config.read().unwrap().cleanup_rules.clone())
}

#[tauri::command]
pub async fn save_scan_rules(
    rules: Vec<CleanupRule>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().unwrap();
    config.cleanup_rules = rules;
    config.save().map_err(|e| e.to_string())
}

// ============================================================================
// Scheduler Commands
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

#[tauri::command]
pub async fn schedule_cleanup(
    task: ScheduledTask,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Implementation would add task to scheduler
    info!("Scheduled cleanup task: {}", task.name);
    Ok(())
}

#[tauri::command]
pub async fn get_scheduled_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<ScheduledTask>, String> {
    // Return scheduled tasks
    Ok(vec![])
}

#[tauri::command]
pub async fn cancel_scheduled_task(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Cancelled scheduled task: {}", task_id);
    Ok(true)
}
