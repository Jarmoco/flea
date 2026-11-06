//! Where the theme lives, decided in one place and read by the window and the terminal interface.
//!
//! Flea used to read its colours only from the desktop shell's current theme. A
//! Linux desktop need not have that shell, so the answer is now a small ladder:
//!
//! * `FLEA_THEME_DIR`, an explicit directory.
//! * The theme named by `FLEA_THEME` or `~/.config/flea/theme`.
//! * `desktop`, the desktop shell's own theme when it installed one.
//! * Flea's own palette, at `~/.config/flea/colors.toml`.
//!
//! Named themes are searched for in Flea's own directories and, when the shell
//! is installed, the shell's collection too.

use std::path::{Path, PathBuf};

/// Names a theme directory outright; the launcher and tests use it.
pub const DIR_ENV: &str = "FLEA_THEME_DIR";
/// Names the theme to use, including the empty name for Flea's own palette.
pub const NAME_ENV: &str = "FLEA_THEME";
/// Names the machine-level `shell.toml`, which layers over a theme's own.
pub const SHELL_ENV: &str = "FLEA_SHELL_TOML";

/// The desktop shell's own current theme, and the shipped default.
pub const DESKTOP: &str = "desktop";
/// No theme: Flea draws with the palette of its own.
pub const NONE: &str = "";
/// Flea's own directory, which holds `colors.toml` and `shell.toml`.
pub const OWN: &str = "flea";

/// A theme's directory and the file that names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub dir: PathBuf,
    pub name: PathBuf,
}

/// One theme the operator can pick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub dir: PathBuf,
    pub file: PathBuf,
}

/// A theme's own name is one path segment, so a stored name can never escape
/// the directories it is looked for in.
pub fn is_theme_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.chars().any(char::is_control)
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
}

/// The directories a theme's own name is looked for in, in order.
pub fn search_dirs(home: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![
        home.join(".config/flea/themes"),
        PathBuf::from("/usr/share/flea/themes"),
        home.join(".config/omarchy/themes"),
        PathBuf::from("/usr/share/omarchy/themes"),
    ];
    dirs.retain(|dir| dir.is_dir());
    dirs
}

/// The desktop shell's own current theme, when it installed one.
///
/// The shell writes it under `~/.local/state` even when `XDG_STATE_HOME` says
/// otherwise, so both roots are checked and the shell's own comes first.
fn desktop_theme(home: &Path, state_home: &Path) -> Option<Theme> {
    let roots = [home.join(".local/state"), state_home.to_owned()];
    for root in roots {
        let dir = root.join("omarchy/current/theme");
        if dir.is_dir() {
            return Some(Theme {
                name: root.join("omarchy/current/theme.name"),
                dir,
            });
        }
    }
    None
}

/// The theme a stored name names, or none when this box has no such theme.
pub fn resolve(home: &Path, state_home: &Path, name: &str) -> Option<Theme> {
    if name == NONE {
        return None;
    }
    if name == DESKTOP {
        return desktop_theme(home, state_home);
    }
    if name == OWN {
        return Some(own_theme(home));
    }
    if !is_theme_name(name) {
        return None;
    }
    for dir in search_dirs(home) {
        let path = dir.join(name);
        if path.join("colors.toml").is_file() {
            return Some(Theme {
                name: path.join("name"),
                dir: path,
            });
        }
    }
    None
}

fn own_theme(home: &Path) -> Theme {
    Theme {
        dir: home.join(".config/flea"),
        name: home.join(".config/flea/theme.name"),
    }
}

/// The name the operator chose, or `desktop` when they chose nothing.
pub fn chosen_name(home: &Path) -> String {
    if let Some(name) = std::env::var_os(NAME_ENV) {
        return name.to_string_lossy().into_owned();
    }
    let stored = std::fs::read_to_string(home.join(".config/flea/theme")).unwrap_or_default();
    let name = stored.lines().next().unwrap_or("").trim();
    if name.is_empty() {
        DESKTOP.to_owned()
    } else {
        name.to_owned()
    }
}

/// The theme this process draws with, resolved once.
pub fn current(home: &Path, state_home: &Path) -> Option<Theme> {
    if let Some(dir) = env_nonempty(DIR_ENV) {
        let dir = PathBuf::from(dir);
        return Some(Theme {
            name: dir.join("name"),
            dir,
        });
    }
    resolve(home, state_home, &chosen_name(home))
}

/// The directory holding the palette and theme shell: the chosen theme, or
/// Flea's own config directory when no desktop shell theme is installed.
pub fn theme_dir(home: &Path, state_home: &Path) -> PathBuf {
    current(home, state_home)
        .map(|theme| theme.dir)
        .unwrap_or_else(|| own_theme(home).dir)
}

pub fn colors_path(home: &Path, state_home: &Path) -> PathBuf {
    theme_dir(home, state_home).join("colors.toml")
}

/// The palette path for this process, read once from `HOME` and `XDG_STATE_HOME`.
pub fn env_colors_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|value| !value.is_empty())?;
    let home = PathBuf::from(home);
    let state = std::env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"));
    Some(colors_path(&home, &state))
}

