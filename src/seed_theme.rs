//! Generate a dark palette from a single RGB seed.
//!
//! The returned text uses the same colour names as an installed theme, so the
//! existing theme resolver continues to own semantic inks and contrast floors.
use crate::ui_logic_pl4::tagtheme::{ensure_ratio, hex_of, parse};

/// A seed is exactly six hexadecimal RGB digits, with an optional leading `#`.
pub fn normalize(seed: &str) -> Option<String> {
    let digits = seed.strip_prefix('#').unwrap_or(seed);
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{}", digits.to_ascii_lowercase()))
}

fn blend(a: [f64; 3], b: [f64; 3], amount: f64) -> [f64; 3] {
    std::array::from_fn(|i| a[i] * (1.0 - amount) + b[i] * amount)
}

/// Derive a palette while keeping text and focus contrast legible for dark,
/// bright, saturated, and neutral seeds. Invalid input is rejected.
pub fn palette(seed: &str) -> Option<String> {
    let seed = normalize(seed)?;
    let rgb = parse(&seed)?;
    // Bounded tinting keeps even white and black seeds within usable dark surfaces.
    let background = hex_of(blend([0.035, 0.039, 0.047], rgb, 0.09));
    let surface = hex_of(blend([0.065, 0.069, 0.078], rgb, 0.12));
    let foreground = hex_of(blend([0.86, 0.87, 0.88], rgb, 0.10));
    let accent = ensure_ratio(&seed, &surface, 4.5);
    let mut palette = format!(
        "background='{background}'\ndark_background='{surface}'\nforeground='{foreground}'\naccent='{accent}'\n"
    );
    // Semantic colors remain distinct, with a small seed tint to fit the palette.
    for (name, base) in [
        ("red", [0.88, 0.35, 0.34]),
        ("orange", [0.90, 0.57, 0.30]),
        ("yellow", [0.88, 0.74, 0.36]),
        ("green", [0.47, 0.75, 0.43]),
        ("cyan", [0.35, 0.72, 0.74]),
        ("blue", [0.40, 0.62, 0.91]),
        ("magenta", [0.76, 0.50, 0.82]),
        ("dark_foreground", [0.58, 0.60, 0.64]),
    ] {
        let color = hex_of(blend(base, rgb, 0.12));
        palette.push_str(&format!("{name}='{color}'\n"));
    }
    Some(palette)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_logic_pl4::tagtheme::{luminance, ratio_of};

    #[test]
    fn rejects_non_rgb_and_normalizes_valid_seeds() {
        assert_eq!(normalize("#aB12ef"), Some("#ab12ef".into()));
        for input in ["", "#abc", "#12345678", "#gg1234", " 123456", "é1234"] {
            assert_eq!(palette(input), None, "{input:?}");
        }
    }

    #[test]
    fn extreme_seeds_keep_surfaces_dark_and_text_and_accent_readable() {
        for seed in [
            "#000000", "#ffffff", "#ff0000", "#00ff00", "#0000ff", "#777777",
        ] {
            let text = palette(seed).unwrap();
            let get = |name: &str| {
                let value = text
                    .lines()
                    .find_map(|line| line.strip_prefix(name))
                    .unwrap();
                parse(value.trim_start_matches('=').trim_matches('\'')).unwrap()
            };
            let background = get("background");
            let surface = get("dark_background");
            assert!(luminance(background) < 0.04, "{seed}");
            assert!(ratio_of(get("foreground"), background) >= 4.5, "{seed}");
            assert!(ratio_of(get("accent"), surface) >= 4.5, "{seed}");
        }
    }
}

#[cfg(test)]
mod persistence_tests {
    use crate::{jsondoc, uischema, uistate};

    #[test]
    fn state_accepts_only_empty_or_rgb_and_preserves_other_settings() {
        let data = uischema::defaults();
        let patch = |text| jsondoc::parse(text).unwrap();
        let saved = uistate::patched(&data, &patch(r##"{"themeSeed":"#12aBcF"}"##)).unwrap();
        assert_eq!(
            saved.get("themeSeed").and_then(|v| v.as_str()),
            Some("#12aBcF")
        );
        assert_eq!(saved.get("view"), data.get("view"));
        assert!(uistate::patched(&saved, &patch(r##"{"themeSeed":"#abc"}"##)).is_err());
        assert!(uistate::patched(&saved, &patch(r#"{"themeSeed":""}"#)).is_ok());
    }
}
