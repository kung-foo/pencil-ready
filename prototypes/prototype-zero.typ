// Zoomed comparison of just the zero glyph for various Fira Mono features.
//
// Render with:
//   typst compile --root . --font-path fonts/ prototypes/prototype-zero.typ output/prototype-zero.png --ppi 300

#set page(width: 14cm, height: auto, margin: 1cm)
#set text(font: "B612", size: 14pt)

#let row(label, body) = grid(
  columns: (3.5cm, auto),
  column-gutter: 1.5em,
  align: horizon + left,
  text(weight: "bold", label),
  text(font: "Fira Code", size: 60pt, body),
)

#row("Default",          [0])
#v(0.3em)
#row("zero: 0",          text(features: (zero: 0))[0])
#v(0.3em)
#row("cv11 = 1",         text(features: (cv11: 1))[0])
#v(0.3em)
#row("cv12 = 1",         text(features: (cv12: 1))[0])
#v(0.3em)
#row("cv13 = 1",         text(features: (cv13: 1))[0])
#v(0.3em)
#row("cv14 = 1",         text(features: (cv14: 1))[0])
#v(0.3em)
#row("cv15 = 1",         text(features: (cv15: 1))[0])
