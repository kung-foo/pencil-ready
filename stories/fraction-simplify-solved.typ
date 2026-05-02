#import "/lib/problems/fraction/simplification.typ": fraction-simplification-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked: 6/8 → 3/4. Exercises the "reduce a proper fraction" branch
// of the answer-rendering logic.
#fraction-simplification-problem((6, 8), mode: "worked", debug: true)
