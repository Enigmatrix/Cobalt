#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(sync_unsafe_cell)]

mod buf;
mod error;
/// Events generated by the Platform
pub mod events;
/// Objects in the Platform
pub mod objects;

use util::error::Result;

use crate::objects::Timestamp;

/// Setup platform for Windows
pub fn setup() -> Result<()> {
    Timestamp::setup()?;
    Ok(())
}
