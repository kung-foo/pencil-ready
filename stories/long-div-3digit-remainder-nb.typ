#import "/lib/problems/division/long.typ": division-long-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Norwegian remainder form: `84 rest 2` instead of `84 r=2`, with the
// slot reserved as the generator does whenever remainders are on. The
// slot is measured from the prefix, so the wider suffix still fits;
// the no-break space keeps "rest" on the digit's line.
#division-long-problem((758, 9, 84), mode: "answer-only", opts: (width: 7em, answer-rows: 6, reserve-remainder: true, remainder-prefix: "rest\u{a0}"), debug: true)
