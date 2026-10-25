// Pixel values aren't proportional to brightness - 128 is about a fifth of 255, not half
// Colors have to be compared on real brightness, so this turns a 0-255 value into linear 0.0-1.0
// Very dark values are scaled linearly, the rest go through a 2.4 power curve
fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}


// In RGB, distance between two colors doesn't match how different they look, so the "nearest" palette color often looks wrong
// This maps a color into a space where it does: [lightness, green-red, blue-yellow]
// R, G, B are mixed into three channels (l, m, s), compressed with a cube root, then mixed again into the final three values
pub fn rgb_to_oklab(rgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(srgb_to_linear);

    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();

    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

// Squared distance - sqrt is skipped because it doesn't change which color is closest
fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

// Checks every palette color, with only 15 of them that's cheap enough
// Takes an already converted color, so dithering can pass in values with error added
// unwrap is safe because the palette is never empty
pub fn nearest_color(lab: [f32; 3], palette: &[([u8; 3], [f32; 3])]) -> ([u8; 3], [f32; 3]) {
    *palette
        .iter()
        .min_by(|x, y| color_distance(lab, x.1).total_cmp(&color_distance(lab, y.1)))
        .unwrap()
}
