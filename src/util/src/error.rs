pub use anyhow::*;

pub trait ResultExt2<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: std::fmt::Debug> ResultExt2<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        match self {
            Err(_) => {
                self.expect("chain of errors");
                std::process::exit(1);
            }
            Ok(x) => x,
        }
    }
}
