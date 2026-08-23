// Thumbnail: one mean problem, worked, with the answer in the
// handwriting font (the brand cue across the homepage card grid).
//
// Deliberately the smallest interesting data set — three 2-digit values
// summing to a 2-digit total (12 + 15 + 9 = 36, 36 ÷ 3 = 12). A
// 3-digit set would need a 4-digit dividend and six rows of long
// division work, which turns to grey mush at card size. This one still
// reads as "a column addition next to a division bracket", which is the
// shape a parent needs to recognise.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/statistics/mean.typ": mean-problem

#show: thumb-page.with(width: 210pt)

// The component self-aligns `left + top` for the worksheet grid; a
// single-cell thumb wants it centred on the page instead.
#mean-problem(
    (12, 15, 9, 36, 12),
    mode: "worked",
    opts: (
      operator: [#sym.plus],
      stack-width: 2.2cm,
      div-width: 2.6cm,
      div-rows: 4,
      body-height: 5.6cm,
      gap: 0.25cm,
      scaffold: true,
      align: center + horizon,
      ..thumb-answer-style,
    ),
)
