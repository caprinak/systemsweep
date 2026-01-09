mod safe_delete;
mod undo;

pub use safe_delete::*;
pub use undo::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub success: bool,
    pub files_deleted: usize,
    pub bytes_freed: u64,
    pub errors: Vec<CleanupError>,
    pub undo_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupError {
    pub path: String,
    pub error: String,
}
