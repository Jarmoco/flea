//@ pragma ShellId flea-themepicker-test

import QtQuick
import Quickshell
import qs.Commons as Commons
import "flea" as Flea
import "flea/js/Settings.js" as Settings

// The Theme page's own plumbing: the picker reads the themes the launcher
// named in the environment, and choosing one sets the palette reader's theme
// directory at once.
//
// The panel and the state file are the two ends of it, and both are proved
// elsewhere: the rows are ui/js/SettingsTheme.js's, and the stored name is
// src/themedir.rs's. What is proved here is the middle, which is the only
// part that can put a theme on the screen.

ShellRoot {
    id: root

    property var failures: []

    function fail(message) {
        root.failures.push(message)
    }

    function check(label, actual, expected) {
        if (actual !== expected)
            root.fail(label + ": got " + JSON.stringify(actual) + ", expected " + JSON.stringify(expected))
    }

    Flea.ThemePicker {
        id: picker
    }

    Component.onCompleted: Qt.callLater(function () { root.answer() })

    // The list the launcher named, and everything the picker does with it.
    function answer() {
        root.check("the picker reads the list the launcher named", picker.themes.length, 2)
        root.check("Flea's own palette carries no directory", picker.entryFor("").dir, "")
        root.check("and names no file either", picker.entryFor("").file, "")
        root.check("a named theme carries its directory",
                   picker.entryFor("rose-pine").dir, "/probe/themes/rose-pine")
        root.check("and the file that names it",
                   picker.entryFor("rose-pine").file, "/probe/themes/rose-pine/name")
        root.check("a name the list does not carry has no entry", picker.entryFor("gone"), null)

        // The theme the operator chose, drawn at once.
        picker.apply("rose-pine")
        root.check("choosing a theme sets the directory the palette reads",
                   Commons.Color.themeDir, "/probe/themes/rose-pine")
        root.check("and the file that names it", Commons.Color.themeNamePath, "/probe/themes/rose-pine/name")

        // A theme that has gone leaves the palette alone: the launcher's own
        // resolution stands for one this picker cannot see.
        picker.apply("gone")
        root.check("a name the list does not carry leaves the theme alone",
                   Commons.Color.themeDir, "/probe/themes/rose-pine")

        // Flea's own palette is no theme file at all.
        picker.apply("")
        root.check("Flea's own palette reads no theme file", Commons.Color.themeDir, "")
        root.check("and names no file either", Commons.Color.themeNamePath, "")

        // The Theme section reads the list the picker holds.
        var rows = Settings.rows("theme", {
            themes: picker.themes,
            themeName: "rose-pine",
            themeColors: {},
            themeDir: Commons.Color.themeDir
        })
        var choice = rows.filter(function (row) { return row.id === "theme.name" })[0] || {}
        root.check("the Theme row offers the themes the picker holds",
                   choice.values.join("|"), "|rose-pine")
        root.check("and shows the chosen one", choice.value, "rose-pine")

        if (root.failures.length > 0) {
            for (var f = 0; f < root.failures.length; f++)
                console.log("THEMEPICKER FAIL " + root.failures[f])
        } else {
            console.log("THEMEPICKER PASS listed=2 applied=rose-pine own=none refused=gone")
        }
        Quickshell.execDetached(["kill", String(Quickshell.processId)])
    }
}
