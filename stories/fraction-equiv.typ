#import "/lib/problems/fraction/equivalence.typ": fraction-equivalence-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Blank — all four missing-slot variants.
#grid(
  columns: 2,
  column-gutter: 0.5cm,
  row-gutter: 0.3cm,
  fraction-equivalence-problem((1, 3, 2, 6,   0)),  // left-num missing
  fraction-equivalence-problem((1, 3, 2, 6,   1)),  // left-den missing
  fraction-equivalence-problem((1, 3, 2, 6,   2)),  // right-num missing
  fraction-equivalence-problem((1, 3, 2, 6,   3)),  // right-den missing
)
