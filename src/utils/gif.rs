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
    AnimationDecoder, GenericImage, ImageDecoder, ImageBuffer,
};
use log::{info, warn};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rusttype::Font;

use crate::utils::{
    appdata_init,
    args::Cli,
    image::{SetUp, TextImage},
    random_name,
};

/// Contains the path to the [Gifsicle](https://www.lcdf.org/gifsicle/) program.
pub struct Gifsicle {
    exe: PathBuf,
}

impl Gifsicle {
    /// Initializes `Gifsicle`'s path.
    ///
    /// # Result
    /// This function will return an error if the `%appdata%` variable is not found on Windows.
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

    /// Runs `Gifsicle` with specified flags.
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

/// Creates the gifcaption.
pub fn process_gif(
    gif: File,
    font: Font<'static>,
    text: &str,
    out_path: &Path,
    name: &str,
    cli: &Cli,
    overwrite: bool,
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
        
        let mut buffer = ImageBuffer::new(gif_w, gif_h + image.height()); 
        
        buffer
            .copy_from(&image, 0, 0)
            .expect("could not copy buffer");

        buffer
            .copy_from(f, 0, image.height())
            .expect("could not copy buffer");

        *f = buffer;
    });

    let (output, output_path) = file_and_path(out_path, name, overwrite)?;

    let mut encoder = GifEncoder::new_with_speed(&output, 30);
    encoder.set_repeat(image::gif::Repeat::Infinite)?;
    encoder.encode_frames(frames)?;
    let outputname = &output_path
        .file_name()
        .context("output file path does not exist.")?
        .to_str()
        .context("output-name was not valid utf-8")?;

    info!(
        "GIF: {} {} at {}",
        outputname,
        "generated".green(),
        out_path
            .to_str()
            .context("invalid output path (not utf-8)")?,
    );
    let opt = cli.opt_level().map(std::borrow::ToOwned::to_owned);
    let lossy = cli.lossy();
    let reduce = cli.reduce();
    Gifsicle::init()
        .context("Gifsicle not found. If using Unix, please install and add it to your path!")?
        .run(opt, lossy, reduce, &output_path)?;
    Ok(())
}

/// Returns the File and the path of the file.
///
/// This takes into account if the overwrite flag was enabled.
///
/// If overwrite was disabled, custom name specified, and a file with a similar name was found,
/// prepends a random name using [`random_name()`].
///
/// # Errors
/// Returns an error if the file creation fails.
fn file_and_path(
    out_path: &Path,
    name: &str,
    overwrite: bool,
) -> Result<(File, PathBuf), anyhow::Error> {
    let default = |file_name: &str | -> Result<(File, PathBuf), anyhow::Error> {
        Ok((
            File::create(&out_path.join(&file_name))?,
            out_path.join(file_name),
        ))
    };

    let (output, output_path) = if out_path.join(&name).exists() {
        if overwrite {
            info!("Overwrite is enabled. Any file with the same name ({}) will be overwritten by the output file", &name);
            default(name)?
        } else {
            warn!("Overwrite is disabled. File with a similar name found. Modifying name.");
            let bind = format!("{}_{}", random_name(), &name);
            default(&bind)?
        }
    } else {
        default(name)?
    };
    Ok((output, output_path))
}
