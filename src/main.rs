mod cli;
mod color;
mod dither;
mod palette;
mod process;

use clap::Parser;
use image::ImageReader;

use cli::Args;
use color::rgb_to_oklab;

fn main() {
    let args = Args::parse();
    let output = args.output_path();
    // convert the palette once up front instead of on every comparison
    let palette = args.palette.colors().map(|c| (c, rgb_to_oklab(c)));

    let img = ImageReader::open(&args.input)
        .expect("failed to open input file")
        .decode()
        .expect("failed to decode image");
    println!("Processing {}", &args.input.display());

    if args.all {
        process::process_all(&img, args.dither, &output);
    } else {
        process::process_single_palette(&img, args.dither, &palette, &output);
    }
}
