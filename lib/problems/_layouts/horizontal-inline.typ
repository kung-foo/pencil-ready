// Horizontal single-line layout: used by multiplication drill.
//
//   7 × 3 = _____
//
// The entire problem is one non-breaking box so nothing wraps.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

// Renders the natural-width problem content. The caller (worksheet grid
// or story page) is responsible for any fill/alignment behavior.
//
// `data` = (a, b, answer). The answer is pre-computed by the generator
// so this component doesn't need to know the operation.
//
// `opts` keys (all required by callers today):
//   operator: typst content (e.g. `[#sym.times]`)
//
// `mode` = "blank" | "worked" | "answer-only". Horizontal has no worked
// steps to suppress, so "worked" and "answer-only" are equivalent here.
#let horizontal-inline-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let solved = mode != "blank"

  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let a = str(data.at(0))
  let b = str(data.at(1))
  let answer = if data.len() > 2 { str(data.at(2)) } else { "" }

  // Fixed-width slot so solved and unsolved problems share the same bounding
  // box. When solved, the answer sits on the slot; otherwise the slot shows
  // a blank line. Use bottom alignment so the answer text sits on the
  // baseline of the surrounding "a × b =" flow — center+horizon put the
  // answer above the baseline, making it look like it was floating.
  let answer-slot = box(
    width: 2em,
    height: 1em,
    stroke: if solved { none } else { (bottom: 0.5pt) },
    align(center + bottom, if solved { text(answer) }),
  )

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // Single non-breaking inline box so nothing wraps.
  let content = box(stroke: debug-box, {
    text(a)
    op-box
    text(b)
    h(0.3em)
    sym.eq
    h(0.3em)
    answer-slot
  })

  // Self-pad + self-align so the worksheet-grid doesn't have to know
  // anything style-specific about this component. 0.3cm left/right to
  // space cells apart; right+top to match the equals-column rhythm
  // across problems in a row.
  align(right + top, pad(left: 0.3cm, right: 0.3cm, content))
}
