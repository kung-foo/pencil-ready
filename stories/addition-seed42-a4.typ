#import "/lib/header.typ": worksheet-header
#import "/lib/layout.typ": worksheet-grid
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font

#set document(
  title: "Addition",
  author: "Pencil Ready",
  description: "Printable math worksheet — https://pencilready.com",
  keywords: ("math", "worksheet", "addition", "pencilready.com"),
)

#set page(paper: "a4", margin: (top: 1.5cm, bottom: 1.0cm, left: 1.5cm, right: 1.5cm))
#set text(font: body-font, size: 10pt)

// Headings exist only to populate the PDF outline (sidebar bookmarks)
// when --include-answers is used. Suppress visible rendering here — the
// worksheet-header already provides the on-page title area.
#show heading: _ => []

#worksheet-header(student-name: "Luke Skywalker", debug: false)

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
  [#sym.plus],
  num-cols: 4,
  width: 2.25cm,
  debug: false,
  style: "vertical",
  answer-rows: 1,
  solve-first: false,
  all-solved: false,
  answer-only: false,
  implicit: false,
  variable: "x",
  pad-width: 0,
)

#worksheet-footer(pencil-ready-content)

