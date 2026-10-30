use std::path::{Path, PathBuf};
use std::process::Command;

// Each test gets its own directory, so they can run in parallel
fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_pineforge")).args(args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}

fn gradient(alpha: bool) -> image::RgbaImage {
    image::RgbaImage::from_fn(20, 10, |x, y| {
        let a = if alpha { (x * 12) as u8 } else { 255 };
        image::Rgba([(x * 12) as u8, (y * 25) as u8, 90, a])
    })
}

#[test]
fn default_output_name_and_size() {
    let dir = temp_dir("default_output");
    let input = dir.join("in.png");
    gradient(false).save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str()]);

    let out = image::open(dir.join("in_rosepine.png")).unwrap();
    assert_eq!((out.width(), out.height()), (20, 10));
}

#[test]
fn png_keeps_alpha() {
    let dir = temp_dir("png_alpha");
    let input = dir.join("in.png");
    let output = dir.join("out.png");
    let img = gradient(true);
    img.save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), output.as_os_str()]);

    let result = image::open(&output).unwrap();
    assert!(result.color().has_alpha());
    let result = result.to_rgba8();
    for (out, orig) in result.pixels().zip(img.pixels()) {
        assert_eq!(out.0[3], orig.0[3]);
    }
}

#[test]
fn opaque_input_drops_alpha() {
    let dir = temp_dir("opaque");
    let input = dir.join("in.jpg");
    let output = dir.join("out.png");
    image::DynamicImage::ImageRgba8(gradient(false)).to_rgb8().save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), output.as_os_str()]);

    assert!(!image::open(&output).unwrap().color().has_alpha());
}

#[test]
fn jpeg_output_from_transparent_input() {
    let dir = temp_dir("jpeg_out");
    let input = dir.join("in.png");
    let output = dir.join("out.jpg");
    gradient(true).save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), output.as_os_str()]);

    assert!(!image::open(&output).unwrap().color().has_alpha());
}

#[test]
fn no_dither_maps_to_single_color() {
    let dir = temp_dir("no_dither");
    let input = dir.join("in.png");
    let output = dir.join("out.png");
    image::RgbaImage::from_pixel(8, 8, image::Rgba([128, 128, 128, 255])).save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), output.as_os_str(), "--no-dither".as_ref(), "-p".as_ref(), "m".as_ref()]);

    let out = image::open(&output).unwrap().to_rgb8();
    let first = out.get_pixel(0, 0);
    assert!(out.pixels().all(|p| p == first));
}

#[test]
fn missing_input_fails() {
    let dir = temp_dir("missing");
    let status = Command::new(env!("CARGO_BIN_EXE_pineforge"))
        .args(["-i".as_ref(), dir.join("nope.png").as_os_str()])
        .output()
        .unwrap()
        .status;
    assert!(!status.success());
}

#[test]
fn all_writes_every_palette_variant() {
    let dir = temp_dir("all_variants");
    let input = dir.join("in.png");
    gradient(false).save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "--all".as_ref()]);

    // each variant must be identical to running that palette on its own
    for (suffix, palette) in [("main", "r"), ("moon", "m"), ("dawn", "d")] {
        let single = dir.join(format!("single_{suffix}.png"));
        run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), single.as_os_str(), "-p".as_ref(), palette.as_ref()]);

        let all = image::open(dir.join(format!("in_rosepine_{suffix}.png"))).unwrap().to_rgb8();
        assert_eq!((all.width(), all.height()), (20, 10));
        assert_eq!(all, image::open(&single).unwrap().to_rgb8(), "{suffix} differs from -p {palette}");
    }
}

#[test]
fn all_with_output_uses_output_name() {
    let dir = temp_dir("all_with_output");
    let input = dir.join("in.png");
    let output = dir.join("out.png");
    gradient(false).save(&input).unwrap();

    run(&["-i".as_ref(), input.as_os_str(), "-o".as_ref(), output.as_os_str(), "--all".as_ref()]);

    for suffix in ["main", "moon", "dawn"] {
        assert!(dir.join(format!("out_rosepine_{suffix}.png")).exists(), "missing out_rosepine_{suffix}.png");
    }
    assert!(!output.exists());
}

#[test]
fn all_with_palette_fails() {
    let dir = temp_dir("all_with_palette");
    let input = dir.join("in.png");
    gradient(false).save(&input).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_pineforge"))
        .args(["-i".as_ref(), input.as_os_str(), "--all".as_ref(), "-p".as_ref(), "m".as_ref()])
        .output()
        .unwrap();

    // the input is valid, so the only reason to fail is the conflicting flags
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot be used with"));
    assert!(!dir.join("in_rosepine_moon.png").exists());
}
