// Thumbnail: one fraction-multiplication problem
// (20 × 3/4 = 60/4 = 15) — the two-row worked layout with the
// intermediate fraction and the final integer in handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem

#show: thumb-page.with(width: 130pt)

// Component returns its tight bounding rect; the thumb owns the
// centering. align(center + horizon, …) puts the visible content at
// the page's geometric center so the A4 page-shape feels balanced.
#align(center + horizon, fraction-multiplication-problem(
  (20, 3, 4),
  mode: "worked",
  // symmetric: false → bbox tracks visible content (no dead space
  // after the narrower RHS) so the centered thumb is visually
  // balanced left/right. Worksheet grid still uses default symmetric.
  opts: (
    operator: [#sym.times],
    symmetric: false,
    ..thumb-answer-style,
  ),
))
