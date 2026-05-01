//! Typst source emission. Takes a `Document` (owning a `Sheet` +
//! `Chrome` + `cols` + pagination fields) and returns the complete
//! `.typ` source — preamble, chrome via page.header/page.footer, then
//! a `#worksheet-grid(...)` call per page plus optional answer pages.
//!
//! Nothing in here generates problems, validates digit ranges, or
//! knows about RNGs — all of that lives in the per-worksheet
//! generators (add.rs, multiply.rs, …) which populate `Sheet`.

use anyhow::Result;

use crate::{
    ComponentOpts, Document, FOOTER_DESCENT_CM, FOOTER_PAD_BOTTOM_CM, HEADER_ASCENT_CM,
    HEADER_PAD_TOP_CM, MARGINS_CM, RenderMode, WorksheetType,
};

fn page_modes(is_answer_page: bool, solve_first: bool, len: usize) -> Vec<RenderMode> {
    if is_answer_page {
        vec![RenderMode::AnswerOnly; len]
    } else if solve_first {
        let mut v = vec![RenderMode::Blank; len];
        if let Some(slot) = v.first_mut() {
            *slot = RenderMode::Worked;
        }
        v
    } else {
        vec![RenderMode::Blank; len]
    }
}

/// Cell width in cm for the vertical-stack and horizontal-inline
/// primitives (and the long-division layout). Drives the `width`
/// opts key emitted for those components. Formulas match what
/// `render_inner_with_pad` computed pre-refactor.
pub(crate) fn box_width_cm(worksheet: &WorksheetType, max_digits: u32) -> f64 {
    match worksheet {
        WorksheetType::LongDivision { .. } => f64::max(3.0, max_digits as f64 * 0.6 + 1.2),
        WorksheetType::MultiplicationDrill { .. } | WorksheetType::DivisionDrill { .. } => {
            f64::max(6.0, max_digits as f64 * 1.2 + 4.0)
        }
        // Fraction + algebra set their widths internally; the value
        // emitted here is a grid hint, not consumed by the component.
        WorksheetType::FractionMultiply { .. } => 6.0,
        WorksheetType::FractionSimplify { .. } => 5.0,
        WorksheetType::AlgebraTwoStep { .. } => 6.0,
        WorksheetType::AlgebraOneStep { .. } => 6.0,
        WorksheetType::AlgebraSquareRoot { .. } => 6.0,
        _ => f64::max(2.2, max_digits as f64 * 0.55 + 0.6),
    }
}

/// Max operand digit count across the actual generated problems.
/// Feeds `box_width_cm` — the UI-level formula wants the real max, not
/// the `WorksheetType::max_digits_bound` worst-case upper bound used
/// by `Document::validate`.
pub(crate) fn max_digits(problems: &[Vec<u32>]) -> u32 {
    problems
        .iter()
        .flat_map(|nums| nums.iter().map(|n| digit_count(*n)))
        .max()
        .unwrap_or(2)
}

pub fn digit_count(n: u32) -> u32 {
    if n == 0 { 1 } else { n.ilog10() + 1 }
}

