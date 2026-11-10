// Greatest common factor by listing factors: `GCF(12, 18) = ___`
// (`SFF(12, 18)` in Norwegian), with a line per number to list its
// factors. The worked solution lists them and rings the common ones;
// the largest ring is the answer. Practice for simplifying fractions:
// the pairs look like a numerator and a denominator.
//
//   GCF(12, 18) = 6
//   12: (1), (2), (3), 4, (6), 12
//   18: (1), (2), (3), (6), 9, 18
//
// 1 is ringed too, as the instruction asks for every common factor; a
// pair whose only ring is 1 has GCF 1.

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features

// `data` = (a, b, gcd). The component lists the factors itself and
// asserts the generator's answer.
// `opts` keys (with defaults):
//   label: the function name, "GCF" or "SFF". Default "GCF".
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   work: reserve (and, when worked, print) the factor lines. Default
//     true.
//   line-width: length of the factor lines. Default auto = the longest
//     list the generator allows; thumbnails pass a shorter one.
//   slots: instead of one line per number, a short blank per factor
//     (`6: __ __ __ __`), so the student knows how many to find.
//     The blanks share the line, up to a handwriting-sized cap, so the
//     cell is the same size either way. Default false.
//   align: cell alignment. Default `left + top`.
// `mode` = "blank" | "worked" | "answer-only". "worked" lists the
// factors; "answer-only" just the answer.
#let gcf-problem(data, mode: "blank", opts: (:), debug: false) = {
  let label = opts.at("label", default: "GCF")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let show-work = opts.at("work", default: true)
  let line-width = opts.at("line-width", default: auto)
  let slots = opts.at("slots", default: false)
  let cell-align = opts.at("align", default: left + top)
  let solved = mode != "blank"
  let worked = mode == "worked"

  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let color = if answer-color != none { answer-color } else { black }
  let handwritten(body) = if answer-font != none {
    text(font: answer-font, fill: color, size: 1.4em, body)
  } else {
    text(fill: color, body)
  }

  let (a, b, g) = data
  assert(calc.gcd(a, b) == g, message: "gcf: generator answer disagrees")
  let factors(n) = range(1, n + 1).filter(d => calc.rem(n, d) == 0)
  let common = factors(g)

  // The problem line. The answer slot is wide enough for two digits,
  // with writing room above the line.
  let slot = box(
    width: 2.2em,
    height: 1.6em,
    stroke: (bottom: 0.5pt),
    align(center + horizon, if solved { handwritten(str(g)) }),
  )
  let expr(a, b) = [#label\(#str(a), #str(b)\)]
  let problem = box(inset: (top: 0.2em, bottom: 0.3em), grid(
    columns: 3,
    column-gutter: 0.3em,
    align: horizon,
    // As wide as the widest pair (`GCF(100, 100)`), so the `=` and
    // answer lines line up down the page.
    context box(width: measure(expr(100, 100)).width, expr(a, b)), sym.eq, slot,
  ))

  // A factor list, common factors ringed, at a smaller size so the
  // longest list fits the line.
  let ring(body) = box(
    stroke: 0.6pt + color,
    radius: 50%,
    inset: (x: 0.18em, y: 0.15em),
    outset: (y: 0.05em),
    body,
  )
  // `ringed`: the factors to ring — this problem's common factors, or
  // every factor when measuring the worst case.
  let mark(d, ringed) = {
    let s = handwritten(str(d))
    if d in ringed { ring(s) } else { s }
  }
  let list-of(n, ringed) = text(size: 0.8em, factors(n).map(d => mark(d, ringed)).join(handwritten[, ]))
  // One blank per factor, `w` wide, with the factor written on it when
  // worked. The blanks are separated by a gap, no comma.
  let sep-w = 0.5em
  // A grid, not inline boxes: inline boxes sit on their text baseline,
  // and a ringed factor's baseline drops its blank below the others.
  let slot-row(n, w) = grid(
    columns: factors(n).len(),
    column-gutter: sep-w,
    ..factors(n).map(d => box(
      width: w,
      height: 1.4em,
      stroke: (bottom: 0.5pt),
      align(center + bottom, pad(bottom: 0.25em, if worked { text(size: 0.8em, mark(d, common)) })),
    )),
  )
  // The line is as long as the longest list the generator allows (96
  // has twelve factors, most of them two digits), every factor ringed,
  // so all cells are the same size.
  let factor-line(n) = grid(
    columns: (2.6em, auto),
    column-gutter: 0.3em,
    align: bottom,
    pad(bottom: 0.25em, align(right, [#str(n):])),
    context {
      let w = if line-width == auto { measure(list-of(96, factors(96))).width } else { line-width }
      if slots {
        // Split the line among this number's factors (twelve at
        // most), but no wider than a comfortable blank.
        let k = factors(n).len()
        assert(w.to-absolute() >= (k - 1) * sep-w.to-absolute(), message: "gcf: line-width is shorter than the slot gutters")
        let slot-w = calc.min((2.2em).to-absolute(), (w - (k - 1) * sep-w.to-absolute()) / k)
        box(width: w, height: 1.4em, align(left + bottom, slot-row(n, slot-w)))
      } else {
        box(
          width: w,
          height: 1.4em,
          stroke: (bottom: 0.5pt),
          align(left + bottom, pad(bottom: 0.25em, if worked { list-of(n, common) })),
        )
      }
    },
  )

  let body = if show-work {
    stack(dir: ttb, spacing: 0.1em, problem, factor-line(a), factor-line(b))
  } else { problem }
  align(cell-align, pad(x: 0.47em, box(stroke: debug-box, inset: 0.2em, body)))
}
