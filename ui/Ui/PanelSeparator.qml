import QtQuick
import qs.Commons

// A rule between two sections of a board: one hairline across the width.
Item {
    id: root

    implicitHeight: 1

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        height: 1
        color: Style.normalBorderColor
    }
}
