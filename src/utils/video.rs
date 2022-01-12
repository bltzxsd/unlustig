use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use colored::Colorize;
use image::GenericImageView;
use log::{info, warn};
use rusttype::Font;

use crate::{
    error::ErrorKind,
    utils::image::{SetUp, TextImage},
};

use super::{appdata_init, image::random_name};

pub struct FFmpeg {
    exe: PathBuf,
    input: PathBuf,
}

impl FFmpeg {
    pub fn init(input: PathBuf) -> Result<Self> {
        if cfg!(windows) {
            appdata_init()?;
            Ok(Self {
                exe: PathBuf::from(env::var("APPDATA")?)
                    .join("unlustig-rs")
                    .join("ffmpeg.exe"),

                input,
            })
        } else {
            Ok(Self {
                exe: PathBuf::from("ffmpeg"),
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
        Command::new(&self.exe)
            .args(&[
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
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
    pub fn process_media(
        &mut self,
        font: Font<'static>,
        text: &str,
        out_path: &Path,
        name: &str,
        overwrite: bool,
    ) -> Result<()> {
        let (width, height) = self.dimensions()?;
        let init = SetUp::init(font).with_dimensions(width, height);
        info!("Creating caption image...");
        let image = TextImage::new(init, text).render()?;
        let mut caption_name = random_name();
        caption_name.push_str(".jpg");
        let caption_location = std::env::temp_dir().join(caption_name);
        image.save(&caption_location)?;
        info!("{}", "Caption image created!".green());
        let (_, caption_height) = image.dimensions();
        let (video_width, video_height) = self.dimensions()?;
        // ffmpeg.exe -i .\cat.mp4 -i .\caption.jpg \
        // -filter_complex "[0:v]pad=640:788:0:148[a];[a][1:v]overlay=0:0,setsar=1"
        // -c:a copy output.mp4
        let mut base_args = vec!["-hide_banner", "-loglevel", "error"];

        let input_args = [
            "-i",
            self.input.to_str().context("cannot convert to str")?,
            "-i",
            caption_location
                .to_str()
                .context("cannot convert to str)")?,
        ];
        let filter_complex = [
            "-filter_complex".into(),
            format!(
                "[0:v]pad={}:{}:0:{}[a];[a][1:v]overlay=0:0,setsar=1",
                video_width,
                video_height + caption_height,
                caption_height
            ),
        ];

        let output = if out_path.join(name).exists() {
            if overwrite {
                info!("Overwrite is enabled. Any file with the same name ({}) will be overwritten by the output file.", name);
                base_args.push("-y");
                out_path.join(name)
            } else {
                warn!("Overwrite is disabled. File with similar name found. Modifying name.");
                out_path.join(format!("{}-{}", random_name(), name))
            }
        } else {
            out_path.join(name)
        };

        let end_args = [
            "-c:a",
            "copy",
            output.to_str().context("cannot convert from path to str")?,
        ];

        Command::new(&self.exe)
            .args(&base_args)
            .args(input_args)
            .args(filter_complex)
            .args(end_args)
            .spawn()?;

        Ok(())
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
