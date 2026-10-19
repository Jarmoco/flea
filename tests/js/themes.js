.import "../../ui/js/Palette.js" as Palette
.import "../../ui/js/Contrast.js" as Contrast
.import "./palettroles.js" as Roles
.import "sourcefixture.js" as Source

// The stock themes installed under /usr/share/omarchy/themes, read live rather than fixtured, so a
// palette that changes under a theme update is caught here. tests/themes.sh compares this list with
// the directory itself, which is where a newly shipped theme reddens.
var THEMES = ["catppuccin", "catppuccin-latte", "ethereal", "everforest", "flexoki-light", "gruvbox",
              "hackerman", "kanagawa", "last-horizon", "lumon", "lupine", "matte-black", "miasma",
              "nord", "osaka-jade", "retro-82", "ristretto", "rose-pine", "solitude", "tokyo-night",
              "vantablack", "white"]
var THEME_DIR = "file:///usr/share/omarchy/themes/"

// The floors the shared role model carries, and the two this sweep adds.
var TEXT_MIN = Roles.TEXT_MIN
var WASH_MIN = Roles.WASH_MIN
var MARK_MIN = Roles.MARK_MIN
var EDGE_MIN = Roles.EDGE_MIN
var WASH = Roles.WASH
var DISABLED_MIN = 1.5      // Containers: a disabled row still reads as a row, never as an empty one.
var SELECTED_FILL = 0.18    // Style.selectedFillAlpha, the OEM default a theme's shell.toml may raise.
var DISABLED = 0.55         // Theme.disabledOpacity.

// gvfs-free file read: qml6 allows it only with QML_XHR_ALLOW_FILE_READ, which tests/js.sh sets.
// Measured on this box: a file that reads answers status 200, and one that is not there answers 0.
function read(url) {
    var request = new XMLHttpRequest()
    request.open("GET", url, false)
    request.send()
    return { body: String(request.responseText || ""), status: request.status }
}

// The two palette shapes no installed theme has: one setting no muted at all, and one setting color8
// instead. ui/Theme.qml takes neither as the caption, so the mirror must not either.
function mirrorFallbacks(check) {
    var ground = "background = \"#1e1e2e\"\nforeground = \"#cdd6f4\"\naccent = \"#89b4fa\"\nred = \"#f38ba8\"\n"
    var none = Roles.roles(ground)
    // The value tests/themes.sh measured off the running window for this exact palette, which is what
    // binds this mirror's darker() to ui/Theme.qml's own Qt.darker(c, 1.4).
    check("a palette with no muted takes the darkened foreground the window renders", none.muted, "#9299ae")
    Roles.atLeast(check, "a palette with no muted", "its derived caption still clears the floor",
            Contrast.ratio(none.muted, none.background), TEXT_MIN)
    var eight = Roles.roles(ground + "color8 = \"#585b70\"\n")
    check("and color8 is not the caption either, because ui/Theme.qml does not read it",
          eight.muted, none.muted)
}

// The heading ink: no bright key keeps the foreground, a dimmer one keeps it too, a brighter one wins; color15 stands in for an ANSI-ring palette.
function brightSamples(check) {
    var applied = Source.source("ui/Theme.qml") + Source.source("ui/ThemeColors.qml")
    check("Theme.qml derives the heading ink as color.foregroundBright from bright_foreground",
          applied.indexOf("foregroundBright") >= 0 && applied.indexOf('["bright_foreground", "color15"]') >= 0, true)
    var dark = "background = \"#1a1b26\"\nforeground = \"#a9b1d6\"\n"
    check("a palette with no bright key keeps the foreground", Roles.roles(dark).foregroundBright, "#a9b1d6")
    check("a bright_foreground with less contrast keeps the foreground", Roles.roles(dark + "bright_foreground = \"#445066\"\n").foregroundBright, "#a9b1d6")
    check("Tokyo Night's bright_foreground beats its foreground, the board's #c0caf5", Roles.roles(dark + "bright_foreground = \"#c0caf5\"\n").foregroundBright, "#c0caf5")
    check("an ANSI-ring palette's color15 stands in when it sets no bright_foreground", Roles.roles(dark + "color15 = \"#c0caf5\"\n").foregroundBright, "#c0caf5")
    check("bright_foreground wins over color15 when both are set", Roles.roles(dark + "color15 = \"#ffffff\"\nbright_foreground = \"#c0caf5\"\n").foregroundBright, "#c0caf5")
    var light = "background = \"#eff1f5\"\nforeground = \"#4c4f69\"\n"
    check("on a light ground a lighter bright_foreground loses to the foreground", Roles.roles(light + "bright_foreground = \"#bcc0cc\"\n").foregroundBright, "#4c4f69")
}

// The status bar draws error ink on the surface, where the background lift lands under 4.5.
var ERROR_ON_SURFACE_SAMPLES = [
    "background = \"#eff1f5\"\ndark_background = \"#e6e9ef\"\nforeground = \"#4c4f69\"\nred = \"#d20f39\"\n",
    "background = \"#faf4d6\"\ndark_background = \"#f0e9d0\"\nforeground = \"#4c4f69\"\nred = \"#b4637a\"\n",
    "background = \"#eef0f5\"\ndark_background = \"#dfe3ec\"\nforeground = \"#4c4f69\"\nred = \"#c2435c\"\n",
    "background = \"#f5f0e8\"\ndark_background = \"#e8dfd0\"\nforeground = \"#4c4f69\"\nred = \"#a8435f\"\n"
];

