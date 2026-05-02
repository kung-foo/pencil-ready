// Equivalent fractions: find the missing number in  a/b = c/d.
//
//   1     2         4      □         □     4
//   ─  =  ─         ─  =  ──         ─  =  ─
//   3     □         8     16         2     4
//
// Single-row layout: two stacked fractions separated by `=`. One of the
// four slots (left-num, left-den, right-num, right-den) is an outlined
// box for the student to fill in. In solved mode the answer is written
// inside the box.
//
// Data: (left-num, left-den, right-num, right-den, missing)
//   missing: 0 = left-num, 1 = left-den, 2 = right-num, 3 = right-den

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features

// `data` = (ln, ld, rn, rd, missing).
// `opts` keys (with defaults):
//   answer-font: typst font for the answer written into the box.
//     Default: none = inherit problem-font.
//   answer-color: color for the answer. Default: none = inherit.
//   align: alignment of the cell in its container. Default
//     `center + horizon` (already symmetric).
// `mode` = "blank" | "worked" | "answer-only".
#let fraction-equivalence-problem(data, mode: "blank", opts: (:), debug: false) = {
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: center + horizon)
  let solved = mode != "blank"
  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }

  let ln = data.at(0)
  let ld = data.at(1)
  let rn = data.at(2)
  let rd = data.at(3)
  let missing = data.at(4)

  let blank-w = 2.0em
  let blank-h = 1.6em

  // Returns a box (blank or filled) when this is the missing slot,
  // or a plain string for all other slots. Solved value is styled
  // via the answer-font/answer-color opts when set; without an
  // explicit override we render plain to keep baseline rendering
  // (and the visual-regression stories) unchanged. Handwritten
  // answers get a size bump (1.4em) so they read at roughly the
  // same x-height as the printed digits — handwriting fonts are
  // visually smaller than the equivalent printed glyph at the same
  // pt size.
  let slot(val, idx) = if missing == idx {
    box(width: blank-w, height: blank-h, stroke: 0.8pt,
      align(center + horizon, if solved {
        if answer-font != none {
          text(
            font: resolved-answer-font,
            fill: resolved-answer-color,
            size: 1.4em,
            str(val),
          )
        } else {
          str(val)
        }
      }))
  } else {
    str(val)
  }

  let a = slot(ln, 0)
  let b = slot(ld, 1)
  let c = slot(rn, 2)
  let d = slot(rd, 3)

  // Explicit height so the bounding rect matches the visible extent
  // — typst's math layout reports a frame smaller than the slot's
  // 1.6em visible height, which makes any caller's `align(center +
  // horizon)` center an undersized box. 2.4em = slot extent above
  // the bar (~1.5em) + denom below (~1em) less a touch of overlap
  // since the slot's bottom and the bar's top share a hair of space.
  // Inner `align(horizon)` sits the equation at the box's vertical
  // center so a centered outer box has the equation at page center.
  let content = box(
    stroke: debug-box,
    inset: (x: 0.4em, y: 0.3em),
    height: 2.4em,
    align(horizon, $#a / #b = #c / #d$),
  )

  align(cell-align, pad(left: 0.2cm, right: 0.2cm, content))
}
