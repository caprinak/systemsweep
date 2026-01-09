// src-tauri/src/platform/mod.rs

// Platform-specific functionality

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::commands::StartupItem;

pub async fn get_startup_items() -> Result<Vec<StartupItem>, String> {
    #[cfg(windows)]
    return windows::get_startup_items().await;
    
    #[cfg(target_os = "macos")]
    return Ok(Vec::new()); // Placeholder
    
    #[cfg(target_os = "linux")]
    return linux::get_startup_items().await;
    
    #[allow(unreachable_code)]
    Ok(Vec::new())
}

pub async fn toggle_startup_item(name: &str, enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    return windows::toggle_startup_item(name, enabled).await;
    
    #[cfg(target_os = "macos")]
    return Ok(false); // Placeholder
    
    #[cfg(target_os = "linux")]
    return linux::toggle_startup_item(name, enabled).await;
    
    #[allow(unreachable_code)]
    Ok(false)
}

/// Get platform-specific cleanup paths
pub fn get_cleanup_paths() -> Vec<std::path::PathBuf> {
    #[cfg(windows)]
    return windows::get_cleanup_paths();
    
    #[cfg(target_os = "macos")]
    return Vec::new(); // Placeholder
    
    #[cfg(target_os = "linux")]
    return Vec::new(); // Placeholder
    
    #[allow(unreachable_code)]
    Vec::new()
}