// Sample input: one palette body above; the bar's own ground is the surface it sits on.
function errorOnSurfaceSamples(check) {
    var applied = Source.source("ui/Theme.qml")
    check("Theme.qml lifts the status bar red on its surface", applied.indexOf("Contrast.ensureRatio(Commons.Color.urgent, surface,") >= 0, true)
    var weak = 0
    for (var i = 0; i < ERROR_ON_SURFACE_SAMPLES.length; i++) {
        var r = Roles.roles(ERROR_ON_SURFACE_SAMPLES[i])
        Roles.atLeast(check, "surface sample " + i, "error on surface", Contrast.ratio(r.errorOnSurface, r.surface), TEXT_MIN)
        if (Contrast.ratio(Contrast.ensureRatio(r.urgent, r.background, TEXT_MIN), r.surface) < TEXT_MIN)
            weak += 1
    }
    check("the background lift alone lands under 4.5 on at least one surface", weak >= 1, true)
}

// The stock themes are read live, so a palette that changes under a theme update is caught here.
// A box with no desktop shell ships none of them, and Flea's own palette is the gate there.
function stockThemesPresent() {
    return read(THEME_DIR + THEMES[0] + "/colors.toml").status === 200
}

function run(check) {
    mirrorFallbacks(check)
    brightSamples(check)
    errorOnSurfaceSamples(check)
    // A truncated table would iterate few times and report every check it did run as green, so the
    // table's own size is checked first; tests/themes.sh compares it with the directory itself.
    check("the table still names the themes this box ships", THEMES.length >= 22, true)
    if (!stockThemesPresent()) {
        // No stock themes on this box, so the sweep below has nothing to read. Flea's own palette
        // was checked above, and it is the gate here.
        return
    }
    for (var i = 0; i < THEMES.length; i++) {
        var name = THEMES[i]
        var got = read(THEME_DIR + name + "/colors.toml")
        var body = got.body
        check(name + ": colors.toml is there to read", got.status, 200)
        check(name + ": colors.toml parses to a palette", Palette.isPalette(Palette.parse(body)), true)
        // A theme that did not read is one red check, not a throw that leaves the rest unmeasured.
        if (body.length === 0)
            continue
        var r = Roles.roles(body)

        // Text, on both grounds a listing row can sit on.
        Roles.atLeast(check, name, "foreground on background", Contrast.ratio(r.foreground, r.background), TEXT_MIN)
        Roles.atLeast(check, name, "foreground on surface", Contrast.ratio(r.foreground, r.surface), TEXT_MIN)
        Roles.atLeast(check, name, "muted text on background", Contrast.ratio(r.muted, r.background), TEXT_MIN)
        Roles.atLeast(check, name, "muted text on surface", Contrast.ratio(r.muted, r.surface), TEXT_MIN)
        Roles.atLeast(check, name, "metadata text on background", Contrast.ratio(r.metadata, r.background), TEXT_MIN)
        Roles.atLeast(check, name, "metadata text on surface", Contrast.ratio(r.metadata, r.surface), TEXT_MIN)
        Roles.atLeast(check, name, "muted frame on background", Contrast.ratio(r.mutedFrame, r.background), MARK_MIN)

        // The heading ink never reads weaker than the body ink it sits over.
        Roles.atLeast(check, name, "heading ink against the foreground",
                Contrast.ratio(r.foregroundBright, r.background), Contrast.ratio(r.foreground, r.background))

        if (name === "tokyo-night")
            check(name + ": the heading ink is the board's #c0caf5 over the body's " + r.foreground, r.foregroundBright, "#c0caf5")

        // Disabled is muted at 0.55 on the ground: still legible, and visibly weaker than muted.
        var disabled = Roles.washed(r.muted, DISABLED, r.background)
        Roles.atLeast(check, name, "disabled on background", Contrast.ratio(disabled, r.background), DISABLED_MIN)
        check(name + ": disabled reads below muted",
              Contrast.ratio(disabled, r.background) < Contrast.ratio(r.muted, r.background), true)

        // The one wash, on both of its grounds, and the search match run drawn with the same recipe.
        Roles.atLeast(check, name, "accent wash clears background", Contrast.ratio(Roles.washed(r.accent, WASH, r.background), r.background), WASH_MIN)
        Roles.atLeast(check, name, "accent wash clears surface", Contrast.ratio(Roles.washed(r.accent, WASH, r.surface), r.surface), WASH_MIN)
        Roles.atLeast(check, name, "a matched run still reads on its wash",
                Contrast.ratio(r.foreground, Roles.washed(r.accent, WASH, r.background)), TEXT_MIN)

        // The checkbox draws in the two roles above and nothing of its own: a foreground fill with the
        // ground cut out for its tick, and the muted frame when empty, so it needs no rule here.

        // The error role on its ground, and beside the drawn caption it is never the quieter of: ethereal's caption lands at 4.90.
        Roles.atLeast(check, name, "error on background", Contrast.ratio(r.error, r.background), TEXT_MIN)
        Roles.atLeast(check, name, "error is never dimmer than a caption",
                Contrast.ratio(r.error, r.background), Contrast.ratio(r.muted, r.background))
        Roles.atLeast(check, name, "error on surface is never dimmer than a caption",
                Contrast.ratio(r.errorOnSurface, r.surface), Contrast.ratio(r.muted, r.surface))

        // The cursor's edge borders the unselected ground, which is what a low-chroma theme breaks.
        Roles.atLeast(check, name, "cursor edge on the unselected ground", Contrast.ratio(r.accent, r.background), EDGE_MIN)
        Roles.atLeast(check, name, "cursor edge against the selected fill",
                Contrast.ratio(r.accent, Roles.washed(r.accent, SELECTED_FILL, r.background)), EDGE_MIN)

        // A pane without focus keeps its marked rows in the selected rung: above the ground and the hover rung, below the focused selection.
        var inactive = Roles.washed(r.foreground, SELECTED_FILL, r.background)
        Roles.atLeast(check, name, "unfocused selection clears the background", Contrast.ratio(inactive, r.background), WASH_MIN)
        Roles.atLeast(check, name, "unfocused selection clears the hover rung",
                Contrast.ratio(inactive, Roles.washed(r.foreground, HOVER_FILL, r.background)), WASH_MIN)
        Roles.atLeast(check, name, "unfocused selection clears the focused selection",
                Contrast.ratio(inactive, Roles.washed(r.foreground, SELECTION_FILL, r.background)), WASH_MIN)
        Roles.atLeast(check, name, "unfocused cursor edge against the unfocused selection", Contrast.ratio(r.muted, inactive), EDGE_MIN)

        // The primary button carries its identity in a frame, on the card's own surface.
        Roles.atLeast(check, name, "primary frame on surface", Contrast.ratio(r.accentFrame, r.surface), MARK_MIN)
    }
    // Measured: while the walk bounded red alone, 46 colour and ground pairs with a channel at 0 or 255
    // came back under the floor, this one at 2.9996 for a requested 3.
    Roles.atLeast(check, "a channel at the extreme", "the lift toward black still delivers the floor",
            Contrast.ratio(Contrast.ensureRatio("#0099ff", "#ffffff", MARK_MIN), "#ffffff"), MARK_MIN)
    // The other direction, where the bounded channel is the one already at the extreme: measured at
    // 4.4943 for a requested 4.5, on the ground catppuccin actually ships.
    Roles.atLeast(check, "a channel at the extreme", "the lift toward white still delivers the floor",
            Contrast.ratio(Contrast.ensureRatio("#ff0000", "#1e1e2e", TEXT_MIN), "#1e1e2e"), TEXT_MIN)
    tagThemeSamples(check)
}

