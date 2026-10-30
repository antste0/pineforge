use super::*;

fn parse(args: &[&str]) -> Args {
    Args::try_parse_from(std::iter::once("pineforge").chain(args.iter().copied())).unwrap()
}

#[test]
fn output_path_next_to_input() {
    let args = parse(&["-i", "dir/sub/photo.jpg"]);
    assert_eq!(args.output_path(), PathBuf::from("dir/sub/photo_rosepine.jpg"));
}

#[test]
fn output_path_without_extension_falls_back_to_png() {
    let args = parse(&["-i", "photo"]);
    assert_eq!(args.output_path(), PathBuf::from("photo_rosepine.png"));
}

// SetFalse is what makes dithering default to on, the flag only turns it off
#[test]
fn dither_is_on_unless_disabled() {
    assert!(parse(&["-i", "photo.png"]).dither);
    assert!(!parse(&["-i", "photo.png", "--no-dither"]).dither);
}
