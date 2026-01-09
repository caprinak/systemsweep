// src-tauri/src/startup/windows.rs
#[cfg(target_os = "windows")]
use super::{StartupItem, StartupSource};
#[cfg(target_os = "windows")]
use crate::error::{CleanerError, Result};
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
pub async fn get_startup_items() -> Result<Vec<StartupItem>> {
    let mut items = Vec::new();

    // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
    {
        items.extend(scan_registry_key(&hkcu, StartupSource::Registry)?);
    }

    // HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Run
    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
    {
        items.extend(scan_registry_key(&hklm, StartupSource::System)?);
    }

    // Startup folder
    if let Some(startup) = dirs::data_local_dir()
        .map(|d| d.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"))
    {
        if startup.exists() {
            items.extend(scan_startup_folder(&startup)?);
        }
    }

    Ok(items)
}

#[cfg(target_os = "windows")]
fn scan_registry_key(key: &RegKey, source: StartupSource) -> Result<Vec<StartupItem>> {
    let mut items = Vec::new();

    for (name, value) in key.enum_values().filter_map(|v| v.ok()) {
        if let Ok(command) = value.to_string() {
            items.push(StartupItem {
                name,
                path: PathBuf::new(),
                command: Some(command),
                enabled: true,
                source: source.clone(),
                description: None,
            });
        }
    }

    Ok(items)
}

#[cfg(target_os = "windows")]
fn scan_startup_folder(folder: &PathBuf) -> Result<Vec<StartupItem>> {
    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(folder) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            items.push(StartupItem {
                name,
                path: path.clone(),
                command: Some(path.to_string_lossy().to_string()),
                enabled: true,
                source: StartupSource::User,
                description: None,
            });
        }
    }

    Ok(items)
}

#[cfg(target_os = "windows")]
pub async fn toggle_startup_item(name: &str, enabled: bool) -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    if enabled {
        // Re-enable by moving from RunDisabled to Run
        if let Ok(disabled_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run-disabled") {
            if let Ok(value) = disabled_key.get_value::<String, _>(name) {
                let run_key = hkcu.open_subkey_with_flags(
                    "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    KEY_SET_VALUE,
                )?;
                run_key.set_value(name, &value)?;
                disabled_key.delete_value(name).ok();
                return Ok(true);
            }
        }
    } else {
        // Disable by moving from Run to RunDisabled
        if let Ok(run_key) = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_READ | KEY_SET_VALUE,
        ) {
            if let Ok(value) = run_key.get_value::<String, _>(name) {
                let (disabled_key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run-disabled")?;
                disabled_key.set_value(name, &value)?;
                run_key.delete_value(name).ok();
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

#[cfg(target_os = "windows")]
pub async fn add_startup_item(
    name: &str,
    command: &str,
    _description: Option<&str>,
) -> Result<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE,
    )?;
    
    run_key.set_value(name, &command)?;
    
    Ok(PathBuf::from(format!("HKCU\\...\\Run\\{}", name)))
}

#[cfg(target_os = "windows")]
pub async fn remove_startup_item(name: &str) -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    if let Ok(run_key) = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE,
    ) {
        if run_key.delete_value(name).is_ok() {
            return Ok(true);
        }
    }
    
    Ok(false)
}
