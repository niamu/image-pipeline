# image-pipeline

This is a [Rust](https://www.rust-lang.org) CLI tool used to compress and scale images into WebP.

## Usage

Using the compiled binary:

```
$ image-pipeline --help
image-pipeline 0.2.0

USAGE:
    image-pipeline [FLAGS] [OPTIONS] --src <input-dir> --dest <output-dir>

FLAGS:
        --help             Prints help information
        --scale-contain    Maintain aspect ratio and scale image to contain within <width> and <height>
    -V, --version          Prints version information

OPTIONS:
    -h, --height <height>       [default: 600]
        --src <input-dir>
        --dest <output-dir>
    -w, --width <width>         [default: 430]
```
