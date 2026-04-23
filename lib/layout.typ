// Grid that lays problems out on a page. Per-problem render mode:
//
//   modes: a list of "blank" | "worked" | "answer-only", one entry per
//          problem. "blank" = unsolved. "worked" = filled-in example
//          with partial work shown. "answer-only" = just the final
//          numeric answer (used on answer-key pages).
//
// Defaults to all-blank when the caller omits modes, so stories and
// standalone invocations keep their existing behavior.
#let worksheet-grid(
  problems,
  operator,
  num-cols: 4,
  width: 2.2cm,
  debug: false,
  style: "vertical",
  answer-rows: 1,
  modes: none,
  implicit: false,
  variable: "x",
  pad-width: 0,
) = {
  import "/lib/problems/vertical.typ": vertical-problem
  import "/lib/problems/long-division.typ": long-division-problem
  import "/lib/problems/horizontal.typ": horizontal-problem
  import "/lib/problems/horizontal-fraction.typ": horizontal-fraction-problem
  import "/lib/problems/algebra-two-step.typ": algebra-two-step-problem

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
      align: if style == "vertical" {
        center + top
      } else if style == "horizontal" or style == "horizontal-fraction" or style == "algebra-two-step" {
        // Right-align problems within each cell so the = and answer
        // blanks line up vertically down each column. Top-align so every
        // problem starts flush with the top of its cell instead of being
        // vertically centered — the visual rhythm across rows is cleaner.
        right + top
      } else {
        left + top
      },
      stroke: debug-grid,
      ..range(num-problems).map(idx => {
        let nums = problems.at(idx)
        let mode = mode-at(idx)
        let solved = mode != "blank"
        let answer-only = mode == "answer-only"
        if style == "long-division" {
          long-division-problem(nums, mode: mode, opts: (width: width, answer-rows: answer-rows), debug: debug)
        } else if style == "horizontal" {
          horizontal-problem(nums, mode: mode, opts: (operator: operator), debug: debug)
        } else if style == "horizontal-fraction" {
          pad(left: 0.3cm, right: 0.3cm, horizontal-fraction-problem(nums, mode: mode, opts: (operator: operator), debug: debug))
        } else if style == "algebra-two-step" {
          pad(left: 0.3cm, right: 1.5cm, algebra-two-step-problem(nums, mode: mode, opts: (operator: operator, implicit: implicit, variable: variable), debug: debug))
        } else {
          vertical-problem(nums, mode: mode, opts: (operator: operator, width: width, answer-rows: answer-rows, pad-width: pad-width), debug: debug)
        }
      })
    )
  })
}
