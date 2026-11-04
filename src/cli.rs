use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::filter::Mix;
use crate::palette::Palette;
use crate::process::Method;

#[derive(Parser, Debug)]
#[command(version, about = "Transform any image to fit the Rose Pine palette")]
pub struct Args {
    #[arg(short, long, value_name = "file")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "file")]
    pub output: Option<PathBuf>,

    /// Palette variant to map colors to
    #[arg(short, long, value_enum, default_value_t = Palette::R)]
    pub palette: Palette,

    /// Replace every pixel with a palette color instead of tinting the image toward the palette
    #[arg(long = "no-soft", action = ArgAction::SetFalse)]
    pub soft: bool,

    /// Replace every pixel with a palette color without Floyd-Steinberg dithering, implies --no-soft (better for flat graphics, logos, pixel art)
    // Only replacing pixels makes color bands, tinting has nothing for dithering to hide, so turning dithering off only makes sense together with --no-soft
    #[arg(long = "no-dither", action = ArgAction::SetFalse)]
    pub dither: bool,

    /// Generate all 3 palette variants, named <name>_rosepine_<main|moon|dawn>.<ext>
    #[arg(long, conflicts_with = "palette")]
    pub all: bool,

    /// How far the colors move toward the palette, 0 keeps the original
    #[arg(long, value_name = "0-1", default_value_t = 0.7, value_parser = parse_strength, conflicts_with_all = ["soft", "dither"])]
    pub strength: f32,

    /// How many palette colors get blended, small values stick to the nearest one
    #[arg(long, value_name = "n", default_value_t = 0.02, value_parser = parse_softness, conflicts_with_all = ["soft", "dither"])]
    pub softness: f32,

    /// Keep the original brightness, only hue and saturation change
    #[arg(long, conflicts_with_all = ["soft", "dither"])]
    pub keep_lightness: bool,
}

fn parse_strength(s: &str) -> Result<f32, String> {
    let v: f32 = s.parse().map_err(|e| format!("{e}"))?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err("must be between 0 and 1".into())
    }
}

fn parse_softness(s: &str) -> Result<f32, String> {
    let v: f32 = s.parse().map_err(|e| format!("{e}"))?;
    if v > 0.0 && v.is_finite() {
        Ok(v)
    } else {
        Err("must be greater than 0".into())
    }
}

impl Args {
    pub fn method(&self) -> Method {
        if self.soft && self.dither {
            Method::Soft {
                softness: self.softness,
                mix: Mix {
                    strength: self.strength,
                    keep_lightness: self.keep_lightness,
                },
            }
        } else {
            Method::Quantize {
                dither: self.dither,
            }
        }
    }

    // Uses --output if given, otherwise writes next to the input as `<name>_rosepine.<ext>`
    pub fn output_path(&self) -> PathBuf {
        if let Some(o) = &self.output {
            return o.clone();
        }
        let stem = self
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = self
            .input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        self.input.with_file_name(format!("{stem}_rosepine.{ext}"))
    }

    // Name for --all to build on, without _rosepine so the variant names don't get it twice
    pub fn base_path(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| self.input.clone())
    }
}

#[cfg(test)]
mod tests;
