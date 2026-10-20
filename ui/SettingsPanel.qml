import QtQuick
import Quickshell
import qs.Commons
import qs.Commons as Commons
import "." as Flea
import "js/Keymap.js" as Keymap
import "js/Settings.js" as Settings
import "js/SettingsTheme.js" as SettingsTheme

// The settings panel the Settings board draws: one floating surface, a fixed rail of sections and a pane that scrolls inside the work-area clamp. A plain overlay and not a QQC Popup, the call ui/ContextMenu.qml already made, because the one Controls import cost 10 ms of warm startup.
Item {
    id: root

    property bool opened: false
    property Item focusHolder: null
    // Reopening keeps the last section selected in this process.
    property string section: "view"
    property int cursor: 0
    property int selectedFavourite: -1
    property int favouriteMoveTarget: -1
    property bool favouriteActionPending: false
    // "rail" or "pane", which side Tab last gave the cursor to.
    property string side: "pane"

    // Border-box 560 wide with a 150 rail and a 408 pane, the board's resolved geometry at base 14;
    // ui/Theme.qml holds the derivation and the seam reports all three.
    readonly property int panelWidth: Theme.settings.panelWidth
    readonly property int railWidth: Theme.settings.railWidth
    // The Menus board's work-area clamp: a floating surface never renders taller than its bounds
    // less this margin, and the pane scrolls inside that while the rail stays put.
    readonly property int clampMargin: 8
    // The compact card keeps View's measured size across sections, clamped to the window.
    readonly property int chromeAndBorder: Theme.chromeHeight + 2 * Theme.spacing.hairline
    readonly property int paneBottomPadding: Math.round(8 * Theme.font.bodySmall / 13)
    readonly property real groundOpacity: 0.5
    // The card's title, for ui/Ipc.qml: a driven click on it proves the card swallows what its controls do not.
    readonly property alias titleItem: title
    // var, not Item: BorderSurface is a qs.Ui type qmllint cannot resolve, and Item would read as incompatible.
    readonly property var cardItem: card

    // Everything ui/js/Settings.js rows() reads, built once here for the keyboard's rows and the pane's.
    readonly property var settingsState: ({
        data: ViewState.state,
        home: Quickshell.env("HOME"),
        favouriteStatuses: Favourites.statuses,
        pins: ShelfPins.records,
        selectedFavourite: root.selectedFavourite,
        about: aboutFacts.facts,
        saveStatus: ViewState.saveStatus,
        textSize: ViewState.textSize,
        hidden: ViewState.menuHidden,
        keyHints: ViewState.keyHints,
        preset: ViewState.keysPreset,
        baseSize: Theme.baseSize,
        monitorScale: Theme.monitorScale,
        cornerRadius: Style.cornerRadius,
        presetKeys: Keymap.PRESET_KEYS,
        // The Theme section's own: the themes this box has and the choice among them.
        themes: root.themePicker.themes,
        themeName: ViewState.themeName,
        // The operator's own colours, one hex per role the app draws with.
        themeColors: ViewState.themeColors,
        // The directory the palette is reading, which is the truth about where
        // the theme in use lives.
        themeDir: Commons.Color.themeDir
    })
    readonly property var rows: Settings.rows(root.section, root.settingsState)

    // The open submenu, for a test that reads it without walking children.
    readonly property var menuRef: choiceMenu

    // What a test reads instead of running OCR over the panel, the same idiom ui/KeymapSheet.qml's
    // rows() uses: one row per line, its kind, its wording and whatever value it currently holds.
    function rowsText() {
        var out = []
        for (var i = 0; i < root.rows.length; i++) {
            var row = root.rows[i]
            out.push(row.kind + "|" + row.label + "|" + (row.value !== undefined ? row.value : ""))
        }
        return out.join("\n")
    }

    // The themes this box has, and the one it draws with. It is built with the panel,
    // so a window that never opens Settings carries none of it.
    property Item themePicker: Flea.ThemePicker { }

    function open(holder) {
        root.focusHolder = holder
        root.cursor = Settings.firstRow(root.rows)
        root.side = "pane"
        pane.contentY = 0
        root.opened = true
        keys.forceActiveFocus()
    }

    function close() {
        if (!root.opened)
            return
        root.opened = false
        if (root.focusHolder)
            root.focusHolder.forceActiveFocus()
    }

    function showSection(id) {
        root.section = id
        root.cursor = Settings.firstRow(root.rows)
        pane.contentY = 0
    }

    // Enter and Space both land here. A choice steps rather than opening a menu of its own, because
    // no control the panel draws has more values than a walk can reach.
    function activate(index) {
        var row = root.rows[index]
        if (!row || !Settings.focusable(row))
            return
        if (row.kind === "favourite") {
            // A pin opens the folder it names, the way the sidebar opens a favourite.
            if (row.pinPath !== undefined) { root.openPin(row.pinPath); return }
            root.selectedFavourite = row.favouriteIndex
            if (root.focusHolder && root.focusHolder.sidebar) root.focusHolder.sidebar.openFavourite(row.favouriteIndex)
            return
        }
        // The only headings the cursor can reach: the one carrying its group's master, and the one carrying Places' own Add.
        if (row.kind === "group") {
            if (row.action === "addFavourite") root.addFavourite()
            else if (row.action === "pinFolder") root.pinFolder()
            // The shelf's master is one stored boolean rather than a set of menu ids.
            else if (row.id === "shelf.enabled") ViewState.changeSetting(row.id, row.state !== "all")
            else ViewState.toggleMenuGroup(row.ids)
            return
        }
        if (row.id === "columns") { root.showSection("columns"); return }
        // A colour row opens its own editor, where a hex is typed, and the panel's keyboard
        // goes with it until the editor closes.
        if (row.kind === "color") { root.beginEdit(index); return }
        // Every colour back to the theme's own, which is the one row that clears the map.
        if (row.id === "theme.colors.reset") { ViewState.clearThemeColors(); return }
        // The folder the panel was opened over, which is the pane behind it: the same folder Places' own "Add this folder" takes, and the only one on screen to mean.
        if (row.id === "startFolder") {
            if (root.focusHolder) ViewState.setStartFolder(root.focusHolder.path)
            return
        }
        if (row.id === "backView") { root.showSection("view"); return }
        if (row.id === "keyboardSheet") {
            var holder = root.focusHolder
            root.close()
            if (holder && holder.keymapSheet) holder.keymapSheet.open(holder)
            return
        }
        // About's two web routes carry their own destination; ui/js/SettingsAbout.js names both.
        if (row.url !== undefined) { Qt.openUrlExternally(row.url); return }
        // Two About actions with a live state: Update Flea's value shows a check and the footer what a launch did, and Make Flea the default runs flea instead of writing ui.json.
        if (row.id === "updateFlea") { UpdateCheck.activate(root.focusHolder); return }
        if (row.id === "makeDefault") { DefaultClaim.toggle(); return }
        if (row.id.indexOf("column:") === 0) { ViewState.toggleColumn(row.id.substring(7)); return }
        if (row.kind === "check" && root.section !== "menus") {
            ViewState.changeSetting(row.id, !row.on)
            return
        }
        // Menu checks share their visibility writer; other sections write their own leaves.
        if (row.id === "keyHints")
            ViewState.toggleKeyHints()
        else if (row.kind === "check")
            ViewState.toggleMenuAction(row.id)
        else if (row.menu === true && row.values !== undefined)
            root.openMenu(index)
        else
            root.stepRowValue(index, 1)
    }

    // The writer every path a choice can take shares, so a click, an Enter and an h/l step all
    // land the same setting, and a theme is drawn at once whichever one the operator used.
    function setChoice(row, value, index) {
        // A colour row writes the operator's own colour for its role, and the menu's own custom
        // entry opens the row's editor rather than storing a colour.
        if (row.colorRole !== undefined) {
            if (value === SettingsTheme.CUSTOM) {
                root.beginEdit(index)
                return
            }
            ViewState.setThemeColor(row.colorRole, value)
            return
        }
        ViewState.changeSetting(row.id, value)
        if (row.id === "theme.name")
            root.themePicker.apply(value)
    }

    // The colour row's own editor, which takes the keyboard while it is open and gives it back
    // when it closes.
    function beginEdit(index) {
        var item = pane.rowItem(index)
        if (item)
            item.beginEdit()
    }

    // A choice row's submenu: all of its options in a card under the row, so a long list reads as
    // a menu rather than a cycle. The row's own geometry places it, and the panel's window clamps it.
    function openMenu(index) {
        var row = root.rows[index]
        if (!row || row.values === undefined)
            return
        var item = pane.rowItem(index)
        var at = pane.mapToItem(root, item ? item.x : 0, item ? item.y + item.height : 0)
        choiceMenu.open(row.labels, row.value, at.x, at.y, item ? item.width : root.width,
                      Math.max(Theme.rowHeight * 2, root.height - at.y - root.clampMargin), row.swatches)
    }

    // h, l and the two chevrons. Every writer is ui/ViewState.qml's own, which is the same state the
    // Ctrl+Shift chords reach, so a keystroke and a control can never hold two different sizes.
    function stepRowValue(index, direction) {
        var row = root.rows[index]
        if (!row || !Settings.focusable(row))
            return
        if (row.id === "textMode") {
            ViewState.toggleTextFollow()
            return
        }
        if (row.id === "textStop") {
            ViewState.stepTextSize(direction)
            return
        }
        // A check or a group's master is toggled by activate(), never walked, so h and l stop here.
        if (row.kind !== "choice")
            return
        if (row.values !== undefined) {
            // A colour row's step walks the colours it can pick, and never the menu's own
            // custom entry, which is the editor and not a colour.
            if (row.colorRole !== undefined) {
                root.setChoice(row, SettingsTheme.stepValue(row.values, row.selected, direction), index)
                return
            }
            var index = row.values.indexOf(row.selected)
            var target = (index + direction + row.values.length) % row.values.length
            root.setChoice(row, row.values[target], index)
            return
        }
        var at = Settings.PRESETS.indexOf(ViewState.keysPreset)
        var next = (at + direction + Settings.PRESETS.length) % Settings.PRESETS.length
        ViewState.setKeysPreset(Settings.PRESETS[next])
    }

    // The folder the panel was opened over, which is the pane behind it: SettingsRest rule 3 puts this on the Favorites heading, where it governs the whole list.
    function addFavourite() {
        if (!root.focusHolder)
            return
        var path = root.focusHolder.path
        root.favouriteActionPending = Favourites.add(path, path.substring(path.lastIndexOf("/") + 1) || path)
    }

    // Rule 4: removal is addressed by the row it acts on, from the row's own mark or from the x key on it.
    // A pin is the shelf's own entry, so its x unpins it rather than touching Places.
    function removeFavourite(index) {
        var row = root.rows[index]
        if (!row || row.kind !== "favourite") return
        if (row.pinPath !== undefined) { ShelfPins.unpin(row.pinPath); return }
        root.favouriteActionPending = Favourites.remove(row.favouriteIndex)
    }

    // Rule 4 again: a pin is reordered in the shelf's own pile, and a favourite in Places.
    function stepFavourite(index, direction) {
        var row = root.rows[index]
        if (!row) return
        if (row.pinPath !== undefined) { ShelfPins.move(row.pinPath, direction); return }
        var to = Math.max(0, Math.min(Favourites.records.length - 1, row.favouriteIndex + direction))
        if (to !== row.favouriteIndex && Favourites.move(row.favouriteIndex, to)) root.favouriteMoveTarget = to
    }

    function moveFavourite(index, to) {
        var row = root.rows[index]
        if (!row) return
        if (row.pinPath !== undefined) { ShelfPins.moveTo(row.pinPath, to); return }
        if (Favourites.move(row.favouriteIndex, to)) root.favouriteMoveTarget = to
    }

    // SettingsRest rule 4: the shelf's own Add, the folder the panel was opened over.
    function pinFolder() { if (root.focusHolder) ShelfPins.pin(root.focusHolder.path) }

    function openPin(path) { if (root.focusHolder && root.focusHolder.sidebar) root.focusHolder.sidebar.opened(path) }

    function pickRowStop(index, stop) {
        var row = root.rows[index]
        if (!row || !Settings.focusable(row))
            return
        // A segment names the value it was clicked on where a ruler tick names a stop outright, so both arrive here addressed by index, the row decides which writer that index belongs to, and a tick lands in the same ViewState writer stepTextSize itself calls: a click, an h and a Ctrl+Shift+Plus cannot leave two different sizes stored.
        if (row.id === "textMode") {
            ViewState.toggleTextFollow()
            return
        }
        if (row.values !== undefined) {
            root.setChoice(row, row.values[stop], index)
            return
        }
        if (row.kind === "choice") {
            ViewState.setKeysPreset(Settings.PRESETS[stop])
            return
        }
        ViewState.setTextSize({ mode: stop })
    }

    function moveCursor(delta) {
        if (root.side === "rail") {
            var at = Settings.SECTIONS.map(function (s) { return s.id }).indexOf(root.section)
            var to = Math.max(0, Math.min(Settings.SECTIONS.length - 1, at + delta))
            root.showSection(Settings.SECTIONS[to].id)
            return
        }
        root.cursor = Settings.stepRow(root.rows, root.cursor, delta)
        // A pin is a favourite-shaped row with no favourite behind it, so it selects nothing.
        if (root.rows[root.cursor] && root.rows[root.cursor].favouriteIndex !== undefined)
            root.selectedFavourite = root.rows[root.cursor].favouriteIndex
        root.showCursor()
    }

    function showCursor() { pane.showCursor(root.cursor, Settings.stepRow(root.rows, root.cursor, 1) === root.cursor) }
    function sectionsText() { return JSON.stringify(Settings.SECTIONS) }
    function rowItemForId(id) {
        for (var i = 0; i < root.rows.length; i++) {
            if (root.rows[i].id === id) return pane.rowItem(i)
        }
        return null
    }
    function railItemFor(id) { return rail.itemFor(id) }
    function paneScroll() { return Math.round(pane.contentHeight) + "|" + Math.round(pane.height) }
    function scrollState() {
        return JSON.stringify({compactHeight: pane.compactHeight,
            pane: {y: pane.contentY, height: pane.height, contentHeight: pane.contentHeight},
            rail: {y: rail.contentY, height: rail.height, contentHeight: rail.contentHeight}})
    }

    anchors.fill: parent
    visible: root.opened || root.opacity > 0
    // Closed means no input, even while the fade-out is still drawn.
    enabled: root.opened
    opacity: 0
    // Above the context menu and the keymap sheet: the panel is the surface that opened last.
    z: 3
    Flea.Reveal { target: root; shown: root.opened; lift: card }

    Rectangle {
        anchors.fill: parent
        color: Theme.color.background
        opacity: root.groundOpacity

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            hoverEnabled: true
            onClicked: root.close()
            onWheel: function (wheel) { wheel.accepted = true }
        }
    }

    Rectangle {
        id: card
        x: Theme.cardOrigin(root.width, width)
        y: Theme.cardOrigin(root.height, height)
        width: Theme.cardSpan(root.panelWidth, root.width - 2 * root.clampMargin)
        // Settings.html gives the rail and pane separate vertical insets.
        height: Theme.cardSpan(root.chromeAndBorder + Math.max(rail.implicitHeight + 2 * Theme.settings.railPaddingY,
                                                                pane.compactHeight + root.paneBottomPadding),
                               root.height - 2 * root.clampMargin)
        color: Theme.color.surface
        border.width: Theme.spacing.hairline
        border.color: Theme.color.mutedFrame
        // Mirrors hyprland decoration:rounding, same as ui/KeymapSheet.qml; 0 on a stock box stays square.
        radius: Style.cornerRadius
        clip: true

        // The control handlers above take passive grabs, so without this sink a press on a row fell
        // through the card to the ground below, which closed the panel on release.
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: {}
            onWheel: function (wheel) { wheel.accepted = true }
        }

        // The board's border-box panel: the card's own border is the two outer hairlines, so the
        // 558 they leave is what the chrome, the rail and the pane are laid out inside.
        Item {
            anchors.fill: parent
            anchors.margins: Theme.spacing.hairline

            Item {
                id: chrome
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                height: Theme.chromeHeight

                Flea.Glyph {
                    id: titleMark
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacing.rowPaddingX
                    anchors.verticalCenter: parent.verticalCenter
                    width: Theme.chromeMarkSize
                    height: width
                    name: "sliders"
                    color: Theme.color.accent
                }

                Text {
                    id: title
                    anchors.left: titleMark.right
                    anchors.leftMargin: Theme.spacing.gap
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Settings"
                    color: Theme.color.foreground
                    font.family: Theme.font.family
                    font.pixelSize: Theme.font.caption
                    font.bold: true
                    textFormat: Text.PlainText
                }

                // The board's own header mark, the one surface control a pointer has for closing the panel.
                Item {
                    id: closeMark
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacing.rowPaddingX
                    anchors.verticalCenter: parent.verticalCenter
                    width: Theme.hitMin
                    height: Theme.hitMin

                    Accessible.role: Accessible.Button
                    Accessible.name: "Close settings"
                    Accessible.onPressAction: root.close()

                    Flea.Glyph {
                        anchors.centerIn: parent
                        width: Theme.chromeMarkSize
                        height: Theme.chromeMarkSize
                        name: "x"
                        color: Theme.color.foreground
                    }

                    HoverHandler {
                        cursorShape: Qt.PointingHandCursor
                    }

                    TapHandler {
                        acceptedButtons: Qt.LeftButton
                        gesturePolicy: TapHandler.ReleaseWithinBounds
                        onTapped: root.close()
                    }
                }

                Text {
                    anchors.right: closeMark.left
                    anchors.rightMargin: Theme.spacing.gap
                    anchors.verticalCenter: parent.verticalCenter
                    text: "esc"
                    color: Theme.color.muted
                    font.family: Theme.font.family
                    font.pixelSize: Theme.font.caption
                    textFormat: Text.PlainText
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: Theme.spacing.hairline
                    color: Theme.color.mutedFrame
                    opacity: 0.4
                }
            }

            Flea.SettingsRail {
                id: rail
                anchors.left: parent.left
                anchors.top: chrome.bottom
                anchors.topMargin: Theme.settings.railPaddingY
                anchors.bottom: parent.bottom
                anchors.bottomMargin: Theme.settings.railPaddingY
                width: root.railWidth
                section: root.section === "columns" ? "view" : root.section
                focused: root.side === "rail"
                onChosen: function (id) {
                    root.side = "rail"
                    root.showSection(id)
                }
            }

            // Inside the rail's own 150, not beside it: the board's rail is a border-box whose right
            // edge is this line, which is what leaves the pane the 408 the anatomy note names.
            Rectangle {
                anchors.right: rail.right
                anchors.top: chrome.bottom
                anchors.bottom: parent.bottom
                width: Theme.spacing.hairline
                color: Theme.color.mutedFrame
                opacity: 0.4
            }

            Flea.SettingsPane {
                id: pane
                anchors.left: rail.right
                anchors.right: parent.right
                anchors.top: chrome.bottom
                anchors.bottom: parent.bottom
                anchors.bottomMargin: root.paneBottomPadding
                section: root.section
                values: root.settingsState
                cursor: root.cursor
                side: root.side
                onPointerMoved: function (index) {
                    root.side = "pane"; root.cursor = index
                    if (root.rows[index].favouriteIndex !== undefined) root.selectedFavourite = root.rows[index].favouriteIndex
                }
                onFavouriteRemoved: function (index) { root.removeFavourite(index) }
                onFavouriteMoved: function (index, to) { root.moveFavourite(index, to) }
                onActivated: function (index) { root.side = "pane"; root.cursor = index; root.activate(index) }
                onStepped: function (index, direction) { root.side = "pane"; root.cursor = index; root.stepRowValue(index, direction) }
                onStopPicked: function (index, stop) { root.side = "pane"; root.cursor = index; root.pickRowStop(index, stop) }
                // A colour the operator typed lands in the state, and the palette is laid over
                // again, so the window redraws in it at once.
                onColorCommitted: function (index, role, hex) { ViewState.setThemeColor(role, hex) }
                // The editor gave the keyboard back, so the panel takes it.
                onColorEditing: function (index, editing) { if (!editing) keys.forceActiveFocus() }
            }
        }
    }

    // A choice row's submenu, built only while one is open. It sits above the card, so a press on
    // the panel outside it closes the menu rather than the panel.
    Flea.SettingsMenu {
        id: choiceMenu
        onChosen: function (index) {
            var row = root.rows[root.cursor]
            if (row && row.values !== undefined)
                root.setChoice(row, row.values[index], root.cursor)
        }
        onClosed: root.showCursor()
    }

    Connections {
        target: Favourites
        function onExternalChanged(previousCount) {
            var wasFavourite = root.cursor > 0 && root.cursor <= previousCount + 1
            root.selectedFavourite = -1
            root.favouriteMoveTarget = -1
            root.favouriteActionPending = false
            if (root.opened && root.section === "places") {
                if (wasFavourite) {
                    root.cursor = Settings.firstRow(root.rows)
                } else if (root.cursor > previousCount + 1) {
                    root.cursor += Favourites.records.length - previousCount
                }
                root.showCursor()
            }
        }
        function onWrote() {
            if (root.favouriteActionPending) {
                root.favouriteActionPending = false
                root.selectedFavourite = Math.min(root.selectedFavourite, Favourites.records.length - 1)
                if (root.opened && root.section === "places") {
                    // Adding or removing a row moves every row under it, so the cursor takes the heading that carries Add rather than whatever slid into its index.
                    root.cursor = Settings.firstRow(root.rows)
                    root.showCursor()
                }
            }
            if (root.favouriteMoveTarget < 0) return
            root.selectedFavourite = root.favouriteMoveTarget
            root.cursor = root.favouriteMoveTarget + 1
            root.favouriteMoveTarget = -1
            root.showCursor()
        }
        function onFailed(message) { root.favouriteMoveTarget = -1; root.favouriteActionPending = false }
    }

    Flea.AboutFacts { id: aboutFacts; active: root.opened && root.section === "about" }

    Item {
        id: keys
        anchors.fill: parent
        focus: true

        Keys.onPressed: function (event) {
            event.accepted = true
            // An open submenu takes the keys the panel would otherwise answer, so its own rows move.
            if (choiceMenu.opened) {
                if (event.key === Qt.Key_Escape) {
                    choiceMenu.close()
                    return
                }
                if (event.key === Qt.Key_Down || event.text === "j") {
                    choiceMenu.move(1)
                    return
                }
                if (event.key === Qt.Key_Up || event.text === "k") {
                    choiceMenu.move(-1)
                    return
                }
                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                    choiceMenu.take()
                    return
                }
                return
            }
            if (event.key === Qt.Key_Escape) {
                root.close()
                return
            }
            if (event.key === Qt.Key_Tab || event.key === Qt.Key_Backtab) {
                root.side = root.side === "rail" ? "pane" : "rail"
                return
            }
            var row = root.rows[root.cursor]
            if (root.side === "pane" && row && row.kind === "favourite" && (event.modifiers & Qt.ShiftModifier)
                    && (event.key === Qt.Key_J || event.key === Qt.Key_K)) {
                root.stepFavourite(root.cursor, event.key === Qt.Key_J ? 1 : -1)
                return
            }
            // SettingsRest rule 4: the row's own mark is x, so x on the cursor row is the same action.
            if (root.side === "pane" && row && row.kind === "favourite"
                    && (event.text === "x" || event.key === Qt.Key_Delete)) {
                root.removeFavourite(root.cursor)
                return
            }
            if (event.key === Qt.Key_Down || event.text === "j") {
                root.moveCursor(1)
                return
            }
            if (event.key === Qt.Key_Up || event.text === "k") {
                root.moveCursor(-1)
                return
            }
            // h and l step a control, which only the pane has; from the rail they do nothing rather
            // than reaching across and changing a value the cursor is not on.
            if (event.key === Qt.Key_Right || event.text === "l") {
                if (root.side === "pane")
                    root.stepRowValue(root.cursor, 1)
                return
            }
            if (event.key === Qt.Key_Left || event.text === "h") {
                if (root.side === "pane")
                    root.stepRowValue(root.cursor, -1)
                return
            }
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                if (root.side === "rail")
                    root.side = "pane"
                else if (event.key !== Qt.Key_Space || (root.rows[root.cursor] &&
                         (root.rows[root.cursor].kind === "check" || root.rows[root.cursor].kind === "group")))
                    root.activate(root.cursor)
            }
            // Every other key stops here: an open panel that let one through would move the cursor
            // in the listing behind it, which is the hidden-view keyboard defect AGENTS.md records.
        }
    }
}
