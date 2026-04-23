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
    // --- addition (vertical-stack primitive) ---
    Fixture { id: "addition-basic-d2-op2-blank",  label: "addition 2-digit × 2-operand (blank)",  snippet: r#"addition-basic-problem((12, 34, 46),          opts: (operator: [#sym.plus], width: 2.2cm,  answer-rows: 1))"# },
    Fixture { id: "addition-basic-d3-op2-blank",  label: "addition 3-digit × 2-operand (blank)",  snippet: r#"addition-basic-problem((123, 456, 579),       opts: (operator: [#sym.plus], width: 2.25cm, answer-rows: 1))"# },
    Fixture { id: "addition-basic-d4-op2-blank",  label: "addition 4-digit × 2-operand (blank)",  snippet: r#"addition-basic-problem((1234, 5678, 6912),    opts: (operator: [#sym.plus], width: 2.8cm,  answer-rows: 1))"# },
    Fixture { id: "addition-basic-d5-op2-blank",  label: "addition 5-digit × 2-operand (blank)",  snippet: r#"addition-basic-problem((12345, 67890, 80235), opts: (operator: [#sym.plus], width: 3.35cm, answer-rows: 1))"# },
    Fixture { id: "addition-basic-d2-op3-blank",  label: "addition 2-digit × 3-operand (blank)",  snippet: r#"addition-basic-problem((12, 34, 56, 102),     opts: (operator: [#sym.plus], width: 2.2cm,  answer-rows: 1))"# },
    Fixture { id: "addition-basic-d3-op2-worked", label: "addition 3-digit × 2-operand (worked)", snippet: r#"addition-basic-problem((123, 456, 579),       mode: "worked", opts: (operator: [#sym.plus], width: 2.25cm, answer-rows: 1))"# },

    // --- multiplication (vertical-stack primitive; answer-rows = N+1 partials+sum) ---
    Fixture { id: "multiplication-basic-d2x2-blank", label: "multiplication 2×2-digit (blank)", snippet: r#"multiplication-basic-problem((23, 45, 1035),   opts: (operator: [#sym.times], width: 2.2cm,  answer-rows: 3))"# },
    Fixture { id: "multiplication-basic-d3x2-blank", label: "multiplication 3×2-digit (blank)", snippet: r#"multiplication-basic-problem((123, 45, 5535),  opts: (operator: [#sym.times], width: 2.25cm, answer-rows: 3))"# },
    Fixture { id: "multiplication-basic-d3x3-blank", label: "multiplication 3×3-digit (blank)", snippet: r#"multiplication-basic-problem((123, 456, 56088),opts: (operator: [#sym.times], width: 2.25cm, answer-rows: 4))"# },

    // --- long division (answer-rows = ~2 × dividend digits) ---
    Fixture { id: "division-long-d2-blank", label: "long division, 2-digit dividend (blank)", snippet: r#"division-long-problem((96, 4, 24),     opts: (width: 3.0cm, answer-rows: 4))"# },
    Fixture { id: "division-long-d3-blank", label: "long division, 3-digit dividend (blank)", snippet: r#"division-long-problem((375, 5, 75),    opts: (width: 3.0cm, answer-rows: 6))"# },
    Fixture { id: "division-long-d4-blank", label: "long division, 4-digit dividend (blank)", snippet: r#"division-long-problem((8192, 8, 1024), opts: (width: 3.6cm, answer-rows: 8))"# },

    // --- multiplication drill (horizontal-inline primitive) ---
    Fixture { id: "multiplication-drill-d1x1", label: "multiplication drill 1×1-digit (mult table)", snippet: r#"multiplication-drill-problem((7, 3, 21),  opts: (operator: [#sym.times]))"# },
    Fixture { id: "multiplication-drill-d2x1", label: "multiplication drill 2×1-digit",              snippet: r#"multiplication-drill-problem((12, 7, 84), opts: (operator: [#sym.times]))"# },

    // --- fraction multiplication ---
    Fixture { id: "fraction-multiplication-unit-d2",     label: "fraction multiplication (unit, 2-digit whole)",     snippet: r#"fraction-multiplication-problem((12, 1, 2), opts: (operator: [#sym.times]))"# },
    Fixture { id: "fraction-multiplication-non-unit-d2", label: "fraction multiplication (non-unit, 2-digit whole)", snippet: r#"fraction-multiplication-problem((15, 2, 3), opts: (operator: [#sym.times]))"# },

    // --- algebra two-step ---
    Fixture { id: "algebra-two-step-small-form0", label: "algebra two-step (small, form 0)", snippet: r#"algebra-two-step-problem((4, 5, 4, 21, 0),    opts: (operator: [#sym.dot.op]))"# },
    Fixture { id: "algebra-two-step-large-form0", label: "algebra two-step (large, form 0)", snippet: r#"algebra-two-step-problem((12, 87, 3, 123, 0), opts: (operator: [#sym.dot.op]))"# },
];

/// Preamble with the typst imports every fixture needs, plus a
/// caller-supplied additional preamble line (e.g. page/text setup).
pub fn preamble() -> &'static str {
    r##"#import "/lib/problems/addition/basic.typ": addition-basic-problem
#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#import "/lib/problems/multiplication/drill.typ": multiplication-drill-problem
#import "/lib/problems/division/long.typ": division-long-problem
#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem
#import "/lib/problems/algebra/two-step.typ": algebra-two-step-problem
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
