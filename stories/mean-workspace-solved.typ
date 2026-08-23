// Workspace layout, solved: only the answer slot fills in — there is no
// printed working to complete, which is the point of this level.
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
  ),
  debug: true,
)
