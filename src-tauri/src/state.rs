// src-tauri/src/state.rs
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: String,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub files_found: u64,
    pub bytes_found: u64,
    pub phase: String,
    pub percentage: f32,
}

pub struct AppState {
    pub db_path: PathBuf,
    pub scan_cancelled: AtomicBool,
    pub scan_progress: RwLock<ScanProgress>,
    pub progress_sender: broadcast::Sender<ScanProgress>,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            db_path,
            scan_cancelled: AtomicBool::new(false),
            scan_progress: RwLock::new(ScanProgress {
                current_path: String::new(),
                files_scanned: 0,
                bytes_scanned: 0,
                files_found: 0,
                bytes_found: 0,
                phase: "idle".to_string(),
                percentage: 0.0,
            }),
            progress_sender: tx,
        }
    }

    pub fn cancel_scan(&self) {
        self.scan_cancelled.store(true, Ordering::SeqCst);
    }

    pub fn reset_scan(&self) {
        self.scan_cancelled.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.scan_cancelled.load(Ordering::SeqCst)
    }

    pub fn update_progress(&self, progress: ScanProgress) {
        if let Ok(mut p) = self.scan_progress.write() {
            *p = progress.clone();
        }
        let _ = self.progress_sender.send(progress);
    }

    pub fn get_progress(&self) -> ScanProgress {
        self.scan_progress.read().unwrap().clone()
    }
}
