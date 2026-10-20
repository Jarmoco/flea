//! Where the theme lives, decided once at launch and named in the environment.
//!
//! The palette reader reads the theme's `colors.toml` and `shell.toml`, and the
//! window reads the file that names the theme. Both are paths, and a path has
//! to be decided in one place, so the launcher decides here and the interface
//! reads what it is given.
//!
//! The operator chooses a theme by name, and the name is one of three things:
//!
//! * `desktop`, the desktop shell's own current theme, which is the shipped
//!   default so that a box which had one keeps it.
//! * `""`, no theme at all, so Flea draws with the palette of its own.
//! * the name of a theme directory, which is looked for in the theme
//!   directories below.
//!
//! The theme directories, in the order they are searched:
//!
//! * `~/.config/flea/themes`, the operator's own.
//! * `/usr/share/flea/themes`, the packaged ones.
//! * The desktop shell's own collection, so a user who already has themes
//!   keeps them and can pick one of them here.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The environment variable naming the theme's directory.
pub const DIR_ENV: &str = "FLEA_THEME_DIR";
/// The environment variable naming the file that reports a theme swap.
pub const NAME_ENV: &str = "FLEA_THEME_NAME";
/// The environment variable saying the launcher decided and the answer is no
/// theme at all.
///
/// It is what tells "the operator chose the palette of Flea's own" from "the
/// launcher named no theme", so a window started without the launcher still
/// falls back to the desktop shell's own theme while a window started with it
/// does not.
pub const NONE_ENV: &str = "FLEA_THEME_NONE";
/// The environment variable naming the machine-level shell.toml.
pub const SHELL_ENV: &str = "FLEA_SHELL_TOML";
/// The environment variable carrying the whole list of themes this box has.
///
/// The window cannot enumerate a directory, and the launcher already reads the
/// theme directories to resolve the one the operator chose, so the list travels
/// in the environment beside the choice. It is one JSON array, each entry
/// carrying the name the theme is stored under, the directory it lives in and
/// the file that names it.
pub const LIST_ENV: &str = "FLEA_THEMES";

/// The name that means the desktop shell's own theme, and the shipped default.
pub const DESKTOP: &str = "desktop";
/// The name that means no theme at all: Flea's own palette.
pub const NONE: &str = "";

/// A theme's directory and the file that names it.
pub struct Theme {
    pub dir: PathBuf,
    pub name: PathBuf,
}

/// One theme the operator can pick.
pub struct Entry {
    /// The name it is stored and offered under.
    pub name: String,
    /// The directory that holds its `colors.toml`.
    pub dir: PathBuf,
    /// The file that names the theme, which for the desktop shell's own theme
    /// sits beside the theme's directory rather than inside it.
    pub file: PathBuf,
}

/// A theme's own name is one path segment: no separator and no control
/// character, so a stored name can never reach outside the directories it is
/// looked for in.
pub fn is_theme_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.chars().any(|c| c.is_control())
}

/// The directories a theme's own name is looked for in, in order. The ones
/// that are not there are dropped, so a box without them pays nothing.
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

/// The desktop shell's own current theme, when it has one.
fn desktop_theme(state_home: &Path) -> Option<Theme> {
    let dir = state_home.join("omarchy/current/theme");
    if !dir.is_dir() {
        return None;
    }
    // The shell replaces its theme directory wholesale, so the file that names
    // it sits beside that directory and is rewritten in place.
    Some(Theme {
        name: state_home.join("omarchy/current/theme.name"),
        dir,
    })
}

