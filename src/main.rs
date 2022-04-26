#![windows_subsystem = "windows"]
#![warn(clippy::double_comparisons, clippy::missing_errors_doc)]
#![deny(missing_docs)]

//! Unlustig-rs
//!
//! iFunny Gif Caption Maker.

use std::fs::OpenOptions;

use anyhow::{Context, Result};

use klask::Settings;
use log::{debug, error, info, trace, warn};

use rich_presence::Discord;
use rusttype::Font;
use semver::Version;
use serde_json::Value;
use utils::{args::Cli, gif::process_gif, video::FFmpeg, MediaType};
use yansi::Paint;

/// Error module.
pub mod error;

/// Rich Presence module.
mod rich_presence;

/// Utility module.
pub mod utils;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("failed to start logger");

    if let Err(e) = check_updates() {
        debug!("Failed to check for updates: {e}")
    }
    if let Err(e) = Discord::init("930897511743356950") {
        debug!("failed discord RPC initialization: {e}");
    };

    let custom_font = include_bytes!("../font/mononoki-Regular.ttf");
    #[cfg(unix)]
    match ProgramMode::check() {
        ProgramMode::Cli => {
            if let Err(err) = <Cli as clap::Parser>::parse().run() {
                error!("{:?}", err);
            }
        }
        ProgramMode::Gui => {
            let settings = Settings {
                custom_font: Some(custom_font),
                ..Settings::default()
            };

            klask::run_derived::<Cli, _>(settings, |cli| {
                if let Err(err) = cli.run() {
                    error!("{:?}", err);
                }
            });
        }
    }

    #[cfg(windows)]
    {
        let settings = Settings {
            custom_font: Some(custom_font),
            ..Settings::default()
        };

        klask::run_derived::<Cli, _>(settings, |cli| {
            if let Err(err) = cli.run() {
                error!("{:?}", err);
            }
        });
    }
}

impl Cli {
    /// Main logic.
    fn run(&self) -> Result<()> {
        let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
            .context("failed to read font")?;

        let (text, out_path, name, overwrite) =
            (self.text(), self.output()?, self.name()?, self.overwrites());

        if let Ok((file_path, file_ty)) = self.media() {
            let file = OpenOptions::new().read(true).open(&file_path)?;
            if let MediaType::Gif = file_ty {
                process_gif(file, font, self)?
            } else {
                if self.reduce() || self.lossy().is_some() || self.opt_level().is_some() {
                    info!("Optimization flags only work on GIFs.");
                }
                FFmpeg::init(file_path)?.process_media(font, text, &out_path, &name, overwrite)?;
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
}

fn check_updates() -> Result<()> {
    let url = "https://api.github.com/repos/bltzxsd/unlustig/releases";
    let request = ureq::get(url).call()?;
    let data = request.into_string()?;
    let git_tag = serde_json::from_str::<Value>(&data)?[0]["tag_name"]
        .to_string()
        .replace(&['\"', 'v'][..], "");
    let (curr_ver, web_ver): (Version, Version) =
        (clap::crate_version!().parse()?, git_tag.parse()?);

    match curr_ver.cmp(&web_ver) {
        std::cmp::Ordering::Greater => debug!(":face_with_raised_eyebrow:"),
        std::cmp::Ordering::Less => warn!(
            "{}\nUpdate here: https://github.com/bltzxsd/unlustig/releases/latest",
            Paint::red("unlustig is out of date!").bold(),
        ),
        std::cmp::Ordering::Equal => trace!("unlustig is up to date"),
    }
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
