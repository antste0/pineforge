use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

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
// Every pixel is independent here, so it can go through map_unique_colors (dithering can't, each pixel depends on the previous ones)
pub fn map_colors(img: &mut image::RgbaImage, palette: &[PaletteEntry]) {
    map_unique_colors(img, |rgb| nearest_color(rgb_to_oklab(rgb), palette).0);
}

// Replaces the RGB of every pixel with f(RGB), leaving alpha untouched
// The result depends only on the RGB value, and even large photos have about 1-10% unique colors, so each one is computed once
// A table indexed by the whole 0xRRGGBB value makes every lookup a single read, unlike sorting or hashing
// The table is 48 MB, but it starts zeroed so the OS only backs the pages that actually get written
pub fn map_unique_colors(img: &mut image::RgbaImage, f: impl Fn([u8; 3]) -> [u8; 3] + Sync) {
    let key = |p: &[u8]| (p[0] as usize) << 16 | (p[1] as usize) << 8 | p[2] as usize;

    // One bit per possible color, set by many threads at once so it has to be atomic
    let seen: Vec<AtomicU64> = (0..1 << 18).map(|_| AtomicU64::new(0)).collect();
    img.par_chunks(4).for_each(|p| {
        let k = key(p);
        seen[k >> 6].fetch_or(1 << (k & 63), Ordering::Relaxed);
    });

    // Each bitmap word covers 64 neighboring table entries, so threads never write to the same chunk
    let mut table = vec![[0u8; 3]; 1 << 24];
    table
        .par_chunks_mut(64)
        .zip(&seen)
        .enumerate()
        .for_each(|(word, (chunk, bits))| {
            let mut bits = bits.load(Ordering::Relaxed);
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                let k = word << 6 | bit;
                chunk[bit] = f([(k >> 16) as u8, (k >> 8) as u8, k as u8]);
                // Clears the lowest set bit
                bits &= bits - 1;
            }
        });

    img.par_chunks_mut(4).for_each(|p| {
        let rgb = table[key(p)];
        p[..3].copy_from_slice(&rgb);
    });
}

#[cfg(test)]
mod tests;
