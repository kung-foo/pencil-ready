// Long division, equation layout — the form taught in Norway (and
// Germany, Denmark, Poland, …): the problem is one line and the working
// is written under the dividend.
//
//   852 : 4 = 213
//  -8
//   ─
//   05
//   -4
//    ─
//    12
//   -12
//    ──
//     0
//
// Same `data` as `division-long-problem`, so a worksheet can switch
// layouts without touching the generator: `(dividend, divisor,
// quotient)`; the remainder is recomputed here.
//
// The working starts at the shortest leading chunk of the dividend that
// the divisor fits into (no leading-zero quotient digits), then brings
// down one digit per step. A step where the divisor doesn't fit still
// writes `-0`, so every brought-down digit gets its own row.
//
// Set at the horizontal problem size (the drills' 18pt): the problem is
// one line, and at the vertical 22pt a 4-digit problem with a remainder
// only fits one per row on A4.
//
// Bounding box: the solved footprint in every mode. Blank and
// answer-only lay out the full working and `hide` it, so a cell reserves
// exactly the room the student needs (SPEC.md → Layout rules).

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

// `opts` keys:
//   operator: division glyph. Default `[:]`.
//   answer-font / answer-color: student-work styling for solved modes
//     (quotient, remainder and working). Default: inherit.
//   reserve-remainder: size the answer slot for a worst-case
//     `<quotient> rest 8` so every cell's slot is the same width, even
//     ones without a remainder. Default false.
//   remainder-prefix: text before the remainder digit. Default "r=";
//     `document.rs` passes the locale's form (e.g. "rest\u{a0}").
//   hide-dividend: print the line without the dividend (mean worksheet:
//     the dividend is the sum still to be computed). Keeps its space.
//   top-space: extra lead-in above the line, to align with a vertical
//     stack placed beside it (see `mean.typ`). Default 0pt.
//   align: cell alignment. Default `left + top`.
//   pad-left: left padding from the grid cell edge. Default 0.5cm,
//     matching `division-long-problem`.
#let division-equation-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator", default: [:])
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let reserve-remainder = opts.at("reserve-remainder", default: false)
  let remainder-prefix = opts.at("remainder-prefix", default: "r=")
  let hide-dividend = opts.at("hide-dividend", default: false)
  let top-space = opts.at("top-space", default: 0pt)
  let cell-align = opts.at("align", default: left + top)
  let cell-pad-left = opts.at("pad-left", default: 0.5cm)
  let solved = mode != "blank"
  let show-work = mode == "worked"

  let work-font = if answer-font != none { answer-font } else { problem-font }
  let work-fill = if answer-color != none { answer-color } else { black }

  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }

  let dividend = data.at(0)
  let divisor = data.at(1)
  let quotient = if data.len() > 2 { data.at(2) } else { calc.quo(dividend, divisor) }
  let remainder = calc.rem(dividend, divisor)
  let digits = str(dividend).clusters().map(int)
  let n = digits.len()

  // Shortest leading chunk the divisor fits into; the whole dividend if
  // it never does (quotient 0).
  let start = 0
  let current = digits.at(0)
  while start < n - 1 and current < divisor {
    start += 1
    current = current * 10 + digits.at(start)
  }
  // One step per dividend digit from `start` on: what's subtracted, and
  // what's left (before the next digit is brought down).
  let steps = ()
  for i in range(start, n) {
    if i > start { current = current * 10 + digits.at(i) }
    let sub = calc.quo(current, divisor) * divisor
    steps.push((col: i, sub: sub, rem: current - sub))
    current = current - sub
  }

  // The remainder is an annotation after the quotient, set smaller —
  // same treatment as `r=N` beside the bracket — so `rest N` doesn't
  // double the line's width.
  // No tracking either: it only exists to line digits up in columns,
  // and on `rest 8` it costs ~0.5cm per cell.
  let remainder-size = 0.75em
  let remainder-style = (size: remainder-size, tracking: 0pt)
  let remainder-text = remainder-prefix + str(remainder)

  let content = box(stroke: debug-box, {
    v(top-space)
    set block(spacing: 0pt)
    context {
      // Column pitch: one digit advance including tracking, so working
      // digits sit exactly under dividend digits. Column 0 is a gutter
      // for the minus sign of a subtraction that starts at the first
      // dividend digit.
      let pitch = measure(text("00")).width - measure(text("0")).width
      let digit = d => align(right, text(str(d)))

      // Sized to the dividend, not this quotient: the quotient never has
      // more digits, every cell with the same dividend length gets the
      // same slot (a page keeps one rhythm), and a short slot doesn't
      // give away a short answer. Same idea as the bracket's overline.
      let quotient-width = measure(text("0" * n)).width
      let remainder-width = if reserve-remainder or remainder > 0 {
        measure(text(..remainder-style, " " + remainder-prefix + "8")).width
      } else { 0pt }
      let answer-slot = box(
        width: quotient-width + remainder-width,
        stroke: if solved { none } else { (bottom: 0.5pt) },
        if solved {
          set text(font: work-font, fill: work-fill)
          text(str(quotient))
          if remainder > 0 { text(..remainder-style, " " + remainder-text) }
        } else {
          // Strut so the empty slot keeps the line's height.
          hide(text("0"))
        },
      )
      let op = box(width: 1.2em, align(center, text(font: operator-font, operator)))
      let tail = {
        op
        text(str(divisor))
        h(0.3em)
        sym.eq
        h(0.3em)
        answer-slot
      }

      // Row 0: the problem line. Dividend digits in columns 1..n, the
      // rest of the line in a trailing auto column.
      let dividend-cells = digits.map(d => {
        let cell = digit(d)
        if hide-dividend and not solved { hide(cell) } else { cell }
      })
      let cells = ([],) + dividend-cells + (tail,)
      let rows = (auto,)

      // Working rows, three per step: subtraction (with its minus sign
      // one column left), a rule under it, then what's left with the
      // next digit brought down.
      let empty-row = ([],) * (n + 2)
      for (k, step) in steps.enumerate() {
        let last = k == steps.len() - 1
        let sub-digits = str(step.sub).clusters()
        let sub-first = step.col - sub-digits.len() + 1
        let rem-digits = str(step.rem).clusters()
        let rem-first = step.col - rem-digits.len() + 1

        let sub-row = empty-row
        sub-row.at(sub-first) = align(right, text(sym.minus))
        for (j, d) in sub-digits.enumerate() { sub-row.at(sub-first + j + 1) = digit(d) }

        let rule-row = empty-row
        for j in range(sub-digits.len()) {
          rule-row.at(sub-first + j + 1) = align(horizon, line(length: 100%, stroke: 0.5pt))
        }

        let rem-row = empty-row
        for (j, d) in rem-digits.enumerate() { rem-row.at(rem-first + j + 1) = digit(d) }
        if not last { rem-row.at(step.col + 2) = digit(digits.at(step.col + 1)) }

        cells += sub-row + rule-row + rem-row
        rows += (1.0em, 0.4em, 1.0em)
      }

      // Working is laid out in every mode and hidden unless worked, so
      // the box is always the solved footprint.
      let line-cells = cells.slice(0, n + 2)
      let work-cells = cells.slice(n + 2).map(c => {
        let styled = { set text(font: work-font, fill: work-fill); c }
        if show-work { styled } else { hide(styled) }
      })
      grid(
        columns: (pitch,) * (n + 1) + (auto,),
        rows: rows,
        row-gutter: 0pt,
        align: bottom,
        ..line-cells,
        ..work-cells,
      )
    }
  })

  align(cell-align, pad(left: cell-pad-left, content))
}
