#import "/lib/problems/algebra-two-step.typ": algebra-two-step-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Answer-key rendering: the intermediate `ax =` row is suppressed, only
// the final `x = 4` is shown. Keeps the 3-row layout so the answer page
// aligns cell-for-cell with the problem page.
#algebra-two-step-problem((4, 5, 4, 21, 0), mode: "answer-only", opts: (operator: [#sym.times]))
