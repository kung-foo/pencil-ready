//! Long-running version of the cache probe: renders N varied worksheets
//! and samples RSS at intervals to confirm memory plateaus under the
//! `evict(10)` policy that `generate()` applies.
//!
//! Runs two phases back-to-back:
//!   1. `evict(10)` — mirrors what the server does. Expected to plateau.
//!   2. no evict — cache free to grow. Expected to climb linearly.
//!
//! Usage:
//!   cargo run --release --example cache_hammer -p pencil-ready-core -- 2000

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Fonts, Locale, OutputFormat, WorksheetParams, WorksheetType,
    compile_typst, generate_typst_source,
};

fn rss_kb() -> u64 {
    let Ok(s) = std::fs::read_to_string("/proc/self/statm") else {
        return 0;
    };
    let pages: u64 = s
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    pages * 4
}

fn params(kind_ix: u32, seed: u64) -> WorksheetParams {
    let ws = match kind_ix % 3 {
        0 => WorksheetType::Add {
            digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
            carry: CarryMode::Any,
            binary: false,
        },
        1 => WorksheetType::Multiply {
            digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
        },
        _ => WorksheetType::Subtract {
            digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
            borrow: BorrowMode::Any,
        },
    };
    WorksheetParams {
        worksheet: ws,
        num_problems: 12,
        cols: 4,
        paper: "a4".into(),
        debug: false,
        seed: Some(seed),
        symbol: None,
        locale: Locale::Us,
        pages: 1,
        solve_first: false,
        include_answers: false,
        student_name: None,
        teacher_name: None,
    }
}

fn hammer(
    label: &str,
    n: usize,
    sample_every: usize,
    evict_max_age: Option<usize>,
    root: &std::path::Path,
    fonts: &Fonts,
) -> Result<()> {
    println!("\n=== {label} (n={n}, evict={evict_max_age:?}) ===");
    println!("{:>5}  {:>7}  {:>7}  {:>8}", "iter", "RSS MB", "Δ MB", "ms/iter");

    let baseline = rss_kb();
    let mut last_rss = baseline;
    let mut last_sample = Instant::now();

    for i in 1..=n {
        // Vary kind + seed so the cache sees unique work.
        let kind = (i as u32) % 3;
        let seed = i as u64;
        let p = params(kind, seed);
        let source = generate_typst_source(&p)?;
        let _ = compile_typst(&source, OutputFormat::Pdf, root, fonts)?;

        if let Some(age) = evict_max_age {
            typst::comemo::evict(age);
        }

        if i == 1 || i % sample_every == 0 || i == n {
            let now = rss_kb();
            let dt = last_sample.elapsed().as_secs_f64();
            let per_iter_ms = (dt * 1000.0) / (sample_every as f64);
            let delta_mb = (now as f64 - last_rss as f64) / 1024.0;
            println!(
                "{:>5}  {:>7}  {:>+7.2}  {:>8.1}",
                i,
                now / 1024,
                delta_mb,
                per_iter_ms
            );
            last_rss = now;
            last_sample = Instant::now();
        }
    }

    let final_rss = rss_kb();
    typst::comemo::evict(0);
    let after_evict = rss_kb();
    println!(
        "final: {} MB; after evict(0): {} MB (cache held ≈{} MB)",
        final_rss / 1024,
        after_evict / 1024,
        (final_rss as i64 - after_evict as i64) / 1024
    );
    Ok(())
}

fn main() -> Result<()> {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
    let sample_every: usize = (n / 20).max(50);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("no repo root")
        .to_path_buf();

    let fonts = Fonts::load(&root)?;

    println!("baseline RSS: {} MB", rss_kb() / 1024);

    // Phase 1: mimics the server (evict(10) after each compile).
    hammer("evict(10) — production behavior", n, sample_every, Some(10), &root, &fonts)?;

    // Phase 2: no eviction — cache grows freely.
    hammer("no evict — cache unbounded", n, sample_every, None, &root, &fonts)?;

    Ok(())
}
