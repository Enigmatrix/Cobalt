#![feature(backtrace)]

pub mod channel;
pub mod error;
pub mod futures;
pub mod log;

pub use futures as tokio;
pub use log as tracing;

pub use crate::error::*;
pub use log::Instrument;

pub fn setup() -> Result<()> {
    log::setup_log()?;
    Ok(())
}
