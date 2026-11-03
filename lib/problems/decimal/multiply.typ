// Decimal multiplication: the shared `vertical-stack` layout with decimal
// places, so the point takes a half-width column and the digits stay in
// their columns through it. A multi-digit multiplier (4.7 × 5.6) gets
// partial products, written as whole numbers as in the written method,
// and the point is placed in the product.
//
//      4.7
//   ×  5.6
//   ──────
//      282
//     2350
//   ──────
//    26.32
//
// `data` = (top, multiplier, product) as scaled integers. `opts` as for
// `vertical-stack-problem`: `operator`, `width`, `decimal-places` (one
// count per number), `decimal-sep`, `answer-rows` (1 for a one-digit
// multiplier, else digits + 1), `answer-font` / `answer-color`.
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem as decimal-multiply-problem
