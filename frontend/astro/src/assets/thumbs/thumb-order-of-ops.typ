// Thumbnail: four order-of-operations problems stacked on an A4 page.
// Each problem renders with `mode: "worked"` + thumb-answer-style so
// the answer appears in the handwriting font (matching the brand cue
// across the homepage card grid).
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem

#show: thumb-page.with(width: 180pt)

// Right-align inside each row so the fixed-width answer slot pins
// every `=` to the same x position — same convention the worksheet
// grid relies on. Center-aligned content makes `=` drift with the
// LHS width and breaks the vertical line of equals signs.
#let probl-opts = (
  times-op: [#sym.times],
  divide-op: [#sym.div],
  align: right + horizon,
  ..thumb-answer-style,
)
#let probl(data) = order-of-ops-problem(data, mode: "worked", opts: probl-opts)

#grid(
  columns: 1fr,
  rows: (1fr, 1fr, 1fr, 1fr),
  // 5 + 6 × 2 = 17 (form 0)
  probl((0, 5, 0, 6, 2, 2, 17)),
  // (3 + 4) × 5 = 35 (form 2 — parens)
  probl((2, 3, 0, 4, 2, 5, 35)),
  // 9 × 3 − 5 = 22 (form 0)
  probl((0, 9, 2, 3, 1, 5, 22)),
  // 8 ÷ 2 + 3 = 7 (form 0)
  probl((0, 8, 3, 2, 0, 3, 7)),
)
