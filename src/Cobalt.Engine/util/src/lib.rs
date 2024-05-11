pub mod config;
pub mod error;
pub mod tracing;

use crate::config::Config;
use crate::error::Result;

pub fn setup(config: &Config) -> Result<()> {
    error::setup()?;
    tracing::setup(&config.engine_log_filter())?;
    Ok(())
}
