import QtQuick

// The colour row's own editor: the row's value becomes a field, and the colour is
// typed into it.
//
// The row owns the swatch and the writer; this owns the field, the hex and whether
// it is one yet. An empty field is the theme's own colour for that role, and a hex
// that is not a colour is refused and the field stays open for it to be typed again.
TextInput {
    id: root

    // The name a test finds the field by, the way the menu's wash names itself.
    objectName: "colorField"

    // Whether the editor is open, which is the row's own state and nothing else.
    property bool opened: false
    // The colour the operator typed, and whether it is a colour yet. The hash is
    // optional while it is typed, and is added when the colour lands.
    property string draft: ""
    readonly property bool valid: root.draft === "" || /^#?[0-9a-fA-F]{6}$/.test(root.draft)

    // The colour the operator set, and the editor closing without one.
    signal committed(string hex)
    signal cancelled()

    // The editor opens on the colour the row holds, so a colour the operator set is
    // one keystroke from being changed again, and a row that follows the theme opens
    // empty.
    function open(hex) {
        root.draft = hex || ""
        root.text = root.draft
        root.opened = true
        root.forceActiveFocus()
        root.selectAll()
    }

    function close() {
        root.opened = false
        root.draft = ""
    }

    onTextChanged: root.draft = text
    onAccepted: {
        var typed = String(root.text).trim()
        if (typed.length > 0 && typed.charAt(0) !== "#")
            typed = "#" + typed
        if (typed.length > 0 && !/^#[0-9a-fA-F]{6}$/.test(typed)) {
            root.draft = typed
            return
        }
        root.committed(typed.toLowerCase())
    }
    Keys.onEscapePressed: function (event) { root.cancelled(); event.accepted = true }
}
