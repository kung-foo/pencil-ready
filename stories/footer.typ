#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font

// worksheet-footer now renders as a fixed-height box (0.8cm tall,
// 100% wide). Pin the page width to the A4 content-area width (18cm)
// so `width: 100%` resolves the way it would in production, and
// height: auto lets the rendered PNG match the box's 0.8cm exactly.
#set page(width: 18cm, height: auto, margin: 0pt)
#set text(font: body-font)

#worksheet-footer(pencil-ready-content, debug: true)
