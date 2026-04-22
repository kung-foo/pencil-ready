#import "/lib/problems/long-division.typ": long-division-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Answer-key rendering: the quotient sits above the bracket, but the
// divide-multiply-subtract-bring-down work below is suppressed.
#long-division-problem((756, 3, 252), mode: "answer-only", opts: (answer-rows: 6))
