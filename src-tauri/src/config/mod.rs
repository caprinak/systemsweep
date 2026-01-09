// src-tauri/src/config/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub auto_scan: bool,
    pub scan_hidden: bool,
    pub use_trash: bool,
    pub secure_delete_passes: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            language: "en".into(),
            auto_scan: false,
            scan_hidden: false,
            use_trash: true,
            secure_delete_passes: 3,
        }
    }
}
