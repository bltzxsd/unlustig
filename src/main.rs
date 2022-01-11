#![windows_subsystem = "windows"]

use anyhow::{Context, Result};

use klask::Settings;
use log::error;

use rusttype::Font;
use utils::{args::Cli, gif::process_gif};

mod error;
mod utils;

fn main() {
    pretty_env_logger::init();

    let settings = Settings {
        custom_font: Some(include_bytes!("../font/mononoki-Regular.ttf")),
        ..Settings::default()
    };

    klask::run_derived::<Cli, _>(settings, |cli| {
        if let Err(e) = run(&cli) {
            error!("{:?}", e);
        }
    });
}

fn run(cli: &Cli) -> Result<()> {
    let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
        .context("font could not be read")?;

    let (media, text, out_path, name) = (cli.media()?, cli.text(), cli.output()?, cli.name());

    process_gif(media, font, text, &out_path, &name, cli)?;

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
