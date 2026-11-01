use std::path::Path;

use image::{DynamicImage, ImageFormat};

use crate::color::map_colors;
use crate::dither::dither;
use crate::filter::{Mix, soft_map};
use crate::palette::{Palette, PaletteEntry};

// Quantize limits the output to palette colors, Soft only tints the image toward them
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    Quantize { dither: bool },
    Soft { softness: f32, mix: Mix },
}

pub fn process_single_palette(
    img: &DynamicImage,
    method: Method,
    palette: &[PaletteEntry],
    output: &Path,
) {
    // remember whether the original had transparency
    let has_alpha = img.color().has_alpha();
    // normalize every input format to 8-bit RGBA
    let mut img = img.to_rgba8();

    match method {
        Method::Quantize { dither: true } => dither(&mut img, palette),
        Method::Quantize { dither: false } => map_colors(&mut img, palette),
        Method::Soft { softness, mix } => soft_map(&mut img, palette, softness, mix),
    }

    // drop alpha when unsupported/unused
    let supports_alpha = !matches!(ImageFormat::from_path(output), Ok(ImageFormat::Jpeg));
    if has_alpha && supports_alpha {
        img.save(output)
    } else {
        DynamicImage::ImageRgba8(img).to_rgb8().save(output)
    }
    .unwrap_or_else(|e| crate::fail(&format!("failed to save {}: {e}", output.display())));
    println!("Saved to {}", output.display());
}

pub fn process_all(img: &DynamicImage, method: Method, output: &Path) {
    let stem = output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let extension = output.extension().and_then(|e| e.to_str()).unwrap_or("png");

    // dithering is sequential within one image, so the parallelism is across palettes
    std::thread::scope(|s| {
        for variant in Palette::ALL {
            s.spawn(move || {
                let path = output
                    .with_file_name(format!("{stem}_rosepine_{}.{extension}", variant.suffix()));
                process_single_palette(img, method, &variant.entries(), &path);
            });
        }
    });
}
