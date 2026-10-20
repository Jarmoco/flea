.pragma library

// The Settings panel's Theme section: which theme to draw with, and the colours the
// operator's own over it.
//
// A theme is a directory with a colors.toml in it, and the panel lists the ones
// this box has. The backend reads them, because the window cannot list a
// directory itself; each entry carries the name it is stored under and the
// directory it lives in, so the window can set the theme itself when the
// operator picks one.
//
// The colours are the operator's own over the theme's: one hex for every role
// the app draws with, and a role the map does not name keeps the theme's own.

// The accents the operator can choose, the theme's own first.
var ACCENTS = [
    { name: "The theme's own", hex: "" },
    { name: "Blue", hex: "#4c8bf5" },
    { name: "Sky", hex: "#2fa8d4" },
    { name: "Teal", hex: "#2aa198" },
    { name: "Green", hex: "#4a9e5c" },
    { name: "Yellow", hex: "#c99a1e" },
    { name: "Orange", hex: "#d1703a" },
    { name: "Red", hex: "#cf5c6e" },
    { name: "Pink", hex: "#c86bb0" },
    { name: "Purple", hex: "#8a6bd1" },
    { name: "Grey", hex: "#7a8391" }
]

// The word Flea's own palette is shown under, which is also the name it is
// stored under: no theme at all.
var OWN_NAME = ""
var OWN_LABEL = "Flea's own"

// The desktop shell's own theme is offered under its own name, but the name is
// an implementation word, so the panel shows what it means.
var DESKTOP_NAME = "desktop"
var DESKTOP_LABEL = "The desktop shell's theme"

// The menu's own entry, which opens the row's editor instead of storing a
// colour: the way the accent takes a colour no named one carries.
var CUSTOM = "custom"

// The colour roles the operator can set, one per colour the app draws with, in
// the order they are drawn: the two grounds, then the inks, then the frames and
// the three marks a file kind carries. src/uischema.rs holds the same list, and
// its test reads this file to hold the two together.
var ROLES = [
    { role: "background", label: "Background" },
    { role: "surface", label: "Surface" },
    { role: "foreground", label: "Text" },
    { role: "foregroundBright", label: "Bright text" },
    { role: "muted", label: "Muted text" },
    { role: "mutedFrame", label: "Muted frames" },
    { role: "metadata", label: "Metadata" },
    { role: "metadataFrame", label: "Metadata glyphs" },
    { role: "accent", label: "Accent" },
    { role: "accentFrame", label: "Accent frames" },
    { role: "error", label: "Error" },
    { role: "errorOnSurface", label: "Status error" },
    { role: "symlink", label: "Symlink" },
    { role: "executable", label: "Executable" }
]

// The label of a stored value, or the value itself when the list does not carry
// it: a theme that was removed, or a colour a hand edit put there.
function labelFor(labels, values, stored) {
    var at = values.indexOf(stored)
    if (at >= 0)
        return labels[at]
    return stored === OWN_NAME ? OWN_LABEL : stored
}

// The themes this box has, as the names they are stored under.
function themeNames(themes) {
    var names = []
    for (var i = 0; themes && i < themes.length; i++)
        names.push(themes[i].name)
    return names
}

// The next value a step lands on. The menu's own custom entry is never one of
// them: a step walks the colours that can be picked, and the editor is reached
// with Enter.
function stepValue(values, selected, direction) {
    if (!values || values.length === 0)
        return ""
    var at = values.indexOf(selected)
    var count = values.length
    for (var i = 1; i <= count; i++) {
        var next = (at + direction * i + count * count) % count
        if (values[next] !== CUSTOM)
            return values[next]
    }
    return ""
}

// A colour row: the swatch in its mark slot is the colour the app draws with,
// and Enter opens the row's own editor, where a hex is typed.
function colorRow(entry, hex) {
    return { kind: "color", id: "theme.colors." + entry.role, colorRole: entry.role,
             label: entry.label, hex: hex,
             value: hex === "" ? "The theme's own" : hex }
}

// The accent's own row: the named mid-tones the operator can pick, and the
// row's editor for any other colour. Every colour offered is a mid-tone, which
// reads on a dark ground and on a light one, so one list serves both.
function accentRow(hex) {
    var values = []
    var labels = []
    var swatches = []
    for (var i = 0; i < ACCENTS.length; i++) {
        values.push(ACCENTS[i].hex)
        labels.push(ACCENTS[i].name)
        swatches.push(ACCENTS[i].hex)
    }
    values.push(CUSTOM)
    labels.push("Custom colour…")
    swatches.push("")
    return { kind: "choice", id: "theme.colors.accent", colorRole: "accent", label: "Accent",
             values: values, labels: labels, swatches: swatches, hex: hex,
             value: labelFor(labels, values, hex), selected: hex, menu: true }
}

// state: { themes, themeName, themeColors, themeDir }
function rows(state) {
    var themes = state.themes || []
    var names = themeNames(themes)
    var labels = []
    for (var i = 0; i < names.length; i++)
        labels.push(names[i] === OWN_NAME ? OWN_LABEL : names[i] === DESKTOP_NAME ? DESKTOP_LABEL : names[i])
    var stored = state.themeName === undefined ? OWN_NAME : state.themeName
    var colors = state.themeColors || {}
    var out = [
        { kind: "group", label: "Theme" },
        { kind: "choice", id: "theme.name", label: "Theme", glyph: "folder",
          values: names, labels: labels,
          value: labelFor(labels, names, stored), selected: stored, menu: true },
        { kind: "hint", label: "A theme is a directory with a colors.toml in it." },
        { kind: "fact", label: "Reading", glyph: "folder-open",
          value: state.themeDir ? state.themeDir : "Flea's own palette" },
        { kind: "group", label: "Colors" }
    ]
    for (var r = 0; r < ROLES.length; r++) {
        var entry = ROLES[r]
        var hex = typeof colors[entry.role] === "string" ? colors[entry.role] : ""
        out.push(entry.role === "accent" ? accentRow(hex) : colorRow(entry, hex))
    }
    out.push({ kind: "hint", label: "Enter edits a colour: type #rrggbb and press Enter. An empty value follows the theme." })
    out.push({ kind: "action", id: "theme.colors.reset", label: "Reset every colour", value: "Reset" })
    out.push({ kind: "group", label: "Your own themes" })
    out.push({ kind: "hint", label: "Put one in ~/.config/flea/themes/<name>/ and Flea offers it here." })
    return out
}
