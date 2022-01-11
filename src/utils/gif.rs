use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use colored::Colorize;
use image::{
    gif::{GifDecoder, GifEncoder},
    AnimationDecoder, GenericImage, ImageDecoder, RgbaImage,
};
use log::info;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rusttype::Font;

use crate::utils::image::{SetUp, TextImage};

use super::{appdata_init, args::Cli};

pub struct Gifsicle {
    exe: PathBuf,
}

impl Gifsicle {
    pub fn init() -> Result<Self> {
        if cfg!(windows) {
            appdata_init()?;
            Ok(Self {
                // program: AppData/unlustig-rs/gifsicle.rs
                exe: PathBuf::from(env::var("APPDATA")?)
                    .join("unlustig-rs")
                    .join("gifsicle.exe"),
            })
        } else {
            Ok(Self {
                exe: PathBuf::new(),
            })
        }
    }

    ///
    pub fn run(
        self,
        opt: Option<String>,
        lossy: Option<u32>,
        reduce: bool,
        imagepath: &Path,
    ) -> Result<()> {
        let mut args = vec![
            "--no-conserve-memory".into(),
            "-w".into(),
            "-b".into(),
            imagepath.display().to_string(),
        ];
        if let Some(v) = opt {
            args.push(format!("-{}", v));
        }
        if let Some(l) = lossy {
            args.push(format!("--lossy={}", l));
        }
        if reduce {
            args.push("--colors".into());
            args.push("256".into());
        }
        // No optimization called for.
        if args.len() == 4 {
            return Ok(());
        }
        info!("Optimization is enabled. Optimizing GIF...");
        info!("GIF optimization may take some time.");
        Command::new(self.exe).args(args).spawn()?;
        info!("The optimization will be complete when the terminal window closes.");
        Ok(())
    }
}

pub fn process_gif(
    gif: File,
    font: Font<'static>,
    text: &str,
    out_path: &Path,
    name: &str,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    let decoder = GifDecoder::new(gif)?;
    let (gif_w, gif_h) = decoder.dimensions();
    let init = SetUp::init(font).with_dimensions(gif_w, gif_h);
    info!("Creating caption image...");
    let image = TextImage::new(init, text).render()?;
    info!("{}", "Caption image created!".green());
    let mut frames = decoder.into_frames().collect_frames()?;
    info!("{}", "Rendering GIF...".blue());
    frames.par_iter_mut().for_each(|f| {
        let f = f.buffer_mut();
        let mut buffer = RgbaImage::new(gif_w, gif_h + image.height());

        buffer
            .copy_from(&image, 0, 0)
            .expect("could not copy buffer");

        buffer
            .copy_from(f, 0, image.height())
            .expect("could not copy buffer");

        *f = buffer;
    });
    let file_out = File::create(&out_path.join(&name))?;
    let file_out_path = out_path.join(&name);
    let mut encoder = GifEncoder::new_with_speed(&file_out, 30);
    encoder.set_repeat(image::gif::Repeat::Infinite)?;
    encoder.encode_frames(frames)?;
    info!(
        "GIF: {} {} at {}",
        &name,
        "generated".green(),
        out_path.to_str().expect("invalid output path"),
    );
    let opt = cli.opt_level().map(std::borrow::ToOwned::to_owned);
    let lossy = cli.lossy();
    let reduce = cli.reduce();
    Gifsicle::init()
        .context("Gifsicle not found. If using Unix, please install and add it to your path!")?
        .run(opt, lossy, reduce, &file_out_path)?;
    Ok(())
}
