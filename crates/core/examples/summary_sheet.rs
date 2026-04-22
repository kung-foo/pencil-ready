//! A single-document review sheet showing all fixtures at once.
//!
//! Renders every fixture from `examples/common/fixtures.rs` into a
//! 2-column grid on A4 with an explicit gray cell border. Each cell
//! shows the fixture id + description up top, the rendered problem
//! (with standard cell-padding applied) in the middle — outlined by
//! a red box so the exact bbox is visible — and the declared
//! dimensions from re-measuring below.
//!
//! Usage:
//!   cargo run --release --example summary_sheet -p pencil-ready-core
//!
//! Output: out/summary-sheet.pdf

use std::path::PathBuf;

use anyhow::{Context, Result};
use pencil_ready_core::{Fonts, OutputFormat, compile_typst, measure_typst};

#[path = "common/fixtures.rs"]
mod fixtures;
use fixtures::{FIXTURES, Fixture, ceil_mm, padded, preamble};

struct Measured<'a> {
    fixture: &'a Fixture,
    measured_w: f32,
    measured_h: f32,
    declared_w: f32,
    declared_h: f32,
}

fn measure_fixture(f: &Fixture, root: &std::path::Path, fonts: &Fonts) -> Result<(f32, f32)> {
    let src = format!(
        "{preamble}\n#set page(width: auto, height: auto, margin: 0pt)\n#set text(font: body-font, size: 10pt)\n\n#{padded}\n",
        preamble = preamble(),
        padded = padded(f.snippet),
    );
    measure_typst(&src, root, fonts)
        .with_context(|| format!("measuring fixture {}", f.id))
}

fn build_source(measured: &[Measured]) -> String {
    let mut cells = String::new();
    for m in measured {
        let f = m.fixture;
        // Cell body: id (bold), label, padded problem wrapped in red
        // bbox, then a dim caption. Code mode inside the cell body so
        // the snippet is a direct function call.
        cells.push_str(&format!(
            r##"  box(
    stroke: 1pt + rgb("#bbbbbb"),
    inset: 1cm,
    width: 100%,
    {{
      set align(left + top)
      text(size: 8pt, fill: rgb("#333333"), weight: "bold")[{id}]
      linebreak()
      text(size: 7pt, fill: rgb("#888888"))[{label}]
      v(0.4cm)
      align(center, box(stroke: 1pt + red, {padded}))
      v(0.3cm)
      align(center, text(size: 7pt, fill: rgb("#888888"))[
        declared: {dw} × {dh} cm · measured: {mw} × {mh} cm
      ])
    }}
  ),
"##,
            id = f.id,
            label = f.label,
            padded = padded(f.snippet),
            dw = format!("{:.1}", m.declared_w),
            dh = format!("{:.1}", m.declared_h),
            mw = format!("{:.2}", m.measured_w),
            mh = format!("{:.2}", m.measured_h),
        ));
    }

    format!(
        r##"{preamble}
#set page(paper: "a4", margin: 1.5cm)
#set text(font: body-font, size: 10pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 0.5cm,
  row-gutter: 0.5cm,
{cells})
"##,
        preamble = preamble(),
        cells = cells,
    )
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("no repo root")
        .to_path_buf();

    let fonts = Fonts::load(&root)?;

    // Measure first so the sheet can label each cell with its declared
    // size (ceil to 0.1cm) alongside the raw measurement.
    let mut measured = Vec::with_capacity(FIXTURES.len());
    for f in FIXTURES {
        let (mw, mh) = measure_fixture(f, &root, &fonts)?;
        measured.push(Measured {
            fixture: f,
            measured_w: mw, measured_h: mh,
            declared_w: ceil_mm(mw), declared_h: ceil_mm(mh),
        });
    }

    let source = build_source(&measured);
    let pdf = compile_typst(&source, OutputFormat::Pdf, &root, &fonts)?;

    let out_path = root.join("out/summary-sheet.pdf");
    std::fs::create_dir_all(out_path.parent().unwrap())?;
    std::fs::write(&out_path, &pdf)?;

    println!("wrote {} ({} bytes, {} fixtures)",
             out_path.display(), pdf.len(), FIXTURES.len());
    Ok(())
}
