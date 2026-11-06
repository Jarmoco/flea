//! Discovery of installed icon themes, in precedence order.
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconPack {
    pub name: String,
    pub directory: PathBuf,
}

/// List each theme once, preferring the first root that contains its index.theme.
pub fn discover(roots: &[PathBuf]) -> Vec<IconPack> {
    let mut packs = Vec::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let directory = entry.path();
            if name.len() <= 255
                && name != "."
                && name != ".."
                && !name.chars().any(char::is_control)
                && directory.join("index.theme").is_file()
                && !packs.iter().any(|p: &IconPack| p.name == name)
            {
                packs.push(IconPack { name, directory });
            }
        }
    }
    packs
}

/// The icon search roots for this process.
pub fn session_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    else {
        return Vec::new();
    };
    let data_home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let data_dirs = std::env::var_os("XDG_DATA_DIRS")
        .filter(|value| !value.is_empty())
        .map(|value| std::env::split_paths(&value).collect())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });
    roots(&home, data_home.as_deref(), &data_dirs, Path::new("/snap"))
}

/// Discover themes for this process without creating or modifying any directories.
pub fn installed() -> Vec<IconPack> {
    discover(&session_roots())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackPreview {
    pub pack: IconPack,
    pub icons: [Option<PathBuf>; 6],
}

/// Discover packs and resolve preview paths. Call on a worker, not during painting.
pub fn catalog(roots: &[PathBuf], size: u32, scale: u32) -> Vec<PackPreview> {
    discover(roots)
        .into_iter()
        .map(|pack| {
            let icons = previews(&pack.name, roots.to_vec(), size, scale);
            PackPreview { pack, icons }
        })
        .collect()
}

/// Freedesktop icon names for the sidebar's places and devices.
/// Non-place glyphs return None and continue to use their existing artwork.
pub fn sidebar_icon(glyph: &str) -> Option<&'static str> {
    match glyph {
        "house" => Some("user-home"),
        "folder" | "folder-git-2" => Some("folder"),
        "download" => Some("folder-download"),
        "file-text" => Some("folder-documents"),
        "image" => Some("folder-pictures"),
        "film" => Some("folder-videos"),
        "music" => Some("folder-music"),
        "history" => Some("document-open-recent"),
        "trash" => Some("user-trash"),
        "network" => Some("network-workgroup"),
        "server" => Some("network-server"),
        "drive" => Some("drive-harddisk"),
        "smartphone" => Some("phone"),
        "camera" => Some("camera-photo"),
        _ => None,
    }
}

/// Representative file and place roles shown in the theme chooser.
pub const PREVIEW_ICONS: [&str; 6] = [
    "folder",
    "text-x-generic",
    "image-x-generic",
    "video-x-generic",
    "audio-x-generic",
    "application-x-executable",
];

/// Resolve a pack's previews on a worker; a missing role remains a built-in fallback.
pub fn previews(theme: &str, roots: Vec<PathBuf>, size: u32, scale: u32) -> [Option<PathBuf>; 6] {
    let icons = resolver(theme, roots);
    PREVIEW_ICONS.map(|name| icons.resolve(name, size, scale))
}

/// An icon theme lookup using exactly the same roots as the chooser.
pub fn resolver(theme: &str, roots: Vec<PathBuf>) -> crate::ui_logic_op3::openwith::icons::Icons {
    crate::ui_logic_op3::openwith::icons::Icons {
        theme: theme.to_owned(),
        bases: roots,
        pixmaps: Vec::new(),
    }
}

