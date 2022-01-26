use thiserror::Error;

/// List of general categories defining types of error.
#[derive(Error, Debug)]
pub enum ErrorKind {
    /// Media format is not supported.
    #[error("unsupported media format: {0}")]
    UnsupportedMediaFormat(String),
    /// General IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no text was given")]
    NoTextGiven,
}
