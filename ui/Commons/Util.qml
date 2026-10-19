pragma Singleton
import QtQuick

// The shared helpers. Pure functions, no state: anything stateful belongs on
// Color, Style, or a service.
QtObject {
    id: root

    function clamp(value, min, max) {
        var n = Number(value)
        if (!isFinite(n)) return min
        return Math.max(min, Math.min(max, n))
    }

    // A number between 0 and 1. A value that is not a number is the floor, not
    // the ceiling: a missing alpha is no alpha.
    function clampAlpha(value) {
        return clamp(value, 0, 1)
    }

    // Compose a base colour with an opacity. The alpha is replaced, not blended,
    // so the result keeps the base's own hue at the given opacity.
    function alpha(c, opacity) {
        var a = clampAlpha(opacity)
        if (!c) return Qt.rgba(0, 0, 0, a)
        if (typeof c === "string") c = Qt.color(c)
        return Qt.rgba(c.r, c.g, c.b, a)
    }

    // A file:// URL with each path segment percent-encoded, so a space or a #
    // in a name is read as bytes rather than as a fragment.
    function fileUrl(path) {
        if (!path) return ""
        return "file://" + String(path).split("/").map(encodeURIComponent).join("/")
    }
}
