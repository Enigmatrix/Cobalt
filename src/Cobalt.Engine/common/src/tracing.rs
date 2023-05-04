pub use tracing::*;
pub use tracing_subscriber::prelude::*;

use tracing_subscriber::{fmt, registry, EnvFilter};

use crate::errors::*;

pub fn setup(filter_directives: &str) -> Result<()> {
    let filter = EnvFilter::new(filter_directives);

    registry()
        .with(fmt::layer().with_filter(filter)) // default fmt layer
        .try_init()
        .context("initialize tracing layers")?;
    Ok(())
}
