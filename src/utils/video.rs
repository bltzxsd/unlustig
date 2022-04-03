use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use image::GenericImageView;
use log::{info, warn};
use rusttype::Font;
use yansi::Paint;

use crate::{
    error::ErrorKind,
    utils::{
        image::{SetUp, TextImage},
        DepTy, MediaType,
    },
};

use super::{appdata_init, random_name};

/// [`FFmpeg`] contains the path to the [`FFmpeg`](https://www.ffmpeg.org/) program.
pub struct FFmpeg {
    exe: PathBuf,
    input: PathBuf,
}

impl FFmpeg {
    /// Returns [`FFmpeg`] that you can operate on.
    ///
    /// # Result
    /// Returns an error if [`utils::appdata()`] or [`env::var()`] fail.
    ///
    /// [`utils::appdata()`]: crate::utils
    pub fn init(input: PathBuf) -> Result<Self> {
        let exe = appdata_init(DepTy::Ffmpeg)?;
        Ok(Self { exe, input })
    }

    /// Returns the width and height of the video.
    ///
    /// Runs `FFmpeg` and saves the first frame of the video.
    /// Which is later used to get dimensions from [`dimensions()`]
    ///
    /// [`dimensions()`]: image::GenericImageView::dimensions()
    fn dimensions(&mut self) -> Result<(u32, u32)> {
        let temp_dir = env::temp_dir();
        let mut name = random_name();
        name.push_str(".jpg");
        let file = temp_dir.join(name);
        let file_str = file
            .to_str()
            .context(format!("failed to convert path to str: {}", file.display()))?;
        let input = self.input.to_str().context(format!(
            "failed to convert path to str: {}",
            self.input.display()
        ))?;
        // ffmpeg -ss 0.1 -i .\cat.mp4 -vframes 1 -f image2 imagefile.jpg
        #[rustfmt::skip]
        let args = [
            "-hide_banner", "-loglevel", "error",
            "-y", "-ss", "0.1", "-i", input,
            "-vframes", "1", "-f", "image2", file_str,
        ];

        Command::new(&self.exe)
            .args(&args)
            .spawn()
            .context("failed to start ffmpeg")?
            .wait()?;
        Ok(image::open(file)?.dimensions())
    }

    /// Runs the main logic of video processing.
    ///
    /// `FFmpeg` arguments used:
    ///
    /// ```text
    /// ffmpeg.exe -i media.mp4 -i caption.jpg \
    /// -filter_complex \
    /// "[0:v]pad=640:video_width:0:(video_height + caption_height)[a]; \
    /// [a][1:v]overlay=0:0,setsar=1" \
    /// -c:a copy output.mp4
    /// ```
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
        info!("{}", Paint::green("Caption image created!"));

        let caption_height = image.dimensions().1;
        let (video_width, video_height) = self.dimensions()?;

        // ffmpeg.exe -i .\cat.mp4 -i .\caption.jpg \
        // -filter_complex "[0:v]pad=640:788:0:148[a];[a][1:v]overlay=0:0,setsar=1"
        // -c:a copy output.mp4
        let mut base_args = vec!["-hide_banner", "-loglevel", "error"];

        let input_args = [
            "-i",
            self.input.to_str().context(format!(
                "failed to convert input arg to str: {}",
                self.input.display()
            ))?,
            "-i",
            caption_location.to_str().context(format!(
                "failed to convert input arg to str: {}",
                caption_location.display()
            ))?,
        ];
        let filter_complex = [
            "-filter_complex".into(),
            format!(
                "[0:v]pad={video_width}:{}:0:{caption_height}[a];[a][1:v]overlay=0:0,setsar=1",
                video_height + caption_height,
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
            output.to_str().context(format!(
                "failed to convert output arg to str: {}",
                output.display()
            ))?,
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

/// Validate file formats.
///
/// # Errors
///
/// Returns [`UnsupportedMediaFormat`] if file is unsupported.
///
/// [`UnsupportedMediaFormat`]: crate::error::ErrorKind::UnsupportedMediaFormat
pub fn validate_format(path: &Path) -> Result<MediaType> {
    match path
        .extension()
        .context(format!("failed to get file extension: {}", path.display()))?
        .to_str()
        .context(format!(
            "failed to convert Path->OsStr to str: {}",
            path.display()
        ))? {
        "mp4" => Ok(MediaType::Mp4),
        "avi" => Ok(MediaType::Avi),
        "mkv" => Ok(MediaType::Mkv),
        "webm" => Ok(MediaType::Webm),
        "gif" => Ok(MediaType::Gif),
        ext => Err(ErrorKind::UnsupportedMediaFormat(ext.to_string()).into()),
    }
}
