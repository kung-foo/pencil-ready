#import "/lib/problems/percent/of.typ": percent-of-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved, Norwegian, decimal answer: 37 % av 48 = 17,76
#percent-of-problem((37, 48, 1776, 2), mode: "worked", opts: (percent-suffix: "\u{a0}%", of: "av", decimal-sep: ","), debug: true)
