//! Benchmarks the difference between loading fonts per-render (the
//! previous behavior) vs. loading them once and cloning the `Arc`-backed
//! `Fonts` per render (the current server behavior).
//!
//! Also times the cold `Fonts::load()` itself so we know the one-time
//! startup cost.
//!
//! Usage:
//!   cargo run --release --example font_load_bench -p pencil-ready-core

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use pencil_ready_core::{
    CarryMode, DigitRange, Fonts, Locale, OutputFormat, WorksheetParams, WorksheetType,
    compile_typst, generate_typst_source,
};

const ITERS: usize = 20;

fn params(seed: u64) -> WorksheetParams {
    WorksheetParams {
        worksheet: WorksheetType::Add {
            digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
            carry: CarryMode::Any,
            binary: false,
        },
        num_problems: 12,
        cols: 4,
        paper: pencil_ready_core::Paper::A4,
        debug: false,
        seed: Some(seed),
        symbol: None,
        locale: Locale::Us,
        solve_first: false,
        include_answers: false,
        student_name: None,
        instructions: None,
        share_url: None,
    }
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("no repo root")
        .to_path_buf();

    // One-time font load cost.
    let t = Instant::now();
    let fonts = Fonts::load(&root)?;
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("Fonts::load() (cold from disk): {load_ms:.1} ms");
    println!();

    // Warm-up one render so the comemo cache + allocator arenas are hot.
    // Otherwise iter 1 skews the "old" column because the shared typst
    // stdlib / first-compile setup gets blamed on font reloading.
    let source = generate_typst_source(&params(0))?;
    let _ = compile_typst(&source, OutputFormat::Pdf, &root, &fonts)?;

    // --- Old behavior: reload fonts for every render ---
    let mut total_old = 0.0;
    let mut times_old = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let source = generate_typst_source(&params(i as u64 + 1))?;
        let t = Instant::now();
        let fresh = Fonts::load(&root)?; // reload each time, like pre-refactor
        let _ = compile_typst(&source, OutputFormat::Pdf, &root, &fresh)?;
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        times_old.push(ms);
        total_old += ms;
    }

    // --- New behavior: reuse the pre-loaded Fonts, Arc-cloned per render ---
    let mut total_new = 0.0;
    let mut times_new = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let source = generate_typst_source(&params(i as u64 + 1_000))?;
        let t = Instant::now();
        let _ = compile_typst(&source, OutputFormat::Pdf, &root, &fonts)?;
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        times_new.push(ms);
        total_new += ms;
    }

    fn stats(xs: &[f64]) -> (f64, f64, f64) {
        let mut v = xs.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = xs.iter().sum::<f64>() / xs.len() as f64;
        let p50 = v[v.len() / 2];
        let p95 = v[((v.len() as f64) * 0.95) as usize];
        (avg, p50, p95)
    }

    let (avg_old, p50_old, p95_old) = stats(&times_old);
    let (avg_new, p50_new, p95_new) = stats(&times_new);

    println!("{:<30}  {:>8}  {:>8}  {:>8}  {:>8}", "mode", "avg", "p50", "p95", "total");
    println!(
        "{:<30}  {:>6.1}ms  {:>6.1}ms  {:>6.1}ms  {:>6.1}ms",
        format!("reload fonts per render ({ITERS}×)"),
        avg_old,
        p50_old,
        p95_old,
        total_old
    );
    println!(
        "{:<30}  {:>6.1}ms  {:>6.1}ms  {:>6.1}ms  {:>6.1}ms",
        format!("cached Fonts (Arc clone) ({ITERS}×)"),
        avg_new,
        p50_new,
        p95_new,
        total_new
    );

    let saved_per_render = avg_old - avg_new;
    println!();
    println!(
        "saved per render: ~{saved_per_render:.1} ms ({:.1}× faster)",
        avg_old / avg_new
    );

    Ok(())
}
