// Factors by factor pairs: under the number, a row per pair
// (`1 × 24`, `2 × 12`, …). Trying 1, 2, 3, … in order until the pairs
// meet finds every factor without missing one, and the chart reads off
// as the list: down the left column, then up the right.
//
//        24
//    1 × 24
//    2 × 12
//    3 ×  8
//    4 ×  6
//
// A perfect square ends on a pair with itself (`6 × 6` for 36); a prime
// has the one pair `1 × p`.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features

// `data` = (n,). The component works out the pairs itself.
// `opts` keys (with defaults):
//   operator: multiply sign, per locale. Default `[#sym.times]`.
//   rows: pair rows the cell reserves, the most any number on the page
//     needs, so every cell is the same size and the blank rows don't
//     give away the count. Default 6 (the most for numbers to 100).
//   exact-rows: show only as many rows as this number has pairs, as a
//     hint for how many to find; the rest of the reserved room stays
//     blank, so the cell is the same size. Default false.
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   align: cell alignment. Default `center + top`.
// `mode` = "blank" | "worked" | "answer-only". Both solved modes fill in
// the pairs: they are the answer.
#let factor-pairs-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator", default: [#sym.times])
  let rows = opts.at("rows", default: 6)
  let exact-rows = opts.at("exact-rows", default: false)
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: center + top)
  let solved = mode != "blank"

  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let color = if answer-color != none { answer-color } else { black }
  let handwritten(body) = if answer-font != none {
    text(font: answer-font, fill: color, size: 1.4em, body)
  } else {
    text(fill: color, body)
  }

  let n = data.at(0)
  let pairs = range(1, n + 1)
    .filter(d => d * d <= n and calc.rem(n, d) == 0)
    .map(d => (d, calc.quo(n, d)))
  assert(pairs.len() <= rows, message: "factor-pairs: more pairs than reserved rows")

  // A blank wide enough for three handwritten digits, the number on it
  // when solved.
  let slot(v) = box(
    width: 2.2em,
    height: 1.2em,
    stroke: (bottom: 0.5pt),
    align(center + bottom, pad(bottom: 0.2em, if v != none { handwritten(str(v)) })),
  )
  let op = text(font: operator-font, operator)
  // Every row reserves its room; solved rows past the last pair are
  // hidden, so the cell is the same size in every mode.
  let cells = range(rows).map(i => {
    if not solved and (not exact-rows or i < pairs.len()) {
      (slot(none), op, slot(none))
    } else if solved and i < pairs.len() {
      let (a, b) = pairs.at(i)
      (slot(a), op, slot(b))
    } else {
      (hide(slot(none)), hide(op), hide(slot(none)))
    }
  }).flatten()

  let body = stack(
    dir: ttb,
    spacing: 0.3em,
    align(center, str(n)),
    grid(
      columns: 3,
      column-gutter: 0.4em,
      row-gutter: 0.15em,
      align: (center + bottom, center + horizon, center + bottom),
      ..cells,
    ),
  )
  align(cell-align, pad(x: 0.23em, box(stroke: debug-box, inset: 0.2em, body)))
}