/// Standard per-user, XDG, Flatpak and Snap icon locations.
pub fn roots(
    home: &Path,
    data_home: Option<&Path>,
    data_dirs: &[PathBuf],
    snap: &Path,
) -> Vec<PathBuf> {
    let mut roots = vec![
        data_home
            .filter(|p| p.is_absolute())
            .unwrap_or(&home.join(".local/share"))
            .join("icons"),
        home.join(".local/share/icons"),
        home.join(".icons"),
    ];
    roots.extend(
        data_dirs
            .iter()
            .filter(|p| p.is_absolute())
            .map(|p| p.join("icons")),
    );
    roots.extend([
        PathBuf::from("/usr/local/share/icons"),
        PathBuf::from("/usr/share/icons"),
        home.join(".local/share/flatpak/exports/share/icons"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons"),
        PathBuf::from("/var/lib/snapd/desktop/icons"),
    ]);
    if let Ok(entries) = std::fs::read_dir(snap) {
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.file_name());
        roots.extend(
            entries
                .iter()
                .map(|entry| entry.path().join("current/usr/share/icons")),
        );
    }
    let mut unique = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testdir::TestDir;

    #[test]
    fn sidebar_roles_leave_unrelated_interface_glyphs_alone() {
        assert_eq!(sidebar_icon("house"), Some("user-home"));
        assert_eq!(sidebar_icon("trash"), Some("user-trash"));
        assert_eq!(sidebar_icon("drive"), Some("drive-harddisk"));
        assert_eq!(sidebar_icon("folder"), Some("folder"));
        assert_eq!(sidebar_icon("download"), Some("folder-download"));
        assert_eq!(sidebar_icon("file-text"), Some("folder-documents"));
        assert_eq!(sidebar_icon("image"), Some("folder-pictures"));
        assert_eq!(sidebar_icon("film"), Some("folder-videos"));
        assert_eq!(sidebar_icon("eject"), None);
        assert_eq!(sidebar_icon("tag"), None);
        assert_eq!(sidebar_icon("plugin-unknown"), None);
    }

    #[test]
    fn choice_is_saved_and_rejects_paths() {
        let current = crate::uischema::defaults();
        for name in ["", "Papirus-Dark", "My icons"] {
            let patch = crate::jsondoc::Json::Obj(vec![(
                "iconTheme".into(),
                crate::jsondoc::Json::Str(name.into()),
            )]);
            let saved = crate::uistate::patched(&current, &patch).unwrap();
            assert_eq!(saved.get("iconTheme").and_then(|v| v.as_str()), Some(name));
        }
        for name in ["..", "../other", "/absolute", "bad\nname"] {
            let patch = crate::jsondoc::Json::Obj(vec![(
                "iconTheme".into(),
                crate::jsondoc::Json::Str(name.into()),
            )]);
            assert!(crate::uistate::patched(&current, &patch).is_err());
        }
    }

    #[test]
    fn roots_include_user_flatpak_snap_and_system_locations() {
        let fixture = TestDir::new("icon-pack-roots");
        let home = fixture.path().join("home");
        let snap = fixture.path().join("snap");
        std::fs::create_dir_all(snap.join("editor")).unwrap();
        let found = roots(&home, None, &[], &snap);
        for path in [
            home.join(".local/share/icons"),
            home.join(".icons"),
            PathBuf::from("/usr/share/icons"),
            PathBuf::from("/usr/local/share/icons"),
            home.join(".local/share/flatpak/exports/share/icons"),
            PathBuf::from("/var/lib/flatpak/exports/share/icons"),
            PathBuf::from("/var/lib/snapd/desktop/icons"),
            snap.join("editor/current/usr/share/icons"),
        ] {
            assert!(found.contains(&path), "missing {}", path.display());
        }
    }

    #[test]
    fn resolver_uses_the_discovered_roots_for_the_selected_theme() {
        let fixture = TestDir::new("icon-pack-resolver");
        let root = fixture.path().join("icons");
        let theme = root.join("Example");
        std::fs::create_dir_all(theme.join("24/places")).unwrap();
        std::fs::write(
            theme.join("index.theme"),
            "[Icon Theme]\nDirectories=24/places\n[24/places]\nSize=24\nType=Fixed\n",
        )
        .unwrap();
        let folder = theme.join("24/places/folder.svg");
        std::fs::write(&folder, "<svg/>").unwrap();
        let catalog = catalog(&[root], 24, 1);
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].pack.name, "Example");
        assert_eq!(catalog[0].icons[0], Some(folder));
        assert!(catalog[0].icons[1..].iter().all(Option::is_none));
    }

    #[test]
    fn themes_require_an_index_and_user_roots_win_duplicates() {
        let fixture = TestDir::new("icon-pack-discovery");
        let first = fixture.path().join("first");
        let second = fixture.path().join("second");
        for root in [&first, &second] {
            std::fs::create_dir_all(root.join("Example")).unwrap();
            std::fs::write(
                root.join("Example/index.theme"),
                "[Icon Theme]\nName=Example\n",
            )
            .unwrap();
        }
        std::fs::create_dir_all(first.join("not-a-theme")).unwrap();
        let packs = discover(&[first.clone(), second]);
        assert_eq!(
            packs,
            vec![IconPack {
                name: "Example".into(),
                directory: first.join("Example")
            }]
        );
    }
}
