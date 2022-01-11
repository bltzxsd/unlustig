use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use crate::utils::video::validate_format;
use anyhow::{Context, Result};
use clap::{Parser, ValueHint};

use super::image::random_name;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short = 'T', long, help = "Your caption goes here.", required = true)]
    caption: String,

    #[clap(
        short = 'G',
        long,
        help = "Path to the GIF file",
        value_name = "Path to GIF",
        parse(from_os_str),
        value_hint = ValueHint::FilePath,
        required = true
    )]
    gif: PathBuf,

    #[clap(
        short = 'o',
        long,
        help = "Set the location of the output file\n\nDefaults:\n\tOn Windows: User\\Pictures\\\n\tOn Unix   : Current directory",
        value_name = "Output Directory",
        parse(from_os_str),
        value_hint = ValueHint::DirPath
    )]
    output_directory: Option<PathBuf>,

    #[clap(
        short = 'n',
        long,
        help = "Set the name of the output file\n\nDefault: generates a random alphanumeric name"
    )]
    output_name: Option<String>,

    #[clap(
        short = 'z',
        long,
        help = "Optimizes the output GIF\nCompression and processing time increases with higher values.\nPowered by Gifsicle (https://github.com/kohler/gifsicle) much <3",
        possible_values = ["O1", "O2", "O3"],
    )]
    optimization: Option<String>,

    #[clap(
        short = 'l',
        long,
        help = "Determines how lossy you want the GIF to be.\nHigher values result in smaller file sizes.\nPowered by Gifsicle",
        possible_values = ["20", "40", "60", "80"],
    )]
    lossy: Option<u32>,

    #[clap(
        short = 'r',
        long,
        help = "Reduce the number of distinct colors in each output GIF\nPowered by Gifsicle"
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

    pub fn media(&self) -> Result<File> {
        validate_format(&self.gif)?;
        OpenOptions::new()
            .read(true)
            .open(&self.gif)
            .context("could not read gif")
    }

    pub fn lossy(&self) -> Option<u32> {
        self.lossy
    }

    pub fn name(&self) -> String {
        match &self.output_name {
            Some(string) => {
                if !string.contains(".gif") {
                    return format!("{}.gif", string);
                }
                string.to_string()
            }
            None => format!("{}.gif", random_name()),
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


