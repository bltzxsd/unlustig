use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use colored::Colorize;
use image::GenericImageView;
use log::info;
use rusttype::Font;

use crate::{error::ErrorKind, utils::image::{SetUp, TextImage}};

use super::{appdata_init, image::random_name};

pub struct FFmpeg {
    exe: Command,
    input: PathBuf,
}

impl FFmpeg {
    pub fn init(input: PathBuf) -> Result<Self> {
        if cfg!(windows) {
            appdata_init()?;
            Ok(Self {
                exe: Command::new(
                    PathBuf::from(env::var("APPDATA")?)
                        .join("unlustig-rs")
                        .join("ffmpeg.exe"),
                ),
                input,
            })
        } else {
            Ok(Self {
                exe: Command::new("ffmpeg"),
                input,
            })
        }
    }

    fn dimensions(&mut self) -> Result<(u32, u32)> {
        let temp_dir = env::temp_dir();
        let mut name = random_name();
        name.push_str(".jpg");
        let file = temp_dir.join(name);
        let file_str = file.to_str().context("could not convert path to str")?;

        let input = self
            .input
            .to_str()
            .context("could not get string from os")?;
        // ffmpeg -ss 0.1 -i .\cat.mp4 -vframes 1 -f image2 imagefile.jpg
        self.exe
            .args(&[
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                "0.1",
                "-i",
                input,
                "-vframes",
                "1",
                "-f",
                "image2",
                file_str,
            ])
            .spawn()?
            .wait()?;
        Ok(image::open(file)?.dimensions())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaType {
    Mp4,
    Avi,
    Mkv,
    Webm,
    Gif,
}

pub fn validate_format(path: &Path) -> Result<MediaType> {
    match path
        .extension()
        .context("could not get extension")?
        .to_str()
        .context("could not convert osstr to str")?
    {
        "mp4" => Ok(MediaType::Mp4),
        "avi" => Ok(MediaType::Avi),
        "mkv" => Ok(MediaType::Mkv),
        "webm" => Ok(MediaType::Webm),
        "gif" => Ok(MediaType::Gif),
        ext => Err(anyhow::Error::new(ErrorKind::UnsupportedMediaFormat(
            ext.to_string(),
        ))),
    }
}
