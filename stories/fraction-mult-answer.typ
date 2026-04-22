#import "/lib/problems/horizontal-fraction.typ": horizontal-fraction-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Answer-key rendering: skips the multiply-across intermediate in row 1,
// shows only the simplified integer in row 2. Layout (the two-row grid
// with aligned `=`) stays identical to the unsolved and solved variants.
#horizontal-fraction-problem((30, 7, 10), mode: "answer-only", opts: (operator: [#sym.times]))
