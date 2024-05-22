#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(new_uninit)]

mod buf;
mod error;
pub mod events;
pub mod objects;

use util::error::Result;

pub fn setup() -> Result<()> {
    crate::objects::Timestamp::setup()?;
    // crate::objects::Window::setup()?;
    Ok(())
}
