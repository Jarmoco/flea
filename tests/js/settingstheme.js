.import "../../ui/js/Settings.js" as Settings
.import "../../ui/js/SettingsTheme.js" as SettingsTheme

// The Theme section: the themes this box has and the choice among them, and the
// colours the operator's own over the theme's.

function find(rows, id) {
    return rows.filter(function (row) { return row.id === id })[0] || {}
}

// The three themes the backend answered with: Flea's own, the desktop shell's,
// and one the operator put in their own directory.
var THEMES = [
    { name: "", dir: "", file: "" },
    { name: "desktop", dir: "/home/u/.local/state/omarchy/current/theme", file: "/home/u/.local/state/omarchy/current/theme/name" },
    { name: "rose-pine", dir: "/home/u/.config/flea/themes/rose-pine", file: "/home/u/.config/flea/themes/rose-pine/name" }
]

function rows(themeName, themeColors, themeDir) {
    return Settings.rows("theme", {
        themes: THEMES,
        themeName: themeName === undefined ? "rose-pine" : themeName,
        themeColors: themeColors === undefined ? {} : themeColors,
        themeDir: themeDir === undefined ? "/home/u/.config/flea/themes/rose-pine" : themeDir
    })
}

function run(check) {
    var list = rows()
    check("the section is one the panel can open", Settings.sectionIndex("theme") > 0, true)
    check("and it sits beside Display, which is where the theme's own size lives",
          Settings.SECTIONS[Settings.sectionIndex("theme") - 1].id, "display")

    var theme = find(list, "theme.name")
    check("the Theme row offers every theme the box has",
          theme.values.join("|"), "|desktop|rose-pine")
    check("and shows the chosen one", theme.value, "rose-pine")
    check("which is the one stored", theme.selected, "rose-pine")
    check("Flea's own palette is offered first, and says so", theme.labels[0], "Flea's own")
    check("a theme whose name is its own shows its own name", theme.labels[2], "rose-pine")
    check("the desktop shell's own theme says what it means", theme.labels[1], "The desktop shell's theme")
    check("and it is a stop the cursor rests on", Settings.focusable(theme), true)

    check("the directory in use is a fact, and it is where the theme lives",
          find(list, "theme.name").kind + "|" + find(list, "theme.name").value, "choice|rose-pine")
    var directory = list.filter(function (row) { return row.label === "Reading" })[0] || {}
    check("the reading row names the directory the palette reads",
          directory.value, "/home/u/.config/flea/themes/rose-pine")
    check("and Flea's own palette reads no directory at all",
          rows("rose-pine", {}, "").filter(function (row) { return row.label === "Reading" })[0].value, "Flea's own palette")

    check("a theme the list does not carry still shows its own name",
          find(rows("gone", {}, ""), "theme.name").value, "gone")

    // The colours: one row per role the app draws with, and every one of them a
    // control the cursor rests on.
    var colors = list.filter(function (row) { return row.kind === "color" })
    check("every colour but the accent is a row of its own",
          colors.length, SettingsTheme.ROLES.length - 1)
    check("and every one of them is a stop the cursor rests on",
          colors.every(function (row) { return Settings.focusable(row) }), true)
    check("the background is the first of them", colors[0].colorRole, "background")
    check("and the text is the third", colors[2].colorRole, "foreground")
    check("a colour the operator did not set follows the theme",
          find(list, "theme.colors.background").value, "The theme's own")
    check("and one they did set is named by its hex",
          find(rows("rose-pine", { background: "#101315" }, ""), "theme.colors.background").value, "#101315")
    check("every role the app draws with is offered",
          SettingsTheme.ROLES.map(function (entry) { return entry.role }).join("|"),
          "background|surface|foreground|foregroundBright|muted|mutedFrame|metadata|metadataFrame|accent|accentFrame|error|errorOnSurface|symlink|executable")

    // The accent keeps its named mid-tones, and takes any other colour beside them.
    var accent = find(list, "theme.colors.accent")
    check("the accent offers the theme's own first", accent.labels[0], "The theme's own")
    check("and a set of named ones after it", accent.labels.length > 3, true)
    check("the theme's own accent is the empty one, which is what it stores", accent.values[0], "")
    check("and every colour in the list is a mid-tone hex",
          accent.values.slice(1, accent.values.length - 1).every(function (hex) { return /^#[0-9a-f]{6}$/.test(hex) }), true)
    check("the last entry is the row's own editor, which takes any colour",
          accent.labels[accent.labels.length - 1], "Custom colour…")
    check("and it is not a colour a step can land on",
          SettingsTheme.stepValue(accent.values, accent.values[1], 1), accent.values[2])
    check("the chosen accent is the one stored", accent.selected, "")
    check("both choices open a submenu, so a long list reads as a menu rather than a cycle",
          theme.menu === true && accent.menu === true, true)
    check("the accent's swatches are its own values, so the menu can draw each colour",
          accent.swatches.slice(0, accent.swatches.length - 1).join("|"), accent.values.slice(0, accent.values.length - 1).join("|"))
    check("and an accent a hand edit put there shows itself",
          find(rows("rose-pine", { accent: "#123456" }, ""), "theme.colors.accent").value, "#123456")

    check("every colour back to the theme's own is one action",
          find(list, "theme.colors.reset").kind + "|" + find(list, "theme.colors.reset").value, "action|Reset")

    check("the hint says what a theme is",
          list.filter(function (row) { return row.kind === "hint" })[0].label,
          "A theme is a directory with a colors.toml in it.")
    check("and the last hint says where the operator's own go",
          list.filter(function (row) { return row.kind === "hint" })[list.filter(function (row) { return row.kind === "hint" }).length - 1].label,
          "Put one in ~/.config/flea/themes/<name>/ and Flea offers it here.")
}
