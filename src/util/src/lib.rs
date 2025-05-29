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

use std::sync::{LazyLock, Mutex, Once};

use crate::config::Config;
use crate::error::Result;

/// Configuration Target
#[derive(Debug, Clone, Default)]
pub enum Target {
    /// UI module target
    Ui,
    /// Engine module target
    #[default]
    Engine,
}

static INIT: Once = Once::new();
static TARGET: LazyLock<Mutex<Target>> = LazyLock::new(|| Mutex::new(Target::default()));

/// Set the target for this application
pub fn set_target(target: Target) {
    let mut t = TARGET.lock().unwrap();
    *t = target;
}

/// Setup all utils. This function will only run once, all other invocations
/// will be ignored (vacous success).
pub fn setup(config: &Config) -> Result<()> {
    let mut result = Ok(());
    INIT.call_once(|| {
        fn inner_setup(config: &Config) -> Result<()> {
            error::setup()?;
            tracing::setup(config)?;
            Ok(())
        }
        result = inner_setup(config);
    });
    result
}
