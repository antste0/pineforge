use crate::color::{nearest_color, rgb_to_oklab};

// With only 15 colors, smooth gradients turn into flat bands
// To hide that, each pixel's error (wanted color minus picked color) is passed to neighbors that haven't been processed yet
// They lean the other way, so a small area averages out to the original color
pub fn dither(img: &mut image::RgbaImage, palette: &[([u8; 3], [f32; 3])]) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    // converted copy of the image, neighbors' errors accumulate here
    let mut buf: Vec<[f32; 3]> = img
        .pixels()
        .map(|p| rgb_to_oklab([p.0[0], p.0[1], p.0[2]]))
        .collect();

    for y in 0..h {
        for x in 0..w {
            let wanted = buf[y * w + x];
            let (rgb, lab) = nearest_color(wanted, palette);
            let err = [0, 1, 2].map(|i| wanted[i] - lab[i]);

            let pixel = img.get_pixel_mut(x as u32, y as u32);
            pixel.0 = [rgb[0], rgb[1], rgb[2], pixel.0[3]];

            // add a share of the error to the neighbor at (x + dx, y + dy), skipping ones outside the image
            let mut spread = |dx: isize, dy: usize, factor: f32| {
                let nx = x as isize + dx;
                let ny = y + dy;
                if nx < 0 || nx >= w as isize || ny >= h {
                    return;
                }
                let n = &mut buf[ny * w + nx as usize];
                for i in 0..3 {
                    n[i] += err[i] * factor;
                }
            };
            // weights sum to 1, so the whole error is passed on and none of it is lost
            spread(1, 0, 7.0 / 16.0);  // right
            spread(-1, 1, 3.0 / 16.0); // below left
            spread(0, 1, 5.0 / 16.0);  // below
            spread(1, 1, 1.0 / 16.0);  // below right
        }
    }
}
