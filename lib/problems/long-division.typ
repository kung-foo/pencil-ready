// Long division bracket layout.
//
//         _____
//   4 )  375
//
// Curve drawn as a quadratic bezier, flowing into an overline.
// All dimensions computed from measured text — no scaling functions.

#import "/lib/problems/shared.typ": problem-font, problem-text-size, problem-tracking

// The bracket symbol: curve flowing into overline.
//
//   (0, 0) ---- overline ----> (overline-end, 0)
//     |
//     ) curve (bezier)
//     |
//   (0, h)  <-- bottom of curve
#let division-bracket(text-width, text-height) = {
  let h = text-height * 1.5
  let bulge = h * 0.3
  let overline-end = bulge + text-width + 12pt

  curve(
    stroke: 1.8pt,
    curve.move((0pt, h)),
    curve.quad(
      (bulge, h * 0.45),
      (0pt, 0pt),
    ),
    curve.line((overline-end, 0pt)),
  )
}

#let long-division-problem(numbers, width: 3cm, debug: false) = {
  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking)
  let debug-box = if debug { 1pt + red } else { none }
  let dividend-str = str(numbers.at(0))
  let divisor-str = str(numbers.at(1))

  box(width: width, stroke: debug-box, align(left, {
    context {
      let dividend-content = text(dividend-str)
      let m = measure(dividend-content)

      let bulge = m.height * 1.5 * 0.3
      let answer-space = m.height * 1.0
      let overshoot = m.height * 0.25

      grid(
        columns: (auto, auto),
        column-gutter: 5pt,
        align: bottom,
        pad(bottom: overshoot, text(divisor-str)),
        box({
          v(answer-space)
          pad(left: bulge + 4pt, top: 10pt, dividend-content)
          v(overshoot)
          place(bottom + left, division-bracket(m.width, m.height))
        }),
      )
    }
  }))
}
