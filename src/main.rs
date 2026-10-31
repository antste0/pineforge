mod cli;
mod color;
mod dither;
mod palette;
mod process;

use clap::Parser;
use image::ImageReader;

use cli::Args;

fn main() {
    let args = Args::parse();

    let img = ImageReader::open(&args.input)
        .unwrap_or_else(|e| fail(&format!("failed to open {}: {e}", args.input.display())))
        .decode()
        .unwrap_or_else(|e| fail(&format!("failed to decode {}: {e}", args.input.display())));
    println!("Processing {}", args.input.display());

    if args.all {
        process::process_all(&img, args.dither, &args.base_path());
    } else {
        process::process_single_palette(
            &img,
            args.dither,
            &args.palette.entries(),
            &args.output_path(),
        );
    }
}

// print a plain error instead of a panic message with source locations
pub fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1)
}
