# The theme

Flea draws with a theme of the operator's choosing, and the choice is a setting rather than a
launch-time decision. Three places hold it, and each has to agree: the state file, which
persists it; the launcher, which resolves it so the first paint already carries it; and the
palette reader, which redraws at once when the operator picks another one.

**A theme is a directory with a `colors.toml` in it.** The directories searched, in order, are
`~/.config/flea/themes`, `/usr/share/flea/themes`, the desktop shell's own collection
(`~/.config/omarchy/themes` and `/usr/share/omarchy/themes`) and the shell's own current theme
under `~/.local/state/omarchy/current/theme`. A directory with no `colors.toml` in it is not a
theme and is not offered. Nothing is shipped: the list is what the operator and the desktop
shell have, and Flea's own palette is the first entry and has no directory at all, so there is
no theme file to read.

**The stored value is `theme.name` in `ui.json`,** and it is one of three things: `""`, no theme
at all, which is the palette of Flea's own; `"desktop"`, the desktop shell's own current theme,
which is the shipped default so a box which had one keeps it; or the name of a theme directory.
The name is one path segment with no separator and no control character, so a stored name can
never reach outside the directories it is looked for in; `src/uistate.rs`'s `Rule::ThemeName`
refuses anything else and the default costs the key, so a hand-edited file cannot name a path.
`src/themedir.rs` owns the resolution and the list, and `src/uischema.rs` the shape.

**The launcher resolves the name once and names it in the environment,** so the window's first
paint already carries the theme and there is no flash of the wrong one. `FLEA_THEME_DIR` is the
directory the palette reads, `FLEA_THEME_NAME` the file that reports a theme swap, and
`FLEA_THEMES` the whole list as one JSON array, each entry carrying the name the theme is
stored under, the directory it lives in and the file that names it. `FLEA_THEME_NONE` is the
launcher saying the answer is **no theme**, and it is load-bearing: without it a window would
not be able to tell "the operator chose the palette of Flea's own" from "the launcher named no
theme", and the second would fall back to the desktop shell's own theme. A window the launcher
did not name a theme for falls back to that theme, which is what a direct `qs -p ui/boot`
launch and `tests/glass.sh` both rely on. The list travels beside the
choice because the window cannot enumerate a directory and the launcher is already reading them
to resolve the one the operator chose. **The list is read at launch, so a theme added while
Flea is open is offered at the next launch**; that is the one cost of carrying it in the
environment rather than on the wire.

**The palette reader is Flea's own, in `ui/Commons/Color.qml`,** and the window no longer reads
the desktop shell's copies of the token layer or the widget kit: `ui/Commons/` and `ui/Ui/` are
Flea's own, and the desktop shell is an optdepend rather than a dependency, so the window opens
on a box that has no shell at all. The palette's theme directory is a writable property, so
choosing a theme sets it and the file behind it is read again at once. **A theme directory of
`""` is the palette of Flea's own, and it resets the roles rather than leaving them holding the
last theme's colours**, which is the case the launcher answers with no theme at all.

**Every colour the app draws with is the operator's own over the theme's,** and they are all in
one place: `theme.colors` in `ui.json`, a map of role to hex triple. The roles are the two
grounds (`background`, `surface`), the inks (`foreground`, `foregroundBright`, `muted`,
`mutedFrame`, `metadata`, `metadataFrame`), the accent and its frame (`accent`,
`accentFrame`), the two error inks (`error`, `errorOnSurface`) and the two marks a file kind
carries (`symlink`, `executable`). **A role the map does not name keeps the theme's own colour
for it**, which is every role until the operator sets one; `src/uischema.rs`'s `COLOR_ROLES`
is the list of the roles there are, and `Rule::Colors` refuses a role that is not one of them
and a value that is not a hex triple, so a hand-edited file cannot name a colour Flea does not
draw with. A map patch names one role and leaves the rest standing, and a `null` forgets one
role, which is how a row goes back to the theme's own.

**The accent is one of those colours,** and it was `theme.accent` before: the 4.0 migration
moves a stored accent into the map, so a file that named one keeps it. The accent keeps two
things the other roles do not have. **Its named mid-tones stay**, and every colour offered is a
mid-tone, which reads on a dark ground and on a light one, so one list serves both; they are
the accent row's own menu, and the row's editor takes any other colour beside them. **And the
chosen accent is lifted against the ground it is drawn on before it is drawn,** so a pale accent
on a light theme still reads; `ui/ThemeColors.qml`'s `opaqueAccent` is that lift and the frame
beside it is the same accent lifted against the surface, which is a step lighter. Every other
role is drawn exactly as the operator set it.

**The palette reader owns the colours and the state owns the choice,** the same way the accent
was before them: `ui/Commons/Color.qml` carries the theme's own values and the operator's own
over them, and `ui/ViewState.qml` puts the state's map into the reader on every change of the
document. **A change of the map lays the palette over again,** so the window redraws in the new
colours at once, and the roles Flea derives from others follow the operator's own where they set
one: the ground they set is the ground the frames are lifted against, and the text they set is
the text the metadata is derived from. A role they did not set keeps the theme's own. The
terminal interface draws with the same colours: `src/tui/theme.rs` lays the map over the theme
it reads, for the roles it draws with.

**The Theme page is a section of Settings,** and the section is `ui/js/SettingsTheme.js`'s model
plus `ui/ThemePicker.qml`'s one line that puts a chosen theme's directory into the palette
reader. The page is built with the panel, so a window that never opens Settings carries none of
it. **A name the list does not carry leaves the palette alone**: the launcher resolved the
stored name before the window opened, and a theme that has been removed since is not the
page's to replace. The desktop shell's own theme is always offered, whether or not the shell has
one, because choosing it means following the shell and a box with no shell theme under it draws
with the palette of Flea's own.

**A colour row is a row of its own,** and the swatch in its mark slot is the colour the app
draws with for that role, so the row shows the operator what they are about to change. Enter
opens the row's own editor, `ui/SettingsColorField.qml`, where a hex is typed and the swatch
previews it as it is typed; an empty field is the theme's own colour, and a hex that is not a
colour is refused and the field stays open for it to be typed again. The editor takes the
keyboard while it is open and the panel takes it back when it closes.

`tests/js/settingstheme.js` holds the rows model, `tests/theme-picker.qml` the plumbing that
puts a theme on the screen, driven headless by `tests/theme-picker.sh` with the list named in
the environment, which is how the launcher hands it over, `tests/theme-colors.qml` the
colours themselves reaching the palette, driven headless by `tests/theme-colors.sh`, and
`tests/theme-editor.qml` the panel's own row and its editor, driven headless by
`tests/theme-editor.sh`.
