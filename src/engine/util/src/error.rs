pub use eyre::*;
pub use color_eyre::*;
use std::fmt;

pub trait ResultExt2<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: fmt::Debug> ResultExt2<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        match self {
            Err(_) => {
                self.expect("error'ed due to");
                std::process::exit(1); // kills other threads if there any
            }
            Ok(x) => x,
        }
    }
}

pub fn setup_error() -> Result<()> {
    color_eyre::install().wrap_err("setup error handling and reporting")
}