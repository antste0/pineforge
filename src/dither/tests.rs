use super::*;
use crate::color::rgb_to_oklab;
use crate::palette::Palette;

fn palette() -> Vec<([u8; 3], [f32; 3])> {
    Palette::R.colors().iter().map(|&c| (c, rgb_to_oklab(c))).collect()
}

fn in_palette(img: &image::RgbaImage, palette: &[([u8; 3], [f32; 3])]) -> bool {
    img.pixels().all(|p| palette.iter().any(|c| c.0 == [p.0[0], p.0[1], p.0[2]]))
}

#[test]
fn output_uses_palette_and_keeps_alpha() {
    let palette = palette();
    let mut img = image::RgbaImage::from_fn(32, 32, |x, y| {
        image::Rgba([(x * 8) as u8, (y * 8) as u8, 200, ((x + y) * 4) as u8])
    });
    let original = img.clone();
    dither(&mut img, &palette);

    assert!(in_palette(&img, &palette));
    for (out, orig) in img.pixels().zip(original.pixels()) {
        assert_eq!(out.0[3], orig.0[3]);
    }
}

// A palette color has zero error, so nothing should spread to the neighbors
#[test]
fn palette_colors_are_unchanged() {
    let palette = palette();
    let colors = Palette::R.colors();
    let mut img = image::RgbaImage::from_fn(15, 4, |x, _| {
        let [r, g, b] = colors[x as usize];
        image::Rgba([r, g, b, 255])
    });
    let original = img.clone();
    dither(&mut img, &palette);
    assert_eq!(img, original);
}

// The whole point of dithering is that an area averages out to the original color
// Plain gray is outside what the bluish palette can reach, so the input is a 50/50 mix of base and text
#[test]
fn average_color_is_preserved() {
    let palette = palette();
    let colors = Palette::R.colors();
    let wanted = [0, 1, 2].map(|c| (srgb_to_linear(colors[0][c]) + srgb_to_linear(colors[5][c])) / 2.0);
    // Closest 8-bit value to each mixed channel
    let input = wanted.map(|w| (0..=255u8).min_by(|&a, &b| (srgb_to_linear(a) - w).abs().total_cmp(&(srgb_to_linear(b) - w).abs())).unwrap());

    let mut img = image::RgbaImage::from_pixel(64, 64, image::Rgba([input[0], input[1], input[2], 255]));
    dither(&mut img, &palette);

    for c in 0..3 {
        let mean = img.pixels().map(|p| srgb_to_linear(p.0[c])).sum::<f32>() / img.pixels().len() as f32;
        let wanted = srgb_to_linear(input[c]);
        assert!((mean - wanted).abs() < 0.01, "channel {c}: {mean} vs {wanted}");
    }
}

// Fully transparent pixels must not push their error onto visible ones
#[test]
fn transparent_pixels_do_not_spread_error() {
    let palette = palette();
    let mut img = image::RgbaImage::from_pixel(2, 1, image::Rgba([0, 255, 0, 0]));
    img.put_pixel(1, 0, image::Rgba([0x19, 0x17, 0x24, 255]));
    dither(&mut img, &palette);
    assert_eq!(img.get_pixel(1, 0).0, [0x19, 0x17, 0x24, 255]);
}
