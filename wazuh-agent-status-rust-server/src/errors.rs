use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Platform-specific error: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("Task error: {0}")]
    TaskError(String),

    #[allow(dead_code)]
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[allow(dead_code)]
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ServerError>;
