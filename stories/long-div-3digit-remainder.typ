#import "/lib/problems/division/long.typ": division-long-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit dividend with a non-zero remainder — the answer above the
// bracket reads `84 r 2` (758 / 9 = 84 r 2). The bracket overline
// widens to cover the suffix; cell width is bumped from the
// no-remainder 3.9em to fit the longer answer.
#division-long-problem((758, 9, 84), mode: "answer-only", opts: (width: 7em, answer-rows: 6), debug: true)
