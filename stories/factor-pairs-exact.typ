#import "/lib/problems/factors/pairs.typ": factor-pairs-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// exact-rows: only as many rows as each number has pairs (24: four,
// 36: five, 37: one), in the same six-row cell.
#grid(columns: 3, column-gutter: 0.5em,
  factor-pairs-problem((24,), opts: (rows: 6, exact-rows: true), debug: true),
  factor-pairs-problem((36,), opts: (rows: 6, exact-rows: true), debug: true),
  factor-pairs-problem((37,), opts: (rows: 6, exact-rows: true), debug: true),
)
