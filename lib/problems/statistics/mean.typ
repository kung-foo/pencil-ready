// Mean (average): a data line above a two-step work area.
//
//   132, 128, 141, 135
//
//      132              ______
//      128         4 ⟌ ______
//      141
//    + 135
//    ───────
//      ______
//
// The data line is the question. Below it sit two composed primitives:
// `vertical-stack-problem` for the sum and `division-long-problem` for
// the divide. Neither is re-implemented here — this component only
// arranges them and decides what paints in which mode.
//
// Mode behaviour (see SPEC.md → Bounding box, rule 1):
//
//   blank        data line and an empty "mean = ___" slot. With
//                `scaffold` on, the addition stack is pre-filled and the
//                division bracket is drawn with its divisor — the count
//                is the hint worth giving, since dividing by the wrong
//                count is the classic error; the dividend stays blank
//                because that's the sum she has to work out. With
//                `scaffold` off, both halves are hidden but still
//                reserve their full rect, so she gets exactly the space
//                the worked solution occupies.
//   worked       everything painted, including the full long-division
//                work (brought-down digits, subtractions, remainders).
//   answer-only  stack with its sum, and the quotient above the bracket,
//                but no long-division scratch work. Regardless of
//                `scaffold` — an answer key that hid the answer would be
//                useless.
//
// Because the footprint is always the solved footprint, `scaffold` does
// not change the cell size and the answer-key page lines up cell-for-cell
// with the problem page.

#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem
#import "/lib/problems/division/long.typ": division-long-problem

