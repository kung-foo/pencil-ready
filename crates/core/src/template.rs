//! Shared .typ template rendering.

use anyhow::{Result, bail};

use crate::WorksheetParams;

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
            "couldn't generate enough unique problems: asked for {expected} \
             (num_problems={} × pages={}) but only got {}. \
             Reduce --pages or widen the problem space.",
            params.num_problems,
            params.pages,
            problems.len()
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
    let solve_first_str = if params.solve_first { "true" } else { "false" };
    let implicit_str = if implicit { "true" } else { "false" };
    let cols = params.cols;
    let paper = &params.paper;

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

    // Render each page's problem list + a worksheet-grid + optional pagebreak.
    let mut page_blocks = String::new();
    for (i, page) in pages.iter().enumerate() {
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

        page_blocks.push_str(&format!(
            r#"#worksheet-header(debug: {debug_str})

#worksheet-grid(
  (
  {problem_lines},
  ),
  {operator_arg},
  num-cols: {cols},
  width: {box_width}cm,
  debug: {debug_str},
  style: "{style}",
  answer-rows: {answer_rows},
  solve-first: {solve_first_str},
  implicit: {implicit_str},
  variable: "{variable}",
  pad-width: {pad_width},
)

#worksheet-footer(pencil-ready-content)
"#
        ));
        if i + 1 < pages.len() {
            page_blocks.push_str("\n#pagebreak()\n\n");
        }
    }

    Ok(format!(
        r#"#import "/lib/header.typ": worksheet-header
#import "/lib/layout.typ": worksheet-grid
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font

#set page(paper: "{paper}", margin: (top: 1.5cm, bottom: 1.0cm, left: 1.5cm, right: 1.5cm))
#set text(font: body-font, size: 10pt)

{page_blocks}"#
    ))
}

pub fn digit_count(n: u32) -> u32 {
    if n == 0 { 1 } else { n.ilog10() + 1 }
}
