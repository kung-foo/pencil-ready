#import "/lib/problems/algebra/square-root.typ": algebra-square-root-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Answer-only rendering: the intermediate `x² =` row is suppressed,
// only the final `x = 4` is shown. Keeps the 3-row layout so the
// answer page aligns cell-for-cell with the problem page.
#algebra-square-root-problem((0, 5, 4, 4, 21), mode: "answer-only", debug: true)
