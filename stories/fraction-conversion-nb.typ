#import "/lib/problems/fraction/conversion.typ": fraction-conversion-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved, Norwegian notation — decimal comma, space before %: 3/8 = 0,375 = 37,5 %
#fraction-conversion-problem((3, 8, 0, 375, 3, 375, 1), mode: "worked", opts: (decimal-sep: ",", percent-suffix: "\u{a0}%"), debug: true)
