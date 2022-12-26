#![feature(new_uninit)]
#![feature(maybe_uninit_uninit_array, maybe_uninit_slice)]
#![feature(let_chains)]
#![feature(iter_collect_into)]

pub mod buffers;
pub mod errors;
pub mod events;
pub mod objects;

use utils::errors::*;

pub fn setup() -> Result<()> {
    objects::Timestamp::setup();
    Ok(())
}
