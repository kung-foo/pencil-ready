// Two-step linear equation problem: `ax + b = c`, solve for x.
//
//   4x + 5 = 21
//       4x = ___
//        x = ___
//
// The equation is rendered in math mode so `x` is auto-italicized (Fira
// Math) and visually distinct from text-font letters. A 3-row grid keeps
// the `=` column aligned across all rows. Row 1 renders the given form
// (canonical or const-first); rows 2 and 3 are always canonical work rows.
//
// `numbers` layout: (a, b, x, c, form)
//   form = 0 → canonical plus    `ax + b = c`
//   form = 1 → const-first plus  `b + ax = c`
//   form = 2 → canonical minus   `ax - b = c`

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features, problem-line-height

#let algebra-two-step-problem(numbers, operator, debug: false, solved: false, implicit: false, variable: "x") = {
  let a = numbers.at(0)
  let b = numbers.at(1)
  let x-val = numbers.at(2)
  let c = numbers.at(3)
  let form = numbers.at(4)
  // Intermediate value after isolating the coefficient term:
  // form 0, 1 (ax + b = c or b + ax = c): ax = c - b
  // form 2    (ax - b = c):                ax = c + b
  let intermediate = if form == 2 { c + b } else { c - b }

  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  // Digits and operators render in Fira Math (same as horizontal-fraction,
  // keeps multi-digit numbers as single atoms and avoids letter-spacing
  // bleed).
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  // The variable glyph is rendered in STIX Two Text italic — classical
  // serif variable italic (LaTeX look) — so letter variables are visually
  // distinct from the sans-serif digits. Noto Color Emoji is a fallback
  // so emoji variables (e.g. 🍌) also render.
  //
  // Plain text (not math mode) so the font override applies when this
  // content is injected into the outer equation.
  let x-var = text(
    font: ("STIX Two Text", "Noto Color Emoji"),
    style: "italic",
    variable,
  )

  // Coefficient-variable rendering: implicit (`4x`) or explicit (`4 · x`).
  // Implicit: fuse coefficient and variable with a single box so typst
  //           doesn't insert math spacing between them.
  // `ax-grouped` wraps the explicit form in parens for row 1 (where the
  // coefficient term sits inside a larger expression); `ax-bare` drops
  // the parens for the work rows (where it's already isolated by `=`).
  let ax-bare = if implicit {
    $#box[#a#x-var]$
  } else {
    $#a #operator #x-var$
  }
  let ax-grouped = if implicit {
    ax-bare
  } else {
    $(#a #operator #x-var)$
  }

  // Row 1 LHS depends on form. Wrap in box() to prevent the equation from
  // breaking across lines when the cell gets tight. Row 1 uses the
  // grouped (parenthesized) form.
  let row1-lhs = box(if form == 1 {
    // const-first: b + (ax)
    $#b + #ax-grouped$
  } else if form == 2 {
    // canonical minus: (ax) - b
    $#ax-grouped - #b$
  } else {
    // form 0, canonical plus: (ax) + b
    $#ax-grouped + #b$
  })

  // Work-row LHS. When the problem is solved, we show `ax =` and `x =` as
  // explicit steps so the worked example teaches the procedure. Row 2
  // uses the bare form (no parens) — it's already isolated by `=`. When
  // unsolved, the student gets blank lines to write free-form work.
  let row2-lhs = if solved { box(ax-bare) } else { [] }
  let row3-lhs = if solved { $#x-var$ } else { [] }

  // Reserve fixed horizontal space on the right of each `=` so solved and
  // unsolved problems occupy the same bounding box.
  let slot-width = 2.6em
  let row1-right = $#c$  // always shown; c is part of the given equation
  let row2-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if solved { $#intermediate$ }
  }))
  let row3-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if solved { $#x-val$ }
  }))

  box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.4em, x: 0.2em), grid(
    columns: (auto, auto, auto),
    column-gutter: 0.3em,
    row-gutter: problem-line-height,
    align: (right + horizon, center + horizon, left + horizon),
    row1-lhs, sym.eq, row1-right,
    row2-lhs, sym.eq, row2-right,
    row3-lhs, sym.eq, row3-right,
  ))
}
