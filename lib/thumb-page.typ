// Shared scaffolding for homepage worksheet thumbnails.
//
// Each `frontend/astro/src/assets/thumbs/thumb-<kind>.typ` renders one
// example problem on a tiny A4-ratio "page" with a 1pt soft outline,
// shipped as an SVG and inlined into the homepage card grid. This file
// owns the page-shape and the answer-styling so individual thumbs are
// just (component, data, opts).
//
// Usage:
//
//   #import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
//   #import "/lib/problems/addition/basic.typ": addition-basic-problem
//
//   #show: thumb-page
//   #addition-basic-problem(
//     (47, 28, 75),
//     mode: "worked",
//     opts: (operator: [#sym.plus], ..thumb-answer-style),
//   )

// A4 ratio = 1 : √2. The absolute scale is arbitrary — the embedder
// scales the SVG via CSS — but it must comfortably contain the natural
// width of the problem component (~70pt at 22pt text).
#let thumb-page-width = 90pt
#let thumb-page-height = thumb-page-width * 1.41421356

// Document-wrapper for thumb sources: invoke as `#show: thumb-page`.
// `set page` here applies to the rest of the document via the show
// rule's body scope.
#let thumb-page(body) = {
  set page(
    width: thumb-page-width,
    height: thumb-page-height,
    margin: 8pt,
    // `background` renders behind the content across the full page
    // (margin included) — so this rect *is* the page outline.
    background: rect(
      width: 100%,
      height: 100%,
      stroke: 0.5pt + rgb("#94a3b8"),
      fill: none,
    ),
  )
  body
}

// Spread into a problem component's `opts` dict to render the solved
// answer in a graphite-pencil handwriting style. Pairs with
// `mode: "worked"` (or "answer-only") on the problem call.
#let thumb-answer-style = (
  answer-font: "Architects Daughter",
  answer-color: rgb("#4a4a4a"),
)
