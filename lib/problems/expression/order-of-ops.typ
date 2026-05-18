// Order of operations — horizontal expressions that mix +/− with ×/÷,
// optionally with one parenthesis group.
//
//   5 + 6 × 2 = ___      (3 + 4) × 5 = ___      9 × 3 − 1 + 2 = ___
//
// data layout:
//   form 0 (2-op flat):         (0, a, op1, b, op2, c, answer)
//   form 1 (3-op flat):         (1, a, op1, b, op2, c, op3, d, answer)
//   form 2 (parens around LHS): (2, a, op1, b, op2, c, answer)  ⇒ (a op1 b) op2 c
//   form 3 (parens around RHS): (3, a, op1, b, op2, c, answer)  ⇒ a op1 (b op2 c)
//
// op codes: 0=+, 1=−, 2=×, 3=÷. Plus and minus are universal; × and ÷
// are passed in via opts so the locale (US × / NO ·, US ÷ / NO :) can
// drive their glyphs without the component knowing the rule.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

#let order-of-ops-problem(data, mode: "blank", opts: (:), debug: false) = {
  let times-op = opts.at("times-op", default: [#sym.times])
  let divide-op = opts.at("divide-op", default: [#sym.div])
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: right + top)
  let solved = mode != "blank"

  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }

  let op-glyph(code) = {
    if code == 0 { [#sym.plus] }
    else if code == 1 { [#sym.minus] }
    else if code == 2 { times-op }
    else { divide-op }
  }
  let op-box(code) = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    op-glyph(code)
  }))

  // Resolve answer-text styling once and apply unconditionally inside
  // the answer slot. `set` rules inside an `if` are scoped to the
  // if-block, so we have to materialize concrete values upfront.
  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }

  // Fixed-width answer slot so blank and solved share the same
  // bounding box. The whole expression sits on one baseline, so the
  // slot uses bottom alignment for the worked answer (matches
  // horizontal-inline-problem's convention). 2em fits the
  // worst-case 3-digit answer (cap 100) at problem-text-size while
  // leaving room for the expression on a thumbnail row.
  let answer-slot(answer-text) = box(
    width: 2em,
    height: 1em,
    stroke: if solved { none } else { (bottom: 0.5pt) },
    align(center + bottom, if solved {
      text(font: resolved-answer-font, fill: resolved-answer-color, answer-text)
    }),
  )

  let form = data.at(0)
  let answer-text = str(data.last())

  // Single non-breaking inline box so the expression never wraps. Each
  // form spells out its own operand/operator sequence; the answer "= __"
  // tail is shared across forms.
  //
  // The box uses default text-edges (cap-height/baseline) so the
  // digits and operators sit at their natural inline positions —
  // operators are math-axis-centered against the digit cap-band, the
  // way mult-drill renders them. Inset adds a small top/bottom
  // margin so digits aren't flush to the stroke and so the parens'
  // descender (form 2) stays inside the bounding rect.
  let content = box(
    stroke: debug-box,
    inset: (top: 0.15em, bottom: 0.25em),
    {
    if form == 0 {
      text(str(data.at(1)))
      op-box(data.at(2))
      text(str(data.at(3)))
      op-box(data.at(4))
      text(str(data.at(5)))
    } else if form == 1 {
      text(str(data.at(1)))
      op-box(data.at(2))
      text(str(data.at(3)))
      op-box(data.at(4))
      text(str(data.at(5)))
      op-box(data.at(6))
      text(str(data.at(7)))
    } else if form == 2 {
      text("(")
      text(str(data.at(1)))
      op-box(data.at(2))
      text(str(data.at(3)))
      text(")")
      op-box(data.at(4))
      text(str(data.at(5)))
    } else if form == 3 {
      text(str(data.at(1)))
      op-box(data.at(2))
      text("(")
      text(str(data.at(3)))
      op-box(data.at(4))
      text(str(data.at(5)))
      text(")")
    }
    h(0.3em)
    sym.eq
    h(0.3em)
    answer-slot(answer-text)
    },
  )

  // Self-pad + self-align so the worksheet-grid doesn't need to know
  // anything style-specific about this component. The pad is tight
  // (0.15cm vs the 0.3cm convention) so the widest L3 expression
  // — `(3 + 4) × 5 = 35` — fits a single line in the homepage
  // thumbnail without wrapping.
  align(cell-align, pad(left: 0.15cm, right: 0.15cm, content))
}
