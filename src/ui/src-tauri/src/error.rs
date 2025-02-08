use std::fmt::{self, Debug, Display};

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AppError {
    stack: Vec<String>,
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<util::error::Error> for AppError {
    fn from(e: util::error::Error) -> Self {
        let stack = e.chain().map(|e| e.to_string()).collect();
        Self { stack }
    }
}
