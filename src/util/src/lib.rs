pub mod channels;
pub mod config;
pub mod error;
pub mod future;
pub mod time;
pub mod tracing;

use crate::config::Config;
use crate::error::Result;

pub enum Target {
    Ui,
    Engine,
}

/// Setup all utils
pub fn setup(config: &Config, target: Target) -> Result<()> {
    error::setup()?;
    tracing::setup(config, target)?;
    Ok(())
}
