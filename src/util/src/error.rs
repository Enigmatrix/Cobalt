pub use anyhow::*;

pub trait ResultExt2 {
    fn unwrap_or_exit(self);
}

impl<T> ResultExt2 for Result<T> {
    fn unwrap_or_exit(self) {
        self.unwrap(); // TODO better representation.
    }
}