use std::{
    fs::{File, OpenOptions},
    io::Write,
    iter,
    path::PathBuf,
};

use crate::error::ErrorKind;
use anyhow::{Context, Result};
use clap::{Parser, ValueHint};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[cfg(windows)]
use log::{info, warn};

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
        requires = "optimization"
    )]
    lossy: Option<String>,

    #[clap(
        short = 'r',
        long,
        help = "Reduce the number of distinct colors in each output GIF\nPowered by Gifsicle",
        requires = "optimization"
    )]
    reduce: bool,
}

impl Cli {
    pub fn compress(&self) -> Result<(PathBuf, String)> {
        // this is an exteremely dumb hack of including an exe
        // temporary hack until I figure out how to bundle another exe with wix
        let compression = format!("-{}", self.optimization.as_ref().expect("not a value"));
        #[cfg(windows)]
        {
            let gifsicle = include_bytes!("../../gifsicle/gifsicle.exe");
            let appdata = PathBuf::from(std::env::var("APPDATA")?).join("unlustig-rs");
            if !appdata.exists() {
                warn!("{} does not exist. Creating...", appdata.display());
                std::fs::create_dir(&appdata)?;
                let exe = appdata.join("gifsicle.exe");
                if !exe.exists() {
                    let mut sicle = std::fs::File::create(exe)?;
                    sicle.write_all(gifsicle)?;
                    info!(
                        "Wrote gifsicle.exe to {}",
                        appdata.join("gifsicle.exe").display()
                    );
                }
            }
            Ok((appdata, compression))
        }
        #[cfg(unix)]
        Ok((PathBuf::new(), compression))
    }

    pub fn reduce(&self) -> bool {
        self.reduce
    }

    pub fn gif(&self) -> Result<File> {
        if self
            .gif
            .extension()
            .context("could not get the input file's extension")?
            != "gif"
        {
            return Err(anyhow::Error::from(ErrorKind::InvalidGIF));
        }
        OpenOptions::new()
            .read(true)
            .open(&self.gif)
            .context("could not read gif")
    }

    pub fn lossy(&self) -> Option<&String> {
        self.lossy.as_ref()
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
                    "lacking permissions for current dir or curr dir is invalid",
                )?));
            }
        }
    }

    pub fn text(&self) -> &str {
        self.caption.trim()
    }
}

fn random_name() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(5)
        .collect()
}
