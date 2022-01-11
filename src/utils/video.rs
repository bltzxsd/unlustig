use std::path::Path;

use anyhow::{Context, Result};

use crate::error::ErrorKind;

pub enum MediaTypes {
    Mp4,
    Avi,
    Mkv,
    Webm,
    Gif,
}

pub fn validate_format(path: &Path) -> Result<MediaTypes> {
    match path
        .extension()
        .context("could not get extension")?
        .to_str()
        .context("could not convert osstr to str")?
    {
        "mp4" => Ok(MediaTypes::Mp4),
        "avi" => Ok(MediaTypes::Avi),
        "mkv" => Ok(MediaTypes::Mkv),
        "webm" => Ok(MediaTypes::Webm),
        "gif" => Ok(MediaTypes::Gif),
        ext => Err(anyhow::Error::new(ErrorKind::UnsupportedMediaFormat(
            ext.to_string(),
        ))),
    }
}
