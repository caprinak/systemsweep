// src-tauri/src/scanner/rules.rs
use super::*;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanupRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub category: FileCategory,
    pub patterns: Vec<String>,
    pub extensions: Vec<String>,
    pub directories: Vec<String>,
    pub min_age_days: Option<u32>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    pub description: String,
    pub risk_level: RiskLevel,
}

impl CleanupRule {
    pub fn default_rules() -> Vec<Self> {
        let mut rules = vec![
            Self {
                id: "temp_files".to_string(),
                name: "Temporary Files".to_string(),
                enabled: true,
                category: FileCategory::Temporary,
                patterns: vec!["*.tmp".to_string(), "*.temp".to_string(), "~*".to_string()],
                extensions: vec!["tmp".to_string(), "temp".to_string(), "bak".to_string()],
                directories: vec![],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Remove temporary files created by applications".to_string(),
                risk_level: RiskLevel::Safe,
            },
            Self {
                id: "log_files".to_string(),
                name: "Log Files".to_string(),
                enabled: true,
                category: FileCategory::Log,
                patterns: vec!["*.log".to_string(), "*.log.*".to_string()],
                extensions: vec!["log".to_string()],
                directories: vec![],
                min_age_days: Some(30),
                min_size_bytes: Some(1024 * 1024),
                max_size_bytes: None,
                description: "Remove old log files".to_string(),
                risk_level: RiskLevel::Low,
            },
            Self {
                id: "browser_cache".to_string(),
                name: "Browser Cache".to_string(),
                enabled: true,
                category: FileCategory::BrowserCache,
                patterns: vec![],
                extensions: vec![],
                directories: vec![], // Will be populated dynamically
                min_age_days: Some(7),
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear browser cached data".to_string(),
                risk_level: RiskLevel::Safe,
            },
            Self {
                id: "thumbnails".to_string(),
                name: "Thumbnail Cache".to_string(),
                enabled: true,
                category: FileCategory::Thumbnail,
                patterns: vec!["thumbs.db".to_string(), "Thumbs.db".to_string()],
                extensions: vec![],
                directories: vec![],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Remove thumbnail cache files".to_string(),
                risk_level: RiskLevel::Safe,
            },
        ];

        // Add platform specific directories to rules
        #[cfg(windows)]
        {
             if let Ok(temp) = std::env::var("TEMP") {
                rules.push(Self {
                    id: "windows_temp".to_string(),
                    name: "Windows Temp".to_string(),
                    enabled: true,
                    category: FileCategory::Temporary,
                    patterns: vec![],
                    extensions: vec![],
                    directories: vec![temp],
                    min_age_days: Some(1),
                    min_size_bytes: None,
                    max_size_bytes: None,
                    description: "Clean Windows temporary directory".to_string(),
                    risk_level: RiskLevel::Safe,
                });
             }
        }

        rules
    }
}

pub struct RuleEngine {
    rules: Vec<CleanupRule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<CleanupRule>) -> Self {
        Self { rules }
    }

    pub fn categorize(&self, path: &Path, metadata: &std::fs::Metadata) -> (FileCategory, RiskLevel, Option<String>) {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for rule in &self.rules {
            if !rule.enabled { continue; }

            let mut matched = false;
            
            // Match extension
            if !rule.extensions.is_empty() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if rule.extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        matched = true;
                    }
                }
            }

            // Match directory
            if !matched && !rule.directories.is_empty() {
                if rule.directories.iter().any(|d| path_str.contains(&d.to_lowercase())) {
                    matched = true;
                }
            }

            // Match pattern
            if !matched && !rule.patterns.is_empty() {
                for pattern in &rule.patterns {
                    if file_name.contains(&pattern.replace("*", "")) {
                        matched = true;
                        break;
                    }
                }
            }

            if matched {
                // Check age
                if let Some(days) = rule.min_age_days {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed.as_secs() < (days as u64 * 86400) {
                                continue;
                            }
                        }
                    }
                }
                return (rule.category.clone(), rule.risk_level.clone(), Some(rule.description.clone()));
            }
        }

        (FileCategory::Unknown, RiskLevel::Low, None)
    }
}
