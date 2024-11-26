#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("json deserialization error")]
    JsonError(#[from] serde_json::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
