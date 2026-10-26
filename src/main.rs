mod cli;
mod color;
mod dither;
mod palette;

use clap::Parser;
use image::{DynamicImage, ImageFormat, ImageReader};

use cli::Args;
use color::{map_colors, rgb_to_oklab};
use dither::dither;

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

    // remember whether the original had transparency
    let has_alpha = img.color().has_alpha();
    // normalize every input format to 8-bit RGBA
    let mut img = img.to_rgba8();

    if args.dither {
        dither(&mut img, &palette);
    } else {
        map_colors(&mut img, &palette);
    }

    // drop alpha when unsupported/unused
    let supports_alpha = !matches!(ImageFormat::from_path(&output), Ok(ImageFormat::Jpeg));
    if has_alpha && supports_alpha {
        img.save(&output)
    } else {
        DynamicImage::ImageRgba8(img).to_rgb8().save(&output)
    }
    .expect("failed to save image");
    println!("Saved to {}", output.display());
}
