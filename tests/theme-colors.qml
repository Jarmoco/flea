//@ pragma ShellId flea-theme-colors-test

import QtQuick
import Quickshell
import qs.Commons
import qs.Commons as Commons
import "flea" as Flea

// The Theme page's own colours: the operator's own over the theme's, one hex for
// every role the app draws with.
//
// The theme's own plumbing is proved in tests/theme-picker.qml, and the rows the
// panel draws are proved in tests/js/settingstheme.js. What is proved here is
// the middle again: the colours the operator set reach the palette, and the
// roles they did not set keep the theme's own.

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

    Component.onCompleted: Qt.callLater(function () { root.answer() })

    // The theme the operator picked, and the colours it carries.
    readonly property string body: 'background = "#1a1b26"\nforeground = "#a9b1d6"\naccent = "#7aa2f7"\n' +
                                   'muted = "#414868"\ndark_background = "#24283b"\nred = "#f7768e"\n'

    function answer() {
        var theme = Flea.Theme
        var color = theme.color

        // The state file's own colours, which the window reads before its first
        // paint: the whole path from the file to the palette.
        root.check("the ground the operator set", String(color.background), "#0a0b0c")
        root.check("the accent they set beside it", String(color.accent), "#3a7bd5")

        // The theme's own palette, with nothing of the operator's over it.
        Commons.Color.overrides = ({})
        theme.applyColors(root.body)
        var ownFrame = String(color.mutedFrame)
        var ownMuted = String(color.muted)
        root.check("the theme's own background", String(color.background), "#1a1b26")
        root.check("and its own foreground", String(color.foreground), "#a9b1d6")

        // The palette over the operator's own colours.
        Commons.Color.overrides = ({
            background: "#101010",
            surface: "#181818",
            accent: "#ff8800",
            muted: "#888888",
            mutedFrame: "#666666",
            error: "#ff0000",
            symlink: "#00ff00",
            executable: "#ffff00"
        })
        root.check("the operator's own background", String(color.background), "#101010")
        root.check("the operator's own surface", String(color.surface), "#181818")
        root.check("the operator's own accent", String(color.accent), "#ff8800")
        root.check("the operator's own muted text", String(color.muted), "#888888")
        root.check("the operator's own muted frame", String(color.mutedFrame), "#666666")
        root.check("the operator's own error", String(color.error), "#ff0000")
        root.check("the operator's own symlink", String(color.symlink), "#00ff00")
        root.check("the operator's own executable", String(color.executable), "#ffff00")
        root.check("a role they did not set keeps the theme's own", String(color.foreground), "#a9b1d6")

        // The roles Flea derives follow the operator's own, so the ground and the
        // ink never disagree.
        Commons.Color.overrides = ({ background: "#000000" })
        root.check("the ground is theirs", String(color.background), "#000000")
        root.check("and the frame beside it is lifted against that ground",
                   String(color.mutedFrame) !== ownFrame, true)
        root.check("while the ink they did not set is still the theme's own",
                   String(color.foreground), "#a9b1d6")

        // Every colour back to the theme's own.
        Commons.Color.overrides = ({})
        theme.applyColors(root.body)
        root.check("the theme's own background comes back", String(color.background), "#1a1b26")
        root.check("and its muted frame with it", String(color.mutedFrame), ownFrame)
        root.check("and its muted text", String(color.muted), ownMuted)

        if (root.failures.length > 0) {
            for (var f = 0; f < root.failures.length; f++)
                console.log("THEMECOLORS FAIL " + root.failures[f])
        } else {
            console.log("THEMECOLORS PASS state=2 own=8 kept=1 derived=2 reset=3")
        }
        Quickshell.execDetached(["kill", String(Quickshell.processId)])
    }
}
