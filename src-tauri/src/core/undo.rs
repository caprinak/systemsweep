// src-tauri/src/core/undo.rs

// Undo manager for reversible cleanup operations

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

const MAX_UNDO_HISTORY: usize = 100;

#[derive(Error, Debug)]
pub enum UndoError {
    #[error("Operation not found: {0}")]
    NotFound(String),
    
    #[error("Operation cannot be undone: {0}")]
    CannotUndo(String),
    
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
    
    #[error("Trash error: {0}")]
    TrashError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoOperation {
    pub id: String,
    pub operation_type: UndoOperationType,
    pub files: Vec<(PathBuf, u64)>,
    pub timestamp: DateTime<Utc>,
    pub can_undo: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UndoOperationType {
    MoveToTrash,
    Delete,
    SecureDelete,
    Move,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoHistory {
    pub operations: Vec<UndoOperationSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoOperationSummary {
    pub id: String,
    pub operation_type: String,
    pub file_count: usize,
    pub total_size: u64,
    pub timestamp: DateTime<Utc>,
    pub can_undo: bool,
}

pub struct UndoManager {
    operations: VecDeque<UndoOperation>,
    config_path: PathBuf,
}

impl UndoManager {
    pub fn new() -> Self {
        let config_path = directories::ProjectDirs::from("com", "systemsweep", "SystemSweep")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("undo_history.json");
        
        let mut manager = Self {
            operations: VecDeque::new(),
            config_path,
        };
        
        // Try to load existing history
        if let Ok(data) = std::fs::read_to_string(&manager.config_path) {
            if let Ok(ops) = serde_json::from_str::<Vec<UndoOperation>>(&data) {
                manager.operations = ops.into_iter().collect();
            }
        }
        
        manager
    }
    
    pub fn record_operation(&mut self, operation: UndoOperation) {
        info!(
            "Recording undo operation {}: {} files",
            operation.id,
            operation.files.len()
        );
        
        self.operations.push_front(operation);
        
        // Trim history
        while self.operations.len() > MAX_UNDO_HISTORY {
            self.operations.pop_back();
        }
        
        // Persist to disk
        self.save_history();
    }
    
    pub fn get_history(&self) -> UndoHistory {
        let summaries = self.operations.iter()
            .map(|op| UndoOperationSummary {
                id: op.id.clone(),
                operation_type: format!("{:?}", op.operation_type),
                file_count: op.files.len(),
                total_size: op.files.iter().map(|(_, s)| s).sum(),
                timestamp: op.timestamp,
                can_undo: op.can_undo,
            })
            .collect();
        
        UndoHistory { operations: summaries }
    }
    
    pub fn undo_operation(&mut self, operation_id: &str) -> Result<UndoResult, UndoError> {
        let operation = self.operations.iter()
            .find(|op| op.id == operation_id)
            .cloned()
            .ok_or_else(|| UndoError::NotFound(operation_id.to_string()))?;
        
        if !operation.can_undo {
            return Err(UndoError::CannotUndo(
                "Operation was performed with permanent deletion".to_string()
            ));
        }
        
        info!("Undoing operation {}: {} files", operation_id, operation.files.len());
        
        let mut restored = 0u64;
        let mut failed = 0u64;
        let mut errors = Vec::new();
        
        for (path, _size) in &operation.files {
            match self.restore_file(path) {
                Ok(_) => {
                    restored += 1;
                    info!("Restored: {:?}", path);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{:?}: {}", path, e));
                    warn!("Failed to restore {:?}: {}", path, e);
                }
            }
        }
        
        // Remove from history if successful
        if failed == 0 {
            self.operations.retain(|op| op.id != operation_id);
            self.save_history();
        }
        
        Ok(UndoResult {
            operation_id: operation_id.to_string(),
            files_restored: restored,
            files_failed: failed,
            errors,
        })
    }
    
    pub fn undo_last(&mut self) -> Result<UndoResult, UndoError> {
        let last_undoable = self.operations.iter()
            .find(|op| op.can_undo)
            .map(|op| op.id.clone())
            .ok_or_else(|| UndoError::NotFound("No undoable operations".to_string()))?;
        
        self.undo_operation(&last_undoable)
    }
    
    fn restore_file(&self, path: &PathBuf) -> Result<(), UndoError> {
        // Use trash crate's restore functionality
        // Note: This is platform-specific and may not work on all systems
        
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            // On Windows and macOS, trash crate may support restore
            // For now, we'll indicate it's not always possible
            warn!("Automatic restore not fully implemented for {:?}", path);
            return Err(UndoError::RestoreFailed(
                "Manual restore from trash/recycle bin required".to_string()
            ));
        }
        
        #[cfg(target_os = "linux")]
        {
            // On Linux with freedesktop trash, we might be able to restore
            // This is a simplified implementation
            warn!("Automatic restore not fully implemented for {:?}", path);
            return Err(UndoError::RestoreFailed(
                "Manual restore from trash required".to_string()
            ));
        }
        
        #[allow(unreachable_code)]
        Ok(())
    }
    
    fn save_history(&self) {
        if let Some(parent) = self.config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let ops: Vec<_> = self.operations.iter().cloned().collect();
        if let Ok(json) = serde_json::to_string_pretty(&ops) {
            let _ = std::fs::write(&self.config_path, json);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoResult {
    pub operation_id: String,
    pub files_restored: u64,
    pub files_failed: u64,
    pub errors: Vec<String>,
}
