#![windows_subsystem = "windows"]
#![warn(clippy::double_comparisons, clippy::missing_errors_doc)]
#![deny(missing_docs)]

//! Unlustig-rs
//!
//! iFunny Gif Caption Maker.

use std::fs::OpenOptions;

use anyhow::{Context, Result};

use klask::Settings;
use log::{error, warn};

use rich_presence::Discord;
use rusttype::Font;
use utils::{args::Cli, gif::process_gif, video::FFmpeg, MediaType};

/// Error module.
pub(crate) mod error;

/// Rich Presence module.
mod rich_presence;

/// Utility module.
mod utils;

fn main() {
    pretty_env_logger::init();

    if let Err(e) = Discord::init(include_str!("RPC_ID")) {
        warn!("failed discord RPC initialization: {}", e);
    };

    #[cfg(unix)]
    match ProgramMode::check() {
        ProgramMode::Cli => {
            if let Err(err) = run(&<Cli as clap::StructOpt>::parse()) {
                error!("{:?}", err);
            }
        }
        ProgramMode::Gui => {
            let settings = Settings {
                custom_font: Some(include_bytes!("../font/mononoki-Regular.ttf")),
                ..Settings::default()
            };

            klask::run_derived::<Cli, _>(settings, |cli| {
                if let Err(err) = run(&cli) {
                    error!("{:?}", err);
                }
            });
        }
    }

    #[cfg(windows)]
    {
        let settings = Settings {
            custom_font: Some(include_bytes!("../font/mononoki-Regular.ttf")),
            ..Settings::default()
        };

        klask::run_derived::<Cli, _>(settings, |cli| {
            if let Err(err) = run(&cli) {
                error!("{:?}", err);
            }
        });
    }
}

/// Main logic.
fn run(cli: &Cli) -> Result<()> {
    let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
        .context("font could not be read")?;

    let (text, out_path, name, overwrite) =
        (cli.text(), cli.output()?, cli.name()?, cli.overwrites());

    if let Ok((file_path, file_ty)) = cli.media() {
        let file = OpenOptions::new().read(true).open(&file_path)?;
        match file_ty {
            MediaType::Mp4 | MediaType::Avi | MediaType::Mkv | MediaType::Webm => {
                FFmpeg::init(file_path)?.process_media(font, text, &out_path, &name, overwrite)?;
            }

            MediaType::Gif => process_gif(file, font, text, &out_path, &name, cli, overwrite)?,
        }
    }

    #[cfg(windows)]
    std::process::Command::new("explorer.exe")
        .arg(out_path)
        .spawn()?;

    // Opening File Manager with UNIX is not tested.
    #[cfg(unix)]
    std::process::Command::new("xdg-open")
        .arg(out_path)
        .spawn()?;

    Ok(())
}

/// The implemented ways to interact with the program.
#[cfg(unix)]
#[derive(Debug, Clone, Copy)]
enum ProgramMode {
    Cli,
    Gui,
}

#[cfg(unix)]
impl ProgramMode {
    /// Checks the current mode of the program depending on the number of CLI arguments given.
    pub fn check() -> Self {
        if std::env::args().len() > 1 {
            return ProgramMode::Cli;
        }
        ProgramMode::Gui
    }
}
