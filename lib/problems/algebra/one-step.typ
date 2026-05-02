// One-step linear equation problem: solve for `x` in a single move.
//
//   x + 7 = 12         5 · x = 30          x ÷ 6 = 4
//       x = ___            x = ___             x = ___
//
// Two-row layout via `equation-rows`: row 1 is the given equation in
// one of four forms, row 2 is the canonical `x = ___` solution line.
// `equation-rows` makes col1 = col3 (symmetric around `=`) so equals
// signs line up vertically — and exposes a `col-width` opt that the
// worksheet template can use to keep `=` aligned across every problem
// on the page.
//
// `data` = (form, p, x-val, c)
//   form: 0 → x + p = c    (add, p is constant b)
//   form: 1 → x − p = c    (sub, p is constant b)
//   form: 2 → p · x = c    (mul, p is coefficient a)
//   form: 3 → x ÷ p = c    (div, p is divisor a)

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features
#import "/lib/problems/_layouts/equation-rows.typ": equation-rows

// `opts` keys:
//   mult-operator: typst content for `·` (e.g. [#sym.dot.op]). Algebra
//     pins this to a dot regardless of locale; --symbol overrides.
//   div-operator: typst content for the horizontal divide glyph
//     ([#sym.div] in US, [#sym.colon] in Norway).
//   variable: string (default "x") — the variable glyph
//   col-width: auto | length. When auto, each problem self-sizes to
//     `max(widest LHS, widest RHS)` of its own rows. Pass an explicit
//     length from the worksheet template to align `=` across multiple
//     problems on the same page.
// `mode` = "blank" | "worked" | "answer-only".
//   one-step has no intermediate row, so "worked" and "answer-only"
//   render identically (the answer fills row 2's right slot).
#let algebra-one-step-problem(data, mode: "blank", opts: (:), debug: false) = {
  let mult-op = opts.at("mult-operator", default: [#sym.dot.op])
  let div-op = opts.at("div-operator", default: [#sym.div])
  let variable = opts.at("variable", default: "x")
  let col-width = opts.at("col-width", default: auto)
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

  // RHS slots. Row 2's answer uses `hide(...)` in blank mode so the
  // bounding rect (and `equation-rows`'s width measurement) stays
  // identical between blank and solved.
  let row1-rhs = $#c$
  let row2-rhs = if solved { $#x-val$ } else { hide($#x-val$) }

  equation-rows(
    (
      (row1-lhs, row1-rhs),
      (row2-lhs, row2-rhs),
    ),
    col-width: col-width,
    debug: debug,
  )
}
