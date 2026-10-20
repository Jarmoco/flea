import QtQuick
import Quickshell
import qs.Commons
import "." as Flea

// The Settings panel's submenu: a choice row's options in a card under the row, so a long list
// reads as a menu rather than a cycle. The panel owns the writer and the geometry; this only
// reports the choice, so a click, an Enter and an h/l step all reach the same setting.
Item {
    id: root

    property bool opened: false
    // The row's options, in order: the labels it draws and the values it stores are parallel arrays.
    property var options: []
    // An optional colour per option, drawn as a swatch in the row's mark slot; empty for a plain menu.
    property var swatches: []
    property string value: ""
    property int cursor: 0
    // The card's own geometry, set by the panel: the row's mapped position and the card width.
    property real cardX: 0
    property real cardY: 0
    property real cardWidth: 0
    // The tallest the card may be before it scrolls, set by the panel from the window.
    property real room: 600

    signal chosen(int index)
    signal closed()

    // The option a keyboard or a click picks.
    function open(options, value, x, y, width, room, swatches) {
        root.options = options || []
        root.swatches = swatches || []
        root.value = value || ""
        root.cursor = Math.max(0, root.options.indexOf(root.value))
        root.cardX = x
        root.cardY = y
        root.cardWidth = width
        root.room = room
        root.opened = true
    }

    function close() {
        if (!root.opened)
            return
        root.opened = false
        root.closed()
    }

    function move(delta) {
        if (root.options.length === 0)
            return
        root.cursor = (root.cursor + delta + root.options.length) % root.options.length
        if (frame.item)
            frame.item.reveal(root.cursor)
    }

    function take() {
        if (!root.opened || root.cursor < 0 || root.cursor >= root.options.length)
            return
        root.opened = false
        root.chosen(root.cursor)
    }

    visible: root.opened
    // It fills the panel, so its ground covers the card beneath and a press outside the card closes.
    anchors.fill: parent
    z: 6

    // The ground takes every pointer event the rows leave, so a press outside the card closes.
    MouseArea {
        id: ground
        anchors.fill: root
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        hoverEnabled: true
        onClicked: root.close()
        onWheel: function (wheel) { wheel.accepted = true }
    }

    // The card lives behind a Loader so a panel that never opens a menu builds none of it.
    Loader {
        id: frame
        active: root.opened
        x: root.cardX
        y: root.cardY
        width: root.cardWidth
        sourceComponent: card
    }

    Component {
        id: card
        Rectangle {
            id: box
            color: Theme.color.surface
            border.width: Theme.spacing.hairline
            border.color: Theme.color.mutedFrame
            radius: Style.cornerRadius
            clip: true
            height: Math.max(0, Math.min(rows.implicitHeight + 2 * Theme.spacing.rowPaddingY, root.room))

            function reveal(index) { scroll.reveal(rowItems.itemAt(index)) }
            readonly property int liveCount: rowItems.count
            function liveAt(index) { return rowItems.itemAt(index) }

            // The sink swallows a press the rows do not take, so it never reaches the panel's ground.
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                onClicked: {}
                onWheel: function (wheel) { wheel.accepted = true }
            }

            Flea.CardScroll {
                id: scroll
                anchors.fill: parent
                anchors.topMargin: Theme.spacing.rowPaddingY
                anchors.bottomMargin: Theme.spacing.rowPaddingY
                highlightSteps: true
                stepBy: function (delta) { root.move(delta) }

                Column {
                    id: rows
                    width: parent.width

                    Repeater {
                        id: rowItems
                        model: root.options
                        delegate: Flea.MenuRow {
                            id: row
                            required property var modelData
                            required property int index
                            width: rows.width
                            entry: ({ glyph: "", label: root.options[index], hint: "" })
                            swatch: root.swatches[index] || ""
                            current: root.cursor === index
                            onPointerMoved: { root.cursor = index }
                            onActivated: root.chosen(index)
                        }
                    }
                }
            }

            Flea.MenuEdgeFade {
                anchors.top: parent.top
                visible: scroll.contentY > 0
            }

            Flea.MenuEdgeFade {
                anchors.bottom: parent.bottom
                visible: scroll.contentY + scroll.height < scroll.contentHeight
                rotation: 180
            }

            Component.onCompleted: reveal(root.cursor)
        }
    }
}
