// src-tauri/src/startup/linux.rs
use super::{StartupItem, StartupSource};
use crate::error::{CleanerError, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub async fn get_startup_items() -> Result<Vec<StartupItem>> {
    let mut items = Vec::new();

    // User autostart directory
    if let Some(config) = dirs::config_dir() {
        let autostart = config.join("autostart");
        if autostart.exists() {
            items.extend(scan_autostart_dir(&autostart, StartupSource::User)?);
        }
    }

    // System autostart directories
    let system_dirs = [
        "/etc/xdg/autostart",
        "/usr/share/gnome/autostart",
        "/usr/share/autostart",
    ];

    for dir in &system_dirs {
        let path = PathBuf::from(dir);
        if path.exists() {
            items.extend(scan_autostart_dir(&path, StartupSource::System)?);
        }
    }

    Ok(items)
}

fn scan_autostart_dir(dir: &PathBuf, source: StartupSource) -> Result<Vec<StartupItem>> {
    let mut items = Vec::new();

    let entries = fs::read_dir(dir).map_err(CleanerError::Io)?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "desktop").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(item) = parse_desktop_file(&path, &content, source.clone()) {
                    items.push(item);
                }
            }
        }
    }

    Ok(items)
}

fn parse_desktop_file(path: &PathBuf, content: &str, source: StartupSource) -> Option<StartupItem> {
    let mut name = None;
    let mut exec = None;
    let mut comment = None;
    let mut hidden = false;

    for line in content.lines() {
        if line.starts_with("Name=") {
            name = Some(line[5..].to_string());
        } else if line.starts_with("Exec=") {
            exec = Some(line[5..].to_string());
        } else if line.starts_with("Comment=") {
            comment = Some(line[8..].to_string());
        } else if line.starts_with("Hidden=") {
            hidden = line[7..].trim().eq_ignore_ascii_case("true");
        } else if line.starts_with("X-GNOME-Autostart-enabled=") {
            let enabled = line[26..].trim().eq_ignore_ascii_case("true");
            if !enabled {
                hidden = true;
            }
        }
    }

    name.map(|n| StartupItem {
        name: n,
        path: path.clone(),
        command: exec,
        enabled: !hidden,
        source,
        description: comment,
    })
}

pub async fn toggle_startup_item(name: &str, enabled: bool) -> Result<bool> {
    if let Some(config) = dirs::config_dir() {
        let autostart = config.join("autostart");
        
        if let Ok(entries) = fs::read_dir(&autostart) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(item_name) = parse_desktop_name(&content) {
                        if item_name == name {
                            let new_content = modify_desktop_enabled(&content, enabled);
                            fs::write(&path, new_content).map_err(CleanerError::Io)?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

fn parse_desktop_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Name=") {
            return Some(line[5..].to_string());
        }
    }
    None
}

fn modify_desktop_enabled(content: &str, enabled: bool) -> String {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        if line.starts_with("Hidden=") {
            *line = format!("Hidden={}", !enabled);
            found = true;
            break;
        }
        if line.starts_with("X-GNOME-Autostart-enabled=") {
            *line = format!("X-GNOME-Autostart-enabled={}", enabled);
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("Hidden={}", !enabled));
    }

    lines.join("\n")
}

pub async fn add_startup_item(
    name: &str,
    command: &str,
    description: Option<&str>,
) -> Result<PathBuf> {
    let config = dirs::config_dir().ok_or_else(|| CleanerError::System("No config dir".into()))?;
    let autostart = config.join("autostart");
    fs::create_dir_all(&autostart)?;

    let filename = format!("{}.desktop", name.to_lowercase().replace(' ', "-"));
    let path = autostart.join(&filename);

    let content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nHidden=false\nX-GNOME-Autostart-enabled=true{}",
        name,
        command,
        description.map(|d| format!("\nComment={}", d)).unwrap_or_default()
    );

    fs::write(&path, content)?;
    Ok(path)
}

pub async fn remove_startup_item(name: &str) -> Result<bool> {
    if let Some(config) = dirs::config_dir() {
        let autostart = config.join("autostart");
        
        if let Ok(entries) = fs::read_dir(&autostart) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(item_name) = parse_desktop_name(&content) {
                        if item_name == name {
                            fs::remove_file(&path).map_err(CleanerError::Io)?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}
