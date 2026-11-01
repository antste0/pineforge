use super::*;

#[test]
fn colors_start_with_base() {
    assert_eq!(Palette::R.colors()[0], [0x19, 0x17, 0x24]);
    assert_eq!(Palette::M.colors()[0], [0x23, 0x21, 0x36]);
    assert_eq!(Palette::D.colors()[0], [0xfa, 0xf4, 0xed]);
}

// Duplicates would mean a typo in one of the hex tables
#[test]
fn colors_are_unique() {
    for p in [Palette::R, Palette::M, Palette::D] {
        let colors = p.colors();
        for (i, a) in colors.iter().enumerate() {
            assert!(!colors[i + 1..].contains(a), "{p:?} has duplicate {a:?}");
        }
    }
}
