// src-tauri/src/scanner/cache_scanner.rs
use super::*;
use crate::error::Result;
use crate::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLocation {
    pub name: String,
    pub path: PathBuf,
    pub category: CacheCategory,
    pub safe_to_clean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheCategory {
    Browser,
    System,
    Application,
    Package,
    Thumbnail,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheScanResult {
    pub locations: Vec<CacheLocationResult>,
    pub total_size: u64,
    pub total_files: u64,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLocationResult {
    pub location: CacheLocation,
    pub size: u64,
    pub file_count: u64,
    pub files: Vec<ScannedFile>,
}

pub struct CacheScanner;

impl CacheScanner {
    pub fn get_cache_locations() -> Vec<CacheLocation> {
        let mut locations = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                locations.push(CacheLocation {
                    name: "Chrome Cache".to_string(),
                    path: home.join(".cache/google-chrome"),
                    category: CacheCategory::Browser,
                    safe_to_clean: true,
                });
                locations.push(CacheLocation {
                    name: "User Cache".to_string(),
                    path: home.join(".cache"),
                    category: CacheCategory::System,
                    safe_to_clean: false,
                });
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(local) = dirs::data_local_dir() {
                locations.push(CacheLocation {
                    name: "Windows Temp".to_string(),
                    path: local.join("Temp"),
                    category: CacheCategory::System,
                    safe_to_clean: true,
                });
                locations.push(CacheLocation {
                    name: "Chrome Cache".to_string(),
                    path: local.join("Google/Chrome/User Data/Default/Cache"),
                    category: CacheCategory::Browser,
                    safe_to_clean: true,
                });
            }
            
            locations.push(CacheLocation {
                name: "Windows Temp (System)".to_string(),
                path: PathBuf::from("C:\\Windows\\Temp"),
                category: CacheCategory::System,
                safe_to_clean: true,
            });
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                locations.push(CacheLocation {
                    name: "User Caches".to_string(),
                    path: home.join("Library/Caches"),
                    category: CacheCategory::System,
                    safe_to_clean: false,
                });
            }
        }

        locations.into_iter().filter(|l| l.path.exists()).collect()
    }

    pub fn scan(state: Option<Arc<AppState>>) -> Result<CacheScanResult> {
        let start = std::time::Instant::now();
        let locations = Self::get_cache_locations();
        let mut results = Vec::new();
        let mut total_size = 0u64;
        let mut total_files = 0u64;

        for location in locations {
            if let Some(ref s) = state {
                s.update_progress(crate::state::ScanProgress {
                    current_path: format!("Scanning: {}", location.name),
                    files_scanned: 0,
                    bytes_scanned: 0,
                    files_found: total_files,
                    bytes_found: total_size,
                    phase: "cache_scan".to_string(),
                    percentage: 0.0,
                });
            }

            let scanner = FileScanner::new(ScanOptions::default());
            if let Ok(scan_result) = scanner.scan(&[location.path.clone()], None) {
                total_size += scan_result.total_size;
                total_files += scan_result.total_count;
                
                results.push(CacheLocationResult {
                    location,
                    size: scan_result.total_size,
                    file_count: scan_result.files.len() as u64,
                    files: scan_result.files,
                });
            }
        }

        Ok(CacheScanResult {
            locations: results,
            total_size,
            total_files,
            scan_duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
