// Vertical stacked layout: used by add, subtract, multiply, simple divide.
//
//     123
//   + 456
//   ─────
//
// Supports N operands. Only the last operand gets the operator symbol.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size, problem-tracking, problem-features, problem-line-height, problem-carry-space

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
//   decimal-places: optional array, one count per number in `data`
//     (operands, then the answer), for a decimal multiplication such as
//     48 × 0.37 = 17.76: values are scaled integers (37 with 2 places).
//     The point is drawn as a zero-width mark between two digit columns,
//     so digits stay in their columns and partial products (plain whole
//     numbers, as in the written method) line up under them. Default
//     none.
//   decimal-sep: the mark, "." (default) or ",".
//   decimal-width: width of the point as a fraction of a digit column.
//     Default 0.5. 0 hangs it in the gap between two columns (tighter,
//     but the mark crowds the digits). Above 0, every
//     row — operands, partial products, product — gets a spacer of that
//     width at the product's point position (`dp` digits from the right),
//     so the columns stay aligned through the point; only rows with a
//     point at that position draw the mark. Operands' points must sit at
//     the product's position (true when the other operand is a whole
//     number, e.g. 48 × 0.37).
//   answer-space: writing room reserved below the line. Default auto =
//     `answer-rows` lines. Pass 0em when embedding a worked stack in a
//     larger layout (percent-of) that places its own content below.
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
  let answer-space-opt = opts.at("answer-space", default: auto)
  let dps = opts.at("decimal-places", default: none)
  let decimal-sep = opts.at("decimal-sep", default: ".")
  let decimal-width = opts.at("decimal-width", default: 0.5)
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
  let dp-at(i) = if dps == none { 0 } else { dps.at(i) }
  // Digits of `n`, zero-padded so a decimal has a digit before the
  // point (5 with 2 places → "005" → 0.05).
  let digits-of(n, dp) = {
    let s = fmt(n)
    while s.clusters().len() < dp + 1 { s = "0" + s }
    s
  }
  // Decimal mark between two digit columns, centred in the tracking
  // gap. Inline (operand rows): splitting the digits into two text runs
  // drops the first run's trailing tracking, so the mark's box restores
  // that gap. In the product grid each digit sits right-aligned in its
  // cell with the gap at the cell's left, so the mark is zero-width and
  // hangs into the next cell's gap.
  let sep-text = text(tracking: 0pt, decimal-sep)
  let mark = box(width: problem-tracking, place(bottom + center, sep-text))
  let grid-mark = box(width: 0pt, place(bottom + center, dx: problem-tracking / 2, sep-text))
  // Widened point: a spacer of `decimal-width` digit columns (Fira Code
  // digits advance 0.6em, plus tracking) at the product's point position,
  // `spacer-pos` digits from the right, in every row.
  let spacer-w = decimal-width * (0.6em + problem-tracking)
  let spacer-pos = if decimal-width > 0 and dps != none { dps.last() } else { 0 }
  // A number as content: plain text, or digits with the mark at `dp`.
  let num(n, dp) = if spacer-pos > 0 {
    assert(dp == 0 or dp == spacer-pos, message: "vertical-stack: with decimal-width, operand points must sit at the product's position")
    let s = if dp > 0 { digits-of(n, dp) } else { fmt(n) }
    let len = s.clusters().len()
    let cut = calc.max(0, len - spacer-pos)
    // The spacer's box also restores the tracking the split drops.
    let spacer = box(width: problem-tracking + spacer-w, if dp > 0 { place(bottom + center, sep-text) })
    text(s.slice(0, cut)) + spacer + text(s.slice(cut))
  } else if dp == 0 {
    text(fmt(n))
  } else {
    let s = digits-of(n, dp)
    let pivot = s.clusters().len() - dp
    text(s.slice(0, pivot)) + mark + text(s.slice(pivot))
  }
  let answer = num(numbers.at(operand-count), dp-at(operand-count))
  let first = num(operands.at(0), dp-at(0))
  let rest = operands.slice(1)

  let carry-space = problem-carry-space
  let answer-space = if answer-space-opt == auto { problem-line-height * answer-rows } else { answer-space-opt }

  let content = box(
    width: width,
    stroke: debug-box,
    inset: (top: carry-space, bottom: answer-space),
    {
      set par(leading: 0.3em)
      align(right, first)
      let middle = rest.slice(0, rest.len() - 1)
      let last = rest.last()
      for (i, n) in middle.enumerate() {
        v(-0.65em)
        align(right, num(n, dp-at(i + 1)))
      }
      v(-0.65em)
      grid(
        columns: (auto, 1fr),
        column-gutter: 0.25em,
        align(left, {set text(font: operator-font); operator}), align(right, num(last, dp-at(rest.len()))),
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
          align(right, answer)
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
            let product-dp = dp-at(operand-count)
            let product-str = digits-of(a * b, product-dp)
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
            // With a widened point, every grid row gets the spacer column
            // at the point position, so digits stay in their columns.
            let widened = spacer-pos > 0
            let at-spacer = max-cols - spacer-pos
            let with-spacer(row, spacer-cell) = if widened {
              row.slice(0, at-spacer) + (spacer-cell,) + row.slice(at-spacer)
            } else {
              row
            }
            let cells = ()
            for (i, d) in b-digits.enumerate() {
              let partial-full = a * d * calc.pow(10, i)
              for c in with-spacer(row-cells(partial-full, 0), []) { cells.push(c) }
            }
            // Thin summation stroke spanning the full product width.
            for c in with-spacer(range(max-cols).map(_ => line(length: 100%, stroke: 0.5pt)), line(length: 100%, stroke: 0.5pt)) {
              cells.push(c)
            }
            // Final product row, with the decimal mark after the digit
            // before the point.
            let pivot = max-cols - product-dp
            let product-cells = product-str.clusters().enumerate().map(((i, d)) => {
              align(right, if not widened and product-dp > 0 and i == pivot - 1 { text(d) + grid-mark } else { text(d) })
            })
            // The spacer's whitespace is its width plus the next cell's
            // left tracking gap; centre the mark across both.
            // Same (top) alignment as the digit cells, so it sits on
            // their baseline.
            let product-mark = align(center, move(dx: problem-tracking / 2, sep-text))
            for c in with-spacer(product-cells, product-mark) { cells.push(c) }

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
              columns: with-spacer((digit-pitch,) * max-cols, spacer-w),
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
