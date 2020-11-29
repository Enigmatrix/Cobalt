pub mod error;
pub mod futures;
pub mod log;
pub use log as tracing;

pub use crate::error::*;

pub fn setup() -> Result<()> {
    log::setup_log()?;
    Ok(())
}
