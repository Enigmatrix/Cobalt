#[macro_use]
pub mod error;
pub mod hook;
pub mod time;
pub mod window;

pub use std::mem;
pub use std::ptr;
pub use winapi::km::*;
pub use winapi::shared::*;
pub use winapi::um::*;

pub use error::Error;
pub use time::*;
pub use window::*;

pub use winapi::shared::minwindef::*;
pub use winapi::shared::ntdef::*;
pub use winapi::shared::windef::*;
pub use winapi::um::minwinbase::*;
