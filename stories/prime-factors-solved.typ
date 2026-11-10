#import "/lib/problems/factors/prime.typ": prime-factors-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Blank, then worked trees: 24, 64 (six leaves), 96 (deepest), 30.
#grid(columns: 3, gutter: 0.5em,
  prime-factors-problem((24,), debug: true),
  prime-factors-problem((24,), mode: "worked", debug: true),
  prime-factors-problem((64,), mode: "worked", debug: true),
  prime-factors-problem((96,), mode: "worked", debug: true),
  prime-factors-problem((30,), mode: "worked", debug: true),
  prime-factors-problem((30,), mode: "answer-only", debug: true),
)
