// Decimal addition — multi-digit stacked operands with decimal points
// aligned. Thin wrapper around `decimal-vertical-stack`; the worksheet's
// generator passes `operator: [#sym.plus]` and the per-slot
// `decimal-places` list via opts.
#import "/lib/problems/_layouts/decimal-vertical-stack.typ": decimal-vertical-stack-problem as decimal-add-problem
