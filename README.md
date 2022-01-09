# unlustig-rs

## iFunny GIF Caption maker

Make GIF Captions. Supercharge your discord posting with le epic ironic gifs. 

You can hover over input field names to get help text.
<img align="right" width="348" height="93" src="https://media.discordapp.net/attachments/834076909557645335/929745021148729364/unknown.png">

Does **NOT** support mp4.

## Example
![gif](https://media.discordapp.net/attachments/834076909557645335/929746951757496351/2VUqz.gif)
![gif2](https://media.discordapp.net/attachments/834076909557645335/929748427724701706/ezgif-2-5dbac32931.gif)

## Building from source.
This program is made on Rust Edition 2021. 

The MSRV (Minimum Supported Rust Version) is: 1.56.1

For debugging purposes, omit the `--release` flag.
```
cargo build --release 
```

## Dependencies

* gifsicle

Gifsicle is used for GIF compression. 

It is already packaged in the executable for windows. 
Linux users, if using compression, *must* have [gifsicle](https://www.lcdf.org/gifsicle/) installed and on their PATH.

## License 
this repository is covered under the MIT license