var HOVER_FILL = 0.08       // Style.hoverFillAlpha, the OEM default.
var SELECTION_FILL = 0.35   // Style.selectionFillAlpha, the OEM default.

// Tag swatches lift to 3:1 on both grounds with the digit fallback decided once per theme load.
function tagThemeSamples(check) {
    var latte = { red: "#d20f39", orange: "#d84e2b", yellow: "#df8e1d", green: "#40a02b",
                  blue: "#1e66f5", magenta: "#ea76cb", dark_foreground: "#9ca0b0" }
    var matte = { red: "#d35f5f", orange: "#c63d3d", yellow: "#b91c1c", green: "#ffc107",
                  blue: "#e68e0d", magenta: "#d35f5f", dark_foreground: "#555555" }
    var white = { red: "#2a2a2a", orange: "", yellow: "#4a4a4a", green: "#3a3a3a",
                  blue: "#1a1a1a", magenta: "#2e2e2e", dark_foreground: "#c0c0c0" }
    var bg = "#eff1f5", surface = "#e6e9ef"
    var t = Contrast.tagTheme(latte, bg, surface)
    check("catppuccin latte reads in colour", t.fallback, false)
    check("catppuccin latte names every tag on both grounds",
          Object.keys(t.colors).length + "|" + Object.keys(t.menuColors).length, "7|7")
    check("catppuccin latte lifts Red to 3:1 on the background",
          Contrast.ratio(t.colors.Red, bg) >= MARK_MIN, true)
    check("catppuccin latte lifts Gray to 3:1 on the surface",
          Contrast.ratio(t.menuColors.Gray, surface) >= MARK_MIN, true)
    check("matte black falls back to digits", Contrast.tagTheme(matte, "#121212", "#1a1a1a").fallback, true)
    check("white falls back to digits", Contrast.tagTheme(white, "#ffffff", "#f2f2f2").fallback, true)
    check("a missing colour falls back", Contrast.tagTheme({
        red: "#d20f39", orange: "", yellow: "#df8e1d", green: "#40a02b",
        blue: "#1e66f5", magenta: "#ea76cb", dark_foreground: "#9ca0b0" }, bg, surface).fallback, true)
}
