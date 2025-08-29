//! Error types for the Solana Stream Processor

use std::fmt;

/// Main error type for the application
#[derive(Debug)]
pub enum ProcessorError {
    /// Configuration error
    Config(String),
    
    /// Database error
    Database(mongodb::error::Error),
    
    /// Network error
    Network(String),
    
    /// Serialization error
    Serialization(serde_json::Error),
    
    /// Web server error
    WebServer(String),
    
    /// Vixen runtime error
    VixenRuntime(String),
    
    /// General I/O error
    Io(std::io::Error),
    
    /// Other errors
    Other(anyhow::Error),
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessorError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ProcessorError::Database(err) => write!(f, "Database error: {}", err),
            ProcessorError::Network(msg) => write!(f, "Network error: {}", msg),
            ProcessorError::Serialization(err) => write!(f, "Serialization error: {}", err),
            ProcessorError::WebServer(msg) => write!(f, "Web server error: {}", msg),
            ProcessorError::VixenRuntime(msg) => write!(f, "Vixen runtime error: {}", msg),
            ProcessorError::Io(err) => write!(f, "I/O error: {}", err),
            ProcessorError::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for ProcessorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProcessorError::Database(err) => Some(err),
            ProcessorError::Serialization(err) => Some(err),
            ProcessorError::Io(err) => Some(err),
            ProcessorError::Other(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<mongodb::error::Error> for ProcessorError {
    fn from(err: mongodb::error::Error) -> Self {
        ProcessorError::Database(err)
    }
}

impl From<serde_json::Error> for ProcessorError {
    fn from(err: serde_json::Error) -> Self {
        ProcessorError::Serialization(err)
    }
}

impl From<std::io::Error> for ProcessorError {
    fn from(err: std::io::Error) -> Self {
        ProcessorError::Io(err)
    }
}

impl From<anyhow::Error> for ProcessorError {
    fn from(err: anyhow::Error) -> Self {
        ProcessorError::Other(err)
    }
}

/// Result type alias for the application
pub type ProcessorResult<T> = Result<T, ProcessorError>;