.pragma library

.import "../../ui/js/Palette.js" as Palette
.import "../../ui/js/Contrast.js" as Contrast

// The role model every palette is held to: the four foundational roles, the
// surfaces a listing row can sit on, and the inks the window lifts from them.
//
// One subject, so the stock themes and Flea's own palette are measured by the
// same rules and cannot drift apart.

// Every threshold carries the rule it answers to; none of them is fitted to what the palettes do.
var TEXT_MIN = 4.5          // HANDOFF rule 2: body text on its own ground, WCAG AA.
var WASH_MIN = 1.06         // A 14 percent wash has to be visible against the ground it washes.
var MARK_MIN = 3.0          // A drawn mark or frame is a graphical object, AA at 3:1.
var EDGE_MIN = 1.5          // The cursor's accent edge borders the unselected ground, so a low-chroma theme keeps it.
var WASH = 0.14             // Theme.washActive, the one wash strength on every surface.

// Commons/Color.qml's own mapping from colors.toml, key for key, plus the roles ui/Theme.qml derives
// on top of it; tests/themes.sh reads the running window, which is what catches this mirror drifting.
function roles(body) {
    var found = Palette.parse(body)
    var foreground = Palette.pick(found, ["foreground", "color7"], "#cacccc")
    var background = Palette.pick(found, ["background", "color0"], "#101315")
    var accent = Palette.pick(found, ["accent", "color4"], "#cacccc")
    var urgent = Palette.pick(found, ["red", "color1"], "#a55555")
    var surface = Palette.pick(found, Palette.SURFACE_KEYS, "#181825")
    var mutedFrame = Contrast.ensureRatio(Palette.pick(found, ["muted"], darker(foreground)), background, MARK_MIN)
    var muted = Contrast.ensureRatio(Contrast.ensureRatio(mutedFrame, background, TEXT_MIN), surface, TEXT_MIN)
    var hued = saturation(urgent) > 0.2
    var error = hued ? Contrast.ensureRatio(urgent, background, Math.max(TEXT_MIN, Contrast.ratio(muted, background))) : foreground
    var errorOnSurface = hued ? Contrast.ensureRatio(urgent, surface, Math.max(TEXT_MIN, Contrast.ratio(muted, surface))) : foreground
    return { foreground: foreground, background: background, accent: accent, urgent: urgent,
             // ui/Theme.qml's rule: the palette's muted or a darkened foreground, lifted to the frame floor for frames, then to the text floor on both grounds for text.
             mutedFrame: mutedFrame,
             muted: muted,
             // Metadata ink: the darkened foreground, unlifted for a resting glyph and lifted like muted for text.
             metadataFrame: darker(foreground),
             metadata: Contrast.ensureRatio(Contrast.ensureRatio(darker(foreground), background, TEXT_MIN), surface, TEXT_MIN),
             // ui/Theme.qml: the heading ink is the palette's bright_foreground (color15 without one) when it has more contrast on the ground than the foreground.
             foregroundBright: brighter(foreground, Palette.pick(found, ["bright_foreground", "color15"], foreground), background),
             surface: surface,
             accentFrame: Contrast.ensureRatio(accent, surface, MARK_MIN),
             // ui/Theme.qml: a red with no chroma of its own reads as switched off, so it is dropped
             // for the foreground; every other red is lifted to AA the way symlink and executable are.
             error: error,
             // ui/Theme.qml's status bar role: the same red lifted on the surface it is drawn on.
             errorOnSurface: errorOnSurface }
}

// Qt.darker(c, 1.4) in the value channel, which is ui/Theme.qml's fallback when a palette sets no
// muted of its own. Every stock palette does set one, so this is the path no installed theme takes.
function darker(hex) {
    var c = Contrast.parse(hex)
    var hi = Math.max(c[0], Math.max(c[1], c[2]))
    if (hi <= 0)
        return hex
    var scale = (hi / 1.4) / hi
    return Contrast.hexOf([c[0] * scale, c[1] * scale, c[2] * scale])
}

function brighter(foreground, candidate, background) {
    return Contrast.ratio(candidate, background) > Contrast.ratio(foreground, background) ? candidate : foreground
}

function saturation(hex) {
    var c = Contrast.parse(hex)
    var hi = Math.max(c[0], Math.max(c[1], c[2]))
    var lo = Math.min(c[0], Math.min(c[1], c[2]))
    return hi <= 0 ? 0 : (hi - lo) / hi
}

function washed(colour, alpha, ground) {
    return Contrast.hexOf(Contrast.over(colour, alpha, ground))
}

// No slack: ui/js/Contrast.js delivers the ratio it was asked for after rounding, so a lift that lands
// at 2.99 for a requested 3 is the defect this suite exists to catch rather than a tolerance to grant.
function atLeast(check, theme, rule, got, floor) {
    // Four decimals, because a lift that lands at 2.9996 prints as 3.00 and reads as a false failure.
    check(theme + ": " + rule + " is " + got.toFixed(4) + ", at least " + floor.toFixed(2),
          got >= floor ? "ok" : "under " + floor.toFixed(2) + " at " + got.toFixed(4), "ok")
}
