# ![screenshot](https://media.discordapp.net/attachments/690220522600267780/929755673011580938/unknown.png)

# iFunny GIF Caption maker

<img align="right" width="281" height="255" src="https://i.imgur.com/Gb3Aptm.gif">

Make GIF Captions on your 'puter. 

Supported formats: 
- GIF
- MP4 
- AVI
- MKV 
- WebM
- mov

## Download and Install

You can download the latest release **[here](https://github.com/bltzxsd/unlustig/releases/latest)**.

- Windows users can install the program using the `msi` installer or use the portable executable.
- Arch users can download the `PKGBUILD` file and use an AUR helper. 
- Users on Debian (or any of its derivatives like Ubuntu, can use the `.deb` package file to install the program.  

## Examples
![gif](https://media.discordapp.net/attachments/834076909557645335/929746951757496351/2VUqz.gif)
![gif2](https://media.discordapp.net/attachments/834076909557645335/929748427724701706/ezgif-2-5dbac32931.gif)

## Building from source.
This program is built with Rust Edition 2021. 

The Minimum Supported Rust Version (MSRV) is: 1.58.1

For debugging purposes, omit the `--release` flag.
```
cargo build --release 
```

## Dependencies

* gifsicle
* FFmpeg 

Gifsicle is used for GIF compression. 

FFmpeg is used for video processing.

If Windows users do not have a dependency installed, the program will automatically download it.
Linux users, if using optimization flags or mp4 media, *must* have [gifsicle](https://www.lcdf.org/gifsicle/) and/or [FFmpeg](https://www.ffmpeg.org/) installed and on their PATH.

## Contributing
As this project is mostly a one dev project, all contributions are welcome.

### License 
This repository is covered under the MIT license.
