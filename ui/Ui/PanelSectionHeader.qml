import QtQuick
import qs.Commons
import qs.Commons as Commons

// A section header: the label a board names a part with.
//
// The text is drawn as given, so a caller that wants capitals writes them. The
// label sits on the caller's own fill, so the file manager's column band draws
// one across the whole strip and this draws only the words.
Item {
    id: root

    property string text: ""
    property int fontSize: Style.font.caption
    property int elide: Text.ElideNone
    property int horizontalAlignment: Text.AlignLeft
    property color color: Commons.Color.foreground
    // The label's own font, the one a row's cells use, so a header and the cell
    // under it read as one voice. It is the label's font rather than one built
    // here, because the engine does not make the font type creatable.
    readonly property font font: label.font

    // The label sits a little below the top of its band, so a section header
    // reads as a caption rather than as a heading.
    readonly property int topPadding: Math.ceil(root.fontSize * 0.15)

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight + root.topPadding

    Text {
        id: label
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: root.topPadding
        text: root.text
        elide: root.elide
        horizontalAlignment: root.horizontalAlignment
        verticalAlignment: Text.AlignTop
        color: root.color
        font.family: Style.font.family
        font.pixelSize: root.fontSize
    }
}
