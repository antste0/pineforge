use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::palette::Palette;

#[derive(Parser, Debug)]
#[command(version, about = "Transform any image to fit the Rose Pine palette")]

pub struct Args {
    #[arg(short, long, value_name="file")]
    pub input: PathBuf,

    #[arg(short, long, value_name="file")]
    pub output: Option<PathBuf>,

    /// Palette variant to map colors to
    #[arg(short, long, value_enum, default_value_t = Palette::R)]
    pub palette: Palette,

    /// Disable Floyd-Steinberg dithering (better for flat graphics, logos, pixel art)
    #[arg(long = "no-dither", action = ArgAction::SetFalse)]
    pub dither: bool,
}

impl Args {
    // Uses --output if given, otherwise writes next to the input as `<name>_rosepine.<ext>`
    pub fn output_path(&self) -> PathBuf {
        if let Some(o) = &self.output {
            return o.clone();
        }
        let stem = self.input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let ext = self.input.extension().and_then(|e| e.to_str()).unwrap_or("png");
        self.input.with_file_name(format!("{stem}_rosepine.{ext}"))
    }
}
