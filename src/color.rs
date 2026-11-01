use std::sync::LazyLock;

use rayon::prelude::*;

use crate::palette::PaletteEntry;

// Pixel values aren't proportional to brightness - 128 is about a fifth of 255, not half
// Colors have to be compared on real brightness, so this turns a 0-255 value into linear 0.0-1.0
// Very dark values are scaled linearly, the rest go through a 2.4 power curve
// powf is slow and there are only 256 possible inputs, so every result is computed once and looked up afterwards
pub fn srgb_to_linear(c: u8) -> f32 {
    static LUT: LazyLock<[f32; 256]> =
        LazyLock::new(|| std::array::from_fn(|i| compute_srgb_to_linear(i as u8)));
    LUT[c as usize]
}

fn compute_srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

// In RGB, distance between two colors doesn't match how different they look, so the "nearest" palette color often looks wrong
// This maps a color into a space where it does: [lightness, green-red, blue-yellow]
pub fn rgb_to_oklab(rgb: [u8; 3]) -> [f32; 3] {
    linear_to_oklab(rgb.map(srgb_to_linear))
}

// Same as rgb_to_oklab, for colors already converted with srgb_to_linear
#[allow(clippy::excessive_precision)]
pub fn linear_to_oklab([r, g, b]: [f32; 3]) -> [f32; 3] {
    // l, m and s are cone-like responses (long, medium, short wavelengths), they are intermediate values and not lightness or hue
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();

    // turn the cone responses into lightness and the two opposing color axes
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

// Reverse of rgb_to_oklab, for filters that produce colors outside the palette
// Results outside what a pixel can show are clamped per channel
pub fn oklab_to_rgb(lab: [f32; 3]) -> [u8; 3] {
    oklab_to_linear(lab).map(linear_to_srgb)
}

#[allow(clippy::excessive_precision)]
fn oklab_to_linear([l, a, b]: [f32; 3]) -> [f32; 3] {
    let l_ = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m_ = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s_ = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    [
        4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
        -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
        -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_,
    ]
}

fn linear_to_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

// Squared distance - sqrt is skipped because it doesn't change which color is closest
pub fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

// Checks every palette color, with only 15 of them that's cheap enough
// Takes an already converted color, so dithering can pass in values with error added
// unwrap is safe because the palette is never empty
// Distances are computed up front, comparing them inside min_by would compute each one twice
pub fn nearest_color(lab: [f32; 3], palette: &[PaletteEntry]) -> PaletteEntry {
    *palette
        .iter()
        .map(|p| (color_distance(lab, p.1), p))
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .unwrap()
        .1
}

// Replaces each pixel's color in place, leaving its alpha untouched
// Every pixel is independent here, so they're split across threads (dithering can't do this, each pixel depends on the previous ones)
pub fn map_colors(img: &mut image::RgbaImage, palette: &[PaletteEntry]) {
    img.par_chunks_mut(4).for_each(|pixel| {
        let ([r, g, b], _) = nearest_color(rgb_to_oklab([pixel[0], pixel[1], pixel[2]]), palette);
        pixel[..3].copy_from_slice(&[r, g, b]);
    });
}

#[cfg(test)]
mod tests;
