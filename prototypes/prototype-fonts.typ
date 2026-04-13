// Final font configuration test sheet.
//
// Render with:
//   typst compile --root . --font-path fonts/ prototypes/prototype-fonts.typ output/prototype-fonts.png --ppi 200
//
// Font roles:
//   B612         → labels, header/footer text (proportional)
//   Fira Code    → problem digits (monospaced, with cv11 = plain zero)
//   Fira Math    → operator symbols (×, ÷, ·, :, etc.) AND fractions via math.frac

#set page(width: 22cm, height: auto, margin: 1.2cm)
#set text(font: "B612", size: 11pt)
#show math.equation: set text(font: "Fira Math")

// ───────── Inline components (self-contained) ─────────

#let with-digits(body) = {
  set text(font: "Fira Code", size: 22pt, tracking: 2pt, features: (cv11: 1))
  body
}

#let op(symbol) = box({
  set text(font: "Fira Math")
  symbol
})

#let answer-blank = box(width: 2em, height: 1em, stroke: (bottom: 0.5pt))

#let horizontal-problem(a, op-sym, b) = with-digits({
  box({
    text(str(a))
    h(0.3em); op(op-sym); h(0.3em)
    text(str(b))
    h(0.3em); op(sym.eq); h(0.3em)
    answer-blank
  })
})

#let fraction-problem(whole, n, d) = with-digits({
  box({
    text(str(whole))
    h(0.3em); op(sym.times); h(0.3em)
    $#n / #d$
    h(0.3em); op(sym.eq); h(0.3em)
    answer-blank
  })
})

#let vertical-add(top, bot, op-sym) = with-digits(box({
  set par(leading: 0.3em)
  align(right, text(str(top)))
  v(-0.5em)
  grid(
    columns: (auto, 1fr),
    column-gutter: 0.25em,
    align(left, op(op-sym)),
    align(right, text(str(bot))),
  )
  v(-0.7em)
  line(length: 100%, stroke: 0.8pt)
}))

#let long-div(dividend, divisor) = with-digits(context {
  let dividend-content = text(str(dividend))
  let m = measure(dividend-content)
  let h = m.height * 1.5
  let bulge = h * 0.3
  let overshoot = m.height * 0.25
  grid(
    columns: (auto, auto),
    column-gutter: 0.25em,
    align: bottom,
    pad(bottom: overshoot, text(str(divisor))),
    box({
      v(m.height * 1.0)
      pad(left: bulge + 0.2em, top: 0.45em, dividend-content)
      v(overshoot)
      place(bottom + left, curve(
        stroke: 1.8pt,
        curve.move((0pt, h)),
        curve.quad((bulge, h * 0.45), (0pt, 0pt)),
        curve.line((bulge + m.width + h * 0.7, 0pt)),
      ))
    }),
  )
})

// ───────── Test grid ─────────

#text(weight: "bold", size: 16pt)[Pencil Ready — font configuration test]

#v(0.3em)

#text(size: 10pt, fill: gray)[
  Labels in B612 · digits in Fira Code (cv11 plain zero) · operators and fractions in Fira Math
]

#v(1.2em)

// Compare digit shapes between the fonts that may appear together.
#grid(
  columns: (4.5cm, auto),
  column-gutter: 1em,
  row-gutter: 0.4em,
  align: horizon + left,
  text(weight: "bold")[Fira Code (cv11):],
  text(font: "Fira Code", size: 22pt, tracking: 2pt, features: (cv11: 1))[0123456789],
  text(weight: "bold")[Fira Math (default):],
  text(font: "Fira Math", size: 22pt, tracking: 2pt)[0123456789],
  text(weight: "bold")[Fira Math (math.mono):],
  text(font: "Fira Math", size: 22pt, tracking: 2pt)[
    $ mono(0) mono(1) mono(2) mono(3) mono(4)
      mono(5) mono(6) mono(7) mono(8) mono(9) $
  ],
)

#v(1.2em)

#grid(
  columns: (auto, auto, auto),
  column-gutter: 2em,
  row-gutter: 1.5em,
  align: bottom,

  text(weight: "bold")[Vertical add (with 0)],
  text(weight: "bold")[Vertical multiply],
  text(weight: "bold")[Long division (with 0)],

  vertical-add(307, 490, sym.plus),
  vertical-add(287, 349, sym.times),
  long-div(2050, 5),
)

#v(1.5em)

#grid(
  columns: (auto, auto, auto),
  column-gutter: 2em,
  row-gutter: 1.5em,
  align: horizon,

  text(weight: "bold")[Mult drill (US)],
  text(weight: "bold")[Mult drill (NO)],
  text(weight: "bold")[Div drill (NO, 0s)],

  horizontal-problem(7, sym.times, 8),
  horizontal-problem(7, sym.dot.c, 8),
  horizontal-problem(100, sym.colon, 10),
)

#v(1.5em)

#grid(
  columns: (auto, auto, auto),
  column-gutter: 2em,
  row-gutter: 1.5em,
  align: horizon,

  text(weight: "bold")[Fraction (unit)],
  text(weight: "bold")[Fraction (proper)],
  text(weight: "bold")[Fraction (with 10)],

  fraction-problem(8, 1, 2),
  fraction-problem(20, 4, 5),
  fraction-problem(100, 3, 10),
)
