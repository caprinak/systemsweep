#![cfg(target_os = "macos")]

use super::*;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct MacOSStartupManager;

impl MacOSStartupManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_items(&self) -> Result<Vec<StartupItem>> {
        let mut items = Vec::new();

        // User LaunchAgents
        if let Some(home) = dirs::home_dir() {
            let user_agents = home.join("Library/LaunchAgents");
            items.extend(self.scan_launch_items(&user_agents, StartupSource::LaunchAgent).await?);
        }

        // System LaunchAgents
        let system_agents = PathBuf::from("/Library/LaunchAgents");
        items.extend(self.scan_launch_items(&system_agents, StartupSource::LaunchAgent).await?);

        // Login Items (placeholder)
        items.extend(self.get_login_items().await?);

        Ok(items)
    }

    async fn scan_launch_items(&self, dir: &PathBuf, source: StartupSource) -> Result<Vec<StartupItem>> {
        let mut items = Vec::new();

        if !dir.exists() {
            return Ok(items);
        }

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "plist").unwrap_or(false) {
                if let Ok(plist_data) = plist::from_file::<_, plist::Dictionary>(&path) {
                    let label = plist_data.get("Label")
                        .and_then(|v| v.as_string())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default());

                    let disabled = plist_data.get("Disabled")
                        .and_then(|v| v.as_boolean())
                        .unwrap_or(false);

                    let program = plist_data.get("Program")
                        .and_then(|v| v.as_string())
                        .map(|s| s.to_string());

                    items.push(StartupItem {
                        name: label,
                        path: path.to_string_lossy().to_string(),
                        enabled: !disabled,
                        source: source.clone(),
                        command: program,
                        description: None,
                    });
                }
            }
        }

        Ok(items)
    }

    async fn get_login_items(&self) -> Result<Vec<StartupItem>> {
        // Full implementation would use AppleScript or ServiceManagement
        Ok(Vec::new())
    }

    pub async fn toggle_item(&self, name: &str, enabled: bool) -> Result<bool> {
        let status = if enabled { "enable" } else { "disable" };
        
        let output = tokio::process::Command::new("launchctl")
            .args([status, &format!("gui/{}/{}", users::get_current_uid(), name)])
            .output()
            .await?;

        Ok(output.status.success())
    }

    pub async fn remove_item(&self, name: &str) -> Result<bool> {
        if let Some(home) = dirs::home_dir() {
            let user_agents = home.join("Library/LaunchAgents");
            let plist_path = user_agents.join(format!("{}.plist", name));
            
            if plist_path.exists() {
                let _ = tokio::process::Command::new("launchctl")
                    .args(["unload", &plist_path.to_string_lossy()])
                    .output()
                    .await;

                tokio::fs::remove_file(&plist_path).await?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}
