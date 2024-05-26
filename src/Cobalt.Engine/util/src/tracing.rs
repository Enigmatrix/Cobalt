pub use tracing::*;
use tracing_appender::rolling::daily;
pub use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

use crate::error::*;

/// Extension trait for [Result] to log the error, warn, info, debug,
/// or trace straight to the log.
pub trait ResultTraceExt<T> {
    /// Log the error at error level and return the default value
    fn error(self) -> T;
    /// Log the error at warn level and return the default value
    fn warn(self) -> T;
    /// Log the error at info level and return the default value
    fn info(self) -> T;
    /// Log the error at debug level and return the default value
    fn debug(self) -> T;
    /// Log the error at trace level and return the default value
    fn trace(self) -> T;
}

impl<T: Default> ResultTraceExt<T> for Result<T> {
    fn error(self) -> T {
        self.unwrap_or_else(|report| {
            error!(?report);
            Default::default()
        })
    }
    fn warn(self) -> T {
        self.unwrap_or_else(|report| {
            warn!(?report);
            Default::default()
        })
    }
    fn info(self) -> T {
        self.unwrap_or_else(|report| {
            info!(?report);
            Default::default()
        })
    }
    fn debug(self) -> T {
        self.unwrap_or_else(|report| {
            debug!(?report);
            Default::default()
        })
    }
    fn trace(self) -> T {
        self.unwrap_or_else(|report| {
            trace!(?report);
            Default::default()
        })
    }
}

/// Setup the tracing layer with the given filter directives.
pub fn setup(filter_directives: &str) -> Result<()> {
    let filter = EnvFilter::new(filter_directives);
    let rolling = daily("logs", "Cobalt.Engine.log");

    #[cfg(debug_assertions)]
    let writers = || rolling.and(std::io::stdout);
    #[cfg(not(debug_assertions))]
    let writers = || rolling;

    registry()
        .with(fmt::layer().with_writer(writers()).with_filter(filter)) // default fmt layer
        .try_init()
        .context("initialize tracing layers")?;
    Ok(())
}

// #[test]
// fn warn_feature() -> Result<()> {
//     setup("warn")?;
//     Result::<()>::Err(anyhow!("this is a warning"))
//         .context("what")
//         .context("what2")
//         .warn();
//     Ok(())
// }
