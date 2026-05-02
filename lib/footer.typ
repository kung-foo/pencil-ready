#let worksheet-footer(body, debug: false) = {
  let debug-box = if debug { 1pt + red } else { none }
  set text(size: 8pt, fill: rgb("#999999"))
  box(height: 0.8cm, width: 100%, stroke: debug-box,
    align(center + horizon, body))
}

// Canonical footer content used by every generated worksheet. The baseline
// nudges on the heart and the flag pull those glyphs up into the x-height
// of the surrounding text — without them, both sit noticeably lower than
// the letters (Noto Color Emoji's baseline is near the bottom of each
// glyph, which clashes with the body text's metrics).
#let pencil-ready-content = [
  *Pencil Ready* — made with #box(height: 1.2em, baseline: 20%, image("/assets/rainbow-heart.svg")) in Oslo, #box(baseline: -20%)[🇳🇴] — #link("https://pencilready.com")[pencilready.com] — #link("https://creativecommons.org/licenses/by/4.0/")[CC BY 4.0]
]
