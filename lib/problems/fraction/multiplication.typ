// Whole-number × proper-fraction problem.
//
//   30 × 7/10 = 21/10
//             = 21
//
// Two-row layout via `equation-rows`. Row 1 holds `whole × n/d` on the
// left and the multiply-across intermediate fraction on the right.
// Row 2's LHS is blank — the `=` carries the eye to the simplified
// integer answer on the right. `equation-rows` makes col1 = col3
// (symmetric around `=`) so the equals signs line up vertically.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features
#import "/lib/problems/_layouts/equation-rows.typ": equation-rows

// `data` = (whole, numerator, denominator).
// `opts` keys (with defaults):
//   operator: typst content (e.g. `[#sym.times]`). Required.
//   answer-font: typst font for solved-mode answers. Default: none =
//     inherit problem-font / Fira Math.
//   answer-color: color for solved-mode answers. Default: none = inherit.
//   col-width: auto | length. When auto, each problem self-sizes from
//     its own rows. Pass an explicit length from the worksheet template
//     to align `=` across multiple problems.
// `mode` = "blank" | "worked" | "answer-only". "worked" fills in the
// multiply-across intermediate + simplified answer; "answer-only"
// suppresses the multiply-across and shows only the simplified result.
#let fraction-multiplication-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let col-width = opts.at("col-width", default: auto)
  // symmetric col1 = col3 keeps `=` at the bbox center for cross-cell
  // alignment in the worksheet grid. Default true (worksheet usage).
  // Pass false from a thumb where the LHS (`whole × n/d`) is much
  // wider than the RHS, so the bbox would otherwise have visible
  // dead space after the RHS.
  let symmetric = opts.at("symmetric", default: true)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  // Resolve answer styling once. set rules inside `if` are scoped to
  // the if-block, so we resolve to concrete values up front.
  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }
  // NOTE: do NOT set tracking on the outer text — math.frac inherits
  // the outer text settings and inserts a visible gap between digits
  // of multi-digit numerators/denominators (e.g. "10" rendered as
  // "1 0"). Apply tracking only to the whole-number where we need it.
  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  // Use Fira Math for equations (math font controls the digit
  // rendering inside math.frac — the outer text font doesn't
  // propagate there).
  show math.equation: set text(font: "Fira Math", features: ())

  let whole-v = data.at(0)
  let whole = str(whole-v)
  let n = data.at(1)
  let d = data.at(2)

  // Worked-answer values. The generator guarantees divisibility;
  // assert loudly so a bad input fails the compile instead of
  // silently truncating in `str(int(...))` on the answer-key page.
  let inter-num = whole-v * n
  assert(calc.rem(inter-num, d) == 0,
    message: "fraction-multiplication: whole×num not divisible by den")
  let final = calc.quo(inter-num, d)

  // Wrap solved-mode answer content in a scope that overrides the
  // outer text font and the math.equation show-rule's font, so a
  // configured handwriting font reaches digits inside math.frac too.
  // When `answer-font` is not set we leave equations on Fira Math
  // (the outer show-rule's default) so worksheet/story rendering is
  // unchanged.
  //
  // Handwriting fonts aren't math fonts: the default math.frac bar
  // renders too thin (or invisibly). The show rule on math.frac
  // replaces it with an explicit stack-of-digits-and-line.
  let style-answer = body => if answer-font != none {
    {
      // Bump the answer text size — handwriting fonts read smaller
      // than printed digits at the same pt size.
      set text(font: resolved-answer-font, fill: resolved-answer-color, size: 1.4em)
      show math.equation: set text(font: resolved-answer-font, features: ())
      // Custom math.frac rendering with the bar shifted to the math
      // axis via `box(baseline: …)` — without it the stack reports
      // a baseline at its bottom and the bar lands below `=`. Bar
      // length sized via `context measure()` to the wider of num /
      // denom so multi-digit numerators (e.g. "60") aren't cut off
      // by a fixed 0.9em line.
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

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // LHS for row 1: the printed `whole × n/d` expression. Tracking is
  // applied only to the whole-number (math.frac would expand the
  // numerator's digits if tracking propagated into the equation).
  let row1-lhs = {
    text(tracking: problem-tracking, whole)
    op-box
    box($#str(n)/#str(d)$)
  }

  // Solved-mode row content — always built so `equation-rows` can
  // measure the bounding rect from the solved version. `hide(...)` in
  // non-solved modes keeps the bounding rect identical between blank
  // and worked.
  let row1-rhs-solved = style-answer($#str(inter-num)/#str(d)$)
  let row2-rhs-solved = style-answer(text(tracking: problem-tracking, str(int(final))))

  let row1-rhs = if solved and not answer-only {
    row1-rhs-solved
  } else {
    hide(row1-rhs-solved)
  }
  let row2-rhs = if solved { row2-rhs-solved } else { hide(row2-rhs-solved) }

  // Row 2 LHS is blank — the `=` carries the eye over to the
  // simplified integer.
  equation-rows(
    (
      (row1-lhs, row1-rhs),
      ([], row2-rhs),
    ),
    col-width: col-width,
    symmetric: symmetric,
    debug: debug,
  )
}
