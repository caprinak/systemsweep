// Application state management

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::scanner::ScanProgress;
use super::undo::UndoManager;
use crate::utils::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub undo_manager: Arc<RwLock<UndoManager>>,
    pub active_scans: Arc<RwLock<HashMap<Uuid, ScanSession>>>,
    pub progress_tx: broadcast::Sender<ScanProgress>,
}

pub struct ScanSession {
    pub id: Uuid,
    pub status: ScanStatus,
    pub cancel_flag: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum ScanStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed(String),
}

impl AppState {
    pub fn new() -> Self {
        let (progress_tx, _) = broadcast::channel(100);
        
        Self {
            config: Arc::new(RwLock::new(AppConfig::load().unwrap_or_default())),
            undo_manager: Arc::new(RwLock::new(UndoManager::new())),
            active_scans: Arc::new(RwLock::new(HashMap::new())),
            progress_tx,
        }
    }
    
    pub fn create_scan_session(&self) -> Uuid {
        let id = Uuid::new_v4();
        let session = ScanSession {
            id,
            status: ScanStatus::Running,
            cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        
        self.active_scans.write().unwrap().insert(id, session);
        id
    }
    
    pub fn cancel_scan(&self, scan_id: Uuid) -> bool {
        if let Some(session) = self.active_scans.write().unwrap().get_mut(&scan_id) {
            session.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            session.status = ScanStatus::Cancelled;
            true
        } else {
            false
        }
    }
}
