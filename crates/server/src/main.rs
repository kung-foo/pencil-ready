//! HTTP server for worksheet generation.
//!
//! GET /api/worksheets/{type}?format=pdf&seed=42&...
//!   type  ∈ add | subtract | multiply | simple-divide | long-divide
//!           mult-drill | div-drill | fraction-mult | algebra-two-step
//!   format ∈ pdf (default) | png | svg
//!
//! Response is raw bytes with a matching Content-Type. Unknown query
//! params are ignored; invalid values return 400 with an error message.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use axum::{
    Router,
    extract::{Path as AxPath, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tower_http::compression::CompressionLayer;
use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Locale, OutputFormat, WorksheetParams, WorksheetType,
    generate,
};

#[derive(Clone)]
struct AppState {
    root: PathBuf,
}

#[tokio::main]
async fn main() {
    let root = std::env::var("PENCIL_READY_ROOT").unwrap_or_else(|_| ".".into());
    let root = PathBuf::from(root)
        .canonicalize()
        .expect("canonicalize project root (set PENCIL_READY_ROOT or cd to repo)");
    let state = Arc::new(AppState { root });

    // Compression middleware: negotiates Accept-Encoding and streams the
    // response through gzip or brotli. Skips content-types it knows are
    // already compressed (image/*, application/zip, etc.) so we don't waste
    // CPU double-compressing PNGs.
    let compression = CompressionLayer::new().gzip(true).br(true);

    let app = Router::new()
        .route("/", get(|| async { "hello world\n" }))
        .route("/api/worksheets/{kind}", get(handle_worksheet))
        .layer(compression)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{port}");
    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_worksheet(
    AxPath(kind): AxPath<String>,
    Query(q): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    match build_and_render(&kind, &q, &state.root) {
        Ok((format, bytes)) => {
            let ct = match format {
                OutputFormat::Pdf => "application/pdf",
                OutputFormat::Png => "image/png",
                OutputFormat::Svg => "image/svg+xml",
            };
            ([(header::CONTENT_TYPE, ct)], bytes).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response(),
    }
}

fn build_and_render(
    kind: &str,
    q: &HashMap<String, String>,
    root: &Path,
) -> Result<(OutputFormat, Vec<u8>)> {
    let format = parse_format(q.get("format").map(String::as_str).unwrap_or("pdf"))?;
    let (worksheet, default_problems, default_cols) = build_type(kind, q)?;

    let params = WorksheetParams {
        worksheet,
        num_problems: parse_opt(q, "problems")?.unwrap_or(default_problems),
        cols: parse_opt(q, "cols")?.unwrap_or(default_cols),
        paper: q.get("paper").cloned().unwrap_or_else(|| "a4".into()),
        debug: parse_bool(q, "debug")?.unwrap_or(false),
        seed: parse_opt(q, "seed")?,
        symbol: q.get("symbol").cloned(),
        locale: parse_locale(q.get("locale").map(String::as_str).unwrap_or("us"))?,
        pages: parse_opt(q, "pages")?.unwrap_or(1),
        solve_first: parse_bool(q, "solve_first")?.unwrap_or(false),
    };

    let ws = generate(&params, format, root)?;
    Ok((format, ws.bytes))
}

/// Returns (type, default_problems, default_cols). Defaults mirror the CLI's
/// per-subcommand choices.
fn build_type(kind: &str, q: &HashMap<String, String>) -> Result<(WorksheetType, u32, u32)> {
    match kind {
        "add" => Ok((
            WorksheetType::Add {
                digits: parse_digit_list(q, "digits", default_digits(&[2, 2]))?,
                carry: parse_carry(q.get("carry").map(String::as_str).unwrap_or("any"))?,
                binary: parse_bool(q, "binary")?.unwrap_or(false),
            },
            12,
            4,
        )),
        "subtract" => Ok((
            WorksheetType::Subtract {
                digits: parse_digit_list(q, "digits", default_digits(&[2, 2]))?,
                borrow: parse_borrow(q.get("borrow").map(String::as_str).unwrap_or("any"))?,
            },
            12,
            4,
        )),
        "multiply" => Ok((
            WorksheetType::Multiply {
                digits: parse_digit_list(q, "digits", default_digits(&[2, 2]))?,
            },
            12,
            4,
        )),
        "simple-divide" => Ok((
            WorksheetType::SimpleDivision {
                max_quotient: parse_opt(q, "max_quotient")?.unwrap_or(10),
            },
            12,
            4,
        )),
        "long-divide" => Ok((
            WorksheetType::LongDivision {
                digits: parse_single_digit(q, "digits", DigitRange::fixed(3))?,
                remainder: parse_bool(q, "remainder")?.unwrap_or(false),
            },
            12,
            4,
        )),
        "mult-drill" => Ok((
            WorksheetType::MultiplicationDrill {
                multiplicand: parse_digit_list(q, "multiplicand", vec![DigitRange::new(1, 10)])?,
                multiplier: parse_single_digit(q, "multiplier", DigitRange::new(1, 10))?,
            },
            parse_opt(q, "count")?.unwrap_or(0),
            3,
        )),
        "div-drill" => Ok((
            WorksheetType::DivisionDrill {
                divisor: parse_digit_list(q, "divisor", vec![DigitRange::new(2, 10)])?,
                max_quotient: parse_single_digit(q, "max_quotient", DigitRange::new(2, 10))?,
            },
            parse_opt(q, "count")?.unwrap_or(0),
            3,
        )),
        "fraction-mult" => Ok((
            WorksheetType::FractionMultiply {
                denominators: parse_u32_list(q, "denominators", vec![2, 3, 4, 5, 10])?,
                min_whole: parse_opt(q, "min_whole")?.unwrap_or(2),
                max_whole: parse_opt(q, "max_whole")?.unwrap_or(20),
                unit_only: parse_bool(q, "unit_only")?.unwrap_or(false),
            },
            12,
            3,
        )),
        "algebra-two-step" => Ok((
            WorksheetType::AlgebraTwoStep {
                a_range: parse_single_digit(q, "a_range", DigitRange::new(2, 10))?,
                b_range: parse_single_digit(q, "b_range", DigitRange::new(1, 30))?,
                x_range: parse_single_digit(q, "x_range", DigitRange::new(0, 20))?,
                variable: q.get("variable").cloned().unwrap_or_else(|| "x".into()),
                implicit: parse_bool(q, "implicit")?.unwrap_or(false),
                mix_forms: parse_bool(q, "mix_forms")?.unwrap_or(true),
            },
            6,
            2,
        )),
        other => bail!("unknown worksheet type: {other}"),
    }
}

// --- param helpers ---

fn parse_opt<T: FromStr>(q: &HashMap<String, String>, key: &str) -> Result<Option<T>>
where
    T::Err: std::fmt::Display,
{
    match q.get(key) {
        None => Ok(None),
        Some(s) => s
            .parse::<T>()
            .map(Some)
            .map_err(|e| anyhow!("{key}: {e}")),
    }
}

/// Accept `true/false`, `1/0`, `yes/no`, and bare-flag form (empty string = true).
fn parse_bool(q: &HashMap<String, String>, key: &str) -> Result<Option<bool>> {
    match q.get(key).map(String::as_str) {
        None => Ok(None),
        Some("" | "true" | "1" | "yes") => Ok(Some(true)),
        Some("false" | "0" | "no") => Ok(Some(false)),
        Some(other) => bail!("{key}: bad bool value {other:?}"),
    }
}

fn parse_format(s: &str) -> Result<OutputFormat> {
    match s {
        "pdf" => Ok(OutputFormat::Pdf),
        "png" => Ok(OutputFormat::Png),
        "svg" => Ok(OutputFormat::Svg),
        other => bail!("unknown format: {other} (pdf, png, svg)"),
    }
}

fn parse_locale(s: &str) -> Result<Locale> {
    Locale::from_str(s).ok_or_else(|| anyhow!("unknown locale: {s} (us, no)"))
}

fn parse_carry(s: &str) -> Result<CarryMode> {
    match s {
        "none" => Ok(CarryMode::None),
        "any" => Ok(CarryMode::Any),
        "force" => Ok(CarryMode::Force),
        "ripple" => Ok(CarryMode::Ripple),
        other => bail!("unknown carry: {other} (none, any, force, ripple)"),
    }
}

fn parse_borrow(s: &str) -> Result<BorrowMode> {
    match s {
        "none" => Ok(BorrowMode::None),
        "no-across-zero" => Ok(BorrowMode::NoAcrossZero),
        "any" => Ok(BorrowMode::Any),
        "force" => Ok(BorrowMode::Force),
        "ripple" => Ok(BorrowMode::Ripple),
        other => bail!("unknown borrow: {other}"),
    }
}

fn default_digits(ns: &[u32]) -> Vec<DigitRange> {
    ns.iter().copied().map(DigitRange::fixed).collect()
}

fn parse_digit_list(
    q: &HashMap<String, String>,
    key: &str,
    default: Vec<DigitRange>,
) -> Result<Vec<DigitRange>> {
    match q.get(key) {
        None => Ok(default),
        Some(s) => s
            .split(',')
            .map(|part| DigitRange::from_str(part.trim()).map_err(|e| anyhow!("{key}: {e}")))
            .collect(),
    }
}

fn parse_single_digit(
    q: &HashMap<String, String>,
    key: &str,
    default: DigitRange,
) -> Result<DigitRange> {
    match q.get(key) {
        None => Ok(default),
        Some(s) => DigitRange::from_str(s).map_err(|e| anyhow!("{key}: {e}")),
    }
}

fn parse_u32_list(
    q: &HashMap<String, String>,
    key: &str,
    default: Vec<u32>,
) -> Result<Vec<u32>> {
    match q.get(key) {
        None => Ok(default),
        Some(s) => s
            .split(',')
            .map(|p| {
                p.trim()
                    .parse::<u32>()
                    .map_err(|e| anyhow!("{key}: {e}"))
            })
            .collect(),
    }
}