/// The theme a stored name names, or none when this box has no such theme.
pub fn resolve(home: &Path, state_home: &Path, name: &str) -> Option<Theme> {
    // An operator's own choice is a choice, not a candidate: it is taken as
    // given, whether or not the directory is there yet.
    if let Some(explicit) = std::env::var_os(DIR_ENV).filter(|value| !value.is_empty()) {
        let dir = PathBuf::from(explicit);
        let name = dir.join("name");
        return Some(Theme { dir, name });
    }
    if name == NONE {
        return None;
    }
    if name == DESKTOP {
        return desktop_theme(state_home);
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

/// The themes this box has: the desktop shell's own, and every theme directory
/// found, by name.
pub fn list(home: &Path, state_home: &Path) -> Vec<Entry> {
    // Flea's own palette is the first entry and carries no directory: it is the
    // palette of Flea's own, with no theme file to read.
    let mut out: Vec<Entry> = vec![Entry {
        name: NONE.to_string(),
        dir: PathBuf::new(),
        file: PathBuf::new(),
    }];
    // The desktop shell's own theme is always offered, whether or not the shell
    // has one: choosing it means following the shell, and a box with no shell
    // theme under it draws with the palette of Flea's own.
    match desktop_theme(state_home) {
        Some(theme) => out.push(Entry {
            name: DESKTOP.to_string(),
            dir: theme.dir,
            file: theme.name,
        }),
        None => out.push(Entry {
            name: DESKTOP.to_string(),
            dir: PathBuf::new(),
            file: PathBuf::new(),
        }),
    }
    for dir in search_dirs(home) {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut names: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name != DESKTOP)
            .collect();
        names.sort();
        for name in names {
            if out.iter().any(|entry| entry.name == name) {
                continue;
            }
            let path = dir.join(&name);
            // A theme is a directory with colours in it; anything else is not one.
            if !path.join("colors.toml").is_file() {
                continue;
            }
            out.push(Entry {
                name,
                file: path.join("name"),
                dir: path,
            });
        }
    }
    out
}

/// The list of themes, as the JSON array the window reads out of the
/// environment.
///
/// The window cannot enumerate a directory, so the launcher reads the theme
/// directories once and names the whole list beside the one it chose. Each
/// entry carries the name the theme is stored under, the directory it lives in
/// and the file that names it, because the desktop shell's own theme is named
/// by a file beside its directory and not by one inside it.
pub fn list_json(home: &Path, state_home: &Path) -> String {
    let items: Vec<String> = list(home, state_home)
        .iter()
        .map(|entry| {
            format!(
                r#"{{"name":"{}","dir":"{}","file":"{}"}}"#,
                crate::json::escape(&entry.name),
                crate::json::escape(&entry.dir.to_string_lossy()),
                crate::json::escape(&entry.file.to_string_lossy()),
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// The machine-level shell.toml, which layers over the theme's own.
pub fn user_shell(home: &Path) -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os(SHELL_ENV).filter(|value| !value.is_empty()) {
        let path = PathBuf::from(explicit);
        return path.is_file().then_some(path);
    }
    let own = home.join(".config/flea/shell.toml");
    if own.is_file() {
        return Some(own);
    }
    let shell = home.join(".config/omarchy/shell.toml");
    shell.is_file().then_some(shell)
}

/// The state directory the desktop shell writes its current theme under.
///
/// The shell writes it under the home directory whatever XDG_STATE_HOME says,
/// so this follows the shell rather than the variable.
pub fn state_home(home: &Path) -> PathBuf {
    home.join(".local/state")
}

/// The icon theme the theme names, if it names one.
pub fn icon_theme(theme: Option<&Theme>) -> Option<String> {
    let theme = theme?;
    // Sample input, the whole file: "Yaru-purple\n"
    let text = std::fs::read_to_string(theme.dir.join("icons.theme")).ok()?;
    let name = text.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// The theme this box draws with, resolved once.
pub fn current() -> Option<Theme> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    resolve(&home, &state_home(&home), DESKTOP)
}

/// The theme name a state document holds, or the default when it holds none.
///
/// The document is the settled one, so a name that failed its rule is already the
/// default by the time it arrives here.
pub fn chosen(doc: &crate::jsondoc::Json) -> String {
    doc.get("theme")
        .and_then(|theme| theme.get("name"))
        .and_then(|name| name.as_str())
        .filter(|name| *name == NONE || *name == DESKTOP || is_theme_name(name))
        .unwrap_or(DESKTOP)
        .to_string()
}

/// The operator's own colours, from the state document, as the roles and their hexes.
///
/// A role the terminal interface does not draw with is the window's own, so the
/// map is handed over whole and each reader takes the roles it knows.
pub fn own_colors(doc: &crate::jsondoc::Json) -> Vec<(String, String)> {
    let Some(colors) = doc
        .get("theme")
        .and_then(|theme| theme.get("colors"))
        .and_then(|colors| colors.as_object())
    else {
        return Vec::new();
    };
    colors
        .iter()
        .filter_map(|(role, hex)| {
            let value = hex.as_str()?;
            crate::uistate::is_a_colour(value).then(|| (role.clone(), value.to_string()))
        })
        .collect()
}

/// The theme name the state file holds, or the default when it holds none.
pub fn stored_name() -> String {
    match crate::uistore::Store::user() {
        Ok(store) => chosen(&store.read()),
        Err(_) => DESKTOP.to_string(),
    }
}

/// Name the theme's paths in the environment, so the interface reads one answer
/// rather than resolving a path of its own.
///
/// The whole list of themes travels beside the choice: the window cannot
/// enumerate a directory, and the launcher is already reading them to resolve
/// the one the operator chose.
pub fn apply(cmd: &mut Command, name: &str) {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return;
    };
    let state_home = state_home(&home);
    if let Some(theme) = resolve(&home, &state_home, name) {
        cmd.env(DIR_ENV, &theme.dir);
        cmd.env(NAME_ENV, &theme.name);
        cmd.env_remove(NONE_ENV);
    } else {
        // No theme at all is an answer too: the window draws with the palette of
        // Flea's own rather than with whatever the environment happened to hold.
        cmd.env_remove(DIR_ENV);
        cmd.env_remove(NAME_ENV);
        cmd.env(NONE_ENV, "1");
    }
    cmd.env(LIST_ENV, list_json(&home, &state_home));
    if let Some(shell) = user_shell(&home) {
        cmd.env(SHELL_ENV, &shell);
    }
}

#[cfg(test)]
#[path = "themedir_tests.rs"]
mod tests;
