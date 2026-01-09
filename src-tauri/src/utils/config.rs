// src-tauri/src/utils/config.rs

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::core::rules::CleanupRule;
use anyhow::Result;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub theme: String,
    pub cleanup_rules: Vec<CleanupRule>,
    pub scan_paths: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "dark".to_string(),
            cleanup_rules: CleanupRule::default_rules(),
            scan_paths: vec![],
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            let data = std::fs::read_to_string(config_path)?;
            let config: AppConfig = toml::from_str(&data)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)?;
        std::fs::write(config_path, data)?;
        Ok(())
    }
    
    fn get_config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "systemsweep", "SystemSweep")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("config.toml")
    }
}
