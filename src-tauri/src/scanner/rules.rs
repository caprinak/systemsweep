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
            Self {
                id: "npm_cache".to_string(),
                name: "NPM Cache".to_string(),
                enabled: true,
                category: FileCategory::DeveloperCache,
                patterns: vec![],
                extensions: vec![],
                directories: vec![
                    "npm-cache".to_string(),
                    ".npm".to_string(),
                ],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear Node.js package manager cache".to_string(),
                risk_level: RiskLevel::Safe,
            },
            Self {
                id: "gradle_cache".to_string(),
                name: "Gradle Cache".to_string(),
                enabled: true,
                category: FileCategory::DeveloperCache,
                patterns: vec![],
                extensions: vec![],
                directories: vec![
                    ".gradle/caches".to_string(),
                ],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear Gradle build cache".to_string(),
                risk_level: RiskLevel::Safe,
            },
            Self {
                id: "maven_cache".to_string(),
                name: "Maven Cache".to_string(),
                enabled: true,
                category: FileCategory::DeveloperCache,
                patterns: vec![],
                extensions: vec![],
                directories: vec![
                    ".m2/repository".to_string(),
                ],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear Maven dependency cache".to_string(),
                risk_level: RiskLevel::Safe,
            },
            Self {
                id: "cargo_cache".to_string(),
                name: "Rust Cargo Cache".to_string(),
                enabled: true,
                category: FileCategory::DeveloperCache,
                patterns: vec![],
                extensions: vec![],
                directories: vec![
                    ".cargo/registry/cache".to_string(),
                ],
                min_age_days: None,
                min_size_bytes: None,
                max_size_bytes: None,
                description: "Clear Rust Cargo registry cache".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    fn get_temp_file_metadata(name: &str) -> (PathBuf, std::fs::Metadata) {
        let mut path = std::env::temp_dir();
        let unique_name = format!(
            "systemsweep_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique_name);
        let _file = File::create(&path).unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        (path, metadata)
    }

    fn cleanup_file(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_default_rules_contain_developer_caches() {
        let rules = CleanupRule::default_rules();
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"npm_cache"), "Default rules missing npm_cache");
        assert!(ids.contains(&"gradle_cache"), "Default rules missing gradle_cache");
        assert!(ids.contains(&"maven_cache"), "Default rules missing maven_cache");
        assert!(ids.contains(&"cargo_cache"), "Default rules missing cargo_cache");
    }

    #[test]
    fn test_categorize_by_extension() {
        let rules = CleanupRule::default_rules();
        let engine = RuleEngine::new(rules);
        
        let (path, metadata) = get_temp_file_metadata("test.tmp");
        let (category, risk, _) = engine.categorize(&path, &metadata);
        assert_eq!(category, FileCategory::Temporary);
        assert_eq!(risk, RiskLevel::Safe);
        cleanup_file(path);

        // A file without any matching rule extension
        let (path2, metadata2) = get_temp_file_metadata("test.unknown_ext");
        let (category2, _, _) = engine.categorize(&path2, &metadata2);
        assert_eq!(category2, FileCategory::Unknown);
        cleanup_file(path2);
    }

    #[test]
    fn test_categorize_by_directory() {
        let rules = CleanupRule::default_rules();
        let engine = RuleEngine::new(rules);
        
        let mut path = std::env::temp_dir();
        path.push("npm-cache");
        let _ = std::fs::create_dir_all(&path);
        
        path.push("test_pkg.tgz");
        let _file = File::create(&path).unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        
        let (category, risk, _) = engine.categorize(&path, &metadata);
        assert_eq!(category, FileCategory::DeveloperCache);
        assert_eq!(risk, RiskLevel::Safe);
        
        let _ = std::fs::remove_file(&path);
        let mut dir_path = path.clone();
        dir_path.pop();
        let _ = std::fs::remove_dir(dir_path);
    }
}
