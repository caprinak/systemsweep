// src-tauri/src/core/rules.rs

// Cleanup rules engine with configurable heuristics

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::scanner::FileCategory;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Safe,      // Temp files, caches
    Low,       // Old logs, thumbnails
    Medium,    // Downloads, old files
    High,      // System files (requires confirmation)
}

impl CleanupRule {
    pub fn default_rules() -> Vec<Self> {
        vec![
            // Temporary files
            Self {
                id: "temp_files".to_string(),
                name: "Temporary Files".to_string(),
                enabled: true,
                category: FileCategory::TempFile,
                patterns: vec!["*.tmp".to_string(), "*.temp".to_string(), "~*".to_string()],
                extensions: vec!["tmp".to_string(), "temp".to_string(), "bak".to_string()],
                directories: vec![],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Remove temporary files created by applications".to_string(),
                risk_level: RiskLevel::Safe,
            },
            
            // System temp directories
            Self {
                id: "system_temp".to_string(),
                name: "System Temp Folders".to_string(),
                enabled: true,
                category: FileCategory::TempFile,
                patterns: vec![],
                extensions: vec![],
                directories: Self::get_temp_directories(),
                min_age_days: Some(1),
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clean system temporary directories".to_string(),
                risk_level: RiskLevel::Safe,
            },
            
            // Cache files
            Self {
                id: "cache_files".to_string(),
                name: "Cache Files".to_string(),
                enabled: true,
                category: FileCategory::CacheFile,
                patterns: vec!["*.cache".to_string()],
                extensions: vec!["cache".to_string()],
                directories: Self::get_cache_directories(),
                min_age_days: Some(7),
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear application caches".to_string(),
                risk_level: RiskLevel::Safe,
            },
            
            // Log files
            Self {
                id: "log_files".to_string(),
                name: "Log Files".to_string(),
                enabled: true,
                category: FileCategory::LogFile,
                patterns: vec!["*.log".to_string(), "*.log.*".to_string()],
                extensions: vec!["log".to_string()],
                directories: vec![],
                min_age_days: Some(30),
                min_size_bytes: Some(1024 * 1024), // 1MB
                max_size_bytes: None,
                description: "Remove old log files".to_string(),
                risk_level: RiskLevel::Low,
            },
            
            // Browser cache
            Self {
                id: "browser_cache".to_string(),
                name: "Browser Cache".to_string(),
                enabled: true,
                category: FileCategory::BrowserCache,
                patterns: vec![],
                extensions: vec![],
                directories: Self::get_browser_cache_directories(),
                min_age_days: Some(7),
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear browser cached data".to_string(),
                risk_level: RiskLevel::Safe,
            },
            
            // Thumbnails
            Self {
                id: "thumbnails".to_string(),
                name: "Thumbnail Cache".to_string(),
                enabled: true,
                category: FileCategory::Thumbnail,
                patterns: vec!["thumbs.db".to_string(), "Thumbs.db".to_string()],
                extensions: vec![],
                directories: Self::get_thumbnail_directories(),
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Remove thumbnail cache files".to_string(),
                risk_level: RiskLevel::Safe,
            },
            
            // Large files
            Self {
                id: "large_files".to_string(),
                name: "Large Files".to_string(),
                enabled: false, // Disabled by default
                category: FileCategory::LargeFile,
                patterns: vec![],
                extensions: vec![],
                directories: vec![],
                min_age_days: Some(90),
                min_size_bytes: Some(100 * 1024 * 1024), // 100MB
                max_size_bytes: None,
                description: "Find large files that haven't been accessed".to_string(),
                risk_level: RiskLevel::Medium,
            },
            
            // Old downloads
            Self {
                id: "old_downloads".to_string(),
                name: "Old Downloads".to_string(),
                enabled: false, // Disabled by default
                category: FileCategory::UserDownloads,
                patterns: vec![],
                extensions: vec![],
                directories: Self::get_download_directories(),
                min_age_days: Some(90),
                min_size_bytes: Some(10 * 1024 * 1024), // 10MB
                max_size_bytes: None,
                description: "Find old files in download folders".to_string(),
                risk_level: RiskLevel::Medium,
            },
        ]
    }
    
    fn get_temp_directories() -> Vec<String> {
        let mut dirs = Vec::new();
        
        if let Ok(temp) = std::env::var("TEMP") {
            dirs.push(temp);
        }
        if let Ok(tmp) = std::env::var("TMP") {
            dirs.push(tmp);
        }
        
        #[cfg(unix)]
        {
            dirs.push("/tmp".to_string());
            dirs.push("/var/tmp".to_string());
        }
        
        dirs
    }
    
