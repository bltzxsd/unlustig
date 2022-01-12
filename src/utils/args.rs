use std::path::PathBuf;

use crate::utils::video::validate_format;
use anyhow::{Context, Result};
use clap::{Parser, ValueHint};

use super::{image::random_name, video::MediaType};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short = 'T', long, help = "Your caption goes here.", required = true)]
    caption: String,

    #[clap(
        short = 'G',
        long,
        help = "Path to the media file.",
        parse(from_os_str),
        value_hint = ValueHint::FilePath,
        required = true
    )]
    media: PathBuf,

    #[clap(
        short = 'o',
        long,
        help = "Set the location of the output file.\n\nDefaults:\n\tOn Windows: User\\Pictures\\\n\tOn Unix   : Current directory",
        value_name = "Output Directory",
        parse(from_os_str),
        value_hint = ValueHint::DirPath
    )]
    output_directory: Option<PathBuf>,

    #[clap(
        short = 'n',
        long,
        help = "Set the name of the output file.\n\nDefault: generates a random alphanumeric name"
    )]
    output_name: Option<String>,

    #[clap(
        short = 'f',
        long,
        help = "Force overwrite the output file if one already exists.\n\nNote: if the output and input videos have the same name,\nthe input will not be overwritten."
    )]
    force_overwrite: bool,

    #[clap(
        short = 'z',
        long,
        help = "Optimize the output GIF.\n\nNote: Compression and processing time increases with higher values.\nPowered by Gifsicle (https://github.com/kohler/gifsicle) much <3",
        possible_values = ["O1", "O2", "O3"],
    )]
    optimization: Option<String>,

    #[clap(
        short = 'l',
        long,
        help = "Determines how lossy you want the GIF to be.\n\nHigher values result in smaller file sizes.\nPowered by Gifsicle",
        possible_values = ["20", "40", "60", "80"],
    )]
    lossy: Option<u32>,

    #[clap(
        short = 'r',
        long,
        help = "Reduce the number of distinct colors in each output GIF.\nPowered by Gifsicle"
    )]
    reduce: bool,
}

impl Cli {
    pub fn opt_level(&self) -> Option<&String> {
        self.optimization.as_ref()
    }

    pub fn reduce(&self) -> bool {
        self.reduce
    }

    pub fn media(&self) -> Result<(PathBuf, MediaType)> {
        Ok((self.media.clone(), validate_format(&self.media)?))
    }

    pub fn lossy(&self) -> Option<u32> {
        self.lossy
    }

    pub fn overwrites(&self) -> bool {
        self.force_overwrite
    }

    pub fn name(&self) -> String {
        let (_, ty) = &self.media().expect("no media found");
        let ext = match ty {
            MediaType::Mp4 => ".mp4",
            MediaType::Avi => ".avi",
            MediaType::Mkv => ".mkv",
            MediaType::Webm => ".webm",
            MediaType::Gif => ".gif",
        };
        match &self.output_name {
            Some(string) => {
                if !string.contains(ext) {
                    return format!("{}{}", string, ext);
                }
                string.to_owned()
            }
            None => format!("{}{}", random_name(), ext),
        }
    }

    pub fn output(&self) -> Result<PathBuf> {
        match &self.output_directory {
            Some(output) => Ok(output.clone()),
            None => {
                #[cfg(windows)]
                return Ok(PathBuf::from(
                    std::env::var("UserProfile").context("unable to read userprofile env var")?,
                )
                .join("Pictures"));
                #[cfg(unix)]
                return Ok(PathBuf::from(std::env::current_dir().context(
                    "Current directory is invalid or lacking permissions for access.",
                )?));
            }
        }
    }

    pub fn text(&self) -> &str {
        self.caption.trim()
    }
}
