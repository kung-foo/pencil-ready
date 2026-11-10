// Prime factorization with a factor tree: `24 = ___`, with room under
// it for the tree. Split the number into two factors, split those, and
// so on until every branch ends in a prime (circled); the answer is the
// circled primes multiplied, smallest first.
//
//   24 = 2 · 2 · 2 · 3
//
//        24
//       /  \
//      4    6
//     / \  / \
//    ②  ② ②  ③
//
// The worked tree splits each number into its closest factor pair
// (24 → 4 · 6, not 2 · 12), so it stays balanced and short; any split
// gives the same primes.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-features

// `data` = (n,), a composite number up to 100. The component builds the
// tree and the answer itself.
// `opts` keys (with defaults):
//   operator: multiply sign, per locale. Default `[#sym.dot.op]`.
//   answer-font / answer-color: solved-answer styling. Default: inherit.
//   work: reserve (and, when worked, draw) the tree. Default true.
//   align: cell alignment. Default `left + top`.
// `mode` = "blank" | "worked" | "answer-only". "worked" draws the tree;
// "answer-only" writes just the answer.
#let prime-factors-problem(data, mode: "blank", opts: (:), debug: false) = {
  let operator = opts.at("operator", default: [#sym.dot.op])
  let answer-font = opts.at("answer-font", default: none)
  let answer-color = opts.at("answer-color", default: none)
  let show-work = opts.at("work", default: true)
  let cell-align = opts.at("align", default: left + top)
  let solved = mode != "blank"
  let worked = mode == "worked"

  set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let color = if answer-color != none { answer-color } else { black }
  let handwritten(body) = if answer-font != none {
    text(font: answer-font, fill: color, size: 1.4em, body)
  } else {
    text(fill: color, body)
  }

  let n = data.at(0)
  let is-prime(m) = m > 1 and range(2, m).all(d => d * d > m or calc.rem(m, d) != 0)
  assert(n > 3 and not is-prime(n), message: "prime-factors: number must be composite")
  // Closest factor pair, smaller first.
  let split(m) = {
    let a = range(2, m).filter(d => d * d <= m and calc.rem(m, d) == 0).last()
    (a, calc.quo(m, a))
  }
  let primes(m) = if is-prime(m) { (m,) } else {
    let (a, b) = split(m)
    (primes(a) + primes(b)).sorted()
  }
  // Primes joined by the multiply sign with thin spaces, so the longest
  // answer (six factors) fits a half-page cell.
  let product(m) = primes(m).map(str).join(h(0.15em) + text(font: operator-font, operator) + h(0.15em))
  let answer = product(n)

  // The tree, drawn in handwriting like an answer: each number centred
  // over its two branches, primes circled.
  let ring(body) = box(stroke: 0.6pt + color, radius: 50%, inset: (x: 0.2em, y: 0.15em), body)
  let gap = 0.9em
  let drop = 0.9em
  let tree(m) = if is-prime(m) { ring(handwritten(str(m))) } else {
    let (a, b) = split(m)
    let (l, r) = (tree(a), tree(b))
    context {
      let (lw, rw) = (measure(l).width, measure(r).width)
      let total = lw + gap + rw
      // Children's labels are centred in their subtrees.
      let (lc, rc) = (lw / 2, lw + gap + rw / 2)
      let stroke = 0.6pt + color
      // A stack, not blocks: block spacing would float the branches
      // away from the numbers they join.
      stack(
        dir: ttb,
        spacing: 0.2em,
        box(width: total, align(center, handwritten(str(m)))),
        box(width: total, height: drop, {
          place(line(start: (total / 2, 0pt), end: (lc, drop), stroke: stroke))
          place(line(start: (total / 2, 0pt), end: (rc, drop), stroke: stroke))
        }),
        grid(columns: (lw, rw), column-gutter: gap, align(center + top, l), align(center + top, r)),
      )
    }
  }

  // The problem line: `24 = ___`, the slot as long as the longest
  // answer allowed (64 = 2 · 2 · 2 · 2 · 2 · 2).
  let slot = context box(
    width: measure(handwritten(product(64))).width + 0.8em,
    height: 1.6em,
    stroke: (bottom: 0.5pt),
    align(center + horizon, if solved { handwritten(answer) }),
  )
  let problem = grid(
    columns: 3,
    column-gutter: 0.3em,
    align: horizon,
    // Right-aligned in room for three digits, so the `=` and answer
    // lines line up down the page.
    context box(width: measure("100").width, align(right, str(n))), sym.eq, slot,
  )
  // Room for the biggest tree allowed: 64 has the most leaves (six),
  // 96 the deepest closest-pair tree. Blank cells reserve it for the
  // student's own tree.
  let room = context {
    let (t1, t2) = (measure(tree(64)), measure(tree(96)))
    box(
      width: calc.max(t1.width, t2.width) + 0.6em,
      height: calc.max(t1.height, t2.height),
      if worked { align(center + top, tree(n)) },
    )
  }
  let body = if show-work { stack(dir: ttb, spacing: 0.6em, problem, align(center, room)) } else { problem }
  align(cell-align, pad(x: 0.47em, box(stroke: debug-box, inset: 0.2em, body)))
}