// `data` = (v1, …, vn, sum, mean). The value count is `data.len() - 2`,
// so a page can mix 3- and 4-value problems in the same grid.
//
// `opts` keys:
//   operator: typst content for the addition stack (e.g. `[#sym.plus]`).
//   stack-width: width of the addition half. Required.
//   div-width: width of the division half. Required.
//   div-rows: rows of work space below the bracket. Required.
//   layout: "scaffold" | "workspace". "scaffold" is the two-step cell
//     (addition stack beside a division bracket). "workspace" prints the
//     data line and reserves open space instead — used for wide or
//     decimal sets, where column addition plus single-digit long
//     division can't express the work. Default "scaffold".
//   decimal-places: dp of the incoming scaled integers. 0 = plain ints.
//   data-width / work-height: workspace-layout geometry — where the data
//     line wraps, and how much blank space sits under it.
//   scaffold: paint the working-out scaffold in blank mode — the
//     pre-filled addition stack, plus the division bracket with its
//     divisor (but not the dividend, which is the sum she has yet to
//     compute). Default true.
//   body-height: height of the working area, so the "mean = ___" slot
//     can sit at its bottom without adding to the cell. Required.
//   gap: space between the two halves. Default 0.6cm.
//   answer-font / answer-color: forwarded to both primitives so a thumb
//     can render the solution in the handwriting face.
//   align: alignment of the whole cell. Default `left + top`.
#let mean-problem(data, mode: "blank", opts: (:), debug: false) = {
  let layout = opts.at("layout", default: "scaffold")
  let dp = opts.at("decimal-places", default: 0)
  let operator = opts.at("operator", default: [+])
  let scaffold = opts.at("scaffold", default: true)
  // Scaffold-layout geometry. Unused (and not required) in the
  // open-workspace layout, which prints neither half.
  let stack-width = opts.at("stack-width", default: 0cm)
  let div-width = opts.at("div-width", default: 0cm)
  let div-rows = opts.at("div-rows", default: 1)
  let body-height = opts.at("body-height", default: 0cm)
  // Open-workspace geometry.
  let data-width = opts.at("data-width", default: 16.6cm)
  let data-size = opts.at("data-size", default: problem-text-size-horizontal)
  // Height reserved for the data line (one or two wrapped rows). Fixed
  // rather than natural so the working area below always starts at the
  // same offset — and so a wrap the Rust side didn't predict can't push
  // the bottom-anchored answer slot out of the cell.
  let data-height = opts.at("data-height", default: 1.4cm)
  let work-height = opts.at("work-height", default: 7.8cm)
  let gap = opts.at("gap", default: 0.6cm)
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: left + top)

  let solved = mode != "blank"
  let n = data.len() - 2
  let values = data.slice(0, n)
  let sum = data.at(n)
  let mean = data.at(n + 1)

  let debug-box = if debug { 1pt + red } else { none }

  // The data line always paints — it's the question. Rendered at the
  // horizontal (drill) size rather than the full problem size: it's a
  // given to be read, not a figure to be worked on, and the full size
  // would make a 4×3-digit set wider than half the page.
  //
  // No `problem-tracking` here, unlike every other numeric layout.
  // Tracking exists to column-align stacked digits; on a single
  // comma-separated line it just inflates the width, and the data line
  // is what sets this cell's width — 2pt per glyph cost ~1.2cm on a
  // four-by-three-digit set.
  //
  // Values arrive as scaled integers when `decimal-places` is set (`125`
  // is `12.5`), matching the decimal worksheets' encoding; the point is
  // re-inserted here. Same `fmt` shape as
  // `_layouts/decimal-vertical-stack.typ`.
  let fmt = (v) => {
    if dp == 0 {
      str(v)
    } else {
      let t = str(v)
      while t.clusters().len() < dp + 1 {
        t = "0" + t
      }
      let pivot = t.clusters().len() - dp
      t.slice(0, pivot) + "." + t.slice(pivot)
    }
  }

  let data-line = {
    set text(
      font: problem-font,
      size: data-size,
      features: problem-features,
    )
    values.map(v => fmt(v)).join(", ")
  }

  // Sum half: operands plus the sum as the answer row, which is what
  // `vertical-stack-problem` already expects as `(...operands, answer)`.
  let sum-stack = vertical-stack-problem(
    values + (sum,),
    mode: mode,
    opts: (
      operator: operator,
      width: stack-width,
      answer-rows: 1,
      answer-font: answer-font,
      answer-color: answer-color,
    ),
  )

  // Divide half: dividend is the sum, divisor is the value count, and
  // the quotient is the mean itself.
  let division = division-long-problem(
    (sum, n, mean),
    mode: mode,
    opts: (
      width: div-width,
      answer-rows: div-rows,
      answer-font: answer-font,
      answer-color: answer-color,
      // Blank + scaffold: draw the bracket and the divisor, withhold the
      // dividend. `hide-dividend` is ignored once solved.
      hide-dividend: true,
      // The composite owns its own spacing; rule 4 says a component
      // returns its tight rect with no internal padding.
      pad-left: 0pt,
    ),
  )

  // `hide` reserves the full bounding rect but paints nothing, so the
  // cell keeps the solved footprint in every mode. Don't swap either of
  // these for empty content — that would shrink the box and break both
  // the grid pitch and the answer-key alignment.
  let stack-slot = if solved or scaffold { sum-stack } else { hide(sum-stack) }
  let division-slot = if solved or scaffold { division } else { hide(division) }

  // Final-answer slot. Grading a page means reading one number per
  // problem, and without this the answer is the quotient perched above a
  // bracket — different height in every cell, and absent entirely when
  // the scaffold is off. Anchored to the bottom of the working area so
  // it lands at the same y in every cell of a row and costs no extra
  // height: the space below the addition stack is already reserved by
  // the taller division half.
  let answer-slot = {
    set text(
      font: problem-font,
      size: problem-text-size-horizontal,
      features: problem-features,
    )
    let filled = if solved {
      text(
        font: if answer-font != none { answer-font } else { problem-font },
        fill: if answer-color != none { answer-color } else { black },
        fmt(mean),
      )
    } else {
      []
    }
    [mean =]
    h(0.35em)
    box(
      // Wider when decimals are on: "12.5" needs more room than "12".
      width: if dp == 0 { 2em } else { 3em },
      height: 1em,
      stroke: if solved { none } else { (bottom: 0.5pt) },
      align(center + bottom, filled),
    )
  }

  let content = if layout == "workspace" {
    // Open workspace: the data line wraps inside a fixed width (eight
    // values don't fit one line), then blank space for the student to
    // lay out her own total and division however she likes, then the
    // answer slot pinned to the bottom so a whole page can be graded by
    // reading down one column.
    // Outer height pinned to exactly what `cell_size_cm` reserved. Any
    // internal slack then resolves inside the box instead of making the
    // cell taller than the grid pitch it was paginated against.
    box(
      width: data-width,
      height: data-height + work-height,
      stroke: debug-box,
      // `stack` rather than consecutive blocks: typst puts ~0.5cm of
      // paragraph spacing between sibling blocks, which is invisible in
      // isolation but pushes the bottom-anchored answer slot through the
      // cell's separator line on a full page. A `set block(spacing: 0pt)`
      // would fix that here and break the nested addition stack, whose
      // operand rows *are* blocks — so control the gap structurally
      // instead of with an inherited set rule.
      stack(
        dir: ttb,
        spacing: 0pt,
        box(width: 100%, height: data-height, data-line),
        box(height: work-height, width: 100%, {
          v(1fr)
          answer-slot
        }),
      ),
    )
  } else {
    box(
      height: data-height + body-height,
      stroke: debug-box,
      // See the workspace branch: `stack` avoids both the stray block
      // gap and the set-rule that would collapse the nested addition
      // stack's own rows.
      stack(
        dir: ttb,
        spacing: 0pt,
        box(height: data-height, data-line),
        stack(
          dir: ltr,
          spacing: gap,
          // Fixed-height left column so `v(1fr)` resolves and pins the
          // answer slot to the bottom.
          box(height: body-height, {
            stack-slot
            v(1fr)
            answer-slot
          }),
          box(division-slot),
        ),
      ),
    )
  }

  align(cell-align, content)
}
