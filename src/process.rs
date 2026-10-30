use std::path::Path;

use image::{DynamicImage, ImageFormat};

use crate::color::{map_colors, rgb_to_oklab};
use crate::dither::dither;
use crate::palette::Palette;

pub fn process_single_palette(img: &DynamicImage, has_dither: bool, palette: &[([u8; 3], [f32; 3])], output: &Path) {
    // remember whether the original had transparency
    let has_alpha = img.color().has_alpha();
    // normalize every input format to 8-bit RGBA
    let mut img = img.to_rgba8();

    if has_dither {
        dither(&mut img, palette);
    } else {
        map_colors(&mut img, palette);
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

pub fn process_all(img: &DynamicImage, has_dither: bool, output: &Path) {
    let stem = output.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let extension = output.extension().and_then(|e| e.to_str()).unwrap_or("png");

    // dithering is sequential within one image, so the parallelism is across palettes
    std::thread::scope(|s| {
        for variant in Palette::ALL {
            s.spawn(move || {
                let palette = variant.colors().map(|c| (c, rgb_to_oklab(c)));
                let path = output.with_file_name(format!("{stem}_rosepine_{}.{extension}", variant.suffix()));
                process_single_palette(img, has_dither, &palette, &path);
            });
        }
    });
}
