pragma Singleton
import QtQuick
import Quickshell.Io
import qs.Commons as Commons
import "StyleControls.js" as Controls

// Flea's own token model.
//
// A theme's shell.toml pins a token; everything else takes the default below.
// The names, the defaults and the shell.toml semantics are the ones the shell
// uses, so a theme that pins a token lands on the same number here as it does
// there, and a box with no theme still draws.
//
//   normal       - idle control chrome
//   hover-cursor - mouse hover, or a panel's keyboard cursor
//   selected     - a persistent chosen state
//   focus        - the actual Qt active focus, defaulting to hover-cursor
QtObject {
    id: root

    // ---------------------------------------------------------- state tokens
    property var styleOverrides: ({})

    function styleRawNum(key) {
        var v = root.styleOverrides[key]
        var n = Number(v)
        return isFinite(n) ? n : null
    }
    function styleNum(key, fallback) {
        var n = root.styleRawNum(key)
        return n === null ? fallback : n
    }
    function styleAlpha(key, fallback) {
        return Util.clampAlpha(root.styleNum(key, fallback))
    }
    function styleString(key, fallback) {
        var v = root.styleOverrides[key]
        if (typeof v !== "string") return fallback
        var s = v.replace(/^\s+|\s+$/g, "")
        return s.length > 0 ? s : fallback
    }

    readonly property string normalColorToken: root.styleString("normal-color", "foreground")
    readonly property string hoverColorToken: root.styleString("hover-cursor-color", "foreground")
    readonly property string selectedColorToken: root.styleString("selected-color", "foreground")
    readonly property string pressedColorToken: root.styleString("pressed-color", root.hoverColorToken)
    readonly property string focusColorToken: root.styleString("focus-color", root.hoverColorToken)
    readonly property string selectionColorToken: root.styleString("selection-color", "foreground")

    readonly property int normalBorderWidth: Math.max(0, Math.round(root.styleNum("normal-border-width", 1)))
    readonly property int hoverBorderWidth: Math.max(0, Math.round(root.styleNum("hover-cursor-border-width", root.normalBorderWidth)))
    readonly property int selectedBorderWidth: Math.max(0, Math.round(root.styleNum("selected-border-width", 0)))
    readonly property int focusBorderWidth: Math.max(0, Math.round(root.styleNum("focus-border-width", root.hoverBorderWidth)))

    readonly property real normalFillAlpha: root.styleAlpha("normal-fill-alpha", 0.04)
    readonly property real hoverFillAlpha: root.styleAlpha("hover-cursor-fill-alpha", 0.08)
    readonly property real selectedFillAlpha: root.styleAlpha("selected-fill-alpha", 0.18)
    readonly property real pressedFillAlpha: root.styleAlpha("pressed-fill-alpha", 0.22)
    readonly property real focusFillAlpha: root.styleAlpha("focus-fill-alpha", root.hoverFillAlpha)
    readonly property real selectionFillAlpha: root.styleAlpha("selection-fill-alpha", 0.35)

    readonly property real normalBorderAlpha: root.styleAlpha("normal-border-alpha", 0.4)
    readonly property real hoverBorderAlpha: root.styleAlpha("hover-cursor-border-alpha", 0.25)
    readonly property real selectedBorderAlpha: root.styleAlpha("selected-border-alpha", 1.0)
    readonly property real focusBorderAlpha: root.styleAlpha("focus-border-alpha", root.hoverBorderAlpha)

    // The palette's own colours, once, so a state token resolves against them.
    readonly property color foregroundColor: Commons.Color.foreground
    readonly property color accentColor: Commons.Color.accent
    readonly property color urgentColor: Commons.Color.urgent

    function resolve(token) {
        return Controls.resolveToken(token, root.foregroundColor, root.accentColor, root.urgentColor, root.foregroundColor)
    }
    function stateFill(token, alpha) {
        return Util.alpha(root.resolve(token), alpha)
    }
    function stateBorder(token, alpha) {
        return Util.alpha(root.resolve(token), alpha)
    }

    readonly property color normalFill: root.stateFill(root.normalColorToken, root.normalFillAlpha)
    readonly property color hoverFill: root.stateFill(root.hoverColorToken, root.hoverFillAlpha)
    readonly property color selectedFill: root.stateFill(root.selectedColorToken, root.selectedFillAlpha)
    readonly property color pressedFill: root.stateFill(root.pressedColorToken, root.pressedFillAlpha)
    readonly property color focusFillColor: root.stateFill(root.focusColorToken, root.focusFillAlpha)
    readonly property color selectionFill: root.stateFill(root.selectionColorToken, root.selectionFillAlpha)

    readonly property color normalBorderColor: root.stateBorder(root.normalColorToken, root.normalBorderAlpha)
    readonly property color hoverBorderColor: root.stateBorder(root.hoverColorToken, root.hoverBorderAlpha)
    readonly property color selectedBorderColor: root.stateBorder(root.selectedColorToken, root.selectedBorderAlpha)
    readonly property color focusBorderColor: root.stateBorder(root.focusColorToken, root.focusBorderAlpha)
    readonly property color selectedAccentFill: Util.alpha(root.accentColor, root.selectedFillAlpha)

    // ---------------------------------------------------------- spacing
    //
    // The spacing scale is the shell's equivalent of rem: components keep their
    // proportions by asking for the old pixel value through space(), and a theme
    // makes the shell denser or roomier with [spacing] scale, or pins one token.
    property real spacingScale: 1.0
    property bool spacingScaleWithFont: true
    property var spacingOverrides: ({})

    readonly property real fontScale: Math.max(1 / 12, root.fontBaseSize / 12)
    readonly property real effectiveSpacingScale: root.spacingScale * (root.spacingScaleWithFont ? root.fontScale : 1)

    function spaceReal(px) {
        var n = Number(px)
        if (!isFinite(n) || n <= 0) return 0
        return n * root.effectiveSpacingScale
    }
    function space(px) {
        var n = root.spaceReal(px)
        if (n <= 0) return 0
        return Math.max(1, Math.round(n))
    }
    function spacingToken(key, fallback) {
        var v = root.spacingOverrides[key]
        var n = Number(v)
        return (isFinite(n) && n >= 0) ? Math.round(n) : root.space(fallback)
    }

    readonly property QtObject spacing: QtObject {
        readonly property real scale: root.effectiveSpacingScale

        readonly property int hairline: root.space(1)
        readonly property int xxs: root.spacingToken("xxs", 2)
        readonly property int xs: root.spacingToken("xs", 3)
        readonly property int sm: root.spacingToken("sm", 4)
        readonly property int md: root.spacingToken("md", 6)
        readonly property int lg: root.spacingToken("lg", 8)
        readonly property int xl: root.spacingToken("xl", 10)
        readonly property int xxl: root.spacingToken("xxl", 12)
        readonly property int xxxl: root.spacingToken("xxxl", 14)
        readonly property int huge: root.spacingToken("huge", 18)

        readonly property int controlGap: root.spacingToken("control-gap", 8)
        readonly property int controlPaddingX: root.spacingToken("control-padding-x", 10)
        readonly property int controlPaddingY: root.spacingToken("control-padding-y", 6)
        readonly property int inputPaddingY: root.spacingToken("input-padding-y", 7)
        readonly property int controlHeight: root.spacingToken("control-height", 28)
        readonly property int popupRowHeight: root.spacingToken("popup-row-height", 28)
        readonly property int dropdownWidth: root.spacingToken("dropdown-width", 240)
        readonly property int searchableDropdownWidth: root.spacingToken("searchable-dropdown-width", 260)
        readonly property int numberFieldWidth: root.spacingToken("number-field-width", 120)
        readonly property int searchablePopupMinHeight: root.spacingToken("searchable-popup-min-height", 220)
        readonly property int rowGap: root.spacingToken("row-gap", 8)
        readonly property int rowPaddingX: root.spacingToken("row-padding-x", 12)
        readonly property int labelGap: root.spacingToken("label-gap", 4)
        readonly property int panelGap: root.spacingToken("panel-gap", 14)
        readonly property int panelPadding: root.spacingToken("panel-padding", 18)
        readonly property int popupPadding: root.spacingToken("popup-padding", 14)
    }

    // ---------------------------------------------------------- typography
    property string fontFamily: "monospace"
    property string resolvedFontFamily: "monospace"
    property int fontBaseSize: 12
    property var fontOverrides: ({})

    function fontPx(mult) {
        return Math.max(1, Math.round(root.fontBaseSize * mult))
    }
    function fontToken(key, fallback) {
        var v = root.fontOverrides[key]
        var n = Number(v)
        return (isFinite(n) && n > 0) ? Math.round(n) : fallback
    }

    readonly property QtObject font: QtObject {
        readonly property string family: root.fontFamily
        readonly property string resolvedFamily: root.resolvedFontFamily
        readonly property int baseSize: root.fontBaseSize

        readonly property int caption: root.fontToken("caption", root.fontPx(0.833))
        readonly property int bodySmall: root.fontToken("body-small", root.fontPx(0.917))
        readonly property int body: root.fontToken("body", root.fontPx(1.0))
        readonly property int subtitle: root.fontToken("subtitle", root.fontPx(1.083))
        readonly property int title: root.fontToken("title", root.fontPx(1.167))
        readonly property int heading: root.fontToken("heading", root.fontPx(1.333))
        readonly property int display: root.fontToken("display", root.fontPx(2.0))
        readonly property int displayLarge: root.fontToken("display-large", root.fontPx(2.333))

        readonly property int iconSmall: root.fontToken("icon-small", bodySmall)
        readonly property int icon: root.fontToken("icon", title)
        readonly property int iconLarge: root.fontToken("icon-large", root.fontPx(1.5))
    }

    // ---------------------------------------------------------- shell.toml
    //
    // Pull typography, spacing and state tokens out of the shell.toml dict
    // Color already parsed. One parse feeds both.
    function applyShellValues(values) {
        var parsed = Controls.parseShell(values)
        root.spacingScale = parsed.scale
        root.spacingScaleWithFont = parsed.scaleWithFont
        root.fontBaseSize = parsed.base
        root.fontOverrides = parsed.font
        root.spacingOverrides = parsed.spacing
        root.styleOverrides = parsed.style
    }

    // ---------------------------------------------------------- compositor
    //
    // The corner rounding follows the compositor when there is one. The shell
    // reads the compositor's own rounding setting; a box with no compositor
    // keeps the default below.
    readonly property int cornerRadius: compositor.cornerRadius
    property QtObject compositor: Compositor {
        style: root
    }

    function refresh() {
        root.compositor.refresh()
    }
    function scheduleRefresh() {
        root.compositor.scheduleRefresh()
    }

    // The concrete family "monospace" resolves to right now.
    function resolveFontFamily() {
        fcMatchProc.running = true
    }

    property Process fcMatchProc: Process {
        id: fcMatchProc
        command: ["fc-match", "-f", "%{family[0]}", "monospace"]
        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: {
                var name = String(text).trim()
                if (name.length > 0) root.resolvedFontFamily = name
            }
        }
    }

    Component.onCompleted: root.resolveFontFamily()
}
