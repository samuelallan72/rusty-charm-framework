#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::error::Error),
    // TODO: how to support arbitrary errors with `?` syntax from the user code in event handlers?
    // Maybe `anyhow` lib is better suited for this?
}

pub type Result<T> = std::result::Result<T, Error>;
