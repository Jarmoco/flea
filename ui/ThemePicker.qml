import QtQuick
import Quickshell
import qs.Commons as Commons

// The themes this box has, and the one it draws with.
//
// The window cannot list a directory, so the launcher reads the theme
// directories once and names the whole list in the environment, beside the one
// it chose. The panel reads that list when it opens; choosing a theme sets the
// palette reader's theme directory, so the change is drawn at once, and the
// stored choice carries it across the next launch.
//
// The panel owns one of these, and it is built with the panel, so a window
// that never opens Settings carries none of it.
Item {
    id: root

    // The themes this box has: each a name, the directory it lives in and the
    // file that names it.
    readonly property var themes: {
        var raw = Quickshell.env("FLEA_THEMES")
        if (!raw)
            return []
        var parsed
        try {
            parsed = JSON.parse(raw)
        } catch (e) {
            return []
        }
        return parsed || []
    }

    // The entry a named theme is, or null for a name this list does not carry.
    function entryFor(name) {
        for (var i = 0; i < root.themes.length; i++) {
            if (root.themes[i].name === name)
                return root.themes[i]
        }
        return null
    }

    // The theme the operator chose, drawn at once.
    //
    // A name the list does not carry leaves the palette alone: the launcher
    // resolved the stored name before this window opened, and a theme that has
    // been removed since is not this picker's to replace.
    function apply(name) {
        var entry = root.entryFor(name)
        if (!entry)
            return
        Commons.Color.themeDir = entry.dir
        Commons.Color.themeNamePath = entry.file
    }
}
