// Norwegian workspace layout, solved: decimal comma, `; ` between
// values (a comma would be ambiguous), and the `snitt =` answer slot.
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (668, 630, 621, 356, 450, 524, 393, 350, 3992, 499),
  mode: "worked",
  opts: (
    layout: "workspace",
    decimal-places: 1,
    data-size: 16pt,
    data-width: 17.6cm,
    data-height: 1.4cm,
    work-height: 9.0cm,
    decimal-sep: ",",
    mean-label: "snitt",
  ),
  debug: true,
)
