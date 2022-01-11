use std::{io::Write, path::PathBuf, process::Command};

use anyhow::Result;
use log::{info, warn};

pub struct Gifsicle {
    exe: PathBuf,
}

impl Gifsicle {
    pub fn init() -> Result<Self> {
        #[cfg(unix)]
        return Ok(Self {
            exe: PathBuf::new(),
        });

        #[cfg(windows)]
        let gifsicle = include_bytes!("../../deps/gifsicle/gifsicle.exe");
        // folder: AppData/unlustig-rs/
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

        Ok(Self {
            // program: AppData/unlustig-rs/gifsicle.rs
            exe: appdata.join("gifsicle.exe"),
        })
    }

    ///
    pub fn run(
        self,
        opt: Option<String>,
        lossy: Option<u32>,
        reduce: bool,
        imagepath: PathBuf,
    ) -> Result<()> {
        let mut args = vec!["-b".into(), imagepath.display().to_string()];
        if let Some(v) = opt {
            args.push(format!("-{}", v))
        }
        if let Some(l) = lossy {
            args.push(format!("--lossy={}", l));
        }
        if reduce {
            args.push("--colors".into());
            args.push("256".into());
        }
        // No optimization called for.
        if args.len() == 2 {
            return Ok(());
        }
        info!("Optimization is enabled. Optimizing GIF...");
        info!("GIF optimization may take some time.");
        Command::new(self.exe).args(args).spawn()?;
        info!("The optimization will be complete when the terminal window closes.");
        Ok(())
    }
}
