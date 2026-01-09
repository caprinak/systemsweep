mod optimizer;

pub use optimizer::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub available: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub usage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
    pub memory_percentage: f32,
    pub cpu_percentage: f32,
    pub status: String,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub potential_savings: u64,
    pub action: RecommendedAction,
    pub process_ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    HighMemoryProcess,
    DuplicateProcess,
    InactiveProcess,
    BrowserTabs,
    BackgroundService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    None,
    Suggest,
    SafeToClose,
    RequiresConfirmation,
}
