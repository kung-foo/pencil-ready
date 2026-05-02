// Fraction simplification: write a/b in its simplest form.
//
//   6           3
//   ─ = ___     ─      (reduces)
//   8           4
//
//  11          3
//   ─ = ___   2─        (improper → mixed)
//   4          4
//
//   7           7
//  ── = ___   ──        (already reduced)
//  17          17
//
// Single-row layout: LHS fraction, `=`, answer slot. The answer (when
// solved) is one of:
//   - whole number           (den divides num evenly)
//   - reduced proper fraction
//   - mixed number           (improper fraction that isn't a whole)

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-tracking, problem-features

// `data` = (numerator, denominator).
// `opts` keys (with defaults):
//   answer-font: typst font name for the solved answer. Default: none =
//     inherit problem-font / Fira Math (the equation default).
//   answer-color: color for the solved answer. Default: none = inherit.
//   align: alignment of the cell within its container. Default
//     `left + horizon`. Pass `center + horizon` for single-cell
//     thumbnail rendering.
//   slot-width: width of the answer slot. Default 3.5em — sized to
//     hold a mixed-number answer (e.g. "11 3/4") with writing room
//     above the underline in blank mode. Pass a smaller value for
//     thumbnail use where the slot's blank-line purpose doesn't apply
//     and a wide empty slot pushes the visible content off-center.
// `mode` = "blank" | "worked" | "answer-only". "worked" and "answer-only"
// are rendered identically — there's no intermediate step to suppress.
#let fraction-simplification-problem(data, mode: "blank", opts: (:), debug: false) = {
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: left + horizon)
  let slot-width-opt = opts.at("slot-width", default: 3.5em)
  let solved = mode != "blank"
  // NOTE: do NOT set tracking on outer text — math.frac inherits it and
  // breaks multi-digit numerators / denominators (e.g. "17" → "1 7").
  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  show math.equation: set text(font: "Fira Math", features: ())

  // Resolve answer styling once. set rules inside an `if` are scoped
  // to the if-block, so we resolve to concrete values up front.
  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }
  let debug-box = if debug { 1pt + red } else { none }

  let num = data.at(0)
  let den = data.at(1)

  // Reduced form: num/den = rn/rd with gcd(rn, rd) = 1.
  let g = calc.gcd(num, den)
  let rn = calc.quo(num, g)
  let rd = calc.quo(den, g)

  // Answer is one of three shapes depending on (rn, rd).
  let answer = if rd == 1 {
    // Whole number — den divides num evenly.
    $#str(rn)$
  } else if rn < rd {
    // Proper fraction (already reduced by construction).
    $#str(rn) / #str(rd)$
  } else {
    // Improper — render as mixed number.
    let w = calc.quo(rn, rd)
    let r = calc.rem(rn, rd)
    if r == 0 {
      $#str(w)$
    } else {
      // Thin space between the whole part and the fraction part so the
      // mixed number reads as a unit rather than two glued expressions.
      $#str(w) thin #str(r) / #str(rd)$
    }
  }

  let lhs = $#str(num) / #str(den)$

  // Fixed-width slot so solved and unsolved problems share the same
  // bounding box. Slot is tall enough to leave writing room *above*
  // the underline — with `horizon` alignment from the grid, the box's
  // vertical middle sits at the fraction-bar axis, so a ~2.4em slot
  // pushes the bottom stroke ~1.2em below that axis. That's enough
  // room for a kid to write a fraction or mixed number above the line.
  let slot-width = slot-width-opt
  let slot-height = 2.4em
  // Wrap solved-answer rendering in a scope that overrides both the
  // outer text font and the math.equation show-rule's font, so a
  // configured handwriting font reaches the digit glyphs inside
  // math.frac too. When `answer-font` is not set we leave the math
  // equation on Fira Math (the outer show-rule's default) so the
  // baseline rendering is unchanged for non-thumb callers.
  //
  // Handwriting fonts aren't math fonts, so the default math.frac
  // bar renders too thin (or invisibly). The show rule on math.frac
  // replaces it with an explicit stack-of-digits-and-line so the
  // fraction reads as a fraction in handwritten output too.
  let answer-rendered = if answer-font != none {
    {
      // Bump the answer text size — handwriting fonts read smaller
      // than printed digits at the same pt size.
      set text(font: resolved-answer-font, fill: resolved-answer-color, size: 1.4em)
      show math.equation: set text(font: resolved-answer-font, features: ())
      // Custom math.frac rendering for handwriting (the math font's
      // bar is too thin/invisible). The stack itself doesn't carry
      // the math-axis position through, so wrap in `box(baseline:
      // …)` to shift the stack upward by half its height — that
      // puts the drawn bar at the equation's math axis (where `=`
      // sits) rather than below it.
      show math.frac: it => box(baseline: 0.5em, stack(
        align(center, it.num),
        v(0.15em),
        line(length: 0.9em, stroke: 0.8pt + resolved-answer-color),
        v(0.15em),
        align(center, it.denom),
      ))
      answer
    }
  } else {
    answer
  }

  let right-slot = box(
    width: slot-width,
    height: slot-height,
    stroke: if solved { none } else { (bottom: 0.5pt) },
    // Solved: center the answer vertically on the fraction axis.
    // Blank: content is empty; the bottom stroke defines the line.
    align(center + horizon, if solved { answer-rendered }),
  )

  // 3-column grid keeps the `=` at a consistent axis. `horizon` centers
  // each cell on the fraction's visual middle, so `=` and the answer
  // line up with the fraction bar.
  let content = box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.3em, x: 0.2em), grid(
    columns: (auto, auto, auto),
    column-gutter: 0.3em,
    align: (right + horizon, center + horizon, left + horizon),
    lhs, sym.eq, right-slot,
  ))

  // Self-pad + self-align so the worksheet-grid stays style-agnostic.
  align(cell-align, pad(left: 0.3cm, right: 0.3cm, content))
}
