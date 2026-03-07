//! Error types for AgenticForge.

use thiserror::Error;

pub type ForgeResult<T> = Result<T, ForgeError>;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid magic bytes in file header")]
    InvalidMagic,

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),

    #[error("Blueprint not found: {0}")]
    BlueprintNotFound(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),

    #[error("Test case not found: {0}")]
    TestCaseNotFound(String),

    #[error("Duplicate entity: {0}")]
    DuplicateEntity(String),

    #[error("Duplicate dependency: {0}")]
    DuplicateDependency(String),

    #[error("Invalid parameter: {field} — {reason}")]
    InvalidParameter { field: String, reason: String },

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Capacity exceeded: {resource} — {limit}")]
    CapacityExceeded { resource: String, limit: usize },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Blueprint is locked: {0}")]
    BlueprintLocked(String),

    #[error("File is empty or truncated")]
    Truncated,

    #[error("Corrupt data at offset {0}")]
    Corrupt(u64),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ForgeError {
    pub fn invalid_param(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidParameter {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub fn capacity(resource: impl Into<String>, limit: usize) -> Self {
        Self::CapacityExceeded {
            resource: resource.into(),
            limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ForgeError::BlueprintNotFound("bp-123".into());
        assert!(err.to_string().contains("bp-123"));
    }

    #[test]
    fn test_invalid_param() {
        let err = ForgeError::invalid_param("name", "cannot be empty");
        assert!(err.to_string().contains("name"));
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_capacity_exceeded() {
        let err = ForgeError::capacity("entities", 10_000);
        assert!(err.to_string().contains("entities"));
    }

    #[test]
    fn test_io_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ForgeError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_is_debug() {
        let err = ForgeError::ValidationError("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ValidationError"));
    }
}
