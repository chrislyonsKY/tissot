/// Error types for the Tissot library.
use thiserror::Error;

/// Top-level error type for Tissot operations.
#[derive(Error, Debug)]
pub enum TissotError {
    /// Failed to read or parse an input file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// GeoJSON parsing error.
    #[error("GeoJSON error: {0}")]
    GeoJson(String),

    /// CRS / projection error.
    #[error("Projection error: {0}")]
    Projection(String),

    /// Unsupported file format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Rule does not support autofix.
    #[error("Rule '{0}' does not support autofix")]
    NoAutofix(String),

    /// Configuration error.
    #[error("Config error: {0}")]
    Config(String),

    /// GeoParquet parsing error.
    #[error("GeoParquet error: {0}")]
    GeoParquet(String),

    /// Generic internal error.
    #[error("{0}")]
    Internal(String),
}

/// Convenience Result type for Tissot operations.
pub type Result<T> = std::result::Result<T, TissotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = TissotError::UnsupportedFormat("foobar".into());
        assert_eq!(err.to_string(), "Unsupported format: foobar");
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: TissotError = io_err.into();
        assert!(err.to_string().contains("missing"));
    }
}
