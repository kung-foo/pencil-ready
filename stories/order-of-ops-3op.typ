#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Form 1 (3-op flat): 9 × 3 − 5 + 2 = 24
#order-of-ops-problem(
  (1, 9, 2, 3, 1, 5, 0, 2, 24),
  opts: (times-op: [#sym.times], divide-op: [#sym.div]),
  debug: true,
)
