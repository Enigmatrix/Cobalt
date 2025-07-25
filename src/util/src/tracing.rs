pub use tracing::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
pub use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt, registry};

use crate::config::Config;
use crate::error::*;
use crate::{TARGET, Target};

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
pub fn setup(config: &Config) -> Result<()> {
    let target = TARGET.lock().unwrap().clone();
    let (filter_directives, log_file) = match target {
        Target::Ui => (config.ui_log_filter(), "Cobalt.Ui".to_string()),
        Target::Engine => (config.engine_log_filter(), "Cobalt.Engine".to_string()),
        // TODO: what should the log filter directive be for this?
        Target::Tool { name, .. } => (config.engine_log_filter(), format!("Cobalt.Tool.{name}")),
    };

    let rolling = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(log_file)
        .filename_suffix("log")
        .max_log_files(50)
        .build(config.logs_dir()?)?;

    // Create a non-colored layer for file output
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(rolling)
        .with_filter(EnvFilter::new(filter_directives));

    #[cfg(debug_assertions)]
    {
        // Create a colored layer for stdout
        let stdout_layer = fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_writer(std::io::stdout)
            .with_filter(EnvFilter::new(filter_directives));

        registry()
            .with(stdout_layer)
            .with(file_layer)
            .try_init()
            .context("initialize tracing layers")?;
    }

    #[cfg(not(debug_assertions))]
    {
        registry()
            .with(file_layer)
            .try_init()
            .context("initialize tracing layers")?;
    }

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
