use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use anyhow::Context;
use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short = 'T', long, help = "Your caption goes here.")]
    text: String,

    #[clap(
        short = 'G',
        long,
        help = "Path to the GIF file (without quotation marks)",
        parse(from_os_str),
        value_hint = ValueHint::FilePath
    )]
    gif: PathBuf,

    #[clap(
        short = 'o',
        long,
        help = "Set the location of the output file (On Windows: Pictures folder | On Unix: Current directory))",
        parse(from_os_str),
        value_hint = ValueHint::DirPath
    )]
    output: Option<PathBuf>,

    #[clap(
        short = 'n',
        long,
        help = "Set the name of the output file (default: out.gif)"
    )]
    name: Option<String>,
}

impl Cli {
    pub fn name(&self) -> String {
        match &self.name {
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
        self.text.trim()
    }

    pub fn gif(&self) -> File {
        OpenOptions::new()
            .read(true)
            .open(&self.gif)
            .context("could not read gif")
            .unwrap()
    }

    pub fn output(&self) -> PathBuf {
        match &self.output {
            Some(output) => output.to_path_buf(),
            None => {
                #[cfg(windows)]
                return PathBuf::from(
                    std::env::var("UserProfile").expect("no userprofile env key found"),
                )
                .join("Pictures");
                #[cfg(unix)]
                return PathBuf::from(
                    std::env::current_dir()
                        .expect("lacking permissions for current dir or curr dir is invalid"),
                );
            }
        }
    }
}
