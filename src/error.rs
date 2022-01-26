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

    /// No caption was provided.
    /// This should never occur because the
    /// caption is a required field.
    #[error("no text was given")]
    NoTextGiven,

    /// Gifsicle was not found.
    /// Since unlustig cannot install its
    /// dependencies on unix, users need to
    /// install it themselves.
    #[error(
        "gifsicle not found, if using Unix, please install Gifsicle using your pkg manager: {0}"
    )]
    #[cfg(unix)]
    GifsicleNotFound(#[source] which::Error),

    /// FFmpeg was not found.
    /// Since unlustig cannot install its
    /// dependencies on unix, users need to
    /// install it themselves.
    #[error("FFmpeg not found, if using Unix, please install FFmpeg using your pkg manager: {0}")]
    #[cfg(unix)]
    FfmpegNotFound(#[source] which::Error),
}
