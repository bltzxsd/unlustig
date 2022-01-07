use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::StructOpt;
use colored::Colorize;
use image::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, ImageDecoder, RgbaImage, GenericImage};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rusttype::Font;
use utils::args::{Cli, Mode, ToFile};
use utils::image::{SetUp, TextImage};

mod error;
mod utils;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        let x = "error".red();
        writeln!(io::stderr(), "{}: {:?}", x, e).expect("could not print to stderr");
    }
}

fn run(cli: Cli) -> Result<()> {
    // let now = std::time::Instant::now();

    let font = Font::try_from_bytes(include_bytes!("../font/ifunny.otf"))
        .context("font could not be read")?;
    let mut path = String::new();
    let gif = match cli.gif() {
        Ok(f) => f,
        Err(_) => {
            write!(
                io::stderr(),
                "Enter the path to the GIF (you can also drag it to the terminal): "
            )?;
            io::stdout().flush()?;
            io::stdin().read_line(&mut path)?;
            if path.trim().is_empty() {
                return Err(anyhow::Error::new(crate::error::ErrorKind::InvalidName))
                    .context("no name provided");
            }

            path.trim().to_file()?
        }
    };
    let mut xd = String::new();
    let text = match cli.text() {
        Some(text) => text,
        None => {
            write!(io::stderr(), "Enter the caption for the GIF: ")?;
            io::stdout().flush()?;
            io::stdin().read_line(&mut xd)?;
            if xd.trim().is_empty() {
                return Err(anyhow::Error::new(crate::error::ErrorKind::BadCaption))
                    .context("no caption provided");
            }
            xd.trim()
        }
    };

    let mut temp = String::new();
    let name = match cli.output() {
        Mode::Cli(str) => match str {
            Some(inner) => inner,
            None => "out.gif",
        },
        Mode::InProgram => {
            write!(
            io::stderr(),
            "What would you like to save this GIF as? Leave blank for the default name (out.gif): "
        )?;
            io::stdin().read_line(&mut temp)?;
            if temp.trim().is_empty() {
                "out.gif"
            } else {
                temp.trim()
            }
        }
        Mode::Default => "out.gif",
    };

    let decoder = GifDecoder::new(gif)?;
    let (gif_w, gif_h) = decoder.dimensions();

    let init = SetUp::init(font).with_dimensions(gif_w, gif_h);

    let image = TextImage::new(init, text)?;
    let image = image.render();
    let image_h = image.height();

    let mut frames = decoder.into_frames().collect_frames()?;

    frames.par_iter_mut().for_each(|f| {
        let f = f.buffer_mut();
        let mut buffer = RgbaImage::new(gif_w, gif_h + image_h);
        // image::imageops::overlay(&mut buffer, &image, 0, 0);
        buffer.copy_from(&image, 0, 0).expect("could not copy");
        // image::imageops::overlay(&mut buffer, f, 0, image_h);
        buffer.copy_from(f, 0, image_h).expect("could not copy buffer");
        *f = buffer;
    });

    let image_path = if cfg!(windows) {
        PathBuf::from(std::env::var("UserProfile")?).join("Pictures")
    } else {
        std::env::current_dir()?
    };
    let file_out = File::create(&image_path.join(name))?;
    let mut encoder = GifEncoder::new_with_speed(file_out, 30);
    encoder.set_repeat(image::gif::Repeat::Infinite)?;
    encoder.encode_frames(frames)?;

    
    let generated = "generated!".green();
    writeln!(
        io::stdout(),
        "\nGIF: {} {} at {}, if on Windows, you should see the explorer pop up!",
        name,
        generated,
        image_path.to_str().expect("invalid path"),
    )?;
    // io::stdin().read_line(&mut String::new())?;
    #[cfg(windows)]
    std::process::Command::new("explorer.exe")
        .arg(image_path)
        .spawn()?;
    #[cfg(unix)]
    std::process::Command::new("xdg-open")
        .arg(image_path)
        .spawn()?;
    // dbg!(now.as_micros());
    Ok(())
}
