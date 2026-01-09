// src-tauri/src/platform/linux.rs

#![cfg(target_os = "linux")]

use std::path::PathBuf;
use tracing::{info, warn};
use crate::commands::StartupItem;

pub async fn get_startup_items() -> Result<Vec<StartupItem>, String> {
    let mut items = Vec::new();
    
    // XDG autostart
    if let Some(config) = directories::ProjectDirs::from("com", "systemsweep", "SystemSweep")
        .map(|p| p.config_dir().join("autostart")) {
        if config.exists() {
            if let Ok(entries) = std::fs::read_dir(&config) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "desktop") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let name = parse_desktop_name(&content)
                                .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                            let exec = parse_desktop_exec(&content).unwrap_or_default();
                            let hidden = content.contains("Hidden=true");
                            
                            items.push(StartupItem {
                                name,
                                path: exec,
                                enabled: !hidden,
                                source: "XDG Autostart".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok(items)
}

fn parse_desktop_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Name=") {
            return Some(line[5..].to_string());
        }
    }
    None
}

fn parse_desktop_exec(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Exec=") {
            return Some(line[5..].to_string());
        }
    }
    None
}

pub async fn toggle_startup_item(name: &str, enabled: bool) -> Result<bool, String> {
    if let Some(config) = directories::ProjectDirs::from("com", "systemsweep", "SystemSweep")
        .map(|p| p.config_dir().join("autostart")) {
        
        if let Ok(entries) = std::fs::read_dir(&config) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(item_name) = parse_desktop_name(&content) {
                        if item_name == name {
                            let new_content = if enabled {
                                content.replace("Hidden=true", "Hidden=false")
                            } else if content.contains("Hidden=") {
                                content.replace("Hidden=false", "Hidden=true")
                            } else {
                                format!("{}\nHidden=true", content)
                            };
                            
                            if std::fs::write(&path, new_content).is_ok() {
                                info!("Toggled linux startup item: {} to {}", name, enabled);
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err("Failed to modify linux startup item".to_string())
}

pub fn get_cleanup_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        paths.push(home.join(".cache"));
    }
    paths.push(PathBuf::from("/tmp"));
    paths.push(PathBuf::from("/var/tmp"));
    paths
}
