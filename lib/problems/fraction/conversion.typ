// Fraction ↔ decimal ↔ percent: one value written three ways.
//
//   3                              1
//   ─ = ____ = ____ %      ___ = 0.25 = ___ %      ___ = ___ = 12.5 %
//   4                       ─
//
// One of the three is given; the student fills in the other two. The
// answer line stays under the other two in every mode, so a worked
// example still shows which form was given. Values above 1 are written
// as mixed numbers (1 1/4 = 1.25 = 125 %).
//
// Decimal and percent arrive as scaled integers (`0.125` = 125 with 3
// places) so the component never formats a float. The generator also
// supplies the fraction in lowest terms.

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features

// `data` = (num, den, given, dec, dec-places, pct, pct-places):
//   num/den      the value as a fraction in lowest terms (may be > 1)
//   given        which form is printed: 0 fraction, 1 decimal, 2 percent
//   dec, pct     the decimal and percent, scaled by 10^places
// `opts` keys (with defaults):
//   decimal-sep: "." or ",". Default ".".
//   percent-suffix: text after the percent, e.g. "%" or "\u{a0}%"
//     (Norwegian puts a space before the sign). Default "%".
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   align: cell alignment. Default `left + horizon`.
// `mode` = "blank" | "worked" | "answer-only"; worked and answer-only
// look the same (there's no intermediate step).
#let fraction-conversion-problem(data, mode: "blank", opts: (:), debug: false) = {
  let sep = opts.at("decimal-sep", default: ".")
  let pct-suffix = opts.at("percent-suffix", default: "%")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: left + horizon)
  let solved = mode != "blank"

  // NOTE: no tracking on the outer text — math.frac inherits it.
  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }

  let (num, den, given, dec, dec-places, pct, pct-places) = data
  assert(calc.gcd(num, den) == 1, message: "fraction-conversion: fraction must be in lowest terms")
  // The three forms must agree: num/den = dec / 10^dp = pct / (100 · 10^pp).
  assert(
    num * calc.pow(10, dec-places) == dec * den and num * 100 * calc.pow(10, pct-places) == pct * den,
    message: "fraction-conversion: fraction, decimal and percent disagree",
  )

  let scaled(v, places) = {
    let unit = calc.pow(10, places)
    let whole = str(calc.quo(v, unit))
    if places == 0 {
      whole
    } else {
      let frac = str(calc.rem(v, unit))
      while frac.len() < places { frac = "0" + frac }
      whole + sep + frac
    }
  }
  // Display-size fractions: next to full-size decimals, inline math's
  // shrunken numerator and denominator read as the smaller number.
  let fraction = if num < den {
    $display(#str(num) / #str(den))$
  } else {
    // Thin space so the mixed number reads as one quantity.
    $#str(calc.quo(num, den)) thin display(#str(calc.rem(num, den)) / #str(den))$
  }
  // Decimals and percents in the math font, so their digits match the
  // fraction's (the problem font is monospaced with tracking, built for
  // column arithmetic).
  // `font: none` inherits, so a handwriting `answer-font` set by
  // `style-answer` reaches solved values.
  let num-text(s, font: "Fira Math") = if font == none { text(s) } else { text(font: font, s) }
  let values = (fraction, num-text(scaled(dec, dec-places)), num-text(scaled(pct, pct-places)))

  // Solved answers can be styled as handwriting (thumbnails); the
  // handwriting font isn't a math font, so draw the bar ourselves.
  let style-answer(body) = if answer-font != none {
    let color = if answer-color != none { answer-color } else { black }
    set text(font: answer-font, fill: color, size: 1.4em)
    show math.equation: set text(font: answer-font, features: ())
    show math.frac: it => context {
      let bar-w = calc.max(measure(it.num).width, measure(it.denom).width) + 0.2em
      box(baseline: 0.5em, stack(
        align(center, it.num),
        v(0.15em),
        line(length: bar-w, stroke: 0.8pt + color),
        v(0.15em),
        align(center, it.denom),
      ))
    }
    body
  } else {
    body
  }

  // Fixed slot widths, so every cell has the same box and the `=`
  // signs line up down a column. Sized for the widest values the
  // generator allows: "99/100", "1.875" and "187.5".
  let widths = (2.7em, 3.0em, 3.4em)
  // Tall enough to write a fraction above the line (as in
  // fraction-simplification).
  let slot-height = 2.4em
  // Solved values with a handwriting font are rebuilt without the
  // explicit math font, so the handwriting applies.
  let answer-value(i) = if answer-font != none and i > 0 {
    num-text(if i == 1 { scaled(dec, dec-places) } else { scaled(pct, pct-places) }, font: none)
  } else {
    values.at(i)
  }
  let shown(i) = if i == given { values.at(i) } else if solved { style-answer(answer-value(i)) }
  let slot(i) = box(
    width: widths.at(i),
    height: slot-height,
    stroke: if i != given { (bottom: 0.5pt) } else { none },
    align(center + horizon, shown(i)),
  )
  // The percent sign belongs to the number: right after a printed or
  // written value ("12.5%", "12,5 %"), after the line when blank. The
  // cell keeps a fixed width (line + sign) so columns stay aligned.
  let pct-cell = context {
    let sign = num-text(pct-suffix)
    let body = if 2 == given {
      box(height: slot-height, align(horizon, shown(2) + sign))
    } else {
      box(width: widths.at(2), height: slot-height, stroke: (bottom: 0.5pt),
        align(center + horizon, shown(2))) + sign
    }
    box(width: widths.at(2) + measure(sign).width, align(left + horizon, body))
  }

  let content = box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.3em, x: 0.2em), grid(
    columns: (auto, auto, auto, auto, auto),
    column-gutter: 0.3em,
    align: horizon,
    slot(0), sym.eq, slot(1), sym.eq, pct-cell,
  ))

  // Narrower side padding than the other fraction components: two of
  // these cells must fit across the page.
  align(cell-align, pad(left: 0.15cm, right: 0.15cm, content))
}
