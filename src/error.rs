#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::error::Error),
    //     #[error("error: {0}")]
    //     Generic(#[from] String),
}

pub type Result<T> = std::result::Result<T, Error>;
