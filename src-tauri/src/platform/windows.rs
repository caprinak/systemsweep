// src-tauri/src/platform/windows.rs

// Windows-specific functionality

#![cfg(windows)]

use std::path::PathBuf;
use tracing::{info, warn};

use crate::commands::StartupItem;

pub async fn get_startup_items() -> Result<Vec<StartupItem>, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let mut items = Vec::new();
    
    // User startup entries
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run") {
        for name in run_key.enum_values().filter_map(|r| r.ok()).map(|(n, _)| n) {
            if let Ok(value) = run_key.get_value::<String, _>(&name) {
                items.push(StartupItem {
                    name: name.clone(),
                    path: value,
                    enabled: true,
                    source: "Registry (HKCU)".to_string(),
                });
            }
        }
    }
    
    // System startup entries
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(run_key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run") {
        for name in run_key.enum_values().filter_map(|r| r.ok()).map(|(n, _)| n) {
            if let Ok(value) = run_key.get_value::<String, _>(&name) {
                items.push(StartupItem {
                    name: name.clone(),
                    path: value,
                    enabled: true,
                    source: "Registry (HKLM)".to_string(),
                });
            }
        }
    }
    
    // Startup folder
    if let Some(local) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
        let startup_path = local.join("AppData").join("Roaming").join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("Startup");
        
        if startup_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&startup_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    items.push(StartupItem {
                        name,
                        path: entry.path().to_string_lossy().to_string(),
                        enabled: true,
                        source: "Startup Folder".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(items)
}

pub async fn toggle_startup_item(name: &str, enabled: bool) -> Result<bool, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    // Try HKCU first
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_READ | KEY_WRITE
    ) {
        if enabled {
            // Re-enable not supported without knowing original value
            warn!("Re-enabling startup items requires storing original values");
            return Err("Cannot re-enable item without original path".to_string());
        } else {
            if run_key.delete_value(name).is_ok() {
                info!("Disabled startup item: {}", name);
                return Ok(true);
            }
        }
    }
    
    Err("Failed to modify startup item".to_string())
}

pub fn get_cleanup_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Windows Temp
    if let Ok(temp) = std::env::var("TEMP") {
        paths.push(PathBuf::from(&temp));
    }
    
    // Windows prefetch
    if let Ok(windir) = std::env::var("WINDIR") {
        let win_path = PathBuf::from(&windir);
        paths.push(win_path.join("Prefetch"));
        paths.push(win_path.join("Temp"));
    }
    
    // User temp and cache
    if let Some(local) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
        let appdata_local = local.join("AppData").join("Local");
        paths.push(appdata_local.join("Temp"));
        paths.push(appdata_local.join("Microsoft").join("Windows").join("INetCache"));
        paths.push(appdata_local.join("Microsoft").join("Windows").join("Explorer"));
    }
    
    // Recycle Bin
    for drive in ['C', 'D', 'E', 'F'] {
        let recycle = PathBuf::from(format!("{}:\\$Recycle.Bin", drive));
        if recycle.exists() {
            paths.push(recycle);
        }
    }
    
    paths
}

/// Clean Windows Update cache
pub async fn clean_windows_update_cache() -> Result<u64, String> {
    use std::process::Command;
    
    // Stop Windows Update service
    let _ = Command::new("net")
        .args(["stop", "wuauserv"])
        .output();
    
    let mut freed = 0u64;
    
    // Clean SoftwareDistribution folder
    if let Ok(windir) = std::env::var("WINDIR") {
        let sd_path = PathBuf::from(&windir).join("SoftwareDistribution").join("Download");
        if sd_path.exists() {
            if let Ok(metadata) = fs_extra::dir::get_size(&sd_path) {
                freed += metadata;
                let _ = std::fs::remove_dir_all(&sd_path);
                let _ = std::fs::create_dir_all(&sd_path);
            }
        }
    }
    
    // Restart Windows Update service
    let _ = Command::new("net")
        .args(["start", "wuauserv"])
        .output();
    
    Ok(freed)
}
