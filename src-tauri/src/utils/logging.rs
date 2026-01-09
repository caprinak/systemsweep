// src-tauri/src/utils/logging.rs

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn setup_logging() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
