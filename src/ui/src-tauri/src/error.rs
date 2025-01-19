use serde::Serialize;

pub struct AppError(util::error::Error);
pub type AppResult<T> = Result<T, AppError>;

impl From<util::error::Error> for AppError {
    fn from(e: util::error::Error) -> Self {
        Self(e)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO find a better error representation that just string
        serializer.serialize_str(&self.0.to_string())
    }
}
