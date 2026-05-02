// Thumbnail: one equivalent-fractions problem (1/3 = ?/9, missing
// right-num = 3) — the answer slot is already a visible box on the
// page, so the thumbnail keeps the same layout and just fills it in
// with the handwriting font.
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/fraction/equivalence.typ": fraction-equivalence-problem

#show: thumb-page.with()

// (left-num, left-den, right-num, right-den, missing)
// missing: 0 = left-num, 1 = left-den, 2 = right-num, 3 = right-den
#fraction-equivalence-problem(
  (1, 3, 3, 9, 2),
  mode: "worked",
  opts: thumb-answer-style,
  debug: false,
)
