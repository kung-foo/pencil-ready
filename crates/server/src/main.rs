//! HTTP server for worksheet generation.
//!
//! GET /api/worksheets/{type}?format=pdf&seed=42&...
//!
//! Each endpoint uses two typed query extractors — `SharedParams` for the
//! cross-cutting knobs (format, seed, paper, etc.) and an endpoint-specific
//! struct for the worksheet-type params. Each derives `utoipa::IntoParams`,
//! so the OpenAPI spec at /openapi.json stays in sync with the code.
//! Swagger UI at /docs.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use axum::{
    body::Body,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use clap::Parser;
use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Fonts, Locale, OutputFormat, WorksheetParams, WorksheetType,
    generate,
};
use serde::Deserialize;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use utoipa::{IntoParams, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    root: PathBuf,
    /// Short region slug for the serving machine (e.g. `arn`, `sjc`).
    /// Populated from `FLY_REGION` at boot; `None` for local dev. Both
    /// the structured log and the PDF footer include it.
    region: Option<String>,
    /// Fonts parsed once at startup. Shared (via `Arc`) with every
    /// compile — avoids re-reading `fonts/` from disk per request.
    fonts: Fonts,
    /// Force `debug: true` on every worksheet render regardless of
    /// query params — turns the red/blue layout-debug borders on for
    /// every browser-facing render. Set via `--debug` / `PENCIL_READY_DEBUG`.
    force_debug: bool,
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
    /// Append an answer-key page showing just the final answers (PDF only).
    #[serde(default)]
    include_answers: Option<bool>,
    /// Draw debug borders around problem boxes and grid cells.
    #[serde(default)]
    debug: Option<bool>,
    /// Pre-fill the student name on the header in a handwriting font.
    #[serde(default)]
    student_name: Option<String>,
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
            include_answers: self.include_answers.unwrap_or(false),
            student_name: self.student_name.filter(|s| !s.is_empty()),
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

fn render(
    state: &AppState,
    endpoint: &'static str,
    built: Result<(OutputFormat, WorksheetParams)>,
    headers: &HeaderMap,
) -> Response {
    let ip = client_ip(headers);
    let ua = user_agent(headers);

    let (format, mut params) = match built {
        Ok(p) => p,
        Err(e) => {
            warn!(
                endpoint,
                region = %region_display(&state.region),
                ip = %ip,
                ua = %ua,
                error = %e,
                "worksheet request rejected"
            );
            return (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response();
        }
    };
    if state.force_debug {
        params.debug = true;
    }

    let start = Instant::now();
    let result = generate(&params, format, &state.root, &state.fonts);
    let typst_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(ws) => {
            info!(
                kind = params.kind_slug(),
                format = format_slug(format),
                region = %region_display(&state.region),
                num_problems = params.num_problems,
                cols = params.cols,
                pages = params.pages,
                paper = %params.paper,
                seed = params.seed,
                solve_first = params.solve_first,
                include_answers = params.include_answers,
                bytes = ws.bytes.len(),
                typst_ms,
                ip = %ip,
                ua = %ua,
                "worksheet rendered"
            );

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
        Err(e) => {
            warn!(
                kind = params.kind_slug(),
                format = format_slug(format),
                region = %region_display(&state.region),
                typst_ms,
                ip = %ip,
                ua = %ua,
                error = %e,
                "worksheet generation failed"
            );
            (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response()
        }
    }
}

/// Resolve the real client IP. Behind Fly's edge proxy, `Fly-Client-IP`
/// is the canonical source; `X-Forwarded-For` is a fallback for generic
/// reverse proxies; otherwise we don't know.
fn client_ip(headers: &HeaderMap) -> String {
    if let Some(v) = headers.get("fly-client-ip").and_then(|v| v.to_str().ok()) {
        return v.to_string();
    }
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            return first.trim().to_string();
        }
    }
    "-".to_string()
}

fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string()
}

fn format_slug(f: OutputFormat) -> &'static str {
    match f {
        OutputFormat::Pdf => "pdf",
        OutputFormat::Png => "png",
        OutputFormat::Svg => "svg",
    }
}

