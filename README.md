# Unlustig-rs
## iFunny GIF Caption maker

Make GIF Captions. Supercharge your discord posting with le epic ironic gifs. 

Supports both in-program captioning and env arguments.

Does **NOT** support mp4.

## Help: 
```
unlustig 0.0.6
blitzxd
iFunny GIF Caption maker, but this time, it's in Rust.

USAGE:
    unlustig.exe [OPTIONS]

OPTIONS:
    -G, --gif <GIF>          Path to the GIF file (without quotation marks)
    -h, --help               Print help information
    -o, --output <OUTPUT>    Set the name of the output file (default: out.gif)
    -T, --text <TEXT>        Your caption goes here.
    -V, --version            Print version information
```

## Building from source.
This program is made on Rust Edition 2021. 

The MSRV (Minimum Supported Rust Version) is: 1.56.1

For debugging purposes, omit the `--release` flag.
```
cargo build --release 
```

## License 
this repository is covered under the MIT license
