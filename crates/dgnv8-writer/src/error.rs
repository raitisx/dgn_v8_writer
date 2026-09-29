use thiserror::Error;

/// Errors raised while reading a seed or writing elements.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("seed is not a usable DGN V8 file: {0}")]
    InvalidSeed(String),
    #[error("unsupported seed: {0}")]
    UnsupportedSeed(String),
    #[error("invalid element: {0}")]
    InvalidElement(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, WriteError>;
