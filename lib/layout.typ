#let worksheet-grid(problems, operator, num-cols: 4, width: 2.2cm, debug: false, style: "vertical") = {
  import "/lib/problems/vertical.typ": vertical-problem
  import "/lib/problems/long-division.typ": long-division-problem

  let num-problems = problems.len()
  let num-rows = calc.quo(num-problems, num-cols)
  let header-height = 2.5cm
  let footer-height = 0.8cm
  let content-area = 98% - header-height - footer-height
  let debug-box = if debug { 1pt + red } else { none }
  let debug-grid = if debug { 1pt + blue } else { none }

  block(height: content-area, width: 100%, stroke: debug-box, {
    grid(
      columns: range(num-cols).map(_ => 1fr),
      rows: range(num-rows).map(_ => 1fr),
      align: if style == "long-division" { left + top } else { center + top },
      stroke: debug-grid,
      ..range(num-problems).map(idx => {
        let nums = problems.at(idx)
        if style == "long-division" {
          pad(left: 0.5cm, long-division-problem(nums, width: width, debug: debug))
        } else {
          vertical-problem(nums, operator, width: width, debug: debug)
        }
      })
    )
  })
}