pub fn theme_shell_path(home: &Path, state_home: &Path) -> PathBuf {
    theme_dir(home, state_home).join("shell.toml")
}

/// The machine-level `shell.toml`, which layers over the theme's own.
pub fn user_shell(home: &Path) -> Option<PathBuf> {
    if let Some(path) = env_nonempty(SHELL_ENV) {
        // An explicit path is honoured even before it exists, so the watcher
        // created for it sees the file appear instead of falling back silently.
        return Some(PathBuf::from(path));
    }
    let own = home.join(".config/flea/shell.toml");
    if own.is_file() {
        return Some(own);
    }
    let shell = home.join(".config/omarchy/shell.toml");
    shell.is_file().then_some(shell)
}

/// The themes this box has: Flea's own palette, the desktop shell's own, and
/// every named theme directory found.
pub fn list(home: &Path, state_home: &Path) -> Vec<Entry> {
    let mut out = vec![
        Entry {
            name: NONE.to_owned(),
            dir: PathBuf::new(),
            file: PathBuf::new(),
        },
        Entry {
            name: OWN.to_owned(),
            dir: own_theme(home).dir,
            file: own_theme(home).name,
        },
    ];
    match desktop_theme(home, state_home) {
        Some(theme) => out.push(Entry {
            name: DESKTOP.to_owned(),
            dir: theme.dir,
            file: theme.name,
        }),
        None => out.push(Entry {
            name: DESKTOP.to_owned(),
            dir: PathBuf::new(),
            file: PathBuf::new(),
        }),
    }
    for dir in search_dirs(home) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != DESKTOP && name != OWN)
            .collect();
        names.sort();
        for name in names {
            if out.iter().any(|entry| entry.name == name) {
                continue;
            }
            let path = dir.join(&name);
            if path.join("colors.toml").is_file() {
                out.push(Entry {
                    name,
                    file: path.join("name"),
                    dir: path,
                });
            }
        }
    }
    out
}

/// The icon theme the selected theme names, if it names one.
pub fn icon_theme(theme: Option<&Theme>) -> Option<String> {
    let text = std::fs::read_to_string(theme?.dir.join("icons.theme")).ok()?;
    let name = text.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testdir::TestDir;

    fn install(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir.join(name).join("theme")).unwrap();
        std::fs::write(dir.join(name).join("theme/colors.toml"), body).unwrap();
    }

    #[test]
    fn the_desktop_theme_wins_while_it_is_installed_and_own_colours_take_over_after() {
        let sandbox = TestDir::new("themedir-desktop");
        let home = sandbox.path();
        let state = home.join(".local/state");
        install(home, ".local/state/omarchy/current", "background='#111111'\n");
        assert_eq!(
            resolve(home, &state, DESKTOP).unwrap().dir,
            state.join("omarchy/current/theme")
        );
        std::fs::remove_dir_all(state.join("omarchy")).unwrap();
        assert_eq!(resolve(home, &state, DESKTOP), None);
        assert_eq!(colors_path(home, &state), home.join(".config/flea/colors.toml"));
    }

    #[test]
    fn a_named_theme_is_found_in_fleas_own_collection() {
        let sandbox = TestDir::new("themedir-named");
        let home = sandbox.path();
        let state = home.join(".local/state");
        let theme = home.join(".config/flea/themes/nord");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::write(theme.join("colors.toml"), "background='#2e3440'\n").unwrap();
        assert_eq!(resolve(home, &state, "nord").unwrap().dir, theme);
        assert_eq!(resolve(home, &state, "../nord"), None);
        assert_eq!(resolve(home, &state, "."), None);
        assert_eq!(resolve(home, &state, ".."), None);
        assert_eq!(resolve(home, &state, NONE), None);
    }

    #[test]
    fn the_own_theme_is_the_config_directory_and_a_shell_layers_over_it() {
        let sandbox = TestDir::new("themedir-own");
        let home = sandbox.path();
        let state = home.join(".local/state");
        std::fs::create_dir_all(home.join(".config/flea")).unwrap();
        std::fs::write(home.join(".config/flea/shell.toml"), "[font]\nbase-size=15\n").unwrap();
        assert_eq!(resolve(home, &state, OWN).unwrap().dir, home.join(".config/flea"));
        assert_eq!(theme_dir(home, &state), home.join(".config/flea"));
        assert_eq!(user_shell(home), Some(home.join(".config/flea/shell.toml")));
    }

    #[test]
    fn the_own_palette_and_the_desktop_theme_are_both_offered() {
        let sandbox = TestDir::new("themedir-list");
        let home = sandbox.path();
        let state = home.join(".local/state");
        install(home, ".local/state/omarchy/current", "background='#111111'\n");
        let names: Vec<String> = list(home, &state).into_iter().map(|entry| entry.name).collect();
        assert_eq!(names, [NONE, OWN, DESKTOP]);
    }
}
