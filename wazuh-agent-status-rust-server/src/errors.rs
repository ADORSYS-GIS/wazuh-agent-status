//! Typed error catalogue for the Wazuh agent status server.

use thiserror::Error;

/// All errors that can be produced within this server.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ServerError {
    /// An operation failed because of a platform-specific constraint.
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// A file-system I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON (de)serialisation failure.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// The agent update process failed.
    #[error("Update error: {0}")]
    UpdateError(String),

    /// An unexpected internal error that should not occur in normal operation.
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Alias for `Result<T, ServerError>`.
pub type Result<T> = std::result::Result<T, ServerError>;
