mod cli;
mod color;
mod dither;
mod palette;

use clap::Parser;
use image::{DynamicImage, ImageFormat, ImageReader};

use cli::Args;
use color::{nearest_color, rgb_to_oklab};
use dither::dither;

fn main() {
    let args = Args::parse();
    // convert the palette once up front instead of on every comparison
    let palette = args.palette.colors().map(|c| (c, rgb_to_oklab(c)));

    let img = ImageReader::open(&args.input)
        .expect("failed to open input file")
        .decode()
        .expect("failed to decode image");
    // remember whether the original had transparency
    let has_alpha = img.color().has_alpha();
    // normalize every input format to 8-bit RGBA
    let mut img = img.to_rgba8();

    if args.dither {
        dither(&mut img, &palette);
    } else {
        // replace each pixel's color in place, leaving its alpha untouched
        for pixel in img.pixels_mut() {
            let [r, g, b, a] = pixel.0;
            let ([r, g, b], _) = nearest_color(rgb_to_oklab([r, g, b]), &palette);
            pixel.0 = [r, g, b, a];
        }
    }

    let output = args.output_path();
    // drop alpha when unsupported/unused
    let supports_alpha = !matches!(ImageFormat::from_path(&output), Ok(ImageFormat::Jpeg));
    let result = if has_alpha && supports_alpha {
        img.save(&output)
    } else {
        DynamicImage::ImageRgba8(img).to_rgb8().save(&output)
    };
    result.expect("failed to save image");
    println!("Saved to {}", output.display());
}
