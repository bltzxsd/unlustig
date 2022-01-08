use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::{Parser, ValueHint};

use crate::error::ErrorKind;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short = 'T', long, help = "Your caption goes here.")]
    caption: String,

    #[clap(
        short = 'G',
        long,
        help = "Path to the GIF file",
        value_name = "Path to GIF",
        parse(from_os_str),
        value_hint = ValueHint::FilePath
    )]
    gif: PathBuf,

    #[clap(
        short = 'o',
        long,
        help = "Set the location of the output file (On Windows: User\\Pictures\\ | On Unix: Current directory)",
        value_name = "Output Directory",
        parse(from_os_str),
        value_hint = ValueHint::DirPath
    )]
    output_directory: Option<PathBuf>,

    #[clap(
        short = 'n',
        long,
        value_name = "Outputted GIF's name",
        help = "Set the name of the output file (default: out.gif)"
    )]
    output_name: Option<String>,
}

impl Cli {
    pub fn name(&self) -> String {
        match &self.output_name {
            Some(string) => {
                if !string.contains(".gif") {
                    return format!("{}.gif", string);
                }
                string.to_string()
            }
            None => "out.gif".to_string(),
        }
    }
    pub fn text(&self) -> &str {
        self.caption.trim()
    }

    pub fn gif(&self) -> Result<File> {
        if self
            .gif
            .extension()
            .context("could not get the input file's extension")?
            != "gif"
        {
            return Err(anyhow::Error::from(ErrorKind::NotAGif));
        }
        OpenOptions::new()
            .read(true)
            .open(&self.gif)
            .context("could not read gif")
    }

    pub fn output(&self) -> Result<PathBuf> {
        match &self.output_directory {
            Some(output) => Ok(output.to_path_buf()),
            None => {
                #[cfg(windows)]
                return Ok(PathBuf::from(
                    std::env::var("UserProfile").context("unable to read userprofile env var")?,
                )
                .join("Pictures"));
                #[cfg(unix)]
                return Ok(PathBuf::from(std::env::current_dir().context(
                    "lacking permissions for current dir or curr dir is invalid",
                )?));
            }
        }
    }
}
