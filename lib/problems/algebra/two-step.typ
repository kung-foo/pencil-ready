// Two-step linear equation problem: `ax + b = c`, solve for x.
//
//   (4·x) + 5 = 21
//        4·x = ___
//          x = ___
//
// Three-row layout via `equation-rows`. Row 1 renders the given form
// (canonical or const-first); rows 2 and 3 are always canonical work
// rows. The shared helper makes col1 = col3 (symmetric around `=`)
// so equals signs line up across all three rows automatically.
//
// `numbers` layout: (a, b, x, c, form)
//   form = 0 → canonical plus    `ax + b = c`
//   form = 1 → const-first plus  `b + ax = c`
//   form = 2 → canonical minus   `ax - b = c`

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features
#import "/lib/problems/_layouts/equation-rows.typ": equation-rows

// `data` = (a, b, x-val, c, form).
// `opts` keys:
//   operator: typst content used for implicit × (e.g. [#sym.dot.op])
//   implicit: bool (default false) — coefficient-variable juxtaposition
//   variable: string (default "x") — the variable glyph
//   col-width: auto | length. When auto, each problem self-sizes from
//     its own rows. Pass an explicit length from the worksheet
//     template to align `=` across multiple problems.
// `mode` = "blank" | "worked" | "answer-only".
#let algebra-two-step-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let implicit = opts.at("implicit", default: false)
  let variable = opts.at("variable", default: "x")
  let col-width = opts.at("col-width", default: auto)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  let a = data.at(0)
  let b = data.at(1)
  let x-val = data.at(2)
  let c = data.at(3)
  let form = data.at(4)
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

  // Row 1 LHS — depends on form. Wrap in box() to prevent the equation
  // from breaking across lines.
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

  // Solved-mode content for rows 2 and 3 — always built (so we can
  // measure them) and conditionally `hide(...)` in non-solved modes.
  // Hiding keeps the bounding box stable: blank, answer-only, and
  // worked all reserve identical horizontal/vertical space.
  let show-intermediate = solved and not answer-only
  let row2-lhs-solved = box(ax-bare)
  let row3-lhs-solved = $#x-var$
  let row2-rhs-solved = $#intermediate$
  let row3-rhs-solved = $#x-val$

  let row1-rhs = $#c$
  let row2-lhs = if show-intermediate { row2-lhs-solved } else { hide(row2-lhs-solved) }
  let row2-rhs = if show-intermediate { row2-rhs-solved } else { hide(row2-rhs-solved) }
  let row3-lhs = if solved { row3-lhs-solved } else { hide(row3-lhs-solved) }
  let row3-rhs = if solved { row3-rhs-solved } else { hide(row3-rhs-solved) }

  equation-rows(
    (
      (row1-lhs, row1-rhs),
      (row2-lhs, row2-rhs),
      (row3-lhs, row3-rhs),
    ),
    col-width: col-width,
    debug: debug,
  )
}
