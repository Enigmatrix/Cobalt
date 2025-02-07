pub mod channels;
pub mod config;
pub mod error;
pub mod future;
pub mod time;
pub mod tracing;

use crate::config::Config;
use crate::error::Result;

/// Setup all utils
pub fn setup(config: &Config) -> Result<()> {
    error::setup()?;
    // TODO: configure this for UI as well
    tracing::setup(config.engine_log_filter())?;
    Ok(())
}
