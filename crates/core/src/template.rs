//! Shared .typ template rendering.

use anyhow::{Result, bail};

use crate::{RenderMode, WorksheetParams};

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

/// Render a vertical-style worksheet (add, subtract, multiply, simple divide).
pub fn render(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    answer_rows: u32,
) -> Result<String> {
    render_inner(default_operator, problems, params, "vertical", answer_rows)
}

/// Vertical worksheet with a fixed operand display-width (left-pad with
/// zeros up to `pad_width` characters). Used by binary addition so each
/// operand fills its full bit width.
pub fn render_padded(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    answer_rows: u32,
    pad_width: u32,
) -> Result<String> {
    render_inner_with_pad(
        default_operator, problems, params, "vertical", answer_rows, false, "x", pad_width,
    )
}

/// Render a horizontal-style worksheet (drills: A × B = ___).
pub fn render_horizontal(default_operator: &str, problems: &[Vec<u32>], params: &WorksheetParams) -> Result<String> {
    render_inner(default_operator, problems, params, "horizontal", 1)
}

/// Render a horizontal fraction worksheet (whole × num/den = ___).
pub fn render_horizontal_fraction(default_operator: &str, problems: &[Vec<u32>], params: &WorksheetParams) -> Result<String> {
    render_inner_full(default_operator, problems, params, "horizontal-fraction", 1, false, "x")
}

/// Render an algebra two-step worksheet (ax + b = c, solve for x).
pub fn render_algebra_two_step(default_operator: &str, problems: &[Vec<u32>], params: &WorksheetParams, implicit: bool, variable: &str) -> Result<String> {
    render_inner_full(default_operator, problems, params, "algebra-two-step", 1, implicit, variable)
}

/// Render a long-division-style worksheet.
pub fn render_long_division(
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    answer_rows: u32,
) -> Result<String> {
    render_inner("", problems, params, "long-division", answer_rows)
}

fn render_inner(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    style: &str,
    answer_rows: u32,
) -> Result<String> {
    render_inner_full(default_operator, problems, params, style, answer_rows, false, "x")
}

fn render_inner_full(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    style: &str,
    answer_rows: u32,
    implicit: bool,
    variable: &str,
) -> Result<String> {
    render_inner_with_pad(default_operator, problems, params, style, answer_rows, implicit, variable, 0)
}

