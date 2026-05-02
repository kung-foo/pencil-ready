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
#let worksheet-grid(
  problems,
  component,
  num-cols: 4,
  debug: false,
  modes: none,
  opts: (:),
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
      stroke: debug-grid,
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
