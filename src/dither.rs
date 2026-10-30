use crate::color::{linear_to_oklab, nearest_color, srgb_to_linear};

// With only 15 colors, smooth gradients turn into flat bands
// To hide that, each pixel's error (wanted color minus picked color) is passed to neighbors that haven't been processed yet
// They lean the other way, so a small area averages out to the original color
pub fn dither(img: &mut image::RgbaImage, palette: &[([u8; 3], [f32; 3])]) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    // Neighboring dots blend in the eye by real brightness, so the error is tracked in linear RGB
    // Picking the nearest color still happens in the perceptual space
    let mut buf: Vec<[f32; 3]> = img
        .pixels()
        .map(|p| [p.0[0], p.0[1], p.0[2]].map(srgb_to_linear))
        .collect();

    for y in 0..h {
        // Every other row goes right to left, otherwise the error always flows the same way and forms diagonal "worm" patterns
        let reversed = y % 2 == 1;
        // dx is flipped with the row direction, so "right" always means "next pixel in this row"
        let dir: isize = if reversed { -1 } else { 1 };

        for i in 0..w {
            let x = if reversed { w - 1 - i } else { i };

            // The palette can't reach some colors (e.g. bright green), so for those pixels the error never shrinks and would keep piling up into blotches and streaks
            // Clamping keeps it within what a pixel can show
            let wanted = buf[y * w + x].map(|c| c.clamp(0.0, 1.0));
            let (rgb, _) = nearest_color(linear_to_oklab(wanted), palette);

            let pixel = img.get_pixel_mut(x as u32, y as u32);
            pixel.0 = [rgb[0], rgb[1], rgb[2], pixel.0[3]];

            // the error is weighted by alpha, so pixels that are barely visible don't push their error onto visible neighbors
            let weight = pixel.0[3] as f32 / 255.0;
            let picked = rgb.map(srgb_to_linear);
            let err = [0, 1, 2].map(|c| (wanted[c] - picked[c]) * weight);

            // add a share of the error to the neighbor at (x + dx * dir, y + dy), skipping ones outside the image
            let mut spread = |dx: isize, dy: usize, factor: f32| {
                let nx = x as isize + dx * dir;
                let ny = y + dy;
                if nx < 0 || nx >= w as isize || ny >= h {
                    return;
                }
                let n = &mut buf[ny * w + nx as usize];
                for c in 0..3 {
                    n[c] += err[c] * factor;
                }
            };
            spread(1, 0, 7.0 / 16.0);  // next in row
            spread(-1, 1, 3.0 / 16.0); // below, behind
            spread(0, 1, 5.0 / 16.0);  // below
            spread(1, 1, 1.0 / 16.0);  // below, ahead
        }
    }
}

#[cfg(test)]
mod tests;