fn render_inner_with_pad(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    style: &str,
    answer_rows: u32,
    implicit: bool,
    variable: &str,
    pad_width: u32,
) -> Result<String> {
    let expected = params.total_problems() as usize;
    // Drills with num_problems=0 allow any count. Others must match exactly.
    if params.num_problems > 0 && problems.len() < expected {
        bail!(
            "no valid problems for the given constraints — the combination \
             of digits / carry / borrow / mode rules out every candidate. \
             Widen at least one parameter."
        );
    }

    let operator = params.symbol.as_deref().unwrap_or(default_operator);

    let max_digits = problems
        .iter()
        .flat_map(|nums| nums.iter().map(|n| digit_count(*n)))
        .max()
        .unwrap_or(2);

    let box_width = match style {
        "long-division" => f64::max(3.0, max_digits as f64 * 0.6 + 1.2),
        "horizontal" => f64::max(6.0, max_digits as f64 * 1.2 + 4.0),
        // horizontal-fraction: width is computed by the component itself,
        // but we still need to provide something to the grid.
        "horizontal-fraction" => 6.0,
        "algebra-two-step" => 6.0,
        _ => f64::max(2.2, max_digits as f64 * 0.55 + 0.6),
    };

    let debug_str = if params.debug { "true" } else { "false" };
    let implicit_str = if implicit { "true" } else { "false" };
    let cols = params.cols;
    let paper = &params.paper;

    // Header student name: either `none` or a typst string literal.
    // Escape backslashes and double quotes so arbitrary UTF-8 names drop
    // straight into the generated .typ source.
    let student_name_arg = header_name_arg(params.student_name.as_deref());

    // Chunk problems across pages.
    let per_page = if params.num_problems > 0 {
        params.num_problems as usize
    } else {
        problems.len() // drills with num_problems=0: all on one page
    };
    let pages: Vec<&[Vec<u32>]> = problems.chunks(per_page).collect();

    // Only include operator markup if we have one (long division doesn't).
    let operator_arg = if operator.is_empty() {
        "[]".to_string()
    } else {
        format!("[#{operator}]")
    };

    // Flatten into a sequence of (problems, is_answer_key_page) tuples so we
    // can render problem pages first and answer pages at the end — each page
    // of problems gets a matching answer page.
    let mut page_sequence: Vec<(&[Vec<u32>], bool)> =
        pages.iter().map(|p| (*p, false)).collect();
    if params.include_answers {
        for page in &pages {
            page_sequence.push((*page, true));
        }
    }

    // Render each page's problem list + a worksheet-grid + optional pagebreak.
    let mut page_blocks = String::new();
    let total_page_count = page_sequence.len();
    // Index of the first answer page (== problems.len() when include_answers
    // is true; unused otherwise). Used to attach "Answer Key" outline entry.
    let first_answer_idx = pages.len();
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
        // mode (just the numeric answer, no partial products or worked
        // steps). The `solve-first` knob is respected only on problem
        // pages, where it promotes problem 0 to a worked example.
        let modes = page_modes(*is_answer_page, params.solve_first, page.len());
        // Typst tuple-vs-array: `(x,)` is a 1-tuple-as-array; `(,)` is a
        // syntax error. Emit `()` for the empty case so a zero-problem
        // page (unreachable today, but a cheap footgun to defuse)
        // produces valid source.
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

        // PDF outline entries: when the document has both a problems section
        // and an answer-key section, emit a heading at the top of each to
        // produce sidebar bookmarks. The preamble's show-rule suppresses the
        // visual rendering — only the PDF-outline entry remains.
        let outline_heading = if params.include_answers && i == 0 {
            "#heading(outlined: true, bookmarked: true, level: 1)[Problems]\n"
        } else if *is_answer_page && i == first_answer_idx {
            "#heading(outlined: true, bookmarked: true, level: 1)[Answer Key]\n"
        } else {
            ""
        };

        // Component name is per-worksheet (e.g. addition-basic-problem,
        // multiplication-drill-problem) — the wrapper files under
        // lib/problems/<folder>/ expose distinct aliases even when two
        // worksheets share the same underlying layout primitive.
        //
        // opts keys remain layout-shaped, since vertical-stack vs
        // horizontal-inline vs each one-off layout read different keys.
        let component_name = params.worksheet.component_typst_name();
        let opts_body = match style {
            "vertical" => format!(
                "operator: {operator_arg}, width: {box_width}cm, answer-rows: {answer_rows}, pad-width: {pad_width}"
            ),
            "long-division" => format!(
                "width: {box_width}cm, answer-rows: {answer_rows}"
            ),
            "horizontal" => format!("operator: {operator_arg}"),
            "horizontal-fraction" => format!("operator: {operator_arg}"),
            "algebra-two-step" => format!(
                "operator: {operator_arg}, implicit: {implicit_str}, variable: \"{variable}\""
            ),
            other => bail!("unknown worksheet style: {other}"),
        };

        page_blocks.push_str(&format!(
            r#"{outline_heading}#worksheet-grid(
  (
  {problem_lines},
  ),
  {component_name},
  num-cols: {cols},
  debug: {debug_str},
  modes: {modes_arg},
  opts: ({opts_body}),
)
"#
        ));
        if i + 1 < total_page_count {
            page_blocks.push_str("\n#pagebreak()\n\n");
        }
    }

    // PDF metadata — shows up in the reader's Document Properties panel
    // and gets indexed when the file is ingested by content systems.
    let doc_title = params.title();
    let doc_kind = params.kind_slug();
    Ok(format!(
        r#"#import "/lib/header.typ": worksheet-header
#import "/lib/layout.typ": worksheet-grid
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
#import "/lib/problems/algebra/two-step.typ": algebra-two-step-problem

#set document(
  title: "{doc_title}",
  author: "Pencil Ready",
  description: "Printable math worksheet — https://pencilready.com",
  keywords: ("math", "worksheet", "{doc_kind}", "pencilready.com"),
)

// Header (1.5cm) and footer (0.8cm) are rendered as page chrome via
// typst's page.header/footer callbacks, not body flow. That keeps
// them pinned to the top/bottom margin bands regardless of how much
// the grid fills. Top/bottom margins include the chrome height plus
// a small breathing band (ascent/descent).
#set page(
  paper: "{paper}",
  margin: (top: 3.2cm, bottom: 2.2cm, left: 1.5cm, right: 1.5cm),
  header-ascent: 0.8cm,
  footer-descent: 0.4cm,
  header: pad(top: 0.7cm, worksheet-header(student-name: {student_name_arg}, debug: {debug_str})),
  footer: pad(bottom: 0.7cm, worksheet-footer(pencil-ready-content, debug: {debug_str})),
)
#set text(font: body-font, size: 10pt)

// Headings exist only to populate the PDF outline (sidebar bookmarks)
// when --include-answers is used. Suppress visible rendering here — the
// worksheet-header already provides the on-page title area.
#show heading: _ => []

{page_blocks}"#
    ))
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
