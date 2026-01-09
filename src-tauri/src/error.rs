// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("System error: {0}")]
    System(String),
    
    #[error("Scan error: {0}")]
    ScanError(String),
}

impl serde::Serialize for CleanerError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CleanerError>;
