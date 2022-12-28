pub use tracing::*;
use tracing_subscriber::prelude::*;
pub use tracing_subscriber::*;

use crate::errors::*;

pub fn setup() -> Result<()> {
    registry()
        .with(fmt::layer()) // default fmt layer
        .with(
            filter::EnvFilter::try_from_default_env()
                .or_else(|_| filter::EnvFilter::try_new("trace")) // TODO production should be info
                .context("initialize default info env filter")?,
        )
        .try_init()
        .context("initialize tracing layers")?;
    Ok(())
}
