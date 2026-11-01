use rayon::prelude::*;

use crate::color::{color_distance, oklab_to_rgb, rgb_to_oklab};
use crate::palette::PaletteEntry;

// Controls how much of the palette color ends up in the output
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mix {
    pub strength: f32,
    pub keep_lightness: bool,
}

// Each palette color pulls the pixel toward itself, closer ones much harder, so the result moves smoothly between palette colors instead of jumping
// Alpha is left untouched
pub fn soft_map(img: &mut image::RgbaImage, palette: &[PaletteEntry], softness: f32, mix: Mix) {
    // Index 0 is lightness, skipping it changes only hue and saturation
    let first = if mix.keep_lightness { 1 } else { 0 };
    img.par_chunks_mut(4).for_each(|pixel| {
        let orig = rgb_to_oklab([pixel[0], pixel[1], pixel[2]]);
        let target = soft_target(orig, palette, softness);
        let mut out = orig;
        for c in first..3 {
            out[c] += (target[c] - orig[c]) * mix.strength;
        }
        pixel[..3].copy_from_slice(&oklab_to_rgb(out));
    });
}

fn soft_target(lab: [f32; 3], palette: &[PaletteEntry], softness: f32) -> [f32; 3] {
    // Far from every palette color all weights would underflow to 0 and the division would give NaN
    // Measuring from the closest color keeps its weight at 1, the normalization cancels the shift out
    let min = palette
        .iter()
        .map(|p| color_distance(lab, p.1))
        .fold(f32::INFINITY, f32::min);

    let mut sum = [0.0; 3];
    let mut total = 0.0;
    for p in palette {
        let w = (-(color_distance(lab, p.1) - min) / softness).exp();
        for (s, v) in sum.iter_mut().zip(p.1) {
            *s += v * w;
        }
        total += w;
    }
    sum.map(|c| c / total)
}

#[cfg(test)]
mod tests;
