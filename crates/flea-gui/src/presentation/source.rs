use super::Fonts;
use crate::palette::{ParsedTheme, Theme};
use gpui::App;
use std::{
    io::{self, Read},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq)]
pub struct Snapshot {
    pub theme: Theme,
    pub parsed: ParsedTheme,
    pub fonts: Fonts,
}

impl Snapshot {
    pub fn install(&self, cx: &mut App) {
        cx.set_global(self.theme);
        cx.set_global(self.parsed);
        cx.set_global(self.fonts.clone());
    }
}

pub(super) type Inputs = [Option<String>; 8];
/// The most bytes one input may hold; a larger file is refused, never cut.
pub(super) const INPUT_CAP: usize = 65536;
pub(super) type Stamps = [Option<(u64, u64, u64, i64, i64, i64, i64, u32)>; 8];

/// The seven files a presentation reads, and where the resolved font family is cached.
pub(super) struct Sources {
    inputs: [PathBuf; 8],
    cache: PathBuf,
}

impl Sources {
    #[cfg(test)]
    pub fn at(inputs: [PathBuf; 7], cache: PathBuf) -> Self {
        let mut all: [PathBuf; 8] =
            std::array::from_fn(|_| PathBuf::from("/proc/self/flea-no-state"));
        for (to, from) in all.iter_mut().zip(inputs) {
            *to = from;
        }
        Self { inputs: all, cache }
    }

