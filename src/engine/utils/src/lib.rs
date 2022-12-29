pub mod channels;
pub mod errors;
pub mod futures;
pub mod tracing;

use errors::*;

pub fn setup() -> Result<()> {
    errors::setup().context("setup errors")?;
    tracing::setup().context("setup tracing")?;
    Ok(())
}
