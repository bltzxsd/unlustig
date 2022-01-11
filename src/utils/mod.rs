use std::{fs::File, io::Write, path::PathBuf};

use log::{info, warn};

pub mod args;
pub mod gif;
pub mod image;
pub mod video;

// Writes programs to the appdata folder on windows
pub fn appdata_init() -> anyhow::Result<()> {
    #[cfg(windows)]
    let gifsicle = include_bytes!("../../deps/gifsicle/gifsicle.exe");
    let ffmpeg = include_bytes!("../../deps/ffmpeg/ffmpeg.exe");

    let unlustig = PathBuf::from(std::env::var("APPDATA")?).join("unlustig-rs");
    // if appdata/unlustig-rs doesnt exist, we make a new one and write it
    if !unlustig.exists() {
        warn!("{} does not exist. Trying to create..", unlustig.display());
        std::fs::create_dir(&unlustig)?;
        info!("Created {}", unlustig.display());
    }
    let (gif_exe, ffmpeg_exe) = (unlustig.join("gifsicle.exe"), unlustig.join("ffmpeg.exe"));
    if !gif_exe.exists() {
        let mut sicle = File::create(gif_exe)?;
        sicle.write_all(gifsicle)?;
        info!("Wrote gifsicle.exe to {}", unlustig.display());
    }
    if !ffmpeg_exe.exists() {
        let mut ffm = File::create(ffmpeg_exe)?;
        ffm.write_all(ffmpeg)?;
        info!("Wrote ffmpeg.exe to {}", unlustig.display())
    }

    Ok(())
}
