use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use anyhow::{Context, Result};
use clap::Parser;

use crate::error::ErrorKind;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short = 'T', long, help = "Your caption goes here.")]
    text: Option<String>,

    #[clap(short = 'G', long, help = "Path to the GIF file")]
    gif: Option<String>,

    #[clap(
        short = 'o',
        long,
        help = "Set the name of the output file (default: out.gif)"
    )]
    output: Option<String>,
}

impl Cli {
    pub fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    pub fn gif(&self) -> Result<File> {
        if self.gif.is_none() {
            return Err(anyhow::Error::new(ErrorKind::BadGIF));
        }
        OpenOptions::new()
            .read(true)
            .open(self.gif.as_ref().expect("gif unavailable"))
            .context("could not read gif")
    }

    pub fn output(&self) -> Mode {
        let (text, gif, output) = (
            self.text.is_some(),
            self.gif.is_some(),
            self.output.is_some(),
        );
        if text && gif && !output {
            // output intentionally left out for default
            Mode::Default
        } else if !text && !gif && !output {
            // nothing given, get all during runtime
            Mode::InProgram
        } else {
            // all given
            Mode::Cli(self.output.as_ref())
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode<'m> {
    Cli(Option<&'m String>),
    InProgram,
    Default,
}

pub(crate) trait ToFile {
    fn to_file(&self) -> Result<File, std::io::Error>;
}

impl<T> ToFile for T
where
    T: AsRef<Path>,
{
    fn to_file(&self) -> Result<File, std::io::Error> {
        OpenOptions::new().read(true).open(self)
    }
}
