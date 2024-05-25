pub use tracing::*;
pub use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

use crate::error::*;

pub trait ResultTraceExt<T> {
    fn error(self) -> T;
    fn warn(self) -> T;
    fn info(self) -> T;
    fn debug(self) -> T;
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

pub fn setup(filter_directives: &str) -> Result<()> {
    let filter = EnvFilter::new(filter_directives);

    registry()
        .with(fmt::layer().with_filter(filter)) // default fmt layer
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
