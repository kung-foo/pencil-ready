#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Norwegian locale: × → ·, ÷ → :.
#order-of-ops-problem(
  (2, 8, 0, 4, 3, 2, 6),
  opts: (times-op: [#sym.dot.op], divide-op: [#sym.colon]),
  debug: true,
)