    pub fn current() -> Self {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let state = std::env::var_os("XDG_STATE_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"));
        // Flea's own theme when the desktop shell installed none: the chosen
        // theme's directory, or ~/.config/flea, so a palette and shell.toml work
        // on any Linux desktop without a shell package.
        let theme_dir = flea::themedir::theme_dir(&home, &state);
        let user_shell =
            flea::themedir::user_shell(&home).unwrap_or_else(|| config.join("flea/shell.toml"));
        let toggles = state.join("omarchy/toggles/hypr");
        Self {
            inputs: [
                theme_dir.join("colors.toml"),
                theme_dir.join("shell.toml"),
                user_shell,
                config.join("fontconfig/fonts.conf"),
                config.join("fontconfig/conf.d/50-omarchy-input-method.conf"),
                toggles.join("window-opaque.lua"),
                toggles.join("window-glassy.lua"),
                state.join("flea/ui.json"),
            ],
            cache: state.join("flea/fc-match-monospace"),
        }
    }

    /// The folders that hold the inputs, and the one the theme folder is swapped into, for the waker to watch.
    pub fn dirs(&self) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = self
            .inputs
            .iter()
            .filter_map(|input| input.parent())
            .map(Path::to_path_buf)
            .collect();
        dirs.extend(
            self.inputs[0]
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
        );
        dirs.sort();
        dirs.dedup();
        dirs
    }

    fn read_one(path: &Path, cap: usize) -> io::Result<Option<String>> {
        let mut text = String::new();
        let file = match flea::backend::regfile::open_if_regular(path, 0) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        file.take(cap as u64 + 1).read_to_string(&mut text)?;
        if text.len() > cap {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(Some(text))
    }

    /// Every input or the first failure: a live change that cannot be read keeps the last complete snapshot.
    pub fn read(&self) -> io::Result<Inputs> {
        let mut inputs = std::array::from_fn(|_| None);
        for (at, (path, input)) in self.inputs.iter().zip(&mut inputs).enumerate() {
            *input = Self::read_one(path, if at == 7 { 16 * 1024 * 1024 } else { INPUT_CAP })?;
        }
        Ok(inputs)
    }

    /// The launch read: an input that cannot be read counts as absent and is reported, never refused.
    pub fn read_for_launch(&self) -> (Inputs, Vec<String>) {
        let mut inputs = std::array::from_fn(|_| None);
        let mut failures = Vec::new();
        for (at, (path, input)) in self.inputs.iter().zip(&mut inputs).enumerate() {
            match Self::read_one(path, if at == 7 { 16 * 1024 * 1024 } else { INPUT_CAP }) {
                Ok(text) => *input = text,
                Err(error) => failures.push(format!("{}: {error}", path.display())),
            }
        }
        (inputs, failures)
    }

    /// What identifies each input without reading it, so an unchanged input costs one stat and no probe.
    pub fn stamps(&self) -> Stamps {
        std::array::from_fn(|at| {
            let meta = std::fs::metadata(&self.inputs[at]).ok()?;
            Some((
                meta.dev(),
                meta.ino(),
                meta.size(),
                meta.mtime(),
                meta.mtime_nsec(),
                meta.ctime(),
                meta.ctime_nsec(),
                meta.mode(),
            ))
        })
    }

    /// The snapshot for these inputs. A probe that fails keeps `fallback` for the family and is returned for the log.
    pub fn resolve(&self, input: &Inputs, fallback: &str) -> (Snapshot, Option<io::Error>) {
        self.resolve_with(input, fallback, super::face::fc_match, super::face::LAUNCH)
    }

    /// The retry's resolve after a quick probe timed out: the same inputs, with time for a cold fontconfig cache to answer.
    pub fn resolve_patient(&self, input: &Inputs, fallback: &str) -> (Snapshot, Option<io::Error>) {
        self.resolve_with(input, fallback, super::face::fc_match, super::face::PATIENT)
    }

    fn resolve_with(
        &self,
        input: &Inputs,
        fallback: &str,
        probe: impl FnOnce() -> std::process::Command,
        deadline: std::time::Duration,
    ) -> (Snapshot, Option<io::Error>) {
        let text = |at: usize| input[at].as_deref().unwrap_or("");
        let seed_palette = flea::jsondoc::parse(text(7)).ok().and_then(|data| {
            data.get("themeSeed")
                .and_then(|value| value.as_str())
                .and_then(flea::seed_theme::palette)
        });
        let (mut theme, parsed) =
            Theme::from_text_observed(seed_palette.as_deref().unwrap_or_else(|| text(0)));
        theme.glass = crate::window::glass::Inputs::resolve(
            seed_palette.as_deref().unwrap_or_else(|| text(0)),
            text(1),
            text(2),
            input[5].is_some(),
            input[6].is_some(),
        );
        if let Some(opacity) = flea::jsondoc::parse(text(7))
            .ok()
            .and_then(|data| data.get("themeOpacity").and_then(|v| v.as_f64()))
            .filter(|n| (0.0..=100.0).contains(n))
        {
            let alpha = (opacity * 255.0 / 100.0).round() as u8;
            theme.glass.wanted_alpha = alpha;
            theme.glass.wanted_list_alpha = alpha;
            theme.glass.automatic = false;
            theme.glass.pins_opaque = alpha == 255;
            theme.glass.chrome_inks = None;
        }
        let (family, failure) =
            match super::face_cache::family_within(&input[3..5], &self.cache, probe, deadline) {
                Ok(family) => (family, None),
                Err(error) => (fallback.to_owned(), Some(error)),
            };
        let snapshot = Snapshot {
            theme,
            parsed,
            fonts: Fonts::from_sources(text(1), text(2), family),
        };
        (snapshot, failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn sources(inputs: [PathBuf; 7]) -> Sources {
        Sources {
            inputs: {
                let mut all = std::array::from_fn(|_| PathBuf::from("/proc/self/flea-no-state"));
                for (to, from) in all.iter_mut().zip(inputs) {
                    *to = from;
                }
                all
            },
            cache: PathBuf::from("/proc/self/flea-u01-no-cache/entry"),
        }
    }

    #[test]
    fn seed_overrides_the_source_palette_and_empty_restores_it() {
        let source = sources(std::array::from_fn(|_| {
            PathBuf::from("/proc/self/flea-u01-absent-input")
        }));
        let mut input: Inputs = std::array::from_fn(|_| None);
        input[0] = Some("background='#101010'\naccent='#123456'".into());
        input[7] = Some(r##"{"themeSeed":"#f05533"}"##.into());
        let resolve = |input: &Inputs| {
            source
                .resolve_with(
                    input,
                    "monospace",
                    || {
                        let mut command = Command::new("/bin/sh");
                        command.args(["-c", "exit 1"]);
                        command
                    },
                    super::super::face::LAUNCH,
                )
                .0
        };
        let seeded = resolve(&input);
        assert_eq!(
            seeded.theme,
            Theme::from_text(&flea::seed_theme::palette("#f05533").unwrap())
        );
        input[7] = Some(r#"{"themeOpacity":50}"#.into());
        let glass = resolve(&input).theme.glass;
        assert_eq!(glass.wanted_alpha, 128);
        assert_eq!(glass.wanted_list_alpha, 128);
        assert!(!glass.pins_opaque);
        input[7] = Some(r#"{"themeSeed":""}"#.into());
        assert_eq!(
            resolve(&input).theme,
            Theme::from_text(input[0].as_deref().unwrap())
        );
    }

    #[test]
    fn u01_source_read_failure_stays_distinct_from_absent_font_inputs() {
        let absent = sources(std::array::from_fn(|_| {
            PathBuf::from("/proc/self/flea-u01-absent-input")
        }));
        assert_eq!(
            absent.read().unwrap(),
            [None, None, None, None, None, None, None, None]
        );
        let failed = sources(std::array::from_fn(|_| PathBuf::from("/proc/self")));
        assert_eq!(
            failed.read().unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        let (launch, failures) = failed.read_for_launch();
        assert_eq!(launch, [None, None, None, None, None, None, None, None]);
        assert_eq!(failures.len(), 7);
        let valid = sources(std::array::from_fn(|_| PathBuf::from("/proc/self/status")));
        assert!(valid
            .read()
            .unwrap()
            .iter()
            .take(7)
            .all(|value| value.as_ref().is_some_and(|text| !text.is_empty())));
    }

    #[test]
    fn u01_a_failed_probe_keeps_the_fallback_family_and_a_stamp_moves_only_with_the_input() {
        let fixture = super::super::fixture::Fixture::new("stamp");
        let dir = fixture.path();
        let file = dir.join("fonts.conf");
        let one = sources(std::array::from_fn(|at| {
            if at == 3 {
                file.clone()
            } else {
                dir.join("absent")
            }
        }));
        assert_eq!(one.stamps()[3], None);
        fixture.write(&file, "one");
        let first = one.stamps();
        assert!(first[3].is_some());
        assert_eq!(first, one.stamps());
        fixture.write(&file, "three");
        assert_ne!(first, one.stamps());
        let inputs = one.read().unwrap();
        let (snapshot, failure) = one.resolve_with(
            &inputs,
            "monospace",
            || {
                let mut command = Command::new("/bin/sh");
                command.args(["-c", "exit 1"]);
                command
            },
            super::super::face::LAUNCH,
        );
        assert_eq!(snapshot.fonts.family, "monospace");
        assert_eq!(failure.unwrap().kind(), io::ErrorKind::InvalidData);
        fixture.cleanup();
    }
}

#[cfg(test)]
#[path = "../window/glass_test_bridge.rs"]
mod glass_test_bridge;
