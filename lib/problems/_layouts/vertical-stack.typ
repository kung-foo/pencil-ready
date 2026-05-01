// Vertical stacked layout: used by add, subtract, multiply, simple divide.
//
//     123
//   + 456
//   ─────
//
// Supports N operands. Only the last operand gets the operator symbol.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size, problem-tracking, problem-features, problem-line-height

// `data` = [...operands, answer]. The last element is the final answer
// (sum/difference/product/quotient); it's only rendered when mode != "blank".
//
// `opts` keys (with defaults):
//   operator: typst content (e.g. `[#sym.plus]`). Required.
//   width: cell width (default 2.8em).
//   answer-rows: rows of writing space below the line. 1 for add/subtract
//     (single answer line), ~partial products + 1 for multiply. Default 1.
//   pad-width: when > 0, left-pad operand numbers with "0" up to this many
//     characters. Used by binary addition. Default 0.
//   answer-font: typst font name for the final answer (and partial products).
//     Default: none = inherit `problem-font`. Pass e.g. "Architects Daughter"
//     to make the answer read as if hand-written by the student.
//   answer-color: color for the final answer text. Default: none = inherit
//     (black). Pass e.g. rgb("#4a4a4a") for a graphite-pencil look.
//
// `mode` = "blank" | "worked" | "answer-only". "answer-only" skips worked
// steps (partial products) and renders just the final answer — used by
// answer-key pages.
#let vertical-stack-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator")
  let width = opts.at("width", default: 2.8em)
  let answer-rows = opts.at("answer-rows", default: 1)
  let pad-width = opts.at("pad-width", default: 0)
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  let numbers = data
  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let operand-count = numbers.len() - 1
  let operands = numbers.slice(0, operand-count)
  // Render a value, optionally left-padded with zeros to `pad-width` chars.
  let fmt = (n) => {
    let s = str(n)
    while s.clusters().len() < pad-width {
      s = "0" + s
    }
    s
  }
  let answer = fmt(numbers.at(operand-count))
  let first = fmt(operands.at(0))
  let rest = operands.slice(1)

  let carry-space = 0.5em
  let answer-space = problem-line-height * answer-rows

  let content = box(
    width: width,
    stroke: debug-box,
    inset: (top: carry-space, bottom: answer-space),
    {
      set par(leading: 0.3em)
      align(right, text(first))
      let middle = rest.slice(0, rest.len() - 1)
      let last = rest.last()
      for n in middle {
        v(-0.65em)
        align(right, text(fmt(n)))
      }
      v(-0.65em)
      grid(
        columns: (auto, 1fr),
        column-gutter: 0.25em,
        align(left, {set text(font: operator-font); operator}), align(right, text(fmt(last))),
      )
      v(-0.9em)
      line(length: 100%, stroke: 0.8pt)
      if solved {
        v(-0.65em)
        // Optional font/color overrides for the answer (e.g. a handwriting
        // face + graphite color to make the answer read as if filled in by
        // the student). Each falls back to the inherited setting when not
        // set explicitly. Because typst `set` rules are scoped to the
        // enclosing block, we resolve to a concrete value first and `set`
        // unconditionally — wrapping in `if` would scope the `set` to the
        // if-block alone.
        let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
        let resolved-answer-color = if answer-color != none { answer-color } else { black }
        set text(font: resolved-answer-font, fill: resolved-answer-color)
        if answer-rows == 1 or answer-only {
          // Single-row answers (add, subtract, simple-divide, 1-digit
          // multiply) OR answer-key mode for multi-digit multiply:
          // render just the final numeric answer, no partial products.
          align(right, text(answer))
        } else {
          // Multi-digit multiplication: N partial products + final sum.
          //
          // We render as a fixed-width column grid so each partial's
          // digits land in the same column as the corresponding digit
          // of the multiplier above. Column width is measured from a
          // single digit plus the problem-tracking — that matches the
          // operand row's pitch exactly.
          context {
            let a = numbers.at(0)
            let b = numbers.at(1)
            let product-str = str(a * b)
            let max-cols = product-str.clusters().len()

            // Pitch between consecutive digits in the operand rows.
            // Using (measure "00" − measure "0") gives the advance of the
            // second digit, i.e. glyph width + tracking, which matches
            // exactly how operand digits are laid out above the line.
            let digit-pitch = measure(text("00")).width - measure(text("0")).width

            // Build b's digits LSB-first so partial i corresponds to
            // column shift i.
            let b-digits = ()
            let tmp = b
            while tmp > 0 {
              b-digits.push(calc.rem(tmp, 10))
              tmp = calc.quo(tmp, 10)
            }

            // Turn one number + a right-shift into an array of max-cols
            // cells — MSB-first, empty cells pad the left and right.
            let row-cells(value, shift) = {
              let digits = str(value).clusters()
              let len = digits.len()
              let right-empty = shift
              let left-empty = max-cols - shift - len
              let cells = ()
              for _ in range(left-empty) { cells.push([]) }
              for d in digits { cells.push(align(right, text(d))) }
              for _ in range(right-empty) { cells.push([]) }
              cells
            }

            // Partials include the trailing zeros (e.g. `37 × 40 = 1480`
            // rather than `148` in the tens column with an empty ones
            // column). That makes the column-by-column addition of the
            // final sum more explicit for a student learning the method.
            let cells = ()
            for (i, d) in b-digits.enumerate() {
              let partial-full = a * d * calc.pow(10, i)
              for c in row-cells(partial-full, 0) { cells.push(c) }
            }
            // Thin summation stroke spanning the full product width.
            for _ in range(max-cols) {
              cells.push(line(length: 100%, stroke: 0.5pt))
            }
            // Final product row.
            for c in row-cells(a * b, 0) { cells.push(c) }

            // Row heights: partials + final at 1em each, thin stroke row
            // between the last partial and the final product.
            let work-row-height = 1.0em
            let line-row-height = 0.4em
            let row-heights = ()
            for _ in range(b-digits.len()) {
              row-heights.push(work-row-height)
            }
            row-heights.push(line-row-height)
            row-heights.push(work-row-height)

            align(right, grid(
              columns: (digit-pitch,) * max-cols,
              rows: row-heights,
              row-gutter: 0pt,
              ..cells,
            ))
          }
        }
      }
    },
  )

  // Self-align so the worksheet-grid doesn't have to know anything
  // style-specific about this component. No pad — the box's fixed
  // `width` already defines the cell's horizontal footprint, and the
  // grid's 1fr columns space cells evenly.
  align(center + top, content)
}
