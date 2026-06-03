//! Typed error catalogue for the Wazuh agent status server.

use thiserror::Error;

/// All errors that can be produced within this server.
#[derive(Error, Debug)]
pub enum ServerError {
    /// An operation failed because of a platform-specific constraint.
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// A file-system I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Alias for `Result<T, ServerError>`.
pub type Result<T> = std::result::Result<T, ServerError>;
