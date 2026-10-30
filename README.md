# pineforge
Transform any image to fit the Rose Pine palette, built in Rust

## Examples
<table>
  <tr>
    <td align="center" width="25%"><img src="./thumbnails/pexels-daniqux-30223875.jpg" width="100%" alt="Original"><br>Original</td>
    <td align="center" width="25%"><img src="./thumbnails/pexels-daniqux-30223875_rosepine.jpg" width="100%" alt="Rose Pine"><br>Rose Pine</td>
    <td align="center" width="25%"><img src="./thumbnails/pexels-daniqux-30223875_dawn.jpg" width="100%" alt="Rose Pine Dawn"><br>Rose Pine Dawn</td>
    <td align="center" width="25%"><img src="./thumbnails/pexels-daniqux-30223875_moon.jpg" width="100%" alt="Rose Pine Moon"><br>Rose Pine Moon</td>
  </tr>
</table>

[image used](https://www.pexels.com/photo/historic-architecture-and-clock-tower-in-ghent-30223875/) by Danique Veldhuis

The images in `examples/` and `thumbnails/` are from Pexels and are licensed under the [Pexels License](https://www.pexels.com/license/), not GPL.

## Features
- All the official Rose Pine palettes (Rose Pine, Moon, Dawn)
- Multithreaded for quick processing of large images
- Ability to transform into all three palettes at once

## Features to add
- Mixing colors from the palette with the one from the original image
- Bulk transform
    - Multiple files as input
    - Folders/directories as inputs with optional recursion

## Installation
Requires a [Rust toolchain](https://rustup.rs/), version 1.85 or newer.

```sh
git clone https://github.com/antste0/pineforge
cd pineforge
cargo build --release
```

The binary ends up in `target/release/pineforge`.

### Older CPUs (pre-2015)
On x86-64 the build targets CPUs from about 2015 onward (Intel Haswell, AMD Zen), which makes dithering about 2x faster. This however makes older CPUs unable to build or run the app. Build it for a generic x86-64 CPU instead:

```sh
RUSTFLAGS="-C target-cpu=x86-64" cargo build --release
```

## Usage
The examples assume `pineforge` is on your `PATH`.
Otherwise run `./target/release/pineforge`, or install it with `cargo install --path .`.

```sh
pineforge -i photo.jpg
pineforge -i photo.jpg -o out.png -p m
pineforge -i logo.png --no-dither
pineforge -i photo.jpg --all
```

Alternatively, use the `cargo run --release` command with the flags you want.

```sh
cargo run --release -- -i photo.jpg
```


| Option | Description |
| --- | --- |
| `-i`, `--input <file>` | Image to transform (required) |
| `-o`, `--output <file>` | Output file, defaults to `<name>_rosepine.<ext>` next to the input |
| `-p`, `--palette <r\|m\|d>` | `r` Rose Pine (default), `m` Moon, `d` Dawn |
| `--no-dither` | Disable Floyd-Steinberg dithering, better for flat graphics, logos and pixel art |
| `--all` | Get all 3 palettes as an output |

`--all` writes `<name>_rosepine_main.<ext>`, `<name>_rosepine_moon.<ext>` and `<name>_rosepine_dawn.<ext>`. Mutually exclusive with `-p`.
With `-o`, the palette name is added to the given file name, so `-o out.png` gives `out_main.png`, `out_moon.png` and `out_dawn.png`.
