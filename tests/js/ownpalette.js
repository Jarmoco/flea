.import "../../ui/js/Contrast.js" as Contrast
.import "./palettroles.js" as Roles
.import "./sourcefixture.js" as Source

// Flea's own palette: the one a box with no theme draws with.
//
// It is held to the same floors as every stock palette, because a box without
// a theme is a box that draws with it, and a rule that only the stock themes
// meet is not a rule the product keeps.

// The literals are read off the source, so a change to either file reddens here
// rather than leaving the gate on a palette the product no longer draws.
function literal(text, key) {
    var m = text.match(new RegExp("\\b" + key + ":\\s*\"(#[0-9A-Fa-f]{6})\""))
    return m ? m[1] : null
}

function run(check) {
    var palette = Source.source("ui/Commons/Color.qml")
    var theme = Source.source("ui/Theme.qml")
    var foreground = literal(palette, "foreground")
    var background = literal(palette, "background")
    var accent = literal(palette, "accent")
    var urgent = literal(palette, "urgent")
    var surface = literal(theme, "surface")
    check("Flea carries a palette of its own, with a foreground", foreground, "#cacccc")
    check("Flea carries a palette of its own, with a ground", background, "#101315")
    check("Flea carries a palette of its own, with an accent", accent, "#cacccc")
    check("Flea carries a palette of its own, with an urgent", urgent, "#a55555")
    check("Flea carries a palette of its own, with a surface", surface, "#181825")
    // The muted a palette that sets none takes is the darkened foreground, lifted to the frame
    // floor and then to the text floor, which is the rule every stock palette is held to.
    var r = Roles.roles('background = "' + background + '"\nforeground = "' + foreground + '"\n'
                        + 'accent = "' + accent + '"\nred = "' + urgent + '"\ndark_background = "' + surface + '"\n')
    var name = "Flea's own palette"
    var at = Roles.atLeast
    at(check, name, "foreground on background", Contrast.ratio(r.foreground, r.background), Roles.TEXT_MIN)
    at(check, name, "foreground on surface", Contrast.ratio(r.foreground, r.surface), Roles.TEXT_MIN)
    at(check, name, "muted text on background", Contrast.ratio(r.muted, r.background), Roles.TEXT_MIN)
    at(check, name, "muted text on surface", Contrast.ratio(r.muted, r.surface), Roles.TEXT_MIN)
    at(check, name, "metadata text on background", Contrast.ratio(r.metadata, r.background), Roles.TEXT_MIN)
    at(check, name, "metadata text on surface", Contrast.ratio(r.metadata, r.surface), Roles.TEXT_MIN)
    at(check, name, "muted frame on background", Contrast.ratio(r.mutedFrame, r.background), Roles.MARK_MIN)
    at(check, name, "error on background", Contrast.ratio(r.error, r.background), Roles.TEXT_MIN)
    at(check, name, "error on surface is never dimmer than a caption",
       Contrast.ratio(r.errorOnSurface, r.surface), Contrast.ratio(r.muted, r.surface))
    at(check, name, "accent wash clears background",
       Contrast.ratio(Roles.washed(r.accent, Roles.WASH, r.background), r.background), Roles.WASH_MIN)
    at(check, name, "cursor edge on the unselected ground", Contrast.ratio(r.accent, r.background), Roles.EDGE_MIN)
    at(check, name, "primary frame on surface", Contrast.ratio(r.accentFrame, r.surface), Roles.MARK_MIN)
}
