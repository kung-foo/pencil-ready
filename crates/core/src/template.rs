//! Shared .typ template rendering.
//!
//! Each worksheet type calls `render()` or `render_long_division()` with
//! its generated problems. The operator symbol can be overridden via
//! WorksheetParams for locale differences (e.g. : for division in Norway).

use crate::WorksheetParams;

/// Render a vertical-style worksheet (add, subtract, multiply, simple divide).
pub fn render(default_operator: &str, problems: &[Vec<u32>], params: &WorksheetParams) -> String {
    render_inner(default_operator, problems, params, "vertical")
}

/// Render a long-division-style worksheet.
pub fn render_long_division(problems: &[Vec<u32>], params: &WorksheetParams) -> String {
    // Operator is not used for long division (the bracket is the operator).
    render_inner("", problems, params, "long-division")
}

fn render_inner(
    default_operator: &str,
    problems: &[Vec<u32>],
    params: &WorksheetParams,
    style: &str,
) -> String {
    let operator = params.symbol.as_deref().unwrap_or(default_operator);

    let max_digits = problems
        .iter()
        .flat_map(|nums| nums.iter().map(|n| digit_count(*n)))
        .max()
        .unwrap_or(2);

    // Long division needs wider boxes (divisor + bracket + dividend + answer space).
    let box_width = if style == "long-division" {
        f64::max(3.0, max_digits as f64 * 0.6 + 1.2)
    } else {
        f64::max(2.2, max_digits as f64 * 0.55 + 0.6)
    };

    let debug_str = if params.debug { "true" } else { "false" };
    let cols = params.cols;
    let font = &params.font;
    let paper = &params.paper;

    let problem_lines: String = problems
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

    // Only include operator markup if we have one (long division doesn't).
    let operator_arg = if operator.is_empty() {
        "[]".to_string()
    } else {
        format!("[#{operator}]")
    };

    format!(
        r#"#import "/lib/header.typ": worksheet-header
#import "/lib/layout.typ": worksheet-grid
#import "/lib/footer.typ": worksheet-footer

#set page(paper: "{paper}", margin: (top: 1.5cm, bottom: 1.0cm, left: 1.5cm, right: 1.5cm))
#set text(font: "{font}", size: 10pt)

#worksheet-header(debug: {debug_str})

#worksheet-grid(
  (
  {problem_lines},
  ),
  {operator_arg},
  num-cols: {cols},
  width: {box_width}cm,
  debug: {debug_str},
  style: "{style}",
)

#worksheet-footer[Made with #box(height: 1.2em, baseline: 20%, image("/assets/rainbow-heart.svg")) in Oslo, 🇳🇴]
"#
    )
}

pub fn digit_count(n: u32) -> u32 {
    if n == 0 { 1 } else { n.ilog10() + 1 }
}
