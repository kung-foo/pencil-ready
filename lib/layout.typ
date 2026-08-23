// Grid that lays problems out on a page.
//
//   component: a function reference to the problem component to render
//              each cell with. Signature: `(data, mode, opts, debug)`.
//              Every component is self-padded and self-aligned (see
//              lib/problems/*.typ), so worksheet-grid has no style-
//              specific knowledge.
//
//   modes:     list of "blank" | "worked" | "answer-only", one entry
//              per problem. Defaults to all-blank.
//
//   opts:      dict forwarded to each component unchanged. Keys are
//              component-specific (operator, width, answer-rows,
//              implicit, variable, pad-width, ...).
//
// Callers must import the component function into their scope and
// pass it by reference.
//   separators: draw hairlines between rows and columns, boxing each
//              problem off from its neighbours. Set per worksheet kind
//              (not user-facing) — it earns its keep when a single cell
//              holds several sub-figures, like the mean sheet's data
//              line + addition stack + division bracket, where without
//              a rule it's ambiguous which working belongs to which
//              problem. `cell_size_cm` must include `separator-inset`
//              on both axes for the kinds that turn this on, or the
//              grid will fit one fewer row than the page can hold.
#let separator-stroke = 0.5pt + rgb("#aab2bf")
#let separator-inset = 0.35cm

#let worksheet-grid(
  problems,
  component,
  num-cols: 4,
  debug: false,
  modes: none,
  opts: (:),
  separators: false,
) = {
  let num-problems = problems.len()
  // Ceiling division: handles partial last rows (e.g. 10 problems, 3 cols = 4 rows).
  let num-rows = calc.quo(num-problems + num-cols - 1, num-cols)
  // Header and footer are rendered as page chrome by template.rs
  // (via typst's page.header / page.footer), so the grid fills the
  // full body area.
  let content-area = 100%
  let debug-box = if debug { 1pt + red } else { none }
  let debug-grid = if debug { 1pt + blue } else { none }

  let resolved-modes = if modes == none {
    range(num-problems).map(_ => "blank")
  } else {
    modes
  }
  let mode-at(idx) = resolved-modes.at(idx)

  block(height: content-area, width: 100%, stroke: debug-box, {
    grid(
      columns: range(num-cols).map(_ => 1fr),
      rows: range(num-rows).map(_ => 1fr),
      // Interior lines only — `x > 0` / `y > 0` skip the outer edges, so
      // the grid reads as dividers between problems rather than a box
      // drawn around the page. Debug borders win when both are on.
      stroke: if debug {
        debug-grid
      } else if separators {
        (x, y) => (
          left: if x > 0 { separator-stroke } else { none },
          top: if y > 0 { separator-stroke } else { none },
        )
      } else {
        none
      },
      inset: if separators { separator-inset } else { 0pt },
      // Center each problem horizontally in its cell. When a
      // component's bounding box is symmetric around `=` (col1 = col3
      // in `equation-rows`), centering puts `=` at the cell's
      // horizontal center — and since every cell is the same 1fr
      // width, that's the same x-coordinate across the whole column.
      // So `=` signs line up vertically without the worksheet
      // template having to pre-compute uniform col-widths and pass
      // them down. Vertical alignment stays at the top so the rows
      // read naturally and the writing space stays below the problem.
      align: center + top,
      ..range(num-problems).map(idx => {
        component(problems.at(idx), mode: mode-at(idx), opts: opts, debug: debug)
      })
    )
  })
}
