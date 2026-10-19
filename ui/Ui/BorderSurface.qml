import QtQuick

// A card surface: a rounded, filled rectangle with an optional frame, and the
// insets its content sits inside. The frame is drawn inside the surface, so
// the content insets count it.
Item {
    id: root

    property color color: "transparent"
    property var borderSpec: null
    property int radius: 0
    property int padding: 0

    readonly property int borderWidth: root.borderSpec && root.borderSpec.width !== undefined ? Math.max(0, Math.round(root.borderSpec.width)) : 0
    readonly property color borderColor: root.borderSpec && root.borderSpec.color !== undefined ? root.borderSpec.color : "transparent"

    readonly property int contentTopInset: root.padding + root.borderWidth
    readonly property int contentBottomInset: root.padding + root.borderWidth
    readonly property int contentLeftInset: root.padding + root.borderWidth
    readonly property int contentRightInset: root.padding + root.borderWidth

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.color
        border.width: root.borderWidth
        border.color: root.borderColor
    }
}
