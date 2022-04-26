//! Helper utilities for other media processing activities.
//!
//! The `crate::utils` module contains common functions, and enums.


#[cfg(windows)]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(windows)]
use log::{info, warn};
#[cfg(windows)]
use std::{env, fs::File, io::Read, io::Write};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use anyhow::Context;
use std::{
    iter,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, anyhow::Error>;

use crate::error::ErrorKind;

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
    /// `.gif` files.
    Gif,
    /// `.mov` files.
    Mov,
}

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub enum DepTy {
    Gifsicle,
    Ffmpeg,
}

impl std::fmt::Display for DepTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match *self {
            DepTy::Gifsicle => "Gifsicle",
            DepTy::Ffmpeg => "FFmpeg",
        };
        write!(f, "{x}")
    }
}

/// Writes [`Gifsicle`] and [`FFmpeg`] to the appdata folder on Windows.
///
/// # Errors
/// Returns an error if the `%appdata%` variable does not exist.
///
/// [`Gifsicle`]: https://www.lcdf.org/gifsicle/
/// [`FFmpeg`]: https://www.ffmpeg.org/
pub fn appdata_init(dep: DepTy) -> anyhow::Result<PathBuf> {
    #[cfg(windows)]
    {
        let unlustig = PathBuf::from(env::var("APPDATA")?).join("unlustig-rs");
        let executable = match dep {
            DepTy::Gifsicle => unlustig.join("gifsicle.exe"),
            DepTy::Ffmpeg => unlustig.join("ffmpeg.exe"),
        };

        if !unlustig.exists() || !executable.exists() {
            warn!("{} does not exist. Trying to create..", unlustig.display());
            dep.download()
                .context(format!("failed to download {dep}"))?;
            info!("Created {}", unlustig.display());
        }

        Ok(executable)
    }

    #[cfg(unix)]
    {
        use ErrorKind::{FfmpegNotFound, GifsicleNotFound};
        match dep {
            // since which takes care of path on unix, we can just return that.
            DepTy::Gifsicle => which::which("gifsicle").map_err(|err| GifsicleNotFound(err).into()),
            DepTy::Ffmpeg => which::which("ffmpeg").map_err(|err| FfmpegNotFound(err).into()),
        }
    }
}

/// Generates a random name with 5 alphanumeric chars.
pub fn random_name() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|_| char::from(rng.sample(Alphanumeric)))
        .take(5)
        .collect()
}

impl DepTy {
    #[cfg(windows)]
    /// Downloads the specified dependency.
    ///
    /// # Errors
    /// Returns an error if the download fails.
    pub fn download(&self) -> Result<()> {
        let url = match *self {
            DepTy::Gifsicle => {
                "https://github.com/bltzxsd/unlustig/raw/main/deps/gifsicle/gifsicle.exe"
            }
            DepTy::Ffmpeg => "https://github.com/bltzxsd/unlustig/raw/main/deps/ffmpeg/ffmpeg.exe",
        };

        let request = ureq::get(url).call()?;

        let size: u64 = request
            .header("content-length")
            .context("failed to get download size")?
            .parse()?;

        let fname = url.split('/').last().unwrap_or("unknown");
        let bytes = human_bytes::human_bytes(size as f64);

        info!("Downloading {fname} - {bytes}");

        let chunk_size = 1024usize;

        let pb = ProgressBar::new(size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"));

        let mut buf = Vec::new();
        let mut reader = request.into_reader();

        loop {
            let mut buffer = vec![0; chunk_size];
            let bcount = reader.read(&mut buffer[..])?;
            buffer.truncate(bcount);
            if !buffer.is_empty() {
                buf.extend(buffer.into_boxed_slice().into_vec().iter().copied());
                pb.inc(bcount as _);
            } else {
                break;
            }
        }

        pb.finish();
        let unlustig = PathBuf::from(env::var("APPDATA")?).join("unlustig-rs");
        std::fs::create_dir_all(&unlustig)?;
        let mut file = File::create(&unlustig.join(fname))?;
        file.write_all(&buf)?;

        Ok(())
    }
}

/// Validate file formats.
///
/// # Errors
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
        "mov" => Ok(MediaType::Mov),
        ext => Err(ErrorKind::UnsupportedMediaFormat(ext.to_string()).into()),
    }
}
