//@ pragma ShellId flea-theme-editor-test

import QtQuick
import Quickshell
import "flea" as Flea
import "qml-probe.js" as QmlProbe

// The Theme page's own colour row: Enter opens the row's editor, a hex typed into
// it lands in the state, and the palette is laid over again so the window redraws in
// it at once.
//
// The rows model is proved in tests/js/settingstheme.js, the palette in
// tests/theme-colors.qml, and this is the middle: the panel, the row and the field
// that carry one from the other.

ShellRoot {
    id: shell

    property int checks: 0
    property var failures: []
    property int ticks: 0
    property int step: -1
    property bool done: false

    function log(line) { QmlProbe.log("THEMEEDITOR", line) }
    function quit() { Quickshell.execDetached(["kill", String(Quickshell.processId)]) }
    function check(name, actual, expected) { QmlProbe.checkQuiet(shell, name, actual, expected) }
    function report() {
        QmlProbe.reportCard(shell, ticker)
        shell.quit()
    }

    FloatingWindow {
        implicitWidth: 900
        implicitHeight: 600
        color: "#303030"
        Item {
            width: parent.width
            height: parent.height
            Flea.SettingsPanel { id: panel; anchors.fill: parent }
        }
    }

    // The first item of a kind under an item, the way the menu's own reader finds one.
    function findNamed(item, name) {
        var stack = [item]
        while (stack.length > 0) {
            var o = stack.pop()
            if (o !== item && o.objectName === name) return o
            var kids = o && o.children !== undefined ? o.children : []
            for (var i = 0; i < kids.length; i++) stack.push(kids[i])
        }
        return null
    }

    function tick() {
        shell.ticks += 1
        if (shell.ticks > 400) {
            shell.check("the harness finished", false, true)
            shell.report()
            return
        }
        if (shell.step < 0) {
            shell.step = 0
            panel.open(null)
            panel.showSection("theme")
            return
        }
        if (shell.step === 0) {
            // The section's rows are built when the section is shown, so the row is waited for.
            var row = panel.rowItemForId("theme.colors.background")
            if (!row) return
            shell.step = 1
            var index = -1
            for (var i = 0; i < panel.rows.length; i++)
                if (panel.rows[i].id === "theme.colors.background") index = i
            shell.check("the background row is in the section", index >= 0, true)
            panel.activate(index)
            return
        }
        if (shell.step === 1) {
            var row = panel.rowItemForId("theme.colors.background")
            var field = row ? findNamed(row, "colorField") : null
            if (!field) return
            shell.step = 2
            shell.check("the editor opened on the row", row.editing, true)
            shell.check("and it is the field that took it", field.visible, true)
            shell.check("and the keyboard went with it", field.activeFocus, true)
            // A hex typed without its hash is still a colour, and it lands in lower case.
            field.text = "4C8BF5"
            field.accepted()
            return
        }
        if (shell.step === 2) {
            shell.step = 3
            shell.check("the colour landed in the state",
                        String(Flea.ViewState.themeColors.background), "#4c8bf5")
            shell.check("and the palette is laid over it",
                        String(Flea.Theme.color.background), "#4c8bf5")
            shell.check("while the role beside it keeps the theme's own",
                        String(Flea.Theme.color.foreground), "#a9b1d6")
            return
        }
        if (shell.step === 3) {
            shell.step = 4
            // An empty field is the theme's own colour for that role again.
            var row = panel.rowItemForId("theme.colors.background")
            row.beginEdit()
            var field = findNamed(row, "colorField")
            field.text = ""
            field.accepted()
            return
        }
        if (shell.step === 4) {
            shell.step = 5
            shell.check("an empty field forgets the colour",
                        String(Flea.ViewState.themeColors.background), "undefined")
            shell.check("and the theme's own ground is back",
                        String(Flea.Theme.color.background), "#1a1b26")
            shell.done = true
            shell.report()
        }
    }

    Timer {
        id: ticker
        interval: 40
        repeat: true
        running: true
        onTriggered: shell.tick()
    }

    Component.onCompleted: {
        // The theme the window draws with, so the palette's own ground is the one the
        // checks read.
        Flea.Theme.applyColors('background = "#1a1b26"\nforeground = "#a9b1d6"\naccent = "#7aa2f7"\n' +
                               'muted = "#414868"\ndark_background = "#24283b"\nred = "#f7768e"\n')
    }
}
