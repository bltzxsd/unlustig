# unlustig-rs


## iFunny GIF Caption maker

Make GIF Captions. Supercharge your discord posting with le epic ironic gifs. 

You can hover over fields to get help text.

Does **NOT** support mp4.


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
