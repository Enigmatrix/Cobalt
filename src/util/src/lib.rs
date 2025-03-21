//! Common utils shared by all crates

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

use std::sync::Once;

use crate::config::Config;
use crate::error::Result;

/// Configuration Target
pub enum Target {
    /// UI module target
    Ui,
    /// Engine module target
    Engine,
}

static INIT: Once = Once::new();

/// Setup all utils. This function will only run once, all other invocations
/// will be ignored (vacous success).
pub fn setup(config: &Config, target: Target) -> Result<()> {
    let mut result = Ok(());
    INIT.call_once(|| {
        fn inner_setup(config: &Config, target: Target) -> Result<()> {
            error::setup()?;
            tracing::setup(config, target)?;
            Ok(())
        }
        result = inner_setup(config, target);
    });
    result
}
