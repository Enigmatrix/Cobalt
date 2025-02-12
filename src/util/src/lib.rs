/// Channel base
pub mod channels;
/// Configuration module
pub mod config;
/// Error base
pub mod error;
/// Async base
pub mod future;
///  Common timesystem traits
pub mod time;
/// Logging, tracing helpers
pub mod tracing;

use crate::config::Config;
use crate::error::Result;

/// Configuration Target
pub enum Target {
    /// UI module target
    Ui,
    /// Engine module target
    Engine,
}

/// Setup all utils
pub fn setup(config: &Config, target: Target) -> Result<()> {
    error::setup()?;
    tracing::setup(config, target)?;
    Ok(())
}
