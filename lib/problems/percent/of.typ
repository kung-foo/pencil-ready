// Percent of a number: `20% of 350 = ___` (`20 % av 350` in Norwegian).
//
// One line — the expression, `=`, and an answer slot — over blank room
// for the written multiplication (36% of 75 is 36 × 75 ÷ 100). The
// generator only picks pairs with a whole-number answer.

#import "/lib/problems/shared.typ": problem-text-size-horizontal

// `data` = (percent, base, answer, places): the answer scaled by
// 10^places (17.76 = 1776 with 2 places; whole answers have 0).
// `opts` keys (with defaults):
//   percent-suffix: text after the percent, e.g. "%" or "\u{a0}%".
//     Default "%".
//   of: the word between percent and base ("of", "av"). Default "of".
//   decimal-sep: "." or ",". Default ".".
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   work-height: blank room under the problem for working, part of the
//     cell's box. Default 5.4em, about five handwritten lines: a
//     two-digit by two-digit multiplication. Pass 0em for thumbnails.
//   align: cell alignment. Default `left + top`, so the problems in a
//     row start level and the room is below them.
// `mode` = "blank" | "worked" | "answer-only"; worked and answer-only
// look the same (there's no intermediate step).
#let percent-of-problem(data, mode: "blank", opts: (:), debug: false) = {
  let pct-suffix = opts.at("percent-suffix", default: "%")
  let of-word = opts.at("of", default: "of")
  let sep = opts.at("decimal-sep", default: ".")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let work-height = opts.at("work-height", default: 5.4em)
  let cell-align = opts.at("align", default: left + top)
  let solved = mode != "blank"

  // Math font for digits, like fraction-conversion, so the two percent
  // sheets read the same.
  set text(font: "Fira Math", size: problem-text-size-horizontal)
  let debug-box = if debug { 1pt + red } else { none }

  let (pct, base, answer, places) = data
  let unit = calc.pow(10, places)
  assert(pct * base * unit == answer * 100, message: "percent-of: answer must be pct × base / 100")
  let answer-str = if places == 0 {
    str(answer)
  } else {
    let frac = str(calc.rem(answer, unit))
    while frac.len() < places { frac = "0" + frac }
    str(calc.quo(answer, unit)) + sep + frac
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
  let content = context box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.3em, x: 0.2em), grid(
    columns: (measure(expr(100, 1000)).width, auto, auto),
    column-gutter: 0.3em,
    align: (right + horizon, horizon, horizon),
    expr(pct, base), sym.eq, slot,
  ))
  align(cell-align, pad(left: 0.3cm, right: 0.3cm, stack(content, v(work-height))))
}
