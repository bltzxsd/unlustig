use thiserror::Error;

#[derive(Debug, Error, Clone, Copy)]
pub enum ErrorKind {
    #[error("invalid name")]
    InvalidName,
    #[error("invalid caption")]
    BadCaption,
    #[error("invalid gif")]
    BadGIF,
}
