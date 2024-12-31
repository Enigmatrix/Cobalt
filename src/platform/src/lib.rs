#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(sync_unsafe_cell)]

mod buf;
mod error;
pub mod events;
pub mod objects;

use util::error::Result;

use crate::objects::Timestamp;

/// Setup platform for Windows
pub fn setup() -> Result<()> {
    Timestamp::setup()?;
    Ok(())
}
