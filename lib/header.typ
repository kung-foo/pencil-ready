#let worksheet-header(debug: false) = {
  let debug-box = if debug { 1pt + red } else { none }

  box(height: 2.5cm, width: 100%, stroke: debug-box, {
    grid(
      columns: (auto, 1fr, auto, auto),
      column-gutter: 0.3cm,
      row-gutter: 0.4cm,
      align: bottom,
      [Name:], line(length: 10cm, stroke: 0.5pt), [], line(length: 3cm, stroke: 0.5pt),
      [Teacher:], line(length: 10cm, stroke: 0.5pt), [Date:], line(length: 3cm, stroke: 0.5pt),
    )
    v(0.3cm)
    line(length: 100%, stroke: 1.5pt)
  })
}
