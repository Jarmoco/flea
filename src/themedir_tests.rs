//! The theme module's own tests, beside it so the module itself stays inside its budget.

use super::*;

fn sandbox(tag: &str) -> (PathBuf, PathBuf) {
    let dir = crate::backend::testdir::TestDir::new(tag);
    let home = dir.path().join("home");
    let state = dir.path().join("state");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    (home, state)
}

fn write_theme(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("colors.toml"), "background = \"#101315\"\n").unwrap();
    path
}

#[test]
fn a_box_with_no_theme_at_all_resolves_to_none() {
    let (home, state) = sandbox("theme-none");
    assert!(resolve(&home, &state, DESKTOP).is_none());
    assert!(resolve(&home, &state, NONE).is_none());
    assert!(user_shell(&home).is_none());
}

#[test]
fn the_desktop_shells_own_theme_is_found_when_it_is_there() {
    let (home, state) = sandbox("theme-shell");
    let theme = write_theme(&state.join("omarchy/current"), "theme");
    let found = resolve(&home, &state, DESKTOP).expect("the shell's theme is found");
    assert_eq!(found.dir, theme);
    assert_eq!(found.name, state.join("omarchy/current/theme.name"));
}

#[test]
fn a_named_theme_is_found_in_the_operators_own_directory() {
    let (home, state) = sandbox("theme-own");
    let theme = write_theme(&home.join(".config/flea/themes"), "rose-pine");
    let found = resolve(&home, &state, "rose-pine").expect("the theme is found");
    assert_eq!(found.dir, theme);
    assert_eq!(found.name, theme.join("name"));
}

#[test]
fn no_theme_is_no_theme_at_all() {
    let (home, state) = sandbox("theme-none-named");
    assert!(resolve(&home, &state, NONE).is_none());
}

#[test]
fn a_name_that_is_not_one_segment_never_reaches_outside() {
    let (home, state) = sandbox("theme-escape");
    std::fs::create_dir_all(home.join(".config/flea/themes")).unwrap();
    std::fs::write(home.join(".config/flea/secret"), "x").unwrap();
    assert!(resolve(&home, &state, "../secret").is_none());
    assert!(resolve(&home, &state, "a/b").is_none());
    assert!(!is_theme_name(""));
    assert!(!is_theme_name("a/b"));
    assert!(is_theme_name("rose-pine"));
}

#[test]
fn the_list_carries_the_desktop_and_every_theme_found() {
    let (home, state) = sandbox("theme-list");
    write_theme(&state.join("omarchy/current"), "theme");
    write_theme(&home.join(".config/flea/themes"), "rose-pine");
    write_theme(&home.join(".config/flea/themes"), "catppuccin");
    // A directory without colours in it is not a theme.
    std::fs::create_dir_all(home.join(".config/flea/themes/empty")).unwrap();
    let found = list(&home, &state);
    let names: Vec<&str> = found.iter().map(|entry| entry.name.as_str()).collect();
    // Flea's own palette is first and carries no name, and the desktop's own
    // theme comes before the directories the operator's themes live in.
    assert_eq!(names, vec![NONE, DESKTOP, "catppuccin", "rose-pine"]);
    assert_eq!(found[0].dir, PathBuf::new());
}

#[test]
fn the_machine_level_shell_toml_is_fleas_own_when_it_is_there() {
    let (home, _state) = sandbox("theme-shell-toml");
    let own = home.join(".config/flea");
    std::fs::create_dir_all(&own).unwrap();
    std::fs::write(own.join("shell.toml"), "[flea]\n").unwrap();
    assert_eq!(user_shell(&home), Some(own.join("shell.toml")));
}

#[test]
fn an_icon_theme_is_read_from_the_theme_that_was_resolved() {
    let (home, state) = sandbox("theme-icons");
    let theme = write_theme(&state.join("omarchy/current"), "theme");
    std::fs::write(theme.join("icons.theme"), "Yaru-purple\n").unwrap();
    let found = resolve(&home, &state, DESKTOP).unwrap();
    assert_eq!(icon_theme(Some(&found)).as_deref(), Some("Yaru-purple"));
    assert_eq!(icon_theme(None), None);
}

#[test]
fn the_list_travels_as_one_json_array_the_window_can_read() {
    let (home, state) = sandbox("theme-list-json");
    write_theme(&state.join("omarchy/current"), "theme");
    write_theme(&home.join(".config/flea/themes"), "rose-pine");
    // Sample input: the whole list the launcher names in the environment.
    let json = list_json(&home, &state);
    // Flea's own palette is first and carries no directory and no name file.
    assert!(json.starts_with(r#"[{"name":"","dir":"","file":""}"#), "{}", json);
    // The desktop shell's own theme names itself with a file beside its
    // directory, and that file is what the window watches.
    assert!(
        json.contains(r#"{"name":"desktop","dir":"#) && json.contains("theme.name"),
        "{}",
        json
    );
    // A theme directory's own name file is inside it.
    assert!(json.contains(r#""name":"rose-pine""#), "{}", json);
    // The window reads this with JSON.parse, so it has to be one document.
    assert!(json.starts_with('[') && json.ends_with(']'), "{}", json);
}

#[test]
fn a_name_with_a_quote_in_it_comes_back_readable() {
    let (home, state) = sandbox("theme-quote");
    let odd = home.join(".config/flea/themes/a\"b");
    std::fs::create_dir_all(&odd).unwrap();
    std::fs::write(
        odd.join("colors.toml"),
        "background = \"#101315\"\n",
    )
    .unwrap();
    let json = list_json(&home, &state);
    assert!(json.contains(r#"a\"b"#), "{}", json);
}
