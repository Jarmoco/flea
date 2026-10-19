//! Where the theme lives, decided once at launch and named in the environment.
//!
//! The palette reader reads the theme's `colors.toml` and `shell.toml`, and the
//! window reads the file that names the theme. Both are paths, and a path has
//! to be decided in one place, so the launcher decides here and the interface
//! reads what it is given.
//!
//! The candidates, in order:
//!
//! * `FLEA_THEME_DIR`, which the operator set and which wins outright.
//! * The desktop shell's own theme, so a box that has one keeps following it.
//! * Flea's own, so a box with no desktop shell can still carry a theme.
//!
//! The theme's *name* file sits beside the shell's theme and inside Flea's,
//! because the shell replaces its theme directory wholesale and a file inside
//! it cannot report that, while a name file beside it is rewritten in place.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The environment variable naming the theme's directory.
pub const DIR_ENV: &str = "FLEA_THEME_DIR";
/// The environment variable naming the file that reports a theme swap.
pub const NAME_ENV: &str = "FLEA_THEME_NAME";
/// The environment variable naming the machine-level shell.toml.
pub const SHELL_ENV: &str = "FLEA_SHELL_TOML";

/// A theme's directory and the file that names it.
pub struct Theme {
    pub dir: PathBuf,
    pub name: PathBuf,
}

/// The candidates, in the order they are tried.
fn candidates(home: &Path, state_home: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut list = Vec::new();
    let shell_theme = state_home.join("omarchy/current/theme");
    list.push((shell_theme.clone(), state_home.join("omarchy/current/theme.name")));
    let own_theme = home.join(".config/flea/theme");
    list.push((own_theme.clone(), own_theme.join("name")));
    list
}

/// The theme to draw with, or none when this box carries no theme at all.
pub fn resolve(home: &Path, state_home: &Path) -> Option<Theme> {
    // An operator's own choice is a choice, not a candidate: it is taken as given,
    // whether or not the directory is there yet.
    if let Some(explicit) = std::env::var_os(DIR_ENV).filter(|value| !value.is_empty()) {
        let dir = PathBuf::from(explicit);
        let name = dir.join("name");
        return Some(Theme { dir, name });
    }
    candidates(home, state_home)
        .into_iter()
        .find(|(dir, _)| dir.is_dir())
        .map(|(dir, name)| Theme { dir, name })
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
    resolve(&home, &state_home(&home))
}

/// Name the theme's paths in the environment, so the interface reads one answer
/// rather than resolving a path of its own.
pub fn apply(cmd: &mut Command) {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = &home {
        if let Some(theme) = resolve(home, &state_home(home)) {
            cmd.env(DIR_ENV, &theme.dir);
            cmd.env(NAME_ENV, &theme.name);
        }
        if let Some(shell) = user_shell(home) {
            cmd.env(SHELL_ENV, &shell);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_with_no_theme_at_all_resolves_to_none() {
        let dir = crate::backend::testdir::TestDir::new("theme-none");
        let home = dir.path().join("home");
        let state = dir.path().join("state");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        assert!(resolve(&home, &state).is_none());
        assert!(user_shell(&home).is_none());
    }

    #[test]
    fn the_desktop_shells_own_theme_is_found_when_it_is_there() {
        let dir = crate::backend::testdir::TestDir::new("theme-shell");
        let home = dir.path().join("home");
        let state = dir.path().join("state");
        let theme = state.join("omarchy/current/theme");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let found = resolve(&home, &state).expect("the shell's theme is found");
        assert_eq!(found.dir, theme);
        assert_eq!(found.name, state.join("omarchy/current/theme.name"));
    }

    #[test]
    fn fleas_own_theme_is_found_when_there_is_no_desktop_shell() {
        let dir = crate::backend::testdir::TestDir::new("theme-own");
        let home = dir.path().join("home");
        let state = dir.path().join("state");
        let theme = home.join(".config/flea/theme");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let found = resolve(&home, &state).expect("Flea's own theme is found");
        assert_eq!(found.dir, theme);
        assert_eq!(found.name, theme.join("name"));
    }

    #[test]
    fn the_machine_level_shell_toml_is_fleas_own_when_it_is_there() {
        let dir = crate::backend::testdir::TestDir::new("theme-shell-toml");
        let home = dir.path().join("home");
        let own = home.join(".config/flea");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join("shell.toml"), "[flea]\n").unwrap();
        assert_eq!(user_shell(&home), Some(own.join("shell.toml")));
    }

    #[test]
    fn an_icon_theme_is_read_from_the_theme_that_was_resolved() {
        let dir = crate::backend::testdir::TestDir::new("theme-icons");
        let home = dir.path().join("home");
        let state = dir.path().join("state");
        let theme = state.join("omarchy/current/theme");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(theme.join("icons.theme"), "Yaru-purple\n").unwrap();
        let found = resolve(&home, &state).unwrap();
        assert_eq!(icon_theme(Some(&found)).as_deref(), Some("Yaru-purple"));
        assert_eq!(icon_theme(None), None);
    }
}
