// Open Graph / social share image for pencilready.com
// Render: `typst compile assets/og-image.typ frontend/astro/public/og-image.png --ppi 72`
// Target: 1200 x 630 px.

#set page(
  width: 1200pt,
  height: 630pt,
  margin: 0pt,
  fill: rgb("#fbfdff"),
)

// Graph-paper grid. Pale primary-blue at low alpha, 24pt cells — matches
// the live site's `[data-theme="graph-paper"]` page background.
#let grid-line-color = rgb(37, 99, 235, 20)
#let cell = 24pt

#place(top + left, {
  // Vertical lines
  for i in range(0, 51) {
    place(top + left, dx: i * cell, line(
      start: (0pt, 0pt),
      end: (0pt, 630pt),
      stroke: 1pt + grid-line-color,
    ))
  }
  // Horizontal lines
  for j in range(0, 27) {
    place(top + left, dy: j * cell, line(
      start: (0pt, 0pt),
      end: (1200pt, 0pt),
      stroke: 1pt + grid-line-color,
    ))
  }
})

// Pencil underline — mirrors lib/header.typ brand spec (288:12 viewBox).
// Stretches horizontally; vertical bars keep fixed widths proportionally.
#let pencil-underline(width: 600pt, height: 24pt) = {
  let scale = width / 288pt
  box(width: width, height: height, {
    place(top + left, rect(width: 12pt * scale, height: height, fill: rgb("#E88E8A"), stroke: none))
    place(top + left, dx: 12pt * scale, rect(width: 6pt * scale, height: height, fill: rgb("#3E8948"), stroke: none))
    place(top + left, dx: 18pt * scale, rect(width: 6pt * scale, height: height, fill: rgb("#FDB600"), stroke: none))
    place(top + left, dx: 24pt * scale, rect(width: 6pt * scale, height: height, fill: rgb("#3E8948"), stroke: none))
    place(top + left, dx: 30pt * scale, rect(width: 252pt * scale, height: height, fill: rgb("#FDB600"), stroke: none))
    place(top + left, dx: 282pt * scale, rect(width: 6pt * scale, height: height, fill: rgb("#1A1A1A"), stroke: none))
  })
}

// Left-aligned hero. Placed with a generous left margin and vertically
// centered; the right side of the canvas is reserved for the worksheet
// mockup below.
#place(left + horizon, dx: 72pt, block(width: 720pt, {
  // Hero: wordmark with the pencil bar layered BEHIND the text, stretched
  // to the exact width of "Pencil Ready". `measure` gives us the rendered
  // width so the bar always matches the text no matter the font metrics.
  context {
    let title = text(
      font: "Crimson Text",
      weight: "semibold",
      size: 130pt,
      fill: rgb("#1e293b"),
    )[Pencil Ready]
    let m = measure(title)
    block(width: m.width, height: m.height, {
      // Draw pencil first → lower in paint order → behind the glyphs.
      // Pencil top at the bottom of the measure box so descenders
      // ("y" in Ready) cross the top of the bar.
      place(top + left, dy: m.height, pencil-underline(
        width: m.width,
        height: 20pt,
      ))
      place(top + left, title)
    })
  }
  v(48pt)
  text(
    font: "Roboto Slab",
    size: 34pt,
    fill: rgb("#475569"),
  )[Free printable math worksheets]
}))

// Fake worksheet on the right — A4 portrait (210:297). Uses abstract
// shapes for the problem grid so we're not baking real content into the
// share card. Rotated slightly for visual interest.
#let worksheet-preview(height: 520pt) = {
  let width = height * 210 / 297
  rotate(4deg, origin: center + horizon, box(
    width: width,
    height: height,
    fill: white,
    stroke: 1pt + rgb("#94a3b8"),
    radius: 3pt,
    inset: 18pt,
    {
      set text(font: "Roboto Slab", fill: rgb("#334155"))
      // Name / Date header (matches lib/header.typ structure)
      grid(
        columns: (auto, 1fr, auto, 60pt),
        column-gutter: 5pt,
        align: bottom,
        text(size: 9pt, weight: "bold")[Name:],
        line(length: 100%, stroke: 0.4pt),
        text(size: 9pt, weight: "bold")[Date:],
        line(length: 100%, stroke: 0.4pt),
      )
      v(6pt)
      line(length: 100%, stroke: 1.2pt)
      v(18pt)

      // 4-column x 3-row grid of vertical-addition silhouettes. The
      // operator sits in its own column so the digit placeholders
      // line up vertically on both rows (ones over ones, tens over tens).
      let problem-slot() = {
        let bar-color = rgb("#e2e8f0")
        let digit = rect(width: 10pt, height: 14pt, fill: bar-color, stroke: none, radius: 1pt)
        let op = box(width: 10pt, height: 14pt, align(center + horizon, text(size: 13pt, weight: "bold")[+]))
        align(center, stack(
          spacing: 4pt,
          grid(
            columns: (10pt, 10pt, 10pt),
            column-gutter: 3pt,
            row-gutter: 4pt,
            [], digit, digit,
            op, digit, digit,
          ),
          line(length: 38pt, stroke: 0.6pt),
        ))
      }

      grid(
        columns: (1fr,) * 4,
        rows: (1fr,) * 3,
        align: center + top,
        ..range(12).map(_ => problem-slot()),
      )
    },
  ))
}

#place(right + horizon, dx: -72pt, worksheet-preview(height: 450pt))
