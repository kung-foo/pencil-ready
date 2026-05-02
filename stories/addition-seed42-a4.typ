// Full-page addition worksheet — the only story that exercises
// `worksheet-grid` + the page chrome together. Mirrors what
// `crates/core/src/template.rs` emits for an `add --seed 42 --digits
// 2,2 --problems 12` run, so visual diffs here catch regressions in
// either half of the pipeline.

#import "/lib/header.typ": worksheet-header
#import "/lib/layout.typ": worksheet-grid
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font
#import "/lib/problems/addition/basic.typ": addition-basic-problem

#set document(
  title: "Addition",
  author: "Pencil Ready",
  description: "Printable math worksheet — https://pencilready.com",
  keywords: ("math", "worksheet", "addition", "pencilready.com"),
)

// Margins / ascent / descent / pad values match `MARGINS_CM`,
// `HEADER_ASCENT_CM`, `FOOTER_DESCENT_CM`, `HEADER_PAD_TOP_CM`,
// `FOOTER_PAD_BOTTOM_CM` in `pencil_ready_core`. Keep in sync.
#set page(
  paper: "a4",
  margin: (top: 3.2cm, bottom: 2.2cm, left: 1.5cm, right: 1.5cm),
  header-ascent: 0.8cm,
  footer-descent: 0.4cm,
  header: pad(top: 0.7cm, worksheet-header(student-name: "Luke Skywalker", debug: true)),
  footer: pad(bottom: 0.7cm, worksheet-footer(pencil-ready-content)),
)
#set text(font: body-font, size: 10pt)

#worksheet-grid(
  (
  (37, 75, 112),
  (38, 11, 49),
  (18, 79, 97),
  (45, 67, 112),
  (90, 40, 130),
  (53, 90, 143),
  (71, 48, 119),
  (59, 50, 109),
  (42, 27, 69),
  (70, 44, 114),
  (78, 55, 133),
  (20, 25, 45),
  ),
  addition-basic-problem,
  num-cols: 4,
  debug: true,
  opts: (operator: [#sym.plus], width: 2.25cm, answer-rows: 1, pad-width: 0),
)

