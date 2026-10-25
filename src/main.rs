use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use image::{DynamicImage, ImageFormat, ImageReader};

// Rose Pine variants as 0xRRGGBB, in the order of the official palette:
// base, surface, overlay, muted, subtle, text, love, gold, rose, pine, foam, iris, highlight low, highlight med, highlight high
const ROSE_PINE: [u32; 15] = [0x191724, 0x1f1d2e, 0x26233a, 0x6e6a86, 0x908caa, 0xe0def4, 0xeb6f92, 0xf6c177, 0xebbcba, 0x31748f, 0x9ccfd8, 0xc4a7e7, 0x21202e, 0x403d52, 0x524f67];
const MOON: [u32; 15] = [0x232136, 0x2a273f, 0x393552, 0x6e6a86, 0x908caa, 0xe0def4, 0xeb6f92, 0xf6c177, 0xea9a97, 0x3e8fb0, 0x9ccfd8, 0xc4a7e7, 0x2a283e, 0x44415a, 0x56526e];
const DAWN: [u32; 15] = [0xfaf4ed, 0xfffaf3, 0xf2e9e1, 0x9893a5, 0x797593, 0x464261, 0xb4637a, 0xea9d34, 0xd7827e, 0x286983, 0x56949f, 0x907aa9, 0xf4ede8, 0xdfdad9, 0xcecacd];

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Palette {
    R, // main, the default Rose Pine
    M, // Moon
    D, // Dawn
}

impl Palette {
    fn colors(self) -> [[u8; 3]; 15] {
        let hexes = match self {
            Palette::R => ROSE_PINE,
            Palette::M => MOON,
            Palette::D => DAWN,
        };
        hexes.map(hex_to_rgb)
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Transform any image to fit the Rose Pine palette")]

struct Args {
    #[arg(short, long, value_name="file")]
    input: PathBuf,

    #[arg(short, long, value_name="file")]
    output: Option<PathBuf>,

    #[arg(short, long, value_enum, default_value_t = Palette::R)]
    palette: Palette,
}

impl Args {
    // Uses --output if given, otherwise writes next to the input as `<name>_rosepine.<ext>`
    fn output_path(&self) -> PathBuf {
        if let Some(o) = &self.output {
            return o.clone();
        }
        let stem = self.input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let ext = self.input.extension().and_then(|e| e.to_str()).unwrap_or("png");
        self.input.with_file_name(format!("{stem}_rosepine.{ext}"))
    }
}

const fn hex_to_rgb(hex: u32) -> [u8; 3] {
    // 0xRRGGBB
    // shift each byte down to the lowest position and mask off the rest
    let r: u8 = ((hex >> 16) & 0xFF) as u8;
    let g: u8 = ((hex >> 8) & 0xFF) as u8;
    let b: u8 = (hex & 0xFF) as u8;

    let rgb = [r, g, b];
    return rgb;
}

// Pixel values (0-255) are not proportional to actual brightness
// 128 is not "half as bright" as 255, it's closer to a fifth
// Images store them this way to spend more values on dark tones, where the eye notices differences more
// Before comparing colors we undo that and get the real brightness as 0.0-1.0
// The formula is the official sRGB one: very dark values are just divided, everything else is raised to the power of 2.4
fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}


// Converts a color to OKLab, where the distance between two colors matches how different they look to a human
// In RGB that's not true - e.g. two greens can be far apart in numbers yet look almost the same
// The result has three parts:
//   L - lightness (0 = black, 1 = white)
//   a - from green (negative) to red (positive)
//   b - from blue (negative) to yellow (positive)
// How: first mix R, G, B into three values that roughly match the eye's three kinds of color sensors (the l, m, s below), take their cube root because the eye reacts less and less to extra light, then mix them again into L, a, b
// The constants come from the OKLab spec
fn rgb_to_oklab(rgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(srgb_to_linear);

    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();

    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

// Squared euclidean distance in OKLab
// we only compare values so no need to squre root
fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

// Brute-force search over the palette - with only 15 colors this is faster than any spatial index
// Palette entries carry their precomputed OKLab value next to the RGB one we write back
// unwrap is safe because the palette is never empty
fn nearest_color(pixel: [u8; 3], palette: &[([u8; 3], [f32; 3])]) -> [u8; 3] {
    let lab = rgb_to_oklab(pixel);
    palette
        .iter()
        .min_by(|x, y| color_distance(lab, x.1).total_cmp(&color_distance(lab, y.1)))
        .unwrap()
        .0
}

fn main() {
    let args = Args::parse();
    // Convert the palette to OKLab once up front instead of on every comparison
    let palette = args.palette.colors().map(|c| (c, rgb_to_oklab(c)));

    let img = ImageReader::open(&args.input)
        .expect("failed to open input file")
        .decode()
        .expect("failed to decode image");
    // remember whether the original had transparency
    let has_alpha = img.color().has_alpha();
    // normalize every input format to 8-bit RGBA
    let mut img = img.to_rgba8();

    // replace each pixel's color in place, leaving its alpha untouched
    for pixel in img.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let [r, g, b] = nearest_color([r, g, b], &palette);
        pixel.0 = [r, g, b, a];
    }

    let output = args.output_path();
    // drop alpha when unsupported/unused
    let supports_alpha = !matches!(ImageFormat::from_path(&output), Ok(ImageFormat::Jpeg));
    let result = if has_alpha && supports_alpha {
        img.save(&output)
    } else {
        DynamicImage::ImageRgba8(img).to_rgb8().save(&output)
    };
    result.expect("failed to save image");
    println!("Saved to {}", output.display());
}
