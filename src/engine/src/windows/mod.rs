#[macro_use]
pub mod error;
pub mod event_loop;
pub mod hook;
pub mod window;
pub mod process;

pub use winapi::um::*;
pub use winapi::km::*;
pub use winapi::shared::*;
pub use std::ptr;
pub use std::mem;

pub use crate::windows::error::*;
pub use crate::windows::event_loop::*;
pub use crate::windows::hook::*;
pub use crate::windows::window::*;
pub use crate::windows::process::*;

pub mod wintypes {
    pub use winapi::um::winnt::*;
    pub use winapi::shared::windef::*;
    pub use winapi::shared::minwindef::*;
}