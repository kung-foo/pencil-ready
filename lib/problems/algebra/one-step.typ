// One-step linear equation problem: solve for `x` in a single move.
//
//   x + 7 = 12         5 · x = 30          x ÷ 6 = 4
//       x = ___            x = ___             x = ___
//
// Two-row grid: row 1 is the given equation in one of four forms,
// row 2 is the canonical `x = ___` solution line. The `=` column
// aligns vertically across rows, same rhythm as algebra-two-step
// minus the intermediate-work row.
//
// `data` = (form, p, x-val, c)
//   form: 0 → x + p = c    (add, p is constant b)
//   form: 1 → x − p = c    (sub, p is constant b)
//   form: 2 → p · x = c    (mul, p is coefficient a)
//   form: 3 → x ÷ p = c    (div, p is divisor a)

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features, problem-line-height

// `opts` keys:
//   mult-operator: typst content for `·` (e.g. [#sym.dot.op]). Algebra
//     pins this to a dot regardless of locale; --symbol overrides.
//   div-operator: typst content for the horizontal divide glyph
//     ([#sym.div] in US, [#sym.colon] in Norway).
//   variable: string (default "x") — the variable glyph
// `mode` = "blank" | "worked" | "answer-only".
//   one-step has no intermediate row, so "worked" and "answer-only"
//   render identically (the answer fills row 2's right slot).
#let algebra-one-step-problem(data, mode: "blank", opts: (:), debug: false) = {
  let mult-op = opts.at("mult-operator", default: [#sym.dot.op])
  let div-op = opts.at("div-operator", default: [#sym.div])
  let variable = opts.at("variable", default: "x")
  let solved = mode != "blank"

  let form = data.at(0)
  let p = data.at(1)
  let x-val = data.at(2)
  let c = data.at(3)

  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  // Math equations render in Fira Math — same convention as two-step,
  // keeps multi-digit numbers as single atoms and avoids letter-spacing
  // bleed in things like "10".
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  // Variable in STIX Two Text italic so it reads as a classical LaTeX
  // variable rather than a sans-serif letter. Same rendering as two-step.
  let x-var = text(
    font: ("STIX Two Text", "Noto Color Emoji"),
    style: "italic",
    variable,
  )

  // Row 1 LHS — one of four forms. Wrapped in `box(...)` so the equation
  // doesn't break across lines if a tight cell pushes things.
  let row1-lhs = box(if form == 0 {
    $#x-var + #p$
  } else if form == 1 {
    $#x-var - #p$
  } else if form == 2 {
    $#p #mult-op #x-var$
  } else {
    $#x-var #div-op #p$
  })

  // Row 2 LHS is always `x` — the canonical solution line.
  let row2-lhs = $#x-var$

  // Reserve fixed horizontal space on the right of `=` so blank and
  // worked problems share the same bounding box. Same `slot-width` as
  // two-step.
  let slot-width = 2.6em
  let row1-right = $#c$
  let row2-right = box(
    width: slot-width,
    height: 1em,
    align(left + horizon, {
      if solved { $#x-val$ }
    }),
  )

  let content = box(
    stroke: debug-box,
    inset: (top: 0.2em, bottom: 0.4em, x: 0.2em),
    grid(
      columns: (auto, auto, auto),
      column-gutter: 0.3em,
      row-gutter: problem-line-height,
      align: (right + horizon, center + horizon, left + horizon),
      row1-lhs, sym.eq, row1-right,
      row2-lhs, sym.eq, row2-right,
    ),
  )

  // Self-pad + self-align — same convention as two-step. Right+top to
  // keep the equals-column rhythm across problems in a row.
  align(right + top, pad(left: 0.3cm, right: 1.5cm, content))
}
