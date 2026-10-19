.pragma library

// The pure half of the token model: turning a theme's token names into colours.
//
// A theme's shell.toml names a state colour either as a palette role
// ("foreground", "accent", "urgent", "background") or as a hex colour. This
// file resolves that name to a colour and nothing else: the alpha is composed
// by the caller, so one place owns that.

// A hex colour, or the fallback when the value is not one. Six digits, eight
// with an alpha, or three shorthand.
function colorFromHex(value, fallback) {
    var s = String(value === undefined || value === null ? "" : value).replace(/^\s+|\s+$/g, "")
    var shortHex = s.match(/^#([0-9A-Fa-f]{3})$/)
    if (shortHex) {
        var sh = shortHex[1]
        return Qt.rgba(
            parseInt(sh.charAt(0) + sh.charAt(0), 16) / 255,
            parseInt(sh.charAt(1) + sh.charAt(1), 16) / 255,
            parseInt(sh.charAt(2) + sh.charAt(2), 16) / 255,
            1)
    }
    var hex = s.match(/^#([0-9A-Fa-f]{6})([0-9A-Fa-f]{2})?$/)
    if (!hex) return fallback
    var h = hex[1]
    return Qt.rgba(
        parseInt(h.substr(0, 2), 16) / 255,
        parseInt(h.substr(2, 2), 16) / 255,
        parseInt(h.substr(4, 2), 16) / 255,
        hex[2] ? parseInt(hex[2], 16) / 255 : 1)
}

// A token name to a colour. A role takes the palette's own colour; anything
// else is read as a hex colour and falls back when it is not one.
function resolveToken(token, foreground, accent, urgent, fallback) {
    var fb = fallback === undefined ? foreground : fallback
    var s = String(token === undefined || token === null ? "" : token).replace(/^\s+|\s+$/g, "")
    var role = s.toLowerCase()
    if (role === "foreground" || role === "text") return foreground
    if (role === "accent") return accent
    if (role === "urgent") return urgent
    if (role === "background") return fb
    if (role === "transparent") return Qt.rgba(0, 0, 0, 0)
    return colorFromHex(s, fb)
}

// A shell.toml value that is true or false, spelled the ways a theme spells it.
function boolToken(value, fallback) {
    if (value === undefined || value === null) return fallback
    var s = String(value).replace(/^\s+|\s+$/g, "").toLowerCase()
    if (s === "true" || s === "1" || s === "yes" || s === "on") return true
    if (s === "false" || s === "0" || s === "no" || s === "off") return false
    return fallback
}

// The three sections of the shell.toml dict, split out and read.
//
//   [font]     the base size and the type ladder's own stops
//   [spacing]  the scale, and the tokens a theme pins
//   [controls] the state tokens, under [style] as well
//
// A pinned spacing token is absolute; a scale multiplies the ones that are not.
function parseShell(values) {
    var font = {}
    var style = {}
    var spacing = {}
    var base = 12
    var scale = 1.0
    var scaleWithFont = true
    var v = values || {}
    for (var fullKey in v) {
        var dot = fullKey.indexOf(".")
        if (dot < 0) continue
        var section = fullKey.substr(0, dot)
        var key = fullKey.substr(dot + 1)
        var raw = v[fullKey]
        if (section === "font") {
            var ival = parseInt(raw, 10)
            if (!isFinite(ival)) continue
            if (key === "base-size") base = ival
            else font[key] = ival
        } else if (section === "spacing") {
            if (key === "scale-with-font") {
                scaleWithFont = boolToken(raw, scaleWithFont)
            } else {
                var fval = parseFloat(raw)
                if (!isFinite(fval)) continue
                if (key === "scale") scale = fval
                else spacing[key] = fval
            }
        } else if (section === "controls" || section === "style") {
            style[key] = raw
        }
    }
    if (!isFinite(base) || base < 1) base = 1
    if (!isFinite(scale) || scale < 0) scale = 1.0
    return { font: font, style: style, spacing: spacing, base: base, scale: scale, scaleWithFont: scaleWithFont }
}
