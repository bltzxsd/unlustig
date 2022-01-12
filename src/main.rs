#![windows_subsystem = "windows"]

use std::fs::OpenOptions;

use anyhow::{Context, Result};

use klask::Settings;
use log::{error, info};

use rusttype::Font;
use utils::{
    args::Cli,
    gif::process_gif,
    video::{FFmpeg, MediaType},
};

mod error;
mod utils;

fn main() {
    pretty_env_logger::init();

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

fn run(cli: &Cli) -> Result<()> {
    let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
        .context("font could not be read")?;

    let (text, out_path, name) = (cli.text(), cli.output()?, cli.name());

    if let Ok((file_path, file_ty)) = cli.media() {
        let file = OpenOptions::new().read(true).open(&file_path)?;
        match file_ty {
            MediaType::Mp4 | MediaType::Avi | MediaType::Mkv | MediaType::Webm => {
                info!("Note: Optimization flags do not work on media files.");
                FFmpeg::init(file_path)?.process_media(font, text, &out_path, &name)?;
            }
            MediaType::Gif => process_gif(file, font, text, &out_path, &name, cli)?,
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

#[cfg(unix)]
enum ProgramMode {
    Cli,
    Gui,
}

#[cfg(unix)]
impl ProgramMode {
    pub fn check() -> Self {
        if std::env::args().len() > 1 {
            return ProgramMode::Cli;
        }
        ProgramMode::Gui
    }
}
