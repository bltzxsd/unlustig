use crate::utils::{random_name, video::validate_format, MediaType};
use anyhow::Result;
use clap::{Parser, ValueHint};
use std::{io, path::PathBuf};

/// CLI arguments parser for GUI and TUI.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    /// Caption for the image.
    ///
    /// See also: [`Cli::text()`]   
    #[clap(
        short = 'T',
        long,
        value_name = "Text",
        help = "Your caption goes here.",
        long_help = None,
        required = true,
    )]
    caption: String,

    /// Input media for processing.
    ///
    /// See also: [`Cli::media()`]
    #[clap(
        short = 'G',
        long,
        help = "Path to the media file.",
        parse(from_os_str),
        value_name = "Media: .mp4 / .gif etc.",
        value_hint = ValueHint::FilePath,
        long_help = None,
        required = true
    )]
    media: PathBuf,

    /// The directory where the ouptut should be saved at.
    ///
    /// See also: [`Cli::output()`]
    #[clap(
        short = 'o',
        long,
        help = "Set the location of the output file.\n\nDefaults:\n\tOn Windows: <drive>:\\Users\\<username>\\Pictures\\\n\tOn Unix: /home/<username>/Pictures",
        long_help = None,
        value_name = "Directory", 
        parse(from_os_str),
        value_hint = ValueHint::DirPath
    )]
    output_directory: Option<PathBuf>,

    /// Specified name of the output file.
    ///
    /// See also: [`Cli::name()`]
    #[clap(
        short = 'n',
        long,
        value_name = "Name",
        help = "Set the name of the output file. Generates a random alphanumeric name if none given.",
        long_help = None,
    )]
    output_name: Option<String>,

    /// Determines if the output should overwrite
    /// a pre-existing file.
    ///
    /// See also: [`Cli::overwrites()`]
    #[clap(
        short = 'f',
        long,
        help = "Force overwrite the output file if one already exists.",
        long_help = "Force overwrite the output file if one already exists. Note: if the output and input videos have the same name, the input will not be overwritten"
    )]
    force_overwrite: bool,

    /// Specified the optimization level to be used.
    ///
    /// Optimization levels are implemented only for [`Gif`]s.
    ///
    /// [`Gif`]: crate::utils::MediaType::Gif
    #[clap(
        short = 'z',
        long,
        value_name = "Level",
        help = "Optimize the output GIF.",
        long_help = "Optimize the output GIF. Note: Compression and processing time increases with higher values. Powered by Gifsicle",
        possible_values = ["O1", "O2", "O3"],
    )]
    optimization: Option<String>,

    /// Determines how lossy will the output file be.
    /// Corresponds to `-lossy=<num>` parameter in [Gifsicle](https://www.lcdf.org/gifsicle/).
    ///
    /// Lossy levels are implemented only for [`Gif`]s
    ///
    /// [`Gif`]: crate::utils::MediaType::Gif
    #[clap(
        short = 'l',
        long,
        value_name = "Level",
        help = "Determines how lossy you want the GIF to be. Higher values result in smaller file sizes.",
        long_help = None,
        possible_values = ["20", "40", "60", "80"],
    )]
    lossy: Option<u32>,

    /// Determines whether the output should have its colors reduced to 256.
    /// Corresponds to the `--color reduce 256` argument in [Gifsicle](https://www.lcdf.org/gifsicle/).
    ///
    /// Reduce is implemeted only for [`Gif`]s.
    ///
    /// [`Gif`]: crate::utils::MediaType::Gif
    #[clap(
        short = 'r',
        long,
        help = "Reduce the number of distinct colors in each output GIF. Powered by Gifsicle",
        long_help = None,
    )]
    reduce: bool,
}

impl Cli {
    /// Returns the lossiness level.
    ///
    /// # Option
    /// Returns `None` if no lossiness was given.
    pub fn lossy(&self) -> Option<u32> {
        self.lossy
    }

    /// Returns a tuple of the input media's [`Path`] and [`Type`]
    ///
    /// # Result
    /// Returns an [`UnsupportedMediaFormat`] error if
    /// the input file is unsupported.
    ///
    /// [`UnsupportedMediaFormat`]: crate::error::ErrorKind::UnsupportedMediaFormat
    /// [`Path`]: std::path::Path
    /// [`Type`]: crate::utils::MediaType
    pub fn media(&self) -> Result<(PathBuf, MediaType)> {
        Ok((self.media.clone(), validate_format(&self.media)?))
    }

    /// Returns the name of the output media.
    ///
    /// # Result
    /// Returns an [`UnsupportedMediaFormat`] error if
    /// /// the input file is unsupported.
    ///
    /// [`UnsupportedMediaFormat`]: crate::error::ErrorKind::UnsupportedMediaFormat
    pub fn name(&self) -> Result<String> {
        let (_, ty) = self.media()?;
        let ext = match ty {
            MediaType::Mp4 => ".mp4",
            MediaType::Avi => ".avi",
            MediaType::Mkv => ".mkv",
            MediaType::Webm => ".webm",
            MediaType::Gif => ".gif",
        };
        match &self.output_name {
            Some(string) => {
                if !string.contains(ext) {
                    return Ok(format!("{}{}", string, ext));
                }
                Ok(string.to_owned())
            }
            None => Ok(format!("{}{}", random_name(), ext)),
        }
    }

    /// Returns the Optimization level of output.
    ///
    /// # Option
    /// Returns `None` if no optimization level was specified.
    pub fn opt_level(&self) -> Option<&String> {
        self.optimization.as_ref()
    }

    /// Returns the directory where the output should be saved.
    ///
    /// If the output directory was not specified, either of
    /// the following directories will be returned:
    ///
    /// - On Unix: `/home/<username>/Pictures`
    /// - On Windows: `<drive>:\Users\<username>\Pictures`
    ///
    /// # Result
    /// Returns an [`NotFound`] error if the default pictures folder was not found.
    ///
    /// [`NotFound`]: std::io::ErrorKind::NotFound  
    pub fn output(&self) -> Result<PathBuf> {
        match &self.output_directory {
            Some(output) => Ok(output.clone()),
            None => match dirs::picture_dir() {
                Some(path) => Ok(path),
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "the default pictures folder was not found",
                )
                .into()),
            },
        }
    }

    /// Returns true if force overwrite is enabled.
    pub fn overwrites(&self) -> bool {
        self.force_overwrite
    }

    /// Returns true if `--colors 256` is enabled.
    pub fn reduce(&self) -> bool {
        self.reduce
    }

    /// Returns the caption text with whitespace trimmed.
    pub fn text(&self) -> &str {
        self.caption.trim()
    }
}
