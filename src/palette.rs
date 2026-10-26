use clap::ValueEnum;

// Rose Pine variants as 0xRRGGBB, in the order of the official palette:
// base, surface, overlay, muted, subtle, text, love, gold, rose, pine, foam, iris, highlight low, highlight med, highlight high
const ROSE_PINE: [u32; 15] = [0x191724, 0x1f1d2e, 0x26233a, 0x6e6a86, 0x908caa, 0xe0def4, 0xeb6f92, 0xf6c177, 0xebbcba, 0x31748f, 0x9ccfd8, 0xc4a7e7, 0x21202e, 0x403d52, 0x524f67];
const MOON: [u32; 15] = [0x232136, 0x2a273f, 0x393552, 0x6e6a86, 0x908caa, 0xe0def4, 0xeb6f92, 0xf6c177, 0xea9a97, 0x3e8fb0, 0x9ccfd8, 0xc4a7e7, 0x2a283e, 0x44415a, 0x56526e];
const DAWN: [u32; 15] = [0xfaf4ed, 0xfffaf3, 0xf2e9e1, 0x9893a5, 0x797593, 0x464261, 0xb4637a, 0xea9d34, 0xd7827e, 0x286983, 0x56949f, 0x907aa9, 0xf4ede8, 0xdfdad9, 0xcecacd];

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Palette {
    /// Rose Pine (main)
    R,
    /// Rose Pine Moon
    M,
    /// Rose Pine Dawn
    D,
}

impl Palette {
    pub fn colors(self) -> [[u8; 3]; 15] {
        let hexes = match self {
            Palette::R => ROSE_PINE,
            Palette::M => MOON,
            Palette::D => DAWN,
        };
        hexes.map(hex_to_rgb)
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

#[cfg(test)]
mod tests;
