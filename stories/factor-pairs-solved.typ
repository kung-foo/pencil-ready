#import "/lib/problems/factors/pairs.typ": factor-pairs-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Blank (six reserved rows), worked 24, a square (36), a prime (37), and
// a twelve-factor number (96).
#grid(columns: 5, column-gutter: 0.5em,
  factor-pairs-problem((24,), debug: true),
  factor-pairs-problem((24,), mode: "worked", debug: true),
  factor-pairs-problem((36,), mode: "worked", debug: true),
  factor-pairs-problem((37,), mode: "worked", debug: true),
  factor-pairs-problem((96,), mode: "worked", debug: true),
)