fn header_name_arg(name: Option<&str>) -> String {
    match name {
        Some(n) if !n.is_empty() => {
            let escaped = n.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
        _ => "none".to_string(),
    }
}

/// Format a ComponentOpts dict as a typst expression, emitting only
/// the keys the current worksheet's component reads. Keeps the emitted
/// source tight (and easier to eyeball during debugging).
fn opts_body(worksheet: &WorksheetType, opts: &ComponentOpts) -> String {
    let operator_arg = if opts.operator.is_empty() {
        "[]".to_string()
    } else {
        format!("[#{}]", opts.operator)
    };
    match worksheet {
        WorksheetType::Add { .. }
        | WorksheetType::Subtract { .. }
        | WorksheetType::Multiply { .. }
        | WorksheetType::SimpleDivision { .. } => format!(
            "operator: {operator_arg}, width: {w}cm, answer-rows: {r}, pad-width: {p}",
            w = opts.width_cm,
            r = opts.answer_rows,
            p = opts.pad_width,
        ),
        WorksheetType::LongDivision { .. } => format!(
            "width: {w}cm, answer-rows: {r}",
            w = opts.width_cm,
            r = opts.answer_rows,
        ),
        WorksheetType::MultiplicationDrill { .. } | WorksheetType::DivisionDrill { .. } => {
            format!("operator: {operator_arg}")
        }
        WorksheetType::FractionMultiply { .. } => format!("operator: {operator_arg}"),
        // Simplification has no operator / no opts — the fraction bar
        // and `=` are universal. `:` interpolates into `(:)` which is
        // typst's empty-dict literal (as opposed to `()`, an empty array).
        WorksheetType::FractionSimplify { .. } => ":".to_string(),
        WorksheetType::AlgebraTwoStep { .. } => format!(
            "operator: {operator_arg}, implicit: {i}, variable: \"{v}\"",
            i = opts.implicit,
            // Escape backslashes and double-quotes so a variable like `"`
            // or `\` drops into the generated .typ source without
            // breaking string literal syntax. Upstream validation caps
            // variable at one unicode scalar, so this is defence-in-depth.
            v = opts.variable.replace('\\', "\\\\").replace('"', "\\\""),
        ),
        WorksheetType::FractionEquiv { .. } => ":".to_string(),
        WorksheetType::AlgebraOneStep { .. } => {
            // Two operators: `·` for multiply (uses the shared
            // `operator` slot) and `÷`/`:` for divide (its own slot).
            // Empty divide_operator falls back to a literal so typst
            // doesn't choke on `[#]`.
            let div_arg = if opts.divide_operator.is_empty() {
                "[]".to_string()
            } else {
                format!("[#{}]", opts.divide_operator)
            };
            format!(
                "mult-operator: {operator_arg}, div-operator: {div_arg}, variable: \"{v}\"",
                v = opts.variable.replace('\\', "\\\\").replace('"', "\\\""),
            )
        }
        WorksheetType::AlgebraSquareRoot { .. } => format!(
            "variable: \"{v}\"",
            v = opts.variable.replace('\\', "\\\\").replace('"', "\\\""),
        ),
    }
}

pub(crate) fn render_document(doc: &Document) -> Result<String> {
    let sheet = &doc.sheet;
    let chrome = &doc.chrome;

    let debug_str = if chrome.debug { "true" } else { "false" };
    let cols = doc.cols;
    let paper_name = chrome.paper.typst_name();

    // Header student name: either `none` or a typst string literal.
    // Escape backslashes and double quotes so arbitrary UTF-8 names drop
    // straight into the generated .typ source.
    let student_name_arg = header_name_arg(chrome.student_name.as_deref());

    // Chunk problems across pages — `cells_per_page` is `cols ×
    // rows_per_page` computed upstream in `Document::from_params`.
    let per_page = doc.cells_per_page.max(1) as usize;
    let pages: Vec<&[Vec<u32>]> = if sheet.problems.is_empty() {
        // Degenerate: no problems. Emit one empty grid so the page
        // chrome still renders.
        vec![&[]]
    } else {
        sheet.problems.chunks(per_page).collect()
    };

    // Flatten into a sequence of (problems, is_answer_key_page) tuples so we
    // can render problem pages first and answer pages at the end — each page
    // of problems gets a matching answer page.
    let mut page_sequence: Vec<(&[Vec<u32>], bool)> =
        pages.iter().map(|p| (*p, false)).collect();
    if chrome.include_answers {
        for page in &pages {
            page_sequence.push((*page, true));
        }
    }

    // Render each page's problem list + a worksheet-grid + optional pagebreak.
    let mut page_blocks = String::new();
    let total_page_count = page_sequence.len();
    let first_answer_idx = pages.len();
    let component_name = sheet.worksheet.component_typst_name();
    let opts_text = opts_body(&sheet.worksheet, &sheet.opts);

    for (i, (page, is_answer_page)) in page_sequence.iter().enumerate() {
        let problem_lines: String = page
            .iter()
            .map(|nums| {
                let inner: String = nums
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({inner})")
            })
            .collect::<Vec<_>>()
            .join(",\n  ");

        // On answer-key pages we force every problem into answer-only
        // mode. The `solve-first` knob only promotes cell 0 of the
        // very first problem page — paginated worksheets don't get a
        // fresh worked example on each page.
        let solve_first_here = chrome.solve_first && !*is_answer_page && i == 0;
        let modes = page_modes(*is_answer_page, solve_first_here, page.len());
        // Typst `(x,)` is a 1-tuple-as-array; `(,)` is a syntax error.
        // Emit `()` for empty so a zero-problem page produces valid source.
        let modes_arg = if modes.is_empty() {
            "()".to_string()
        } else {
            let inner = modes
                .iter()
                .map(|m| format!("\"{}\"", m.as_tag()))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({inner},)")
        };

        // PDF outline entries — sidebar bookmarks that `worksheet-page`
        // emits as suppressed-rendering headings. The preamble's show-rule
        // prevents visual output.
        let outline_key = if chrome.include_answers && i == 0 {
            "problems"
        } else if *is_answer_page && i == first_answer_idx {
            "answer-key"
        } else {
            ""
        };

        page_blocks.push_str(&format!(
            r#"#worksheet-page(
  (
  {problem_lines},
  ),
  {component_name},
  cols: {cols},
  debug: {debug_str},
  modes: {modes_arg},
  opts: ({opts_text}),
  outline: "{outline_key}",
)
"#
        ));
        if i + 1 < total_page_count {
            page_blocks.push_str("\n#pagebreak()\n\n");
        }
    }

    // PDF metadata — shows up in the reader's Document Properties panel
    // and gets indexed when the file is ingested by content systems.
    let doc_title = sheet.worksheet.title(chrome.solve_first);
    let doc_kind = sheet.worksheet.kind_slug();

    // Interpolate chrome dimensions from Rust constants.
    let margin_top = MARGINS_CM.top;
    let margin_bottom = MARGINS_CM.bottom;
    let margin_left = MARGINS_CM.left;
    let margin_right = MARGINS_CM.right;
    let header_ascent = HEADER_ASCENT_CM;
    let footer_descent = FOOTER_DESCENT_CM;
    let header_pad_top = HEADER_PAD_TOP_CM;
    let footer_pad_bottom = FOOTER_PAD_BOTTOM_CM;

    Ok(format!(
        r#"#import "/lib/header.typ": worksheet-header
#import "/lib/page.typ": worksheet-page
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font
// Problem components are passed to worksheet-grid by reference, so
// they must be in scope at the call site. Each worksheet has its own
// wrapper file under lib/problems/<folder>/ exposing a distinct alias
// even when two worksheets share the same underlying layout primitive.
#import "/lib/problems/addition/basic.typ": addition-basic-problem
#import "/lib/problems/subtraction/basic.typ": subtraction-basic-problem
#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#import "/lib/problems/multiplication/drill.typ": multiplication-drill-problem
#import "/lib/problems/division/simple.typ": division-simple-problem
#import "/lib/problems/division/long.typ": division-long-problem
#import "/lib/problems/division/drill.typ": division-drill-problem
#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem
#import "/lib/problems/fraction/simplification.typ": fraction-simplification-problem
#import "/lib/problems/algebra/two-step.typ": algebra-two-step-problem
#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#import "/lib/problems/algebra/square-root.typ": algebra-square-root-problem
#import "/lib/problems/fraction/equivalence.typ": fraction-equivalence-problem

#set document(
  title: "{doc_title}",
  author: "Pencil Ready",
  description: "Printable math worksheet — https://pencilready.com",
  keywords: ("math", "worksheet", "{doc_kind}", "pencilready.com"),
)

// Header (HEADER_HEIGHT_CM) and footer (FOOTER_HEIGHT_CM) render as
// page chrome via typst's page.header / page.footer callbacks, not
// body flow. Margin / ascent / descent / header-pad / footer-pad
// values are interpolated from Rust constants (MARGINS_CM,
// HEADER_ASCENT_CM, …) — single source of truth.
#set page(
  paper: "{paper_name}",
  margin: (top: {margin_top}cm, bottom: {margin_bottom}cm, left: {margin_left}cm, right: {margin_right}cm),
  header-ascent: {header_ascent}cm,
  footer-descent: {footer_descent}cm,
  header: pad(top: {header_pad_top}cm, worksheet-header(student-name: {student_name_arg}, debug: {debug_str})),
  footer: pad(bottom: {footer_pad_bottom}cm, worksheet-footer(pencil-ready-content, debug: {debug_str})),
)
#set text(font: body-font, size: 10pt)

// Headings exist only to populate the PDF outline (sidebar bookmarks)
// when --include-answers is used. Suppress visible rendering here — the
// worksheet-header already provides the on-page title area.
#show heading: _ => []

{page_blocks}"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_modes_plain_problem_page() {
        assert_eq!(
            page_modes(false, false, 3),
            vec![RenderMode::Blank, RenderMode::Blank, RenderMode::Blank],
        );
    }

    #[test]
    fn page_modes_solve_first_problem_page() {
        assert_eq!(
            page_modes(false, true, 3),
            vec![RenderMode::Worked, RenderMode::Blank, RenderMode::Blank],
        );
    }

    #[test]
    fn page_modes_answer_page_ignores_solve_first() {
        let expected = vec![
            RenderMode::AnswerOnly,
            RenderMode::AnswerOnly,
            RenderMode::AnswerOnly,
        ];
        assert_eq!(page_modes(true, false, 3), expected);
        assert_eq!(page_modes(true, true, 3), expected);
    }
}
