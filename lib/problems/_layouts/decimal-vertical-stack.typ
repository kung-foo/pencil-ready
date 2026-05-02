// Vertical stacked layout for decimal arithmetic — add, subtract, and
// (single-digit-multiplier) multiply.
//
//      1.23
//   +  4.56
//   ───────
//
// Operand and answer values are encoded as scaled integers (e.g. 1.23 →
// 123). The component formats each one with a decimal point inserted at
// `decimal-places[i]` from the right, padding with leading zeros so a
// value like 5 with dp=2 renders as `0.05`.
//
// `data` = [...operands, answer]. The last element is rendered only when
// `mode != "blank"`.
//
// `opts` keys:
//   operator: required (e.g. `[#sym.plus]`).
//   width: cell width. Default 3.5em (wider than vertical-stack's 2.8em
//          to make room for the decimal point and trailing digits).
//   decimal-places: array of dp counts, one per number in `data` (operands
//          and answer). e.g. (2, 2, 2) for 1.23 + 4.56 = 5.79;
//          (1, 0, 1) for 2.5 × 3 = 7.5.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size, problem-tracking, problem-features, problem-line-height

#let decimal-vertical-stack-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let width = opts.at("width", default: 3.5em)
  let dp-list = opts.at("decimal-places", default: (0,) * data.len())
  // Optional answer-font / answer-color: matches the vertical-stack
  // primitive so homepage thumbs can pass `..thumb-answer-style` and
  // get the handwriting + graphite-pencil look on the filled answer.
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let solved = mode != "blank"

  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let operand-count = data.len() - 1
  let operands = data.slice(0, operand-count)
  let answer = data.at(operand-count)

  // Format an integer-encoded number as `int.frac` with `dp` decimal
  // places. Pads with leading zeros so values smaller than 10^dp still
  // render with a leading "0.": e.g. `5` with dp=2 → `"0.05"`.
  let fmt = (n, dp) => {
    if dp == 0 {
      str(n)
    } else {
      let s = str(n)
      while s.clusters().len() < dp + 1 {
        s = "0" + s
      }
      let pivot = s.clusters().len() - dp
      s.slice(0, pivot) + "." + s.slice(pivot)
    }
  }

  let first = fmt(operands.at(0), dp-list.at(0))
  let rest = operands.slice(1)
  let answer-str = fmt(answer, dp-list.at(operand-count))

  let carry-space = 0.5em
  let answer-space = problem-line-height

  align(center + top, box(
    width: width,
    stroke: debug-box,
    inset: (top: carry-space, bottom: answer-space),
    {
      set par(leading: 0.3em)
      align(right, text(first))
      let middle = rest.slice(0, rest.len() - 1)
      let last = rest.last()
      let last-dp = dp-list.at(operand-count - 1)
      for (i, n) in middle.enumerate() {
        v(-0.65em)
        align(right, text(fmt(n, dp-list.at(i + 1))))
      }
      v(-0.65em)
      grid(
        columns: (auto, 1fr),
        column-gutter: 0.25em,
        align(left, {set text(font: operator-font); operator}), align(right, text(fmt(last, last-dp))),
      )
      v(-0.9em)
      line(length: 100%, stroke: 0.8pt)
      if solved {
        v(-0.65em)
        // Shift the answer text horizontally so its decimal point
        // sits at the same x as the operand decimal points. A non-
        // monospace handwriting font (e.g. `Architects Daughter`
        // from `thumb-answer-style`) would otherwise leave the dot
        // visibly off-column when right-aligned alongside monospace
        // operands. The shift is the difference in width of the
        // post-dot suffix measured in the operand vs answer font;
        // when handwriting is narrower (the common case) we add
        // right padding to push the answer text left until the dot
        // lines up. No-op when the answer has no decimal point or
        // the fonts happen to match.
        context {
          let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
          let resolved-answer-color = if answer-color != none { answer-color } else { black }
          let dot-pos = answer-str.position(".")
          let delta = if dot-pos != none {
            let after-dot = answer-str.slice(dot-pos + 1)
            let in-op = measure(text(after-dot)).width
            let in-answer = measure(text(after-dot, font: resolved-answer-font)).width
            in-op - in-answer
          } else { 0pt }
          set text(font: resolved-answer-font, fill: resolved-answer-color)
          if delta > 0pt {
            align(right, pad(right: delta, text(answer-str)))
          } else {
            align(right, text(answer-str))
          }
        }
      }
    },
  ))
}
