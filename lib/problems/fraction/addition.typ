// Fraction addition and subtraction: a/b ± c/d, answer in simplest form.
//
//   1   1   3   2      row 1: rewrite over the common denominator
//   ─ + ─ = ─ + ─
//   2   3   6   6
//
//         = 5          row 2: combine the numerators
//           ─
//           6
//
//         = …          row 3: simplify (reduce, or improper → mixed),
//                      only when that changes anything
//
// Same-denominator problems skip row 1's rewrite: their first step is
// the combined fraction. Built on `equation-rows`, so the `=` signs line
// up down the cell, like fraction multiplication.
//
// Subtraction is the same component with `subtract: true`;
// `subtraction.typ` re-exports it under its own name.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features
#import "/lib/problems/_layouts/equation-rows.typ": equation-rows

// `data` = (a, b, c, d, whole, num, den): the problem a/b ± c/d and the
// generator's answer in simplest form, read as `whole num/den`
// (num = 0: a whole number; whole = 0: a proper fraction). The
// component re-derives the steps and asserts the answer matches.
//
// `opts` keys (with defaults):
//   operator: typst content, e.g. `[#sym.plus]`. Required.
//   subtract: bool, default false. Must agree with `operator`.
//   rows: rows the cell reserves, default 3. Pass 2 when every problem
//     on the page shares its denominator (no rewrite step), so the
//     cell is shorter. Missing steps are hidden placeholders, so every
//     cell on a page has the same height and the problems line up.
//   answer-font / answer-color: solved-answer styling, as in
//     fraction-multiplication. Default: inherit.
//   col-width / symmetric / rhs-align: forwarded to `equation-rows`.
//
// `mode` = "blank" | "worked" | "answer-only". "blank" prints the
// problem and its `=`; "worked" shows every step; "answer-only" puts the
// final answer straight after the problem.
#let fraction-addition-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let subtract = opts.at("subtract", default: false)
  let rows = opts.at("rows", default: 3)
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let col-width = opts.at("col-width", default: auto)
  let symmetric = opts.at("symmetric", default: true)
  let rhs-align = opts.at("rhs-align", default: left + horizon)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }
  // NOTE: no tracking on the outer text — math.frac inherits it and
  // splits multi-digit numerators ("12" → "1 2").
  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  show math.equation: set text(font: "Fira Math", features: ())

  let (a, b, c, d, whole, num, den) = data

  // Steps, derived from the operands.
  let l = calc.lcm(b, d)
  let ea = calc.quo(a * l, b)
  let ec = calc.quo(c * l, d)
  let raw = if subtract { ea - ec } else { ea + ec }
  assert(raw > 0, message: "fraction-addition: result must be positive")
  let g = calc.gcd(raw, l)
  let (rn, rd) = (calc.quo(raw, g), calc.quo(l, g))
  let (fw, fn) = (calc.quo(rn, rd), calc.rem(rn, rd))
  // The denominator is meaningless for a whole-number answer.
  assert(
    (fw, fn, if fn == 0 { 0 } else { rd }) == (whole, num, if num == 0 { 0 } else { den }),
    message: "fraction-addition: generator answer disagrees with the steps",
  )

  let frac(n, d) = $#str(n) / #str(d)$
  let final = if fn == 0 {
    $#str(fw)$
  } else if fw == 0 {
    frac(fn, rd)
  } else {
    // Thin space so the mixed number reads as one quantity.
    $#str(fw) thin #str(fn) / #str(rd)$
  }

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // Solved answers can be styled as handwriting (thumbnails). The
  // handwriting font isn't a math font, so draw the fraction bar
  // ourselves, sized to the wider of numerator and denominator.
  let style-answer = body => if answer-font != none {
    {
      set text(font: resolved-answer-font, fill: resolved-answer-color, size: 1.4em)
      show math.equation: set text(font: resolved-answer-font, features: ())
      show math.frac: it => context {
        let bar-w = calc.max(measure(it.num).width, measure(it.denom).width) + 0.2em
        box(baseline: 0.5em, stack(
          align(center, it.num),
          v(0.15em),
          line(length: bar-w, stroke: 0.8pt + resolved-answer-color),
          v(0.15em),
          align(center, it.denom),
        ))
      }
      body
    }
  } else {
    body
  }

  let steps = ()
  if b != d {
    steps.push({ box(frac(ea, l)); op-box; box(frac(ec, l)) })
  }
  steps.push(frac(raw, l))
  // Simplifying changes nothing for a proper fraction already in lowest
  // terms; anything else (reducible, improper, whole) gets a final row.
  if g != 1 or raw >= l {
    steps.push(final)
  }
  steps = steps.map(style-answer)
  assert(steps.len() <= rows, message: "fraction-addition: more steps than reserved rows")

  let last = steps.len() - 1
  let at(i) = steps.at(calc.min(i, last))
  let lhs = { box(frac(a, b)); op-box; box(frac(c, d)) }
  // Every row holds some step, shown or hidden, so the cell's size is
  // the same in every mode: each step appears exactly once (rows past
  // the last step repeat it, hidden). Only rows that print something
  // get an `=`.
  let cells = range(rows).map(i => {
    let first = if i == 0 { lhs } else { [] }
    if answer-only {
      // The answer sits next to the problem; the steps it replaces
      // move down a row, hidden, to keep their width in the measure.
      if i == 0 { (lhs, steps.at(last)) } else { ([], hide(at(i - 1)), false) }
    } else if solved and i <= last {
      (first, steps.at(i))
    } else {
      (first, hide(at(i)), i == 0)
    }
  })

  equation-rows(
    cells,
    col-width: col-width,
    symmetric: symmetric,
    rhs-align: rhs-align,
    // The last row usually ends in a fraction; leave room for its
    // denominator inside the box. Rows are fraction-tall, so a tighter
    // gap between them still reads clearly and pays for that room.
    bottom-inset: 0.7em,
    row-gutter: 1.1em,
    debug: debug,
  )
}
