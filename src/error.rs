use thiserror::Error;

#[derive(Error, Clone, Copy, Debug)]
pub enum ErrorKind {
    // The supplied file was not a gif
    #[error("not a gif")]
    InvalidGIF,
    #[error("invalid optimization")]
    InvalidOptimization,
}
