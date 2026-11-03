// Percent of a number: `20% of 350 = ___` (`20 % av 350` in Norwegian).
//
// One line — the expression, `=`, and an answer slot — over room for the
// working: the base times the percent written as a decimal, as a column
// multiplication (the shared `vertical-stack` layout with decimal
// places): multiply as whole numbers, partial products and all, then
// place the point in the product. A worked example prints that working;
// blank cells reserve the space it takes, so the room matches what
// solving needs.
//
//   37% of 48 = 17.76
//
//       48
//   × 0.37
//   ──────
//      336
//     1440
//   ──────
//    17.76

#import "/lib/problems/shared.typ": problem-text-size-horizontal, problem-text-size
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem

// `data` = (percent, base, answer, places): the answer scaled by
// 10^places (17.76 = 1776 with 2 places; whole answers have 0).
// `opts` keys (with defaults):
//   percent-suffix: text after the percent, e.g. "%" or "\u{a0}%".
//     Default "%".
//   of: the word between percent and base ("of", "av"). Default "of".
//   decimal-sep: "." or ",". Default ".".
//   operator: multiply sign for the working, per locale. Default
//     `[#sym.times]`.
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   work: reserve (and, when worked, print) the working. Default true;
//     pass false for thumbnails.
//   align: cell alignment. Default `left + top`, so the problems in a
//     row start level and the room is below them.
// `mode` = "blank" | "worked" | "answer-only". "worked" prints the
// working; "answer-only" just the answer.
#let percent-of-problem(data, mode: "blank", opts: (:), debug: false) = {
  let pct-suffix = opts.at("percent-suffix", default: "%")
  let of-word = opts.at("of", default: "of")
  let sep = opts.at("decimal-sep", default: ".")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let show-work = opts.at("work", default: true)
  let mul-op = opts.at("operator", default: [#sym.times])
  let cell-align = opts.at("align", default: left + top)
  let solved = mode != "blank"

  // Math font for digits, like fraction-conversion, so the two percent
  // sheets read the same.
  set text(font: "Fira Math", size: problem-text-size-horizontal)
  let debug-box = if debug { 1pt + red } else { none }

  let (pct, base, answer, places) = data
  let unit = calc.pow(10, places)
  assert(pct * base * unit == answer * 100, message: "percent-of: answer must be pct × base / 100")
  let fmt(v, places) = if places == 0 {
    str(v)
  } else {
    let u = calc.pow(10, places)
    let frac = str(calc.rem(v, u))
    while frac.len() < places { frac = "0" + frac }
    str(calc.quo(v, u)) + sep + frac
  }
  let answer-str = fmt(answer, places)

  // The working: `base × percent-as-decimal` (37% → 0.37, 50% → 0.5,
  // 5% → 0.05) as a column multiplication — partial products when the
  // decimal has two digits — with the point placed in the product.
  let working(p, b, mode) = {
    let (m, places) = (p, 2)
    while places > 0 and calc.rem(m, 10) == 0 {
      m = calc.quo(m, 10)
      places -= 1
    }
    let digits = str(m).len()
    let product = vertical-stack-problem(
      (b, m, m * b),
      mode: mode,
      opts: (
        operator: mul-op,
        width: 3.8em,
        // One row per partial product plus their sum.
        answer-rows: if digits > 1 { digits + 1 } else { 1 },
        decimal-places: (0, places, places),
        decimal-sep: sep,
        // The stack's own blank writing room is replaced by this
        // layout's: the room below is reserved for the whole working.
        answer-space: 0em,
        answer-font: answer-font,
        answer-color: answer-color,
      ),
    )
    // vertical-stack sets its own (column-arithmetic) text size; scale
    // it to this problem's size so the working matches the line above.
    scale(problem-text-size-horizontal / problem-text-size * 100%, reflow: true, origin: left + top, product)
  }
  // Room for the widest working the generator allows (999 × 0.99, two
  // partial products), so every cell is the same size.
  let work-room = context {
    let worst = measure(working(99, 999, "worked"))
    box(
      width: worst.width,
      height: worst.height,
      if solved and mode != "answer-only" { working(pct, base, "worked") },
    )
  }

  let answer-text = if answer-font != none {
    let color = if answer-color != none { answer-color } else { black }
    text(font: answer-font, fill: color, size: 1.4em, answer-str)
  } else {
    answer-str
  }
  // Slot sized for the widest answer ("989.01"), with writing room
  // above the line (as in fraction-simplification).
  let slot = box(
    width: 3.6em,
    height: 1.6em,
    stroke: (bottom: 0.5pt),
    align(center + horizon, if solved { answer-text }),
  )
  let expr(p, b) = [#str(p)#pct-suffix #of-word #str(b)]

  // The expression sits right-aligned in a box as wide as the widest one
  // the generator allows (`100% of 1000`), so the `=` and answer lines
  // line up down a column.
  let content = context box(inset: (top: 0.2em, bottom: 0.3em, x: 0.2em), grid(
    columns: (measure(expr(100, 1000)).width, auto, auto),
    column-gutter: 0.3em,
    align: (right + horizon, horizon, horizon),
    expr(pct, base), sym.eq, slot,
  ))
  let body = if show-work { stack(dir: ttb, spacing: 0.2em, content, pad(left: 1em, work-room)) } else { content }
  // The debug box wraps the whole cell, working included, so it shows
  // the real bounding rect.
  align(cell-align, pad(left: 0.3cm, right: 0.3cm, box(stroke: debug-box, inset: 0.2em, body)))
}
