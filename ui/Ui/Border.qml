pragma Singleton
import QtQuick

// A border specification: which colour, how thick. A surface takes one and
// draws it.
QtObject {
    // A frame of one colour all the way round.
    function flat(color, width) {
        return { "color": color, "width": width }
    }

    // No frame at all, which is not the same as a transparent one of some width.
    function none() {
        return { "color": "transparent", "width": 0 }
    }
}
