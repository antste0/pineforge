use super::*;
use crate::palette::Palette;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
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
    assert!(approx(rgb_to_oklab([0, 0, 0])[0], 0.0));
    assert!(approx(rgb_to_oklab([255, 255, 255])[0], 1.0));
}

#[test]
fn oklab_grays_have_no_hue() {
    for v in [0, 32, 100, 180, 240, 255] {
        let [_, a, b] = rgb_to_oklab([v, v, v]);
        assert!(approx(a, 0.0) && approx(b, 0.0), "gray {v} -> a={a}, b={b}");
    }
}

// Reference values from the Oklab definition, a typo in any coefficient moves at least one of them
#[test]
fn oklab_matches_reference_values() {
    for (rgb, expected) in [
        ([255, 0, 0], [0.62796, 0.22486, 0.12585]),
        ([0, 255, 0], [0.86644, -0.23389, 0.17950]),
        ([0, 0, 255], [0.45201, -0.03246, -0.31153]),
    ] {
        let lab = rgb_to_oklab(rgb);
        // tighter than approx, a small typo can move a value by less than 1e-3
        assert!(
            (0..3).all(|c| (lab[c] - expected[c]).abs() < 1e-4),
            "{rgb:?} -> {lab:?}"
        );
    }
}

#[test]
fn nearest_color_of_palette_color_is_itself() {
    let palette = Palette::R.entries();
    for &(rgb, lab) in &palette {
        assert_eq!(nearest_color(lab, &palette).0, rgb);
    }
}

#[test]
fn nearest_color_extremes() {
    let palette = Palette::R.entries();
    assert_eq!(
        nearest_color(rgb_to_oklab([0, 0, 0]), &palette).0,
        [0x19, 0x17, 0x24]
    );
    assert_eq!(
        nearest_color(rgb_to_oklab([255, 255, 255]), &palette).0,
        [0xe0, 0xde, 0xf4]
    );
}

#[test]
fn map_colors_uses_palette_and_keeps_alpha() {
    let palette = Palette::R.entries();
    let mut img = image::RgbaImage::from_fn(16, 16, |x, y| {
        image::Rgba([(x * 16) as u8, (y * 16) as u8, 128, (x * y) as u8])
    });
    let original = img.clone();
    map_colors(&mut img, &palette);

    for (out, orig) in img.pixels().zip(original.pixels()) {
        let (nearest, _) = nearest_color(rgb_to_oklab([orig.0[0], orig.0[1], orig.0[2]]), &palette);
        assert_eq!([out.0[0], out.0[1], out.0[2]], nearest, "{orig:?}");
        assert_eq!(out.0[3], orig.0[3]);
    }
}

#[test]
fn oklab_round_trip() {
    for rgb in [
        [0, 0, 0],
        [255, 255, 255],
        [128, 64, 200],
        [12, 250, 90],
        [235, 111, 146],
    ] {
        assert_eq!(oklab_to_rgb(rgb_to_oklab(rgb)), rgb);
    }
}

// Mixing can leave the range a pixel can show, those channels have to end up at 0 or 255
#[test]
fn oklab_to_rgb_clamps_out_of_range() {
    assert_eq!(oklab_to_rgb([1.5, 0.0, 0.0]), [255, 255, 255]);
    assert_eq!(oklab_to_rgb([-0.5, 0.0, 0.0]), [0, 0, 0]);
    let [r, g, _] = oklab_to_rgb([0.6, 0.5, 0.0]);
    assert!(r == 255 && g == 0);
}
