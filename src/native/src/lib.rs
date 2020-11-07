#![feature(default_free_fn)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(min_const_generics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(test)]

pub mod raw;
#[macro_use]
pub mod error;
pub mod buffer;
pub mod com;
pub mod wrappers;

pub fn setup() {
    wrappers::Timestamp::calculate_boot_time();
}
