pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

// The palette.
//
// The foundational roles come from the theme's colors.toml and the per-surface
// roles from the theme's shell.toml. Which theme that is, the launcher decides
// once, at launch, and names in FLEA_THEME_DIR; the paths below are the
// defaults for a shell started without one.
QtObject {
    id: root

    readonly property string home: Quickshell.env("HOME")
    // The desktop shell writes its current theme under the home directory
    // whatever XDG_STATE_HOME says, so this follows the shell.
    readonly property string stateHome: root.home + "/.local/state"
    readonly property string omarchyThemeDir: root.stateHome + "/omarchy/current/theme"

    // The directory the theme's colors.toml and shell.toml live in.
    readonly property string themeDir: Quickshell.env("FLEA_THEME_DIR") || root.omarchyThemeDir
    // The file whose change means the theme itself was swapped, which a file
    // inside the theme directory cannot report: the directory is replaced.
    readonly property string themeNamePath: Quickshell.env("FLEA_THEME_NAME") || (root.stateHome + "/omarchy/current/theme.name")
    // The machine-level shell.toml, which layers over the theme's own.
    readonly property string userShellPath: Quickshell.env("FLEA_SHELL_TOML") || (root.home + "/.config/omarchy/shell.toml")

    property color foreground: "#cacccc"
    property color background: "#101315"
    property color accent: "#cacccc"
    property color urgent: "#a55555"
    property color muted: "#707880"

    // Flat dictionary of "section.key" -> raw string from shell.toml.
    property var shellValues: ({})

    function pick(key, fallback) {
        var v = root.shellValues[key]
        return (typeof v === "string" && v.length > 0) ? v : fallback
    }

    function pickAlpha(key, fallback) {
        var v = root.shellValues[key]
        if (typeof v !== "string" || v.length === 0) return fallback
        var n = Number(v)
        if (!isFinite(n)) return fallback
        return Util.clampAlpha(n)
    }

    // The foundational roles out of the theme's colors.toml. A key wins; the
    // numbered colour is the fallback for a theme that has no key of its own.
    function loadColors(raw) {
        var lines = String(raw || "").split("\n")
        var foundAccent = false
        var foundMuted = false
        var loadedForeground = false
        var loadedBackground = false
        var color0Value = ""
        var color4Value = ""
        var color7Value = ""
        var color8Value = ""
        for (var i = 0; i < lines.length; i++) {
            // Sample input: "accent = \"#89b4fa\"".
            var match = lines[i].match(/^\s*([A-Za-z0-9_-]+)\s*=\s*["']?(#[0-9A-Fa-f]{6})/)
            if (!match) continue
            if (match[1] === "foreground") { root.foreground = match[2]; loadedForeground = true }
            else if (match[1] === "background") { root.background = match[2]; loadedBackground = true }
            else if (match[1] === "accent") { root.accent = match[2]; foundAccent = true }
            else if (match[1] === "muted") { root.muted = match[2]; foundMuted = true }
            else if (match[1] === "color0") color0Value = match[2]
            else if (match[1] === "color4") color4Value = match[2]
            else if (match[1] === "color7") color7Value = match[2]
            else if (match[1] === "color8") color8Value = match[2]
            else if (match[1] === "red" || match[1] === "color1") root.urgent = match[2]
        }
        if (!loadedBackground && color0Value.length > 0) root.background = color0Value
        if (!loadedForeground && color7Value.length > 0) root.foreground = color7Value
        if (!foundAccent && color4Value.length > 0) root.accent = color4Value
        if (!foundMuted) root.muted = color8Value.length > 0 ? color8Value : root.foreground
    }

    // The theme's shell.toml, kept apart from the user's so either can be
    // reloaded without re-reading the other.
    property var themeShellValues: ({})
    property var userShellValues: ({})

    // One walker for shell.toml. A quoted string, a bare number, a bare width
    // list or a bare role name; inline comments are dropped. The values stay
    // strings, and a reader coerces when it takes one.
    function parseShell(raw) {
        var parsed = {}
        var text = String(raw || "")
        if (!text) return parsed
        var lines = text.split("\n")
        var section = ""
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].replace(/^\s+|\s+$/g, "")
            if (!line || line.charAt(0) === "#") continue
            var sectionMatch = line.match(/^\[([A-Za-z0-9_-]+)\]\s*(#.*)?$/)
            if (sectionMatch) { section = sectionMatch[1]; continue }
            var stringKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*["']([^"']+)["']\s*(#.*)?$/)
            var numKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(-?\d+(?:\.\d+)?)\s*(#.*)?$/)
            var widthKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(-?\d+(?:\.\d+)?(?:\s+-?\d+(?:\.\d+)?){1,3})\s*(#.*)?$/)
            var bareKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*([A-Za-z][A-Za-z0-9_-]*)\s*(#.*)?$/)
            var kv = stringKv || numKv || widthKv || bareKv
            if (!kv || !section) continue
            parsed[section + "." + kv[1]] = kv[2]
        }
        return parsed
    }

    // Re-derive the merged values from the theme base and the user's override,
    // and push them into Style. The user's keys win, so a machine-level
    // shell.toml survives a theme switch.
    function mergeShell() {
        var merged = {}
        for (var tk in root.themeShellValues) merged[tk] = root.themeShellValues[tk]
        for (var uk in root.userShellValues) merged[uk] = root.userShellValues[uk]
        root.shellValues = merged
        Style.applyShellValues(merged)
    }

    function loadShell(raw) {
        root.themeShellValues = root.parseShell(raw)
        root.mergeShell()
    }

    function loadUserShell(raw) {
        root.userShellValues = root.parseShell(raw)
        root.mergeShell()
    }

    property FileView colorsFile: FileView {
        id: colorsFile
        path: root.themeDir + "/colors.toml"
        watchChanges: false
        printErrors: false
        onLoaded: root.loadColors(text())
    }

    property FileView shellFile: FileView {
        id: shellFile
        path: root.themeDir + "/shell.toml"
        watchChanges: false
        printErrors: false
        onLoaded: root.loadShell(text())
        onLoadFailed: root.loadShell("")
    }

    // The machine-level override, layered over whatever theme is active.
    property FileView userShellFile: FileView {
        id: userShellFile
        path: root.userShellPath
        watchChanges: true
        printErrors: false
        onLoaded: root.loadUserShell(text())
        onFileChanged: reload()
        onLoadFailed: root.loadUserShell("")
    }
}
