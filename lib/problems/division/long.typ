// Long division bracket layout.
//
//         _____
//   4 )  375
//
// Curve drawn as a quadratic bezier, flowing into an overline.
// All dimensions computed from measured text — no scaling functions.

#import "/lib/problems/shared.typ": problem-font, problem-text-size, problem-tracking, problem-features, problem-line-height

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
  // Extend overline past the dividend by ~half a cap height for breathing room.
  let overline-end = bulge + text-width + text-height * 0.7

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

// `opts` keys:
//   width: cell width (e.g. 3.9em). Required.
//   answer-rows: rows of solve space below the bracket (typically 2×
//     the number of dividend digits). Required.
//   answer-font: typst font name for solved-mode student-work (quotient,
//     sub digits, remainders, brought-down digits). Default: none =
//     inherit problem-font.
//   answer-color: color for solved-mode student-work. Default: none =
//     inherit (black).
//   align: alignment of the cell within its container. Default
//     `left + top` — the worksheet grid relies on this so brackets
//     line up across columns. Pass e.g. `center + horizon` for a
//     single-cell rendering (thumbnails) where you want the problem
//     anchored to the page center.
//   pad-left: left padding around the cell — breathing room from
//     the worksheet-grid cell edge. Default 0.5cm. Pass 0pt for
//     single-cell thumbnails so the content centers symmetrically.
//
// `mode` = "blank" | "worked" | "answer-only". "answer-only" renders
// just the quotient above the bar and skips the work rows.
#let division-long-problem(data, mode: "blank", opts: (:), debug: false) = {
  // Required keys — a silent `answer-rows: 0` would render zero work
  // space and be easy to miss in review, so fail loudly if a caller
  // forgets.
  let width = opts.at("width")
  let answer-rows = opts.at("answer-rows")
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let cell-align = opts.at("align", default: left + top)
  let cell-pad-left = opts.at("pad-left", default: 0.5cm)
  // When the worksheet allows remainders, reserve overline width for
  // a worst-case `dddd r N` answer in every cell — even cells whose
  // actual answer has no remainder. Without this, individual cells
  // would size their overline to their own quotient, so the row
  // would have visually uneven brackets (some short, some long).
  let reserve-remainder = opts.at("reserve-remainder", default: false)
  let solved = mode != "blank"
  let answer-only = mode == "answer-only"

  // Resolve answer styling once. set rules inside an `if` are scoped
  // to the if-block, so we resolve to concrete values and apply
  // unconditionally where the work is rendered.
  let resolved-answer-font = if answer-font != none { answer-font } else { problem-font }
  let resolved-answer-color = if answer-color != none { answer-color } else { black }

  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let dividend-str = str(data.at(0))
  let divisor-str = str(data.at(1))
  // Quotient and remainder are rendered as separate slots: the
  // quotient sits above the dividend (digits column-aligned with the
  // dividend digits, just like a no-remainder problem), and `r=N` is
  // a smaller annotation to the right of the bracket overline. This
  // keeps the overline length identical to the non-remainder case
  // and preserves the standard "digits above digits" alignment.
  let quotient-str = if data.len() > 2 { str(data.at(2)) } else { "" }
  let remainder-val = if data.len() > 2 {
    calc.rem(data.at(0), data.at(1))
  } else { 0 }
  let has-remainder = remainder-val > 0
  let remainder-str = if has-remainder { "r=" + str(remainder-val) } else { "" }
  // Smaller font for the `r=N` annotation so it reads as auxiliary
  // info, not part of the main quotient.
  let remainder-size-factor = 0.75

  // 1.3em per row ≈ one typeset line at this size.
  let work-space = 1.3em * answer-rows

  let content = box(width: width, stroke: debug-box, align(left, {
    // Suppress the default paragraph gap between sibling blocks inside
    // this problem. Otherwise the quotient block sits ~1em above where
    // it should visually (the block spacing stacks between quotient and
    // dividend).
    set block(spacing: 0pt)
    context {
      let dividend-content = text(dividend-str)
      let m = measure(dividend-content)

      let bulge = m.height * 1.5 * 0.3
      let answer-space = m.height * 1.0
      let overshoot = m.height * 0.25
      let column-gutter = 0.25em

      // Width of the `r=N` annotation slot to the right of the
      // overline. Worst case is `r=N` for any single-digit N (divisor
      // is 1-9, so remainder is 0-8, all single-digit at the same
      // monospace width). When `reserve-remainder` is on, every cell
      // reserves this slot so the bounding box (and the bracket
      // position relative to the cell's right edge) stays uniform
      // across blank, solved, and answer-only modes — `r=N` just
      // doesn't paint when there's no remainder.
      let r-slot-width = if reserve-remainder {
        measure(text("r=8", size: problem-text-size * remainder-size-factor)).width + 0.5em
      } else {
        0pt
      }
      // Overline length stays the dividend width — same as the
      // no-remainder case. The bracket curve + overline footprint
      // doesn't grow when `r=N` is present.
      let content-width = m.width

      // The bracket curve is `place`d, so it doesn't contribute to the
      // dividend-area-box's natural width. Without an explicit width,
      // typst's bounding box ends at the dividend's right edge and the
      // overline sticks out into "outside" space — invisible to layout
      // but visible on the page. That's fine for the worksheet (each
      // cell is fixed-width and left-aligned) but breaks centering for
      // single-cell renderings like the homepage thumbs. Pinning the
      // box width to `division-bracket`'s `overline-end` (bulge +
      // content + 0.7em) plus the reserved `r=N` slot means the
      // bounding rect truly bounds what you see.
      let dividend-area-width = bulge + content-width + m.height * 0.7 + r-slot-width
      grid(
        columns: (auto, auto),
        column-gutter: column-gutter,
        align: bottom,
        pad(bottom: overshoot, text(divisor-str)),
        box(width: dividend-area-width, {
          // Space above the overline holds the quotient when solved, or
          // stays empty otherwise. v() preserves the exact pre-solve-first
          // layout for unsolved problems.
          //
          // Solved variant: use cap-height/baseline edges to trim the
          // quotient's text bounding box — removes the ~0.3em of extra
          // leading that was producing a big gap above the overline.
          // Width extends to `bulge + 0.2em + m.width` so the quotient
          // right edge lines up with the dividend's right edge.
          if solved {
            // Two-column layout for the quotient row:
            //   col 1 — pure quotient, right-aligned to dividend's
            //           right edge so digits stack above dividend.
            //   col 2 — `r=N` (smaller) padded a bit past the
            //           dividend's right edge, only painted when
            //           there's a non-zero remainder.
            grid(
              columns: (bulge + 0.2em + content-width, r-slot-width),
              align(right + bottom, text(
                quotient-str,
                font: resolved-answer-font,
                fill: resolved-answer-color,
                top-edge: "cap-height",
                bottom-edge: "baseline",
              )),
              if has-remainder {
                align(left + bottom, pad(left: 0.4em, text(
                  remainder-str,
                  size: problem-text-size * remainder-size-factor,
                  font: resolved-answer-font,
                  fill: resolved-answer-color,
                  top-edge: "cap-height",
                  bottom-edge: "baseline",
                )))
              } else {
                []
              },
            )
          } else {
            v(answer-space)
          }
          pad(left: bulge + 0.2em, top: 0.45em, dividend-content)
          v(overshoot)
          place(bottom + left, division-bracket(content-width, m.height))
        }),
      )

      // Work rows below the bracket, aligned with the dividend columns.
      // When unsolved: empty space (v(work-space)) preserves the pre-
      // solve-first layout so baselines don't shift. Answer-only: the
      // quotient is already rendered above the bracket, so skip the work
      // and reserve the same empty space as an unsolved problem.
      if solved and not answer-only {
        // Column pitch must exactly match the dividend's digit advance so
        // work digits sit directly below the corresponding dividend digit.
        let digit-pitch = measure(text("00")).width - measure(text("0")).width

        // Walk the long-division algorithm: at each step we have a
        // "current" value (prev remainder × 10 + next dividend digit).
        // The sub = (current / divisor) × divisor; the rem = current - sub.
        // The rem carries into the next step paired with the brought-down
        // digit, forming the next current.
        let dividend-digits = dividend-str.clusters().map(c => int(c))
        let divisor = data.at(1)
        let n = dividend-digits.len()

        let steps = ()
        let current = 0
        for d in dividend-digits {
          current = current * 10 + d
          let q = calc.quo(current, divisor)
          let sub = q * divisor
          let rem = current - sub
          steps.push((sub: sub, rem: rem))
          current = rem
        }

        // Each step contributes 3 rows: the subtraction result, a thin
        // line indicating the subtraction operation (spanning just the
        // sub's columns), then the remainder (paired with the brought-
        // down next digit, except on the final step where it stands
        // alone as the overall remainder).
        let cells = ()
        for (i, step) in steps.enumerate() {
          let sub-digits = str(step.sub).clusters()
          let sub-start = i - sub-digits.len() + 1
          let rem-digits = str(step.rem).clusters()
          let rem-start = i - rem-digits.len() + 1

          // Row A: sub value
          for col in range(n) {
            if col >= sub-start and col <= i {
              cells.push(align(right, text(sub-digits.at(col - sub-start))))
            } else {
              cells.push([])
            }
          }
          // Row B: subtraction line — thin stroke in the sub's columns.
          for col in range(n) {
            if col >= sub-start and col <= i {
              cells.push(line(length: 100%, stroke: 0.5pt))
            } else {
              cells.push([])
            }
          }
          // Row C: remainder + brought-down next dividend digit.
          for col in range(n) {
            if col >= rem-start and col <= i {
              cells.push(align(right, text(rem-digits.at(col - rem-start))))
            } else if col == i + 1 and i + 1 < n {
              cells.push(align(right, text(str(dividend-digits.at(i + 1)))))
            } else {
              cells.push([])
            }
          }
        }

        // Horizontal offset to align the work with the dividend.
        // The dividend's own pad is `bulge + 0.2em`, but the 0.2em is
        // breathing room between the bracket curve and the dividend
        // digits — that's not part of the digit column. Work digits
        // below only need `bulge` so they sit directly beneath the
        // dividend columns.
        // block(spacing: 0pt) suppresses the paragraph gap typst would
        // otherwise insert between the divisor|dividend grid above and
        // the work grid below — that gap was pushing the work down into
        // the next worksheet row.
        let divisor-width = measure(text(divisor-str)).width
        // 1.0em per work row is tighter than the shared problem-line-height
        // (1.3em). The cells only hold a single digit each, so we can
        // afford to compress — and the cell allocated by the worksheet
        // grid is only wide enough for the unsolved reading of
        // `answer-rows = 2 × dividend-digits`, so the full-height work
        // would overflow into the next worksheet row.
        let work-row-height = 1.0em
        // Row heights: per step, (sub-row, line-row, rem-row) where
        // sub-row and rem-row are full digit height and line-row holds
        // just the subtraction stroke. 0.4em leaves the stroke visually
        // centred with a clear gap to the digits on either side.
        let line-row-height = 0.4em
        let row-heights = ()
        for _ in range(steps.len()) {
          row-heights.push(work-row-height)
          row-heights.push(line-row-height)
          row-heights.push(work-row-height)
        }

        // Alignment: dividend sits at offset (bulge + 0.2em) inside the
        // inner box; our grid column width (digit-pitch) equals digit
        // advance + tracking, which is one `tracking` wider than the
        // dividend's actual inter-digit pitch. Subtracting tracking from
        // the pad-left puts each grid column's right edge directly under
        // the corresponding dividend digit's right edge.
        block(spacing: 0pt, pad(
          left: divisor-width + column-gutter + bulge + 0.2em - problem-tracking,
          {
            // Style the work as student handwriting (when caller passed
            // answer-font/answer-color). Subtraction lines aren't text,
            // so they're unaffected by `set text` and stay as printed
            // 0.5pt strokes.
            set text(font: resolved-answer-font, fill: resolved-answer-color)
            grid(
              columns: (digit-pitch,) * n,
              rows: row-heights,
              row-gutter: 0pt,
              ..cells,
            )
          },
        ))
      } else {
        // Empty work space — pre-solve-first baseline layout.
        v(work-space)
      }
    }
  }))

  // Self-pad + self-align so the worksheet-grid doesn't have to know
  // anything style-specific about this component. 0.5cm left pad for
  // breathing room from the cell edge; default `left + top` because
  // the bracket glyph is anchored to the dividend's left edge in the
  // multi-column worksheet grid.
  align(cell-align, pad(left: cell-pad-left, content))
}
