use clap::error::ErrorKind;

use super::*;
use crate::filter::Mix;
use crate::process::Method;

fn try_parse(args: &[&str]) -> Result<Args, clap::Error> {
    Args::try_parse_from(std::iter::once("pineforge").chain(args.iter().copied()))
}

fn parse(args: &[&str]) -> Args {
    try_parse(args).unwrap()
}

// Returns the kind, so a test can't pass because of an unrelated error like a misspelled flag
fn error_kind(args: &[&str]) -> ErrorKind {
    try_parse(args).unwrap_err().kind()
}

#[test]
fn output_path_next_to_input() {
    let args = parse(&["-i", "dir/sub/photo.jpg"]);
    assert_eq!(
        args.output_path(),
        PathBuf::from("dir/sub/photo_rosepine.jpg")
    );
}

#[test]
fn output_path_without_extension_falls_back_to_png() {
    let args = parse(&["-i", "photo"]);
    assert_eq!(args.output_path(), PathBuf::from("photo_rosepine.png"));
}

#[test]
fn method_defaults_to_soft() {
    assert_eq!(
        parse(&["-i", "p.png"]).method(),
        Method::Soft {
            softness: 0.02,
            mix: Mix {
                strength: 0.6,
                keep_lightness: false
            }
        }
    );
}

#[test]
fn method_variants() {
    assert_eq!(
        parse(&["-i", "p.png", "--softness", "0.05", "--strength", "0.5"]).method(),
        Method::Soft {
            softness: 0.05,
            mix: Mix {
                strength: 0.5,
                keep_lightness: false
            }
        }
    );
    assert_eq!(
        parse(&["-i", "p.png", "--keep-lightness"]).method(),
        Method::Soft {
            softness: 0.02,
            mix: Mix {
                strength: 0.6,
                keep_lightness: true
            }
        }
    );
    assert_eq!(
        parse(&["-i", "p.png", "--no-soft"]).method(),
        Method::Quantize { dither: true }
    );
    assert_eq!(
        parse(&["-i", "p.png", "--no-soft", "--no-dither"]).method(),
        Method::Quantize { dither: false }
    );
    assert_eq!(
        parse(&["-i", "p.png", "--no-dither"]).method(),
        Method::Quantize { dither: false }
    );
}

#[test]
fn filter_flags_reject_bad_combinations() {
    // these only mean something when tinting, and --no-dither turns tinting off too
    for off in ["--no-soft", "--no-dither"] {
        for flag in [
            &["--strength", "0.5"][..],
            &["--keep-lightness"],
            &["--softness", "0.1"],
        ] {
            let args = [&["-i", "p.png", off], flag].concat();
            assert_eq!(error_kind(&args), ErrorKind::ArgumentConflict, "{args:?}");
        }
    }
}

#[test]
fn filter_values_are_validated() {
    // with a space clap would read -0.1 as an unknown flag and never check the range
    for value in [
        "--strength=1.5",
        "--strength=-0.1",
        "--strength=nan",
        "--softness=0",
        "--softness=nan",
        "--softness=inf",
    ] {
        let args = ["-i", "p.png", value];
        assert_eq!(error_kind(&args), ErrorKind::ValueValidation, "{args:?}");
    }
    // both ends of the strength range are valid
    assert!(try_parse(&["-i", "p.png", "--strength", "0"]).is_ok());
    assert!(try_parse(&["-i", "p.png", "--strength", "1"]).is_ok());
}
