import QtQuick
import Quickshell
import Quickshell.Io

// What the compositor tells the shell about itself.
//
// The corner rounding follows the compositor when there is one: the shell reads
// the compositor's own rounding setting, and a box with no compositor keeps the
// default the caller holds.
QtObject {
    id: root

    // The token model this answers, and the value it answers with.
    property var style: null
    property int cornerRadius: 8

    // The compositor's own environment, which is what says there is one.
    readonly property bool hyprRunning: Quickshell.env("HYPRLAND_INSTANCE_SIGNATURE") !== null

    function refresh() {
        if (root.hyprRunning) hyprctlProc.running = true
    }

    // The compositor is asked once the change has settled, not on every event.
    function scheduleRefresh() {
        refreshTimer.restart()
    }

    function applyRoundingJson(raw) {
        try {
            var json = JSON.parse(raw || "{}")
            var n = Number(json.int)
            if (isFinite(n) && n >= 0) root.cornerRadius = n
        } catch (e) {
            // The compositor did not answer, so the current value stands.
        }
    }

    property Process hyprctlProc: Process {
        id: hyprctlProc
        command: ["hyprctl", "-j", "getoption", "decoration:rounding"]
        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.applyRoundingJson(text)
        }
    }

    property Timer refreshTimer: Timer {
        id: refreshTimer
        interval: 200
        repeat: false
        onTriggered: root.refresh()
    }

    Component.onCompleted: root.refresh()
}
