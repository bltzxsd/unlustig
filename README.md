# ![screenshot](https://media.discordapp.net/attachments/690220522600267780/929755673011580938/unknown.png)

# iFunny GIF Caption maker

<img align="right" width="281" height="255" src="https://i.imgur.com/Gb3Aptm.gif">

Make GIF Captions. Supercharge your discord posting with le epic ironic gifs. 

You can even hover over input field names to get help text! 

Supported formats: 
- GIF
- MP4 
- AVI
- MKV 
- WebM

Optimization only works with GIF files.

## Downloads

You can download the latest release **[here](https://github.com/bltzxsd/unlustig/releases/latest)**.

## Examples
![gif](https://media.discordapp.net/attachments/834076909557645335/929746951757496351/2VUqz.gif)
![gif2](https://media.discordapp.net/attachments/834076909557645335/929748427724701706/ezgif-2-5dbac32931.gif)

## Building from source.
This program is made on Rust Edition 2021. 

The Minimum Supported Rust Version (MSRV) is: 1.56.1

For debugging purposes, omit the `--release` flag.
```
cargo build --release 
```

## Dependencies

* gifsicle
* FFmpeg 

Gifsicle is used for GIF compression. 

FFmpeg is used for video processing.

Both of them are already packaged in the Windows executable. 
Linux users, if using optimization flags or mp4, *must* have [gifsicle](https://www.lcdf.org/gifsicle/) and [FFmpeg](https://www.ffmpeg.org/) installed and on their PATH.

## License 
This repository is covered under the MIT license.
