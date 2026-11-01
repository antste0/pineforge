use super::*;
use crate::color::map_colors;
use crate::palette::Palette;

const FULL: Mix = Mix {
    strength: 1.0,
    keep_lightness: false,
};

fn gradient() -> image::RgbaImage {
    image::RgbaImage::from_fn(32, 32, |x, y| {
        image::Rgba([(x * 8) as u8, (y * 8) as u8, 120, ((x + y) * 4) as u8])
    })
}

// The round trip through Oklab can be off by one per channel
fn close(a: &image::RgbaImage, b: &image::RgbaImage) -> bool {
    a.pixels()
        .zip(b.pixels())
        .all(|(p, q)| (0..4).all(|c| p.0[c].abs_diff(q.0[c]) <= 1))
}

#[test]
fn zero_strength_keeps_the_image() {
    let palette = Palette::R.entries();
    let mix = Mix {
        strength: 0.0,
        keep_lightness: false,
    };
    let original = gradient();

    let mut img = original.clone();
    soft_map(&mut img, &palette, 0.01, mix);
    assert!(close(&img, &original));
}

#[test]
fn keeps_alpha() {
    let palette = Palette::M.entries();
    let original = gradient();
    let mut img = original.clone();
    soft_map(&mut img, &palette, 0.01, FULL);

    for (out, orig) in img.pixels().zip(original.pixels()) {
        assert_eq!(out.0[3], orig.0[3]);
    }
}

// With almost no softness only the nearest color has any weight left
#[test]
fn tiny_softness_matches_quantizing() {
    let palette = Palette::R.entries();
    let mut soft = gradient();
    soft_map(&mut soft, &palette, 1e-6, FULL);
    let mut quantized = gradient();
    map_colors(&mut quantized, &palette);
    assert!(close(&soft, &quantized));
}

#[test]
fn soft_is_not_limited_to_the_palette() {
    let palette = Palette::R.entries();
    let mut img = gradient();
    soft_map(&mut img, &palette, 0.01, FULL);

    let mut colors: Vec<[u8; 3]> = img.pixels().map(|p| [p.0[0], p.0[1], p.0[2]]).collect();
    colors.sort();
    colors.dedup();
    assert!(colors.len() > 15, "only {} colors", colors.len());
}

// Colors that are far from the whole palette must not turn into NaN
#[test]
fn soft_handles_colors_far_from_the_palette() {
    let palette = Palette::D.entries();
    let lab = soft_target(rgb_to_oklab([0, 255, 0]), &palette, 1e-4);
    assert!(lab.iter().all(|c| c.is_finite()));
}

#[test]
fn keep_lightness_keeps_lightness() {
    let palette = Palette::R.entries();
    let mix = Mix {
        strength: 1.0,
        keep_lightness: true,
    };
    // grays stay inside what a pixel can show after tinting, so clamping doesn't move them
    let original = image::RgbaImage::from_fn(8, 1, |x, _| {
        let v = 60 + x as u8 * 20;
        image::Rgba([v, v, v, 255])
    });

    let mut img = original.clone();
    soft_map(&mut img, &palette, 0.01, mix);

    for (out, orig) in img.pixels().zip(original.pixels()) {
        let lab = |p: &image::Rgba<u8>| rgb_to_oklab([p.0[0], p.0[1], p.0[2]]);
        let ([l, a, b], [l0, _, _]) = (lab(out), lab(orig));
        assert!((l - l0).abs() < 0.01, "{out:?} vs {orig:?}");
        // the grays have no hue, so a pixel that wasn't tinted at all would also keep its lightness
        assert!(a.hypot(b) > 0.01, "{out:?} was not tinted");
    }
}

// Each pixel moves along the line toward its target, so a stronger mix lands further from the original
#[test]
fn stronger_mix_moves_further() {
    let palette = Palette::R.entries();
    let original = gradient();
    let moved = |strength| {
        let mut img = original.clone();
        let mix = Mix {
            strength,
            keep_lightness: false,
        };
        soft_map(&mut img, &palette, 0.02, mix);
        img.pixels()
            .zip(original.pixels())
            .map(|(p, q)| {
                let lab = |p: &image::Rgba<u8>| rgb_to_oklab([p.0[0], p.0[1], p.0[2]]);
                color_distance(lab(p), lab(q))
            })
            .sum::<f32>()
    };
    let (low, mid, high) = (moved(0.3), moved(0.6), moved(1.0));
    assert!(low < mid && mid < high, "{low} {mid} {high}");
}

// With huge softness every palette color weighs the same, which is why the README warns about grayer results
#[test]
fn huge_softness_gives_the_palette_average() {
    let palette = Palette::R.entries();
    let n = palette.len() as f32;
    let mean = palette
        .iter()
        .fold([0.0; 3], |acc, p| [0, 1, 2].map(|c| acc[c] + p.1[c] / n));

    for rgb in [[0, 0, 0], [255, 255, 255], [200, 30, 40]] {
        let lab = soft_target(rgb_to_oklab(rgb), &palette, 1e6);
        assert!(
            (0..3).all(|c| (lab[c] - mean[c]).abs() < 1e-4),
            "{rgb:?} -> {lab:?}"
        );
    }
}
