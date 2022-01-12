use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorKind {
    // The supplied file was not a gif
    #[error("unsupported media format: {0}")]
    UnsupportedMediaFormat(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