fn region_display(region: &Option<String>) -> &str {
    region.as_deref().unwrap_or("local")
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
    headers: HeaderMap,
) -> Response {
    render(&s, "add", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "subtract", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "multiply", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "simple-divide", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "long-divide", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "mult-drill", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "div-drill", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-mult", p.build(shared), &headers)
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
    headers: HeaderMap,
) -> Response {
    render(&s, "algebra-two-step", p.build(shared), &headers)
}

/// Text substitution middleware: replaces `SERVER_REGION_PLACEHOLDER`
/// inside `text/html` response bodies with a `(server:<region>)` tag (or
/// an empty string for local dev). Layered *inside* compression so the
/// body is still plaintext when we rewrite it.
async fn region_rewrite(
    State(s): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let response = next.run(req).await;

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("text/html"));
    if !is_html {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    let replacement = match s.region.as_deref() {
        Some(r) => format!("(server:{r})"),
        None => String::new(),
    };

    // Bail fast if the marker isn't present — most non-page HTML won't
    // have it, and we save an allocation.
    let haystack = match std::str::from_utf8(&bytes) {
        Ok(s) if s.contains("SERVER_REGION_PLACEHOLDER") => s,
        _ => return Response::from_parts(parts, Body::from(bytes)),
    };
    let rewritten = haystack.replace("SERVER_REGION_PLACEHOLDER", &replacement);
    // Content-Length no longer matches; drop it so the transport layer
    // re-computes or switches to chunked.
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(rewritten))
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

#[derive(Parser)]
#[command(name = "pencil-ready-server", about = "Pencil Ready worksheet server")]
struct Cli {
    /// Listen port.
    #[arg(long, default_value_t = 8080, env = "PORT")]
    port: u16,
    /// Project root for typst imports (lib/, fonts/, assets/).
    #[arg(long, default_value = ".", env = "PENCIL_READY_ROOT")]
    root: PathBuf,
    /// Static-content directory. If omitted, defaults to
    /// `<root>/frontend/astro/dist`; the server runs API-only if that
    /// path doesn't exist and `--static-dir` isn't overridden.
    #[arg(long, env = "PENCIL_READY_STATIC_DIR")]
    static_dir: Option<PathBuf>,
    /// Run API-only, without serving any static bundle.
    #[arg(long)]
    api_only: bool,
    /// Force `debug: true` on every worksheet render — useful for visual
    /// layout debugging via the browser. Off by default; never set in
    /// production.
    #[arg(long, env = "PENCIL_READY_DEBUG")]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing();

    let root = cli
        .root
        .canonicalize()
        .expect("canonicalize project root (set --root or cd to the repo)");
    // FLY_REGION is set on every Fly machine; absent locally.
    let region = std::env::var("FLY_REGION").ok().filter(|s| !s.is_empty());
    // Parse all bundled fonts once; clone the Arcs per request.
    let fonts = Fonts::load(&root).expect("load fonts from <root>/fonts");
    info!(region = %region_display(&region), "pencil-ready-server starting");
    if cli.debug {
        info!("--debug active: forcing debug borders on every worksheet render");
    }
    let state = Arc::new(AppState {
        root: root.clone(),
        region,
        fonts,
        force_debug: cli.debug,
    });

    // Resolve the static directory. Explicit flag wins; otherwise
    // default to the Astro build alongside the repo. --api-only bypasses
    // static serving entirely.
    let static_dir = if cli.api_only {
        None
    } else {
        cli.static_dir
            .or_else(|| Some(root.join("frontend/astro/dist")))
            .filter(|p| p.join("index.html").is_file())
    };

    let compression = CompressionLayer::new().gzip(true).br(true);

    // Permissive CORS. Routes are all GET-only reads with no credentials,
    // so the blast radius is the generated PDFs/PNGs/SVGs anyway. Enables
    // the Astro dev server on :4321 (or any other frontend on a different
    // origin) to fetch() worksheet bytes cleanly.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        .with_state(state.clone())
        .split_for_parts();

    // SwaggerUi::url("/openapi.json", api) both serves the spec at that
    // path and points the UI at it — no separate route needed.
    let mut app = axum::Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", api))
        .merge(api_router);

    // One frontend per process. Astro's static build emits a real file
    // per route (`/worksheets/add/index.html`, etc.), so ServeDir
    // resolves deep links directly — no SPA-style rewrite-to-index
    // fallback needed. For anything Astro didn't generate (typos, scan
    // traffic hitting `/wp-login.php`, etc.) we return the pre-rendered
    // `404.html` with a real 404 status.
    app = match &static_dir {
        Some(dir) => {
            println!("serving frontend from {}", dir.display());
            let not_found_html = std::fs::read_to_string(dir.join("404.html")).ok();
            let serve_dir = match not_found_html {
                Some(html) => {
                    let html: Arc<str> = Arc::from(html);
                    ServeDir::new(dir).fallback(axum::routing::any(move || {
                        let html = html.clone();
                        async move {
                            (
                                StatusCode::NOT_FOUND,
                                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                                html.as_ref().to_owned(),
                            )
                        }
                    }))
                }
                None => ServeDir::new(dir).fallback(axum::routing::any(|| async {
                    (StatusCode::NOT_FOUND, "not found\n")
                })),
            };
            app.fallback_service(serve_dir)
        }
        None => {
            // No SPA bundle available — expose a cheap root for liveness.
            app.route("/", axum::routing::get(|| async { "hello world\n" }))
        }
    };

    // Layer order (innermost → outermost):
    //   1. region_rewrite — mutates text/html bodies before compression
    //      sees them. Must be innermost so we operate on plaintext.
    //   2. compression — gzips/brs the (possibly rewritten) body.
    //   3. cors — adds headers.
    //   4. TraceLayer — request spans wrap everything above. 2xx/3xx/5xx
    //      log at INFO; 4xx (mostly scan traffic hitting our 404 page)
    //      drops to DEBUG so it doesn't flood the access log.
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<Body>| {
            info_span!(
                "http",
                method = %req.method(),
                uri = %req.uri(),
                ip = %client_ip(req.headers()),
                ua = %user_agent(req.headers()),
            )
        })
        .on_response(
            |response: &Response<Body>, latency: Duration, _span: &tracing::Span| {
                let status = response.status();
                if status.is_client_error() {
                    tracing::debug!(
                        status = status.as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "request"
                    );
                } else {
                    tracing::info!(
                        status = status.as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "request"
                    );
                }
            },
        );

    let app = app
        .layer(middleware::from_fn_with_state(state, region_rewrite))
        .layer(compression)
        .layer(cors)
        .layer(trace_layer);

    let addr = format!("0.0.0.0:{}", cli.port);
    info!(port = cli.port, "pencil-ready-server listening");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Pretty output to a terminal, JSON when stderr is piped (Fly, systemd,
/// containers). Override with `LOG_FORMAT=json|pretty`. Filter via
/// `RUST_LOG=…` (defaults to info for this crate + warn for everything else).
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("warn,pencil_ready_server=info,tower_http=info")
    });

    let force = std::env::var("LOG_FORMAT").ok();
    let json = match force.as_deref() {
        Some("json") => true,
        Some("pretty") => false,
        _ => !std::io::stderr().is_terminal(),
    };

    if json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json().flatten_event(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_target(false).compact())
            .init();
    }
}
