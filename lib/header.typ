#let handwriting-font = "Architects Daughter"

// A filled-in field: handwritten name in a zero-height overlay sitting on
// the signature line position. Zero height keeps the grid row at the same
// height as the blank-underline case so nothing else in the header shifts.
// Long names are shrunk to fit the field width so they can't collide with
// the Date column.
#let handwritten-field(name, width) = {
  let base-size = 18pt
  box(width: width, height: 0pt,
    place(bottom + left, dy: 1pt,
      context {
        let natural = measure(text(font: handwriting-font, size: base-size)[#name])
        let size = if natural.width > width {
          base-size * (width / natural.width)
        } else {
          base-size
        }
        text(font: handwriting-font, size: size)[#name]
      }))
}

#let worksheet-header(student-name: none, teacher-name: none, debug: false) = {
  let debug-box = if debug { 1pt + red } else { none }

  let name-field = if student-name != none {
    handwritten-field(student-name, 10cm)
  } else {
    line(length: 10cm, stroke: 0.5pt)
  }

  let teacher-field = if teacher-name != none {
    handwritten-field(teacher-name, 10cm)
  } else {
    line(length: 10cm, stroke: 0.5pt)
  }

  box(height: 2.5cm, width: 100%, stroke: debug-box, {
    set text(size: 12pt)
    grid(
      columns: (auto, 1fr, auto, auto),
      column-gutter: 0.3cm,
      row-gutter: 0.5cm,
      align: bottom,
      [*Name*:], name-field, [*Date*:], line(length: 4cm, stroke: 0.5pt),
      [*Teacher*:], teacher-field, [], [],
    )
    v(0.3cm)
    line(length: 100%, stroke: 1.5pt)
  })
}
