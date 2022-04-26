use std::{
    borrow::ToOwned,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use image::{
    codecs::gif::{GifDecoder, GifEncoder},
    AnimationDecoder, GenericImage, ImageBuffer, ImageDecoder,
};
use log::{info, warn};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rusttype::Font;
use utils::DepTy;
use yansi::Paint;

use crate::utils::{
    self, appdata_init,
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
    /// # Errors
    /// * On Windows: Returns an error if the `%appdata%` variable
    /// is not found
    /// * On Unix: Returns an error if Gifsicle is not installed
    /// and on the path.
    pub fn init() -> Result<Self> {
        let exe = appdata_init(DepTy::Gifsicle)?;
        Ok(Self { exe })
    }

    /// Runs `Gifsicle` with specified flags.
    ///
    /// # Errors
    /// Returns an error if Gifsicle fails to spawn.
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
            args.push(format!("--lossy={l}"));
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
        Command::new(self.exe)
            .args(args)
            .spawn()
            .context("failed to start gifsicle")?;
        info!("The optimization will be complete when the terminal window closes.");
        Ok(())
    }
}

/// Creates the gifcaption.
#[allow(clippy::missing_errors_doc)]
pub fn process_gif(gif: File, font: Font<'static>, cli: &Cli) -> Result<(), anyhow::Error> {
    let decoder = GifDecoder::new(gif)?;
    let (gif_w, gif_h) = decoder.dimensions();
    let init = SetUp::init(font).with_dimensions(gif_w, gif_h);
    info!("Creating caption image...");
    let image = TextImage::new(init, cli.text()).render()?;

    info!("{}", Paint::green("Caption image created!"));
    let mut frames = decoder.into_frames().collect_frames()?;
    info!("{}", Paint::blue("Rendering GIF..."));
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
    let out_path = cli.output()?;
    let (output, output_path) = file_and_path(&out_path, &cli.name()?, cli.overwrites())?;

    let mut encoder = GifEncoder::new_with_speed(&output, 30);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;
    encoder.encode_frames(frames)?;
    let outputname = &output_path
        .file_name()
        .context("output path does not exist.")?
        .to_str()
        .context("output name is not valid utf-8")?;

    info!(
        "GIF: {outputname} {} at {}",
        Paint::green("generated"),
        out_path.to_str().context("output path is not utf-8")?,
    );

    let opt = cli.opt_level().map(ToOwned::to_owned);
    let lossy = cli.lossy();
    let reduce = cli.reduce();
    Gifsicle::init()?.run(opt, lossy, reduce, &output_path)?;
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
    let default = |file_name: &str| -> Result<(File, PathBuf), anyhow::Error> {
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
