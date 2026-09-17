use serde::Serialize;
use thiserror::Error;

/// High-level typed errors for the Vision application.
/// These errors are safe to expose to the frontend.
#[derive(Debug, Error)]
pub enum VisionError {
    #[error("User error: {0}")]
    UserError(String),

    #[error("Invalid or corrupted document: {0}")]
    DocumentError(String),

    #[error("Engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("Engine error: {0}")]
    EngineError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Security or permission error: {0}")]
    SecurityError(String),

    #[error("Operation cancelled")]
    CancellationError,

    #[error("Resource exhaustion or limit reached: {0}")]
    ResourceError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedError(String),
}

// Convert from standard io::Error
impl From<std::io::Error> for VisionError {
    fn from(err: std::io::Error) -> Self {
        VisionError::IoError(err.to_string())
    }
}

// Implement Serialize so we can return errors to Tauri frontend easily
impl Serialize for VisionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the error as its Display string to the frontend.
        serializer.serialize_str(&self.to_string())
    }
}

pub type VisionResult<T> = Result<T, VisionError>;
