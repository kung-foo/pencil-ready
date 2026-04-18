//! HTTP server for worksheet generation.
//!
//! GET /api/worksheets/{type}?format=pdf&seed=42&...
//!
//! Each endpoint uses two typed query extractors — `SharedParams` for the
//! cross-cutting knobs (format, seed, paper, etc.) and an endpoint-specific
//! struct for the worksheet-type params. Each derives `utoipa::IntoParams`,
//! so the OpenAPI spec at /openapi.json stays in sync with the code.
//! Swagger UI at /docs.

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Locale, OutputFormat, WorksheetParams, WorksheetType,
    generate,
};
use serde::Deserialize;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::{IntoParams, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    root: PathBuf,
}

// ---------------------------------------------------------------------------
// Shared params
// ---------------------------------------------------------------------------

/// Cross-cutting query params common to every worksheet endpoint. Extracted
/// via its own `Query<SharedParams>` on each handler; unknown fields are
/// silently ignored so the type-specific extractor can pick them up.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SharedParams {
    /// Output format (default: pdf).
    #[serde(default)]
    format: Option<OutputFormat>,
    /// Seed for reproducible output.
    #[serde(default)]
    seed: Option<u64>,
    /// Number of problems on the page (per-type default).
    #[serde(default)]
    problems: Option<u32>,
    /// Columns in the problem grid (per-type default).
    #[serde(default)]
    cols: Option<u32>,
    /// Pages (PDF only).
    #[serde(default)]
    pages: Option<u32>,
    /// Paper size passed to typst.
    #[serde(default)]
    #[param(example = "a4")]
    paper: Option<String>,
    /// Regional defaults for operator symbols.
    #[serde(default)]
    locale: Option<Locale>,
    /// Override the operator symbol (typst expression, e.g. "sym.colon").
    #[serde(default)]
    symbol: Option<String>,
    /// Render the first problem as a worked example.
    #[serde(default)]
    solve_first: Option<bool>,
    /// Draw debug borders around problem boxes and grid cells.
    #[serde(default)]
    debug: Option<bool>,
}

impl SharedParams {
    fn fold(
        self,
        worksheet: WorksheetType,
        default_problems: u32,
        default_cols: u32,
    ) -> (OutputFormat, WorksheetParams) {
        let format = self.format.unwrap_or(OutputFormat::Pdf);
        let params = WorksheetParams {
            worksheet,
            num_problems: self.problems.unwrap_or(default_problems),
            cols: self.cols.unwrap_or(default_cols),
            paper: self.paper.unwrap_or_else(|| "a4".into()),
            debug: self.debug.unwrap_or(false),
            seed: self.seed,
            symbol: self.symbol,
            locale: self.locale.unwrap_or_default(),
            pages: self.pages.unwrap_or(1),
            solve_first: self.solve_first.unwrap_or(false),
        };
        (format, params)
    }
}

// ---------------------------------------------------------------------------
// CSV parsing helpers (query strings use `a,b,c` rather than repeated keys)
// ---------------------------------------------------------------------------

fn parse_digits_csv(s: &str, field: &str) -> Result<Vec<DigitRange>> {
    s.split(',')
        .map(|p| DigitRange::from_str(p.trim()).map_err(|e| anyhow!("{field}: {e}")))
        .collect()
}

fn parse_u32_csv(s: &str, field: &str) -> Result<Vec<u32>> {
    s.split(',')
        .map(|p| p.trim().parse::<u32>().map_err(|e| anyhow!("{field}: {e}")))
        .collect()
}

fn parse_digit_range(s: &str, field: &str) -> Result<DigitRange> {
    DigitRange::from_str(s).map_err(|e| anyhow!("{field}: {e}"))
}

fn digits_or(opt: Option<String>, default: &[u32]) -> Result<Vec<DigitRange>> {
    match opt {
        Some(s) => parse_digits_csv(&s, "digits"),
        None => Ok(default.iter().copied().map(DigitRange::fixed).collect()),
    }
}

// ---------------------------------------------------------------------------
// Per-endpoint type-specific params + builders
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AddSpecific {
    /// Comma-separated digit counts per operand (e.g. "2,2" or "2-4,2-4").
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    #[serde(default)]
    carry: Option<CarryMode>,
    /// Binary mode: render in base 2.
    #[serde(default)]
    binary: Option<bool>,
}

