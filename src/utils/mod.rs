//! Helper utilities for other media processing activities.
//!
//! The `crate::utils` module contains common functions, and enums.

use std::{fs::File, io::Write, iter, path::PathBuf};

use log::{info, warn};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

/// Argument handling with [`Clap`].
///
/// [`Clap`]: clap
pub mod args;
/// Gif captioning.
pub mod gif;
/// Caption creation.
pub mod image;
/// Video captioning.
pub mod video;

/// Contains the types of media supported by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaType {
    /// `.mp4` files.
    Mp4,
    /// `.avi` files.
    Avi,
    /// `.mkv` files.
    Mkv,
    /// `.webm` files.
    Webm,
    /// `.gif` files
    Gif,
}

/// Writes [`Gifsicle`] and [`FFmpeg`] to the appdata folder on Windows.
///
/// # Result
/// Returns an error if the `%appdata%` variable does not exist.
///
/// [`Gifsicle`]: https://www.lcdf.org/gifsicle/
/// [`FFmpeg`]: https://www.ffmpeg.org/
pub fn appdata_init() -> anyhow::Result<()> {
    #[cfg(windows)]
    let gifsicle = include_bytes!("../../deps/gifsicle/gifsicle.exe");
    let ffmpeg = include_bytes!("../../deps/ffmpeg/ffmpeg.exe");

    let unlustig = PathBuf::from(std::env::var("APPDATA")?).join("unlustig-rs");
    // if appdata/unlustig-rs doesnt exist, we make a new one and write it

    if !unlustig.exists() {
        warn!("{} does not exist. Trying to create..", unlustig.display());
        std::fs::create_dir(&unlustig)?;
        info!("Created {}", unlustig.display());
    }
    let (gif_exe, ffmpeg_exe) = (unlustig.join("gifsicle.exe"), unlustig.join("ffmpeg.exe"));
    if !gif_exe.exists() {
        let mut sicle = File::create(gif_exe)?;
        sicle.write_all(gifsicle)?;
        info!("Wrote gifsicle.exe to {}", unlustig.display());
    }
    if !ffmpeg_exe.exists() {
        let mut ffm = File::create(ffmpeg_exe)?;
        ffm.write_all(ffmpeg)?;
        info!("Wrote ffmpeg.exe to {}", unlustig.display())
    }

    Ok(())
}

/// Generates a random name with 5 alphanumeric chars.
pub fn random_name() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(5)
        .collect()
}
