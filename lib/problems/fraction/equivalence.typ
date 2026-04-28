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
// `opts` is ignored — no operator / locale-specific symbols.
// `mode` = "blank" | "worked" | "answer-only".
#let fraction-equivalence-problem(data, mode: "blank", opts: (:), debug: false) = {
  let solved = mode != "blank"
  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  let ln = data.at(0)
  let ld = data.at(1)
  let rn = data.at(2)
  let rd = data.at(3)
  let missing = data.at(4)

  let blank-w = 2.0em
  let blank-h = 1.6em

  // Returns a box (blank or filled) when this is the missing slot,
  // or a plain string for all other slots.
  let slot(val, idx) = if missing == idx {
    box(width: blank-w, height: blank-h, stroke: 0.8pt,
      align(center + horizon, if solved { str(val) }))
  } else {
    str(val)
  }

  let a = slot(ln, 0)
  let b = slot(ld, 1)
  let c = slot(rn, 2)
  let d = slot(rd, 3)

  let content = box(
    stroke: debug-box,
    inset: (x: 0.4em, y: 0.3em),
    $#a / #b = #c / #d$,
  )

  align(center + horizon, pad(left: 0.2cm, right: 0.2cm, content))
}