impl AddSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Add {
                digits: digits_or(self.digits, &[2, 2])?,
                carry: self.carry.unwrap_or(CarryMode::Any),
                binary: self.binary.unwrap_or(false),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SubtractSpecific {
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    #[serde(default)]
    borrow: Option<BorrowMode>,
}

impl SubtractSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Subtract {
                digits: digits_or(self.digits, &[2, 2])?,
                borrow: self.borrow.unwrap_or(BorrowMode::Any),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct MultiplySpecific {
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
}

impl MultiplySpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Multiply {
                digits: digits_or(self.digits, &[2, 2])?,
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SimpleDivideSpecific {
    /// Max quotient (answer). 2-12.
    #[serde(default)]
    max_quotient: Option<u32>,
}

impl SimpleDivideSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::SimpleDivision {
                max_quotient: self.max_quotient.unwrap_or(10),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct LongDivideSpecific {
    /// Dividend digit count. "N" or "N-M", e.g. "3" or "2-4".
    #[serde(default)]
    #[param(value_type = String, example = "3")]
    digits: Option<String>,
    #[serde(default)]
    remainder: Option<bool>,
}

impl LongDivideSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let digits = match self.digits {
            Some(s) => parse_digit_range(&s, "digits")?,
            None => DigitRange::fixed(3),
        };
        Ok(shared.fold(
            WorksheetType::LongDivision {
                digits,
                remainder: self.remainder.unwrap_or(false),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct MultDrillSpecific {
    /// Which tables to drill, comma-separated. e.g. "2,3" or "1-10".
    #[serde(default)]
    #[param(value_type = String, example = "1-10")]
    multiplicand: Option<String>,
    /// Range of the other factor.
    #[serde(default)]
    #[param(value_type = String, example = "1-10")]
    multiplier: Option<String>,
    /// Problem count (0 = all problems from the enumerated tables).
    #[serde(default)]
    count: Option<u32>,
}

impl MultDrillSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let multiplicand = match self.multiplicand {
            Some(s) => parse_digits_csv(&s, "multiplicand")?,
            None => vec![DigitRange::new(1, 10)],
        };
        let multiplier = match self.multiplier {
            Some(s) => parse_digit_range(&s, "multiplier")?,
            None => DigitRange::new(1, 10),
        };
        let (fmt, mut params) = shared.fold(
            WorksheetType::MultiplicationDrill {
                multiplicand,
                multiplier,
            },
            0,
            3,
        );
        if let Some(c) = self.count {
            params.num_problems = c;
        }
        Ok((fmt, params))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DivDrillSpecific {
    #[serde(default)]
    #[param(value_type = String, example = "2-10")]
    divisor: Option<String>,
    #[serde(default)]
    #[param(value_type = String, example = "2-10")]
    max_quotient: Option<String>,
    #[serde(default)]
    count: Option<u32>,
}

impl DivDrillSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let divisor = match self.divisor {
            Some(s) => parse_digits_csv(&s, "divisor")?,
            None => vec![DigitRange::new(2, 10)],
        };
        let max_quotient = match self.max_quotient {
            Some(s) => parse_digit_range(&s, "max_quotient")?,
            None => DigitRange::new(2, 10),
        };
        let (fmt, mut params) = shared.fold(
            WorksheetType::DivisionDrill {
                divisor,
                max_quotient,
            },
            0,
            3,
        );
        if let Some(c) = self.count {
            params.num_problems = c;
        }
        Ok((fmt, params))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionMultSpecific {
    /// Allowed denominators, comma-separated.
    #[serde(default)]
    #[param(value_type = String, example = "2,3,4,5,10")]
    denominators: Option<String>,
    #[serde(default)]
    min_whole: Option<u32>,
    #[serde(default)]
    max_whole: Option<u32>,
    #[serde(default)]
    unit_only: Option<bool>,
}

impl FractionMultSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 3, 4, 5, 10],
        };
        Ok(shared.fold(
            WorksheetType::FractionMultiply {
                denominators,
                min_whole: self.min_whole.unwrap_or(2),
                max_whole: self.max_whole.unwrap_or(20),
                unit_only: self.unit_only.unwrap_or(false),
            },
            12,
            3,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AlgebraTwoStepSpecific {
    #[serde(default)]
    #[param(value_type = String, example = "2-10")]
    a_range: Option<String>,
    #[serde(default)]
    #[param(value_type = String, example = "1-30")]
    b_range: Option<String>,
    #[serde(default)]
    #[param(value_type = String, example = "0-20")]
    x_range: Option<String>,
    /// Variable glyph (single character).
    #[serde(default)]
    variable: Option<String>,
    /// Render coefficient-variable as `4x` (no operator).
    #[serde(default)]
    implicit: Option<bool>,
    #[serde(default)]
    mix_forms: Option<bool>,
}

impl AlgebraTwoStepSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let a_range = match self.a_range {
            Some(s) => parse_digit_range(&s, "a_range")?,
            None => DigitRange::new(2, 10),
        };
        let b_range = match self.b_range {
            Some(s) => parse_digit_range(&s, "b_range")?,
            None => DigitRange::new(1, 30),
        };
        let x_range = match self.x_range {
            Some(s) => parse_digit_range(&s, "x_range")?,
            None => DigitRange::new(0, 20),
        };
        Ok(shared.fold(
            WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                variable: self.variable.unwrap_or_else(|| "x".into()),
                implicit: self.implicit.unwrap_or(false),
                mix_forms: self.mix_forms.unwrap_or(true),
            },
            6,
            2,
        ))
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(root: &Path, built: Result<(OutputFormat, WorksheetParams)>) -> Response {
    let (format, params) = match built {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response(),
    };
    match generate(&params, format, root) {
        Ok(ws) => {
            let (ct, ext) = match format {
                OutputFormat::Pdf => ("application/pdf", "pdf"),
                OutputFormat::Png => ("image/png", "png"),
                OutputFormat::Svg => ("image/svg+xml", "svg"),
            };
            // `inline` (not `attachment`) so browsers show a preview; the
            // filename hint is still used when the user chooses Save As.
            let filename = format!("pencil-ready-{}.{ext}", params.slug());
            let disposition = format!("inline; filename=\"{filename}\"");
            (
                [
                    (header::CONTENT_TYPE, ct.to_string()),
                    (header::CONTENT_DISPOSITION, disposition),
                ],
                ws.bytes,
            )
                .into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/worksheets/add",
    params(SharedParams, AddSpecific),
    responses((status = 200, description = "Worksheet bytes (PDF/PNG/SVG)")),
    tag = "worksheets",
)]
async fn handle_add(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AddSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/subtract",
    params(SharedParams, SubtractSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_subtract(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<SubtractSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/multiply",
    params(SharedParams, MultiplySpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_multiply(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<MultiplySpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/simple-divide",
    params(SharedParams, SimpleDivideSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_simple_divide(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<SimpleDivideSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/long-divide",
    params(SharedParams, LongDivideSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_long_divide(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<LongDivideSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/mult-drill",
    params(SharedParams, MultDrillSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_mult_drill(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<MultDrillSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/div-drill",
    params(SharedParams, DivDrillSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_div_drill(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<DivDrillSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-mult",
    params(SharedParams, FractionMultSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_fraction_mult(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionMultSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

#[utoipa::path(
    get,
    path = "/api/worksheets/algebra-two-step",
    params(SharedParams, AlgebraTwoStepSpecific),
    responses((status = 200, description = "Worksheet bytes")),
    tag = "worksheets",
)]
async fn handle_algebra_two_step(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AlgebraTwoStepSpecific>,
) -> Response {
    render(&s.root, p.build(shared))
}

// ---------------------------------------------------------------------------
// OpenAPI + entry
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pencil Ready",
        version = "0.1.0",
        description = "Generate printable math worksheets.",
    ),
    tags((name = "worksheets", description = "Worksheet generation endpoints")),
    components(schemas(CarryMode, BorrowMode, Locale, OutputFormat)),
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let root = std::env::var("PENCIL_READY_ROOT").unwrap_or_else(|_| ".".into());
    let root = PathBuf::from(root)
        .canonicalize()
        .expect("canonicalize project root (set PENCIL_READY_ROOT or cd to repo)");
    let state = Arc::new(AppState { root });

    let compression = CompressionLayer::new().gzip(true).br(true);

    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(handle_add))
        .routes(routes!(handle_subtract))
        .routes(routes!(handle_multiply))
        .routes(routes!(handle_simple_divide))
        .routes(routes!(handle_long_divide))
        .routes(routes!(handle_mult_drill))
        .routes(routes!(handle_div_drill))
        .routes(routes!(handle_fraction_mult))
        .routes(routes!(handle_algebra_two_step))
        .with_state(state)
        .split_for_parts();

    // SwaggerUi::url("/openapi.json", api) both serves the spec at that
    // path and points the UI at it — no separate route needed.
    let mut app = axum::Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", api))
        .merge(api_router);

    // In production the SPA bundle lives at PENCIL_READY_STATIC_DIR (set by
    // the Dockerfile). ServeDir serves real files; not_found_service falls
    // back to index.html so client-side routing works for any deep link.
    // In dev we skip this entirely and let Vite serve the frontend.
    app = match std::env::var("PENCIL_READY_STATIC_DIR") {
        Ok(dir) => {
            let dir = PathBuf::from(dir);
            let index = dir.join("index.html");
            let serve = ServeDir::new(&dir).not_found_service(ServeFile::new(&index));
            println!("serving SPA from {}", dir.display());
            app.fallback_service(serve)
        }
        Err(_) => {
            // No SPA bundle available — expose a cheap root for liveness.
            app.route("/", axum::routing::get(|| async { "hello world\n" }))
        }
    };

    let app = app.layer(compression);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{port}");
    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
