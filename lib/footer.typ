#let worksheet-footer(body) = {
  set text(size: 8pt, fill: rgb("#999999"))
  align(center, body)
}

// Canonical footer content used by every generated worksheet. The baseline
// nudges on the heart and the flag pull those glyphs up into the x-height
// of the surrounding text — without them, both sit noticeably lower than
// the letters (Noto Color Emoji's baseline is near the bottom of each
// glyph, which clashes with the body text's metrics).
#let pencil-ready-content = [
  *Pencil Ready* — made with #box(height: 1.2em, baseline: 20%, image("/assets/rainbow-heart.svg")) in Oslo, #box(baseline: -20%)[🇳🇴] — #link("https://pencilready.com")[pencilready.com]
]
