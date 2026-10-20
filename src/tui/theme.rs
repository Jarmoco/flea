use std::collections::HashMap;

// The one background fallback, and ui/Theme.qml carries the same literal for the same reason.
const BACKGROUND_FALLBACK: &str = "#101315";

pub struct Theme {
    pub foreground: String,
    pub muted: String,
    pub accent: String,
    pub error: String,
    pub background: String,
    pub symlink: String,
    pub executable: String,
    pub selected: String,
    pub border: String,
}
impl Theme {
    /// The theme the operator chose, the same one the window draws with, and the
    /// operator's own colours over it.
    pub fn load_named(name: &str, own: &[(String, String)]) -> Self {
        // The same theme the window draws with, resolved once and named in the environment.
        let home = std::env::var("HOME").unwrap_or_default();
        let home = std::path::Path::new(&home);
        let state_home = crate::themedir::state_home(home);
        let text = crate::themedir::resolve(home, &state_home, name)
            .map(|theme| std::fs::read_to_string(theme.dir.join("colors.toml")).unwrap_or_default())
            .unwrap_or_default();
        Self::from_text_with(&text, own)
    }
    /// The theme's own colours, with the operator's own over the roles the terminal
    /// interface draws with. A role it does not draw with is the window's own.
    pub(super) fn from_text_with(text: &str, own: &[(String, String)]) -> Self {
        let values = parse(text);
        let pick = |keys: &[&str], fallback: &str| {
            keys.iter().find_map(|key| values.get(*key)).map(String::as_str).unwrap_or(fallback).to_owned()
        };
        // The operator's own colour for a role, or the theme's own value for it.
        let own_pick = |role: &str, fallback: &str| {
            own.iter()
                .find(|(name, _)| name == role)
                .map(|(_, hex)| hex.clone())
                .unwrap_or_else(|| fallback.to_string())
        };
        let foreground = own_pick("foreground", &pick(&["foreground", "color7"], "#cacccc"));
        let background = own_pick("background", &pick(&["background", "color0"], BACKGROUND_FALLBACK));
        let accent = own_pick("accent", &pick(&["accent", "color4"], "#cacccc"));
        let surface = pick(&["dark_background", "background", "selection"], "#181825");
        let role_background = own_pick("background", &pick(&["background"], BACKGROUND_FALLBACK));
        let muted = own_pick(
            "muted",
            &contrast(&contrast(&pick(&["muted"], "#707880"), &role_background), &surface),
        );
        let error = own_pick("error", &pick(&["urgent"], "#a55555"));
        let symlink = own_pick(
            "symlink",
            &contrast(&pick(&["cyan", "color6"], "#94e2d5"), &role_background),
        );
        let executable = own_pick(
            "executable",
            &contrast(&pick(&["green", "color2"], "#a6e3a1"), &role_background),
        );
        Self {
            foreground: ansi(&foreground, false),
            muted: ansi(&muted, false),
            accent: ansi(&accent, false),
            error: ansi(&error, false),
            background: ansi(&background, true),
            symlink: ansi(&symlink, false),
            executable: ansi(&executable, false),
            selected: ansi(&blend(&accent, &background, 0.22), true),
            border: ansi(&muted, false),
        }
    }
}
fn blend(foreground: &str, background: &str, opacity: f64) -> String {
    let channel = |i| {
        let fg = u8::from_str_radix(&foreground[i..i + 2], 16).unwrap_or(0) as f64;
        let bg = u8::from_str_radix(&background[i..i + 2], 16).unwrap_or(0) as f64;
        (fg * opacity + bg * (1.0 - opacity)).round() as u8
    };
    format!("#{:02x}{:02x}{:02x}", channel(1), channel(3), channel(5))
}
// Sample input: accent = "#a9b665"; values outside six-digit RGB are ignored.
fn parse(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_start_matches(['"', '\'']);
            let value = value.get(..7).unwrap_or("");
            if value.len() == 7
                && value.starts_with('#')
                && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                out.insert(key.trim().into(), value.into());
                if matches!(key.trim(), "red" | "color1") { out.insert("urgent".into(), value.into()); }
            }
        }
    }
    out
}
fn rgb(hex: &str) -> [f64; 3] {
    [1, 3, 5].map(|start| u8::from_str_radix(&hex[start..start + 2], 16).unwrap_or(0) as f64 / 255.0)
}
fn luminance(rgb: [f64; 3]) -> f64 {
    let linear = rgb.map(|c| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) });
    linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722
}
fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
fn contrast(foreground: &str, background: &str) -> String {
    // Keep this calculation aligned with ui/js/Contrast.js, including its rounded RGB result.
    const MIN_RATIO: f64 = 4.5;
    const SEARCH_STEPS: usize = 18;
    let (fg, bg) = (rgb(foreground), rgb(background));
    if ratio(fg, bg) >= MIN_RATIO { return foreground.into(); }
    let toward = if luminance(bg) > 0.179 { 0.0 } else { 1.0 };
    let (mut low, mut high) = (0.0, 1.0);
    let mut best = [toward; 3];
    for _ in 0..SEARCH_STEPS {
        let mid = (low + high) / 2.0;
        let candidate = fg.map(|c| c + (toward - c) * mid);
        if ratio(candidate, bg) >= MIN_RATIO { best = candidate; high = mid; }
        else { low = mid; }
    }
    let [red, green, blue] = best.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("#{red:02x}{green:02x}{blue:02x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_matches_oem_precedence_and_flea_roles() {
        let theme = Theme::from_text_with("color7='#112233'\ncolor0='#010203'\ncolor4='#445566'\naccent='#123456'\nred='#654321'\ncolor1='#abcdef'", &[]);
        assert_eq!(theme.foreground, ansi("#112233", false));
        assert_eq!(theme.background, ansi("#010203", true));
        assert_eq!(theme.accent, ansi("#123456", false));
        assert_eq!(theme.error, ansi("#abcdef", false));
        assert_eq!(Theme::from_text_with("color1='#abcdef'\nred='#654321'", &[]).error, ansi("#654321", false));
        assert_eq!(Theme::from_text_with("foreground='oops'", &[]).foreground, ansi("#cacccc", false));
        assert_eq!(contrast("#ffffff", "#000000"), "#ffffff");
        assert_eq!(contrast("#707880", "#101315"), "#767e85");
    }

    // The operator's own colours are the window's, so the terminal interface draws
    // with them too, and a role it does not draw with is the window's own.
    #[test]
    fn the_operators_own_colours_are_laid_over_the_themes_own() {
        let own = vec![("foreground".to_string(), "#fefefe".to_string())];
        let theme = Theme::from_text_with("color7='#112233'\ncolor0='#010203'\ncolor4='#445566'", &own);
        assert_eq!(theme.foreground, ansi("#fefefe", false));
        assert_eq!(theme.accent, ansi("#445566", false));
        assert_eq!(theme.background, ansi("#010203", true));
        // The selected row is the accent over the ground, so it follows both.
        assert_eq!(theme.selected, ansi(&blend("#445566", "#010203", 0.22), true));
    }
}
fn ansi(hex: &str, bg: bool) -> String {
    let channel = |start| u8::from_str_radix(&hex[start..start + 2], 16).unwrap_or(0);
    format!(
        "\x1b[{};2;{};{};{}m",
        if bg { 48 } else { 38 },
        channel(1),
        channel(3),
        channel(5)
    )
}
