//! Standalone Rust → typst layout preview.
//!
//! Emits a multi-page PDF showing header + grid + footer with debug borders
//! (red = box bounds, blue = grid cells). No problem content — just the
//! page-layout skeleton, so we can compare grid configurations, paper
//! sizes, and future layout ideas without involving problem components.
//!
//! Usage:
//!   cargo run --example layout_preview -p pencil-ready-core
//!
//! Output: out/layout-preview.pdf

use std::path::PathBuf;

use anyhow::Result;
use pencil_ready_core::{Fonts, OutputFormat, compile_typst};

struct Config {
    label: &'static str,
    paper: &'static str,
    cols: u32,
    rows: u32,
    /// When `Some(h)`, emit TWO pages for this config: a "first page"
    /// reserving `h` cm above the grid for an intro block (H1 + help
    /// text) and a "continuation" page with no intro (full grid area).
    /// Demonstrates the two-page-layout model from LAYOUT_REFACTOR.md.
    intro_cm: Option<f32>,
}

const CONFIGS: &[Config] = &[
    Config { label: "A4 · 4×4 — vertical stacked (add/sub/mult)", paper: "a4",        cols: 4, rows: 4, intro_cm: None },
    Config { label: "A4 · 2×3 — algebra-sized (wide, tall cells)", paper: "a4",        cols: 2, rows: 3, intro_cm: None },
    Config { label: "A4 · 5×8 — drill density",                    paper: "a4",        cols: 5, rows: 8, intro_cm: None },
    Config { label: "A4 · 3×3 — long-division",                    paper: "a4",        cols: 3, rows: 3, intro_cm: None },
    Config { label: "US-Letter · 4×4 — compare with A4 4×4",       paper: "us-letter", cols: 4, rows: 4, intro_cm: None },
    Config { label: "A4 · 2×3 algebra — with 4cm intro block",     paper: "a4",        cols: 2, rows: 3, intro_cm: Some(4.0) },
];

fn build_source() -> String {
    // Preamble: imports, shared helpers. Paper is set per-page below so we
    // can mix A4 and Letter in one document.
    let preamble = r##"#import "/lib/header.typ": worksheet-header
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font

#set text(font: body-font, size: 10pt)

// Empty grid with debug borders. Red = outer content-area box; blue =
// per-cell borders. Mirrors what worksheet-grid does without requiring
// problems / operators. `intro` reserves that much vertical space above
// the grid for an H1 + help-text block (drawn purple-bordered).
#let preview-grid(cols, rows, intro: 0cm) = {
  let header-h = 1.5cm
  let footer-h = 0.8cm
  let content-area = 98% - header-h - footer-h - intro
  if intro > 0cm {
    block(height: intro, width: 100%, stroke: 1pt + purple,
      align(center + horizon, {
        set text(fill: rgb("#888888"), size: 9pt)
        [Intro block — H1 + help text reservation (#intro)]
      }))
  }
  block(height: content-area, width: 100%, stroke: 1pt + red, {
    grid(
      columns: range(cols).map(_ => 1fr),
      rows: range(rows).map(_ => 1fr),
      stroke: 1pt + blue,
      ..range(cols * rows).map(_ => []),
    )
  })
}

// Small label floating in the top margin. `place()` doesn't consume
// layout space, so the caption can't push the footer onto page 2.
// dy: -1cm puts it roughly in the middle of the 1.5cm top margin.
#let caption(body) = place(top + left, dy: -1cm, text(size: 8pt, fill: rgb("#888888"), body))

"##;

    // Every config emits 1 page; configs with `intro_cm: Some(_)` emit
    // 2 (page 1 reserves the intro; "continuation" page uses full grid).
    let mut pages = String::new();
    let mut emit_page =
        |cfg: &Config, label_override: Option<&str>, intro: Option<f32>, first: bool| {
            if !first {
                pages.push_str("#pagebreak()\n\n");
            }
            let label = label_override.unwrap_or(cfg.label);
            let intro_arg = match intro {
                Some(h) => format!(", intro: {h}cm"),
                None => String::new(),
            };
            // stack(spacing: 0pt) flushes the header, grid, and footer
            // together without the global `set block(spacing: 0pt)` leaking
            // into the header's internal layout (which was collapsing the
            // gap between the Name/Date grid and the thick rule).
            pages.push_str(&format!(
                r##"#set page(paper: "{paper}", margin: (top: 1.5cm, bottom: 1.0cm, left: 1.5cm, right: 1.5cm))
#caption[{label}]
#stack(
  spacing: 0pt,
  worksheet-header(debug: true),
  preview-grid({cols}, {rows}{intro_arg}),
  worksheet-footer(pencil-ready-content, debug: true),
)
"##,
                paper = cfg.paper,
                label = label,
                cols = cfg.cols,
                rows = cfg.rows,
                intro_arg = intro_arg,
            ));
        };

    let mut first = true;
    for cfg in CONFIGS.iter() {
        match cfg.intro_cm {
            None => {
                emit_page(cfg, None, None, first);
                first = false;
            }
            Some(h) => {
                let p1 = format!("{} — page 1 (with intro)", cfg.label);
                let p2 = format!("{} — continuation (no intro, full grid)", cfg.label);
                emit_page(cfg, Some(&p1), Some(h), first);
                first = false;
                emit_page(cfg, Some(&p2), None, false);
            }
        }
    }

    format!("{preamble}{pages}")
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("no repo root")
        .to_path_buf();

    let fonts = Fonts::load(&root)?;
    let source = build_source();
    let pdf = compile_typst(&source, OutputFormat::Pdf, &root, &fonts)?;

    let out_dir = root.join("out");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("layout-preview.pdf");
    std::fs::write(&out_path, &pdf)?;

    println!("wrote {} ({} bytes, {} configs)", out_path.display(), pdf.len(), CONFIGS.len());
    Ok(())
}