    fn get_cache_directories() -> Vec<String> {
        let mut dirs = Vec::new();
        
        if let Some(home) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
            #[cfg(windows)]
            {
                dirs.push(home.join("AppData").join("Local").join("Temp").to_string_lossy().to_string());
            }
            
            #[cfg(target_os = "macos")]
            {
                dirs.push(home.join("Library").join("Caches").to_string_lossy().to_string());
            }
            
            #[cfg(target_os = "linux")]
            {
                dirs.push(home.join(".cache").to_string_lossy().to_string());
            }
        }
        
        dirs
    }
    
    fn get_browser_cache_directories() -> Vec<String> {
        let mut dirs = Vec::new();
        
        if let Some(home) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
            #[cfg(windows)]
            {
                let local = home.join("AppData").join("Local");
                dirs.push(local.join("Google").join("Chrome").join("User Data").join("Default").join("Cache").to_string_lossy().to_string());
                dirs.push(local.join("Microsoft").join("Edge").join("User Data").join("Default").join("Cache").to_string_lossy().to_string());
                dirs.push(local.join("Mozilla").join("Firefox").join("Profiles").to_string_lossy().to_string());
            }
            
            #[cfg(target_os = "macos")]
            {
                let caches = home.join("Library").join("Caches");
                dirs.push(caches.join("Google").join("Chrome").to_string_lossy().to_string());
                dirs.push(caches.join("com.apple.Safari").to_string_lossy().to_string());
                dirs.push(caches.join("Firefox").to_string_lossy().to_string());
            }
            
            #[cfg(target_os = "linux")]
            {
                let cache = home.join(".cache");
                dirs.push(cache.join("google-chrome").to_string_lossy().to_string());
                dirs.push(cache.join("chromium").to_string_lossy().to_string());
                dirs.push(cache.join("mozilla").to_string_lossy().to_string());
            }
        }
        
        dirs
    }
    
    fn get_thumbnail_directories() -> Vec<String> {
        let mut dirs = Vec::new();
        
        if let Some(home) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
            #[cfg(windows)]
            {
                dirs.push(home.join("AppData").join("Local").join("Microsoft").join("Windows").join("Explorer").to_string_lossy().to_string());
            }
            
            #[cfg(target_os = "linux")]
            {
                dirs.push(home.join(".cache").join("thumbnails").to_string_lossy().to_string());
            }
        }
        
        dirs
    }
    
    fn get_download_directories() -> Vec<String> {
        let mut dirs = Vec::new();
        
        if let Some(download) = directories::UserDirs::new().and_then(|u| Some(u.download_dir().map(|d| d.to_path_buf()).unwrap_or_default())) {
            if download.exists() {
                dirs.push(download.to_string_lossy().to_string());
            }
        }
        
        dirs
    }
}

pub struct RuleEngine {
    rules: Vec<CleanupRule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<CleanupRule>) -> Self {
        Self { rules }
    }
    
    pub fn categorize_file(
        &self,
        path: &Path,
        metadata: &std::fs::Metadata
    ) -> (FileCategory, Option<String>) {
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            
            if self.matches_rule(path, metadata, rule) {
                return (rule.category.clone(), Some(rule.description.clone()));
            }
        }
        
        (FileCategory::Unknown, None)
    }
    
    fn matches_rule(&self, path: &Path, metadata: &std::fs::Metadata, rule: &CleanupRule) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        
        // Check extension match
        if !rule.extensions.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if rule.extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
                    return self.check_age_and_size(metadata, rule);
                }
            }
        }
        
        // Check pattern match
        for pattern in &rule.patterns {
            if self.matches_pattern(&file_name, pattern) {
                return self.check_age_and_size(metadata, rule);
            }
        }
        
        // Check directory match
        for dir in &rule.directories {
            if path_str.contains(&dir.to_lowercase()) {
                return self.check_age_and_size(metadata, rule);
            }
        }
        
        false
    }
    
    fn matches_pattern(&self, name: &str, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        
        if pattern_lower.starts_with('*') && pattern_lower.ends_with('*') {
            let middle = &pattern_lower[1..pattern_lower.len()-1];
            name.contains(middle)
        } else if pattern_lower.starts_with('*') {
            name.ends_with(&pattern_lower[1..])
        } else if pattern_lower.ends_with('*') {
            name.starts_with(&pattern_lower[..pattern_lower.len()-1])
        } else {
            name == pattern_lower
        }
    }
    
    fn check_age_and_size(&self, metadata: &std::fs::Metadata, rule: &CleanupRule) -> bool {
        let size = metadata.len();
        
        // Check minimum size
        if let Some(min_size) = rule.min_size_bytes {
            if size < min_size {
                return false;
            }
        }
        
        // Check maximum size
        if let Some(max_size) = rule.max_size_bytes {
            if size > max_size {
                return false;
            }
        }
        
        // Check age
        if let Some(min_age_days) = rule.min_age_days {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::ZERO);
                
                if age.as_secs() < (min_age_days as u64 * 24 * 60 * 60) {
                    return false;
                }
            }
        }
        
        true
    }
}
