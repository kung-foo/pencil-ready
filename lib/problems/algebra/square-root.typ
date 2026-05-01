// Squares-and-roots problem: solve `x² ± b = c` or `√x ± b = c` for x.
//
//   x² + 5 = 21        √x + 4 = 9
//      x² = ___           √x = ___
//       x = ___            x = ___
//
// Six equation forms covering both families. Three-row grid mirrors
// algebra-two-step so the algebra worksheets feel uniform: row 1 is
// the given equation, row 2 isolates the squared / rooted term, row
// 3 is `x = ___`. The `=` column aligns vertically across all rows.
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

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features, problem-line-height

// `opts` keys:
//   variable: string (default "x") — the variable glyph
// `mode` = "blank" | "worked" | "answer-only".
#let algebra-square-root-problem(data, mode: "blank", opts: (:), debug: false) = {
  let variable = opts.at("variable", default: "x")
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
  // Keeps multi-digit numbers as single atoms and gives proper √ / ^
  // metrics.
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  // Variable in STIX Two Text italic so it reads as a classical LaTeX
  // variable; same rendering as one-step / two-step. Noto Color Emoji
  // is a fallback for emoji variables (e.g. 🍌).
  let x-var = text(
    font: ("STIX Two Text", "Noto Color Emoji"),
    style: "italic",
    variable,
  )

  // The "inner" expression on the LHS — `x²` for square forms,
  // `√x` for root forms. Both are single math atoms so they slot into
  // the row-1 equation alongside `+ b` / `− b` without parens.
  let is-root = form >= 3
  let lhs-inner = if is-root { $sqrt(#x-var)$ } else { $#x-var^2$ }

  // Row 1 LHS depends on form. Wrap in box() to prevent the equation
  // from breaking across lines under tight cells.
  let row1-lhs = box(if form == 0 or form == 3 {
    // canonical plus
    $#lhs-inner + #b$
  } else if form == 1 or form == 4 {
    // const-first plus
    $#b + #lhs-inner$
  } else {
    // forms 2 / 5: canonical minus
    $#lhs-inner - #b$
  })

  // Work-row LHS: when solved, row 2 shows the isolated `x² =` or
  // `√x =`, then row 3 shows `x = answer`. When unsolved, both rows
  // are blank for free-form work. Answer-only mode skips row 2 (the
  // intermediate is scratch work — an answer key doesn't need it) but
  // keeps row 3 so the answer-page grid aligns with the problem grid.
  let show-intermediate = solved and not answer-only
  let row2-lhs = if show-intermediate { box(lhs-inner) } else { [] }
  let row3-lhs = if solved { $#x-var$ } else { [] }

  // Reserve a fixed-width slot to the right of each `=` so blank /
  // worked / answer-only problems share the same bounding box.
  let slot-width = 2.6em
  let row1-right = $#c$
  let row2-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if show-intermediate { $#intermediate$ }
  }))
  let row3-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if solved { $#answer$ }
  }))

  let content = box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.4em, x: 0.2em), grid(
    columns: (auto, auto, auto),
    column-gutter: 0.3em,
    row-gutter: problem-line-height,
    align: (right + horizon, center + horizon, left + horizon),
    row1-lhs, sym.eq, row1-right,
    row2-lhs, sym.eq, row2-right,
    row3-lhs, sym.eq, row3-right,
  ))

  // Self-pad + self-align: same convention as one-step / two-step. The
  // 1.5cm right pad keeps the answer column from kissing the next cell.
  align(right + top, pad(left: 0.3cm, right: 1.5cm, content))
}
