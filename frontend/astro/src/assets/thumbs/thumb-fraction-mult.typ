// Thumbnail: one fraction-multiplication problem
// (20 × 3/4 = 60/4 = 15) — the two-row worked layout with the
// intermediate fraction and the final integer in handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem

#show: thumb-page.with(width: 150pt)

#fraction-multiplication-problem(
  (20, 3, 4),
  mode: "worked",
  opts: (
    operator: [#sym.times],
    align: center + horizon,
    // Tighter slot than worksheet default (3.2em → 1.5em) so the
    // visible content tracks the bounding box and centers properly.
    "slot-width": 1.5em,
    ..thumb-answer-style,
  ),
)
