use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum ErrorKind {
    // The supplied file was not a gif
    #[error("unsupported media format: {0}")]
    UnsupportedMediaFormat(String),
}
