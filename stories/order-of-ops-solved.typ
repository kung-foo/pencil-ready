#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked variant: same problem with the answer filled in.
#order-of-ops-problem(
  (0, 5, 0, 6, 2, 2, 17),
  mode: "worked",
  opts: (times-op: [#sym.times], divide-op: [#sym.div]),
  debug: true,
)
