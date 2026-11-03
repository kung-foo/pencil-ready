// Percent of a number: `20% of 350 = ___` (`20 % av 350` in Norwegian).
//
// One line: the expression, `=`, and an answer slot. The generator only
// picks pairs with a whole-number answer.

#import "/lib/problems/shared.typ": problem-text-size-horizontal

// `data` = (percent, base, answer).
// `opts` keys (with defaults):
//   percent-suffix: text after the percent, e.g. "%" or "\u{a0}%".
//     Default "%".
//   of: the word between percent and base ("of", "av"). Default "of".
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   align: cell alignment. Default `left + horizon`.
// `mode` = "blank" | "worked" | "answer-only"; worked and answer-only
// look the same (there's no intermediate step).
#let percent-of-problem(data, mode: "blank", opts: (:), debug: false) = {
  let pct-suffix = opts.at("percent-suffix", default: "%")
  let of-word = opts.at("of", default: "of")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: left + horizon)
  let solved = mode != "blank"

  // Math font for digits, like fraction-conversion, so the two percent
  // sheets read the same.
  set text(font: "Fira Math", size: problem-text-size-horizontal)
  let debug-box = if debug { 1pt + red } else { none }

  let (pct, base, answer) = data
  assert(pct * base == answer * 100, message: "percent-of: answer must be pct × base / 100")

  let answer-text = if answer-font != none {
    let color = if answer-color != none { answer-color } else { black }
    text(font: answer-font, fill: color, size: 1.4em, str(answer))
  } else {
    str(answer)
  }
  // Slot sized for a four-digit answer, with writing room above the
  // line (as in fraction-simplification).
  let slot = box(
    width: 3.0em,
    height: 1.6em,
    stroke: (bottom: 0.5pt),
    align(center + horizon, if solved { answer-text }),
  )
  let expr(p, b) = [#str(p)#pct-suffix #of-word #str(b)]

  // The expression sits right-aligned in a box as wide as the widest one
  // the generator allows (`100% of 1000`), so the `=` and answer lines
  // line up down a column.
  let content = context box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.3em, x: 0.2em), grid(
    columns: (measure(expr(100, 1000)).width, auto, auto),
    column-gutter: 0.3em,
    align: (right + horizon, horizon, horizon),
    expr(pct, base), sym.eq, slot,
  ))
  align(cell-align, pad(left: 0.3cm, right: 0.3cm, content))
}
