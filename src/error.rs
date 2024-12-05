#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::error::Error),

    #[error("error reading environment variable: {0}")]
    VarError(#[from] std::env::VarError),

    #[error("invalid utf8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("error reading from stdin")]
    StdinError(),
}

pub type Result<T> = std::result::Result<T, Error>;
