use std::fs::File;

use anyhow::{Context, Result};
use colored::Colorize;
use image::{
    gif::{GifDecoder, GifEncoder},
    AnimationDecoder, GenericImage, ImageDecoder, RgbaImage,
};
use klask::Settings;
use log::{error, info};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rusttype::Font;
use utils::{
    args::Cli,
    image::{SetUp, TextImage},
};

mod utils;

fn main() {
    pretty_env_logger::init();
    klask::run_derived::<Cli, _>(Settings::default(), |cli| {
        if let Err(e) = run(cli) {
            for error in e.chain() {
                error!("{}", error);
            }
        }
    });
}

fn run(cli: Cli) -> Result<()> {
    let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
        .context("font could not be read")?;

    let (gif, text, out_path, name) = (cli.gif(), cli.text(), cli.output(), cli.name());

    let decoder = GifDecoder::new(gif)?;
    let (gif_w, gif_h) = decoder.dimensions();

    let init = SetUp::init(font).with_dimensions(gif_w, gif_h);
    info!("Creating caption image..");

    let image = TextImage::new(init, text)?.render();
    info!("{}", "Caption image created!".green());

    let mut frames = decoder.into_frames().collect_frames()?;
    info!("{}", "Rendering GIF...".blue());

    frames.par_iter_mut().for_each(|f| {
        let f = f.buffer_mut();
        let mut buffer = RgbaImage::new(gif_w, gif_h + image.height());
        // image::imageops::overlay(&mut buffer, &image, 0, 0);
        buffer.copy_from(&image, 0, 0).expect("could not copy");
        // image::imageops::overlay(&mut buffer, f, 0, image_h);
        buffer
            .copy_from(f, 0, image.height())
            .expect("could not copy buffer");
        *f = buffer;
    });

    let file_out = File::create(&out_path.join(&name))?;

    let mut encoder = GifEncoder::new_with_speed(file_out, 30);
    encoder.set_repeat(image::gif::Repeat::Infinite)?;
    encoder.encode_frames(frames)?;

    info!(
        "GIF: {} {} at {}, if on Windows, you should see the explorer pop up!",
        &name,
        "generated".green(),
        out_path.to_str().expect("invalid output path"),
    );

    #[cfg(windows)]
    std::process::Command::new("explorer.exe")
        .arg(out_path)
        .spawn()?;
    #[cfg(unix)]
    std::process::Command::new("xdg-open")
        .arg(out_path)
        .spawn()?;

    Ok(())
}
