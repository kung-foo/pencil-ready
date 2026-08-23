// Level 3: the open-workspace layout. Eight 2-digit values with tenths,
// so the data line wraps and there is no printed scaffold — column
// addition can't hold eight decimal rows in a two-per-page cell, and the
// long-division component has no decimal support. The student lays out
// the work in the reserved space; the answer slot is pinned to the
// bottom so a page can be graded down one column.
//
// Values arrive scaled by 10 (668 renders as 66.8).
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (668, 630, 621, 356, 450, 524, 393, 350, 3992, 499),
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
