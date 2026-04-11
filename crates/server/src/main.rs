//! HTTP server for worksheet generation — stub.
//!
//! This is the axum equivalent of a Go net/http server.
//! Axum is the most popular Rust web framework — similar philosophy to
//! Go's stdlib http: handlers are just functions, routing is explicit.
//!
//! NOT wired up yet — this shows the shape and compiles, but the actual
//! endpoint is a TODO.

use axum::{Router, routing::post, Json};
use serde::{Deserialize, Serialize};

/// Request body for worksheet generation.
/// `#[derive(Deserialize)]` makes this parseable from JSON automatically.
/// Like json.Unmarshal in Go, but the struct tags are derive macros.
#[allow(dead_code)] // fields will be used when we wire up core
#[derive(Deserialize)]
struct GenerateRequest {
    operation: String,
    #[serde(default = "default_digits")]
    top_digits: u32,
    #[serde(default = "default_digits")]
    bottom_digits: u32,
    #[serde(default = "default_problems")]
    num_problems: u32,
    #[serde(default = "default_cols")]
    cols: u32,
    format: Option<String>,
}

fn default_digits() -> u32 { 2 }
fn default_problems() -> u32 { 12 }
fn default_cols() -> u32 { 4 }

/// Response — for now just confirms what was requested.
#[derive(Serialize)]
struct GenerateResponse {
    message: String,
}

/// Handler for POST /generate.
///
/// In Go: `func handleGenerate(w http.ResponseWriter, r *http.Request)`
/// In axum: async function that takes typed extractors and returns a response.
/// Axum automatically deserializes JSON body → GenerateRequest and
/// serializes GenerateResponse → JSON response.
async fn handle_generate(
    Json(req): Json<GenerateRequest>,
) -> Json<GenerateResponse> {
    // TODO: call mathsheet_core::generate() here.
    // For now, just echo back what was requested.
    Json(GenerateResponse {
        message: format!(
            "would generate {} worksheet: {}x{} digits, {} problems",
            req.operation, req.top_digits, req.bottom_digits, req.num_problems,
        ),
    })
}

/// Entry point. `#[tokio::main]` sets up the async runtime.
/// In Go, goroutines are built in. In Rust, you need an explicit async
/// runtime (tokio). `#[tokio::main]` is sugar for:
///   fn main() { tokio::runtime::Runtime::new().block_on(async { ... }) }
#[tokio::main]
async fn main() {
    // Build the router — like http.NewServeMux() in Go.
    let app = Router::new()
        .route("/generate", post(handle_generate));

    let addr = "0.0.0.0:3000";
    println!("listening on {addr}");

    // In Go: http.ListenAndServe(addr, mux)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
