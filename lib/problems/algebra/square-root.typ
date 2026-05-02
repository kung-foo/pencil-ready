// Squares-and-roots problem: solve `x² ± b = c` or `√x ± b = c` for x.
//
//   x² + 5 = 21        √x + 4 = 9
//      x² = ___           √x = ___
//       x = ___            x = ___
//
// Six equation forms covering both families. Three-row layout via
// `equation-rows` mirrors algebra-two-step so the algebra worksheets
// feel uniform: row 1 is the given equation, row 2 isolates the
// squared / rooted term, row 3 is `x = ___`.
//
// `data` = (form, b, inner, answer, c)
//   form = 0 → x² + b = c       (square family, canonical plus)
//   form = 1 → b + x² = c       (square family, const-first)
//   form = 2 → x² − b = c       (square family, canonical minus)
//   form = 3 → √x + b = c       (root family, canonical plus)
//   form = 4 → b + √x = c       (root family, const-first)
//   form = 5 → √x − b = c       (root family, canonical minus)
//
// `inner` = integer beneath the operator (x for squares, √x = r for roots).
// `answer` = solution shown in row 3 (= inner for squares, = inner² for roots).

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features
#import "/lib/problems/_layouts/equation-rows.typ": equation-rows

// `opts` keys:
//   variable: string (default "x") — the variable glyph
//   col-width: auto | length. When auto, each problem self-sizes from
//     its own rows. Pass an explicit length from the worksheet
//     template to align `=` across multiple problems.
// `mode` = "blank" | "worked" | "answer-only".
#let algebra-square-root-problem(data, mode: "blank", opts: (:), debug: false) = {
  let variable = opts.at("variable", default: "x")
  let col-width = opts.at("col-width", default: auto)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  let form = data.at(0)
  let b = data.at(1)
  let inner = data.at(2)
  let answer = data.at(3)
  let c = data.at(4)

  // Intermediate value (RHS of row 2, after isolating the operator term):
  //   forms 0/1/3/4 (plus): inner-op = c − b
  //   forms 2/5    (minus): inner-op = c + b
  let intermediate = if form == 2 or form == 5 { c + b } else { c - b }

  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  // Math mode pinned to Fira Math — same convention as algebra-two-step.
  show math.equation: set text(font: "Fira Math", features: ())

  // Variable in STIX Two Text italic so it reads as a classical LaTeX
  // variable; same rendering as one-step / two-step. Noto Color Emoji
  // is a fallback for emoji variables (e.g. 🍌).
  let x-var = text(
    font: ("STIX Two Text", "Noto Color Emoji"),
    style: "italic",
    variable,
  )

  // The "inner" expression on the LHS — `x²` for square forms,
  // `√x` for root forms.
  let is-root = form >= 3
  let lhs-inner = if is-root { $sqrt(#x-var)$ } else { $#x-var^2$ }

  // Row 1 LHS depends on form.
  let row1-lhs = box(if form == 0 or form == 3 {
    $#lhs-inner + #b$
  } else if form == 1 or form == 4 {
    $#b + #lhs-inner$
  } else {
    $#lhs-inner - #b$
  })

  // Solved-mode content for rows 2 and 3 — always built (so we can
  // measure them) and conditionally `hide(...)` in non-solved modes.
  // Hiding keeps the bounding box stable across blank / answer-only /
  // worked: each mode reserves identical horizontal/vertical space.
  let show-intermediate = solved and not answer-only
  let row2-lhs-solved = box(lhs-inner)
  let row3-lhs-solved = $#x-var$
  let row2-rhs-solved = $#intermediate$
  let row3-rhs-solved = $#answer$

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
