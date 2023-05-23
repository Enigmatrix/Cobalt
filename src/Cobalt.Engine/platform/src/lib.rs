#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_uninit_array)]
#![feature(let_chains)]

pub mod buffers;
pub mod errors;
pub mod objects;
pub mod watchers;

pub use buffers::*;
pub use errors::*;

use common::errors::*;

pub fn setup() -> Result<()> {
    crate::objects::Timestamp::setup().context("setup timestamp")?;
    crate::objects::Window::setup().context("setup window")?;
    Ok(())
}
