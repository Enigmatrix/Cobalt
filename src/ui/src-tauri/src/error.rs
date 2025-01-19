use serde::Serialize;

#[derive(Serialize)]
pub struct AppError {
    stack: Vec<String>
}
pub type AppResult<T> = Result<T, AppError>;

impl From<util::error::Error> for AppError {
    fn from(e: util::error::Error) -> Self {
        let stack = e.chain().map(|e| e.to_string()).collect();
        Self { stack }
    }
}
