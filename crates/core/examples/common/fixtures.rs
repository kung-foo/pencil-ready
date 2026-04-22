//! Shared fixture list for the `measure_cells` and `summary_sheet`
//! example binaries. Lives in a subdirectory so Cargo doesn't treat it
//! as an example itself; both examples include it via
//! `#[path = "common/fixtures.rs"] mod fixtures;`.

pub struct Fixture {
    /// Stable slug used as the lookup key in cell-sizes.toml and as
    /// the per-fixture PNG filename.
    pub id: &'static str,
    /// Human-readable description.
    pub label: &'static str,
    /// typst expression calling a problem component. Evaluates to a
    /// layout element; the per-tool preamble imports the components.
    pub snippet: &'static str,
}

// Vertical and long-division take an explicit `width`; their internal
// grids use a 1fr column that collapses to 0 under `width: auto`, so
// we pass widths matching template.rs's current formula. Components
// without a `width` parameter size themselves naturally.
//
// Vertical widths:   max(2.2, max_digits * 0.55 + 0.6) cm
// Long-div widths:   max(3.0, max_digits * 0.6  + 1.2) cm
pub const FIXTURES: &[Fixture] = &[
    // --- vertical (add / subtract / simple-div) ---
    Fixture { id: "vertical-add-d2-op2-blank",  label: "vertical 2-digit × 2-operand (add, blank)",  snippet: r#"vertical-problem((12, 34, 46),          [#sym.plus], width: 2.2cm,  answer-rows: 1)"# },
    Fixture { id: "vertical-add-d3-op2-blank",  label: "vertical 3-digit × 2-operand (add, blank)",  snippet: r#"vertical-problem((123, 456, 579),       [#sym.plus], width: 2.25cm, answer-rows: 1)"# },
    Fixture { id: "vertical-add-d4-op2-blank",  label: "vertical 4-digit × 2-operand (add, blank)",  snippet: r#"vertical-problem((1234, 5678, 6912),    [#sym.plus], width: 2.8cm,  answer-rows: 1)"# },
    Fixture { id: "vertical-add-d5-op2-blank",  label: "vertical 5-digit × 2-operand (add, blank)",  snippet: r#"vertical-problem((12345, 67890, 80235), [#sym.plus], width: 3.35cm, answer-rows: 1)"# },
    Fixture { id: "vertical-add-d2-op3-blank",  label: "vertical 2-digit × 3-operand (add, blank)",  snippet: r#"vertical-problem((12, 34, 56, 102),     [#sym.plus], width: 2.2cm,  answer-rows: 1)"# },
    Fixture { id: "vertical-add-d3-op2-worked", label: "vertical 3-digit × 2-operand (add, worked)", snippet: r#"vertical-problem((123, 456, 579),       [#sym.plus], width: 2.25cm, answer-rows: 1, solved: true)"# },

    // --- vertical multiply (answer-rows = N+1 partials+sum) ---
    Fixture { id: "multiply-d2x2-blank", label: "multiply 2×2-digit (blank)", snippet: r#"vertical-problem((23, 45, 1035),   [#sym.times], width: 2.2cm,  answer-rows: 3)"# },
    Fixture { id: "multiply-d3x2-blank", label: "multiply 3×2-digit (blank)", snippet: r#"vertical-problem((123, 45, 5535),  [#sym.times], width: 2.25cm, answer-rows: 3)"# },
    Fixture { id: "multiply-d3x3-blank", label: "multiply 3×3-digit (blank)", snippet: r#"vertical-problem((123, 456, 56088),[#sym.times], width: 2.25cm, answer-rows: 4)"# },

    // --- long division (answer-rows = ~2 × dividend digits) ---
    Fixture { id: "long-div-d2-blank", label: "long-div 2-digit dividend (blank)", snippet: r#"long-division-problem((96, 4, 24),     width: 3.0cm, answer-rows: 4)"# },
    Fixture { id: "long-div-d3-blank", label: "long-div 3-digit dividend (blank)", snippet: r#"long-division-problem((375, 5, 75),    width: 3.0cm, answer-rows: 6)"# },
    Fixture { id: "long-div-d4-blank", label: "long-div 4-digit dividend (blank)", snippet: r#"long-division-problem((8192, 8, 1024), width: 3.6cm, answer-rows: 8)"# },

    // --- horizontal drill ---
    Fixture { id: "horizontal-drill-d1x1", label: "horizontal drill 1×1-digit (mult table)", snippet: r#"horizontal-problem((7, 3, 21),  [#sym.times])"# },
    Fixture { id: "horizontal-drill-d2x1", label: "horizontal drill 2×1-digit",              snippet: r#"horizontal-problem((12, 7, 84), [#sym.times])"# },

    // --- horizontal fraction ---
    Fixture { id: "fraction-unit-d2",     label: "horizontal fraction (unit, 2-digit whole)",     snippet: r#"horizontal-fraction-problem((12, 1, 2), [#sym.times])"# },
    Fixture { id: "fraction-non-unit-d2", label: "horizontal fraction (non-unit, 2-digit whole)", snippet: r#"horizontal-fraction-problem((15, 2, 3), [#sym.times])"# },

    // --- algebra two-step ---
    Fixture { id: "algebra-small-form0", label: "algebra two-step (small, form 0)", snippet: r#"algebra-two-step-problem((4, 5, 4, 21, 0),    [#sym.dot.op])"# },
    Fixture { id: "algebra-large-form0", label: "algebra two-step (large, form 0)", snippet: r#"algebra-two-step-problem((12, 87, 3, 123, 0), [#sym.dot.op])"# },
];

/// Preamble with the typst imports every fixture needs, plus a
/// caller-supplied additional preamble line (e.g. page/text setup).
pub fn preamble() -> &'static str {
    r##"#import "/lib/problems/vertical.typ": vertical-problem
#import "/lib/problems/horizontal.typ": horizontal-problem
#import "/lib/problems/horizontal-fraction.typ": horizontal-fraction-problem
#import "/lib/problems/long-division.typ": long-division-problem
#import "/lib/problems/algebra-two-step.typ": algebra-two-step-problem
#import "/lib/problems/shared.typ": body-font
"##
}

/// Small fixed padding applied around every rendered problem. Gives
/// content breathing room against its container border and makes the
/// padding behavior consistent across components. Applied at display
/// time until the components self-pad (LAYOUT_REFACTOR step 3).
pub const CELL_PADDING_CM: f32 = 0.15;

/// Wrap a raw component-call snippet in the standard cell padding.
/// Returns a typst expression that evaluates to the padded content.
pub fn padded(snippet: &str) -> String {
    format!("pad({}cm, {})", CELL_PADDING_CM, snippet)
}

/// Ceil to the next 0.1cm (1mm) so declared cell sizes carry a small
/// safety margin over the measured natural size. Shared by both the
/// measurement tool and the summary sheet renderer.
pub fn ceil_mm(x: f32) -> f32 {
    (x * 10.0).ceil() / 10.0
}
