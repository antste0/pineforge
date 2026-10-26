use super::*;
use crate::palette::Palette;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

fn palette() -> Vec<([u8; 3], [f32; 3])> {
    Palette::R.colors().iter().map(|&c| (c, rgb_to_oklab(c))).collect()
}

#[test]
fn srgb_to_linear_endpoints() {
    assert_eq!(srgb_to_linear(0), 0.0);
    assert!(approx(srgb_to_linear(255), 1.0));
    // Mid gray is far darker than half in linear light
    assert!(approx(srgb_to_linear(128), 0.2158));
}

#[test]
fn srgb_to_linear_lut_matches_formula() {
    for c in 0..=255u8 {
        assert_eq!(srgb_to_linear(c), compute_srgb_to_linear(c));
    }
}

#[test]
fn oklab_black_and_white() {
    let [l, a, b] = rgb_to_oklab([0, 0, 0]);
    assert!(approx(l, 0.0) && approx(a, 0.0) && approx(b, 0.0));
    let [l, a, b] = rgb_to_oklab([255, 255, 255]);
    assert!(approx(l, 1.0) && approx(a, 0.0) && approx(b, 0.0));
}

#[test]
fn oklab_grays_have_no_hue() {
    for v in [32, 100, 180, 240] {
        let [_, a, b] = rgb_to_oklab([v, v, v]);
        assert!(approx(a, 0.0) && approx(b, 0.0), "gray {v} -> a={a}, b={b}");
    }
}

#[test]
fn oklab_red_and_blue_hues() {
    // Positive a is red, negative b is blue
    assert!(rgb_to_oklab([255, 0, 0])[1] > 0.1);
    assert!(rgb_to_oklab([0, 0, 255])[2] < -0.1);
}

#[test]
fn nearest_color_of_palette_color_is_itself() {
    let palette = palette();
    for &(rgb, lab) in &palette {
        assert_eq!(nearest_color(lab, &palette).0, rgb);
    }
}

#[test]
fn nearest_color_extremes() {
    let palette = palette();
    assert_eq!(nearest_color(rgb_to_oklab([0, 0, 0]), &palette).0, [0x19, 0x17, 0x24]);
    assert_eq!(nearest_color(rgb_to_oklab([255, 255, 255]), &palette).0, [0xe0, 0xde, 0xf4]);
}

#[test]
fn map_colors_uses_palette_and_keeps_alpha() {
    let palette = palette();
    let mut img = image::RgbaImage::from_fn(16, 16, |x, y| {
        image::Rgba([(x * 16) as u8, (y * 16) as u8, 128, (x * y) as u8])
    });
    let original = img.clone();
    map_colors(&mut img, &palette);

    for (out, orig) in img.pixels().zip(original.pixels()) {
        let rgb = [out.0[0], out.0[1], out.0[2]];
        assert!(palette.iter().any(|c| c.0 == rgb), "{rgb:?} not in palette");
        assert_eq!(out.0[3], orig.0[3]);
    }
}
