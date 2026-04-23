//! Probe typst's comemo cache growth across successive renders.
//!
//! Renders a sequence of worksheets and prints process RSS after each,
//! so we can see how much the memoization cache grows per render and
//! how much reuse different traffic shapes produce.
//!
//! Runs without eviction so the cache is free to accumulate. A final
//! `evict(0)` pass shows how much of the retained RSS the cache was
//! actually holding.
//!
//! Usage:
//!   cargo run --release --example cache_growth -p pencil-ready-core

use std::path::PathBuf;

use anyhow::Result;
use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Fonts, Locale, OutputFormat, WorksheetParams, WorksheetType,
    compile_typst, generate_typst_source,
};

fn rss_kb() -> u64 {
    let Ok(s) = std::fs::read_to_string("/proc/self/statm") else {
        return 0;
    };
    // statm format: size resident shared text lib data dt (all in pages)
    let pages: u64 = s
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    pages * 4 // 4 KiB pages → KiB
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
        paper: pencil_ready_core::Paper::A4,
        debug: false,
        seed: Some(seed),
        symbol: None,
        locale: Locale::Us,
        solve_first: false,
        include_answers: false,
        student_name: None,
    }
}

fn render(kind_ix: u32, seed: u64, root: &std::path::Path, fonts: &Fonts) -> Result<()> {
    let p = params(kind_ix, seed);
    let source = generate_typst_source(&p)?;
    // Use compile_typst (not `generate`) so there's no implicit evict.
    let _ = compile_typst(&source, OutputFormat::Pdf, root, fonts)?;
    Ok(())
}

fn main() -> Result<()> {
    // Walk up from crates/core to the repo root so typst imports resolve.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("no repo root")
        .to_path_buf();

    let fonts = Fonts::load(&root)?;

    let mut last = rss_kb();
    println!("baseline RSS: {} MB", last / 1024);
    println!();
    println!("{:<4}  {:>7}  {:>7}  {}", "i", "RSS MB", "Δ KB", "scenario");

    let mut i = 0;
    let run = |label: &str, kind: u32, seed: u64, i: &mut u32, last: &mut u64| -> Result<()> {
        *i += 1;
        render(kind, seed, &root, &fonts)?;
        let now = rss_kb();
        let delta = now as i64 - *last as i64;
        println!("{:<4}  {:>7}  {:>+7}  {label}", i, now / 1024, delta);
        *last = now;
        Ok(())
    };

    println!("\n-- same params, repeated --");
    for _ in 0..5 {
        run("add seed=42", 0, 42, &mut i, &mut last)?;
    }

    println!("\n-- different seeds, same kind --");
    for s in 1..=5 {
        run(&format!("add seed={s}"), 0, s, &mut i, &mut last)?;
    }

    println!("\n-- cycling kinds --");
    for k in 0..6 {
        run(&format!("kind={}", k % 3), k, 42, &mut i, &mut last)?;
    }

    println!("\n-- all unique --");
    for k in 0..10 {
        run(
            &format!("kind={}, seed={}", k % 3, k * 100),
            k,
            (k as u64) * 100,
            &mut i,
            &mut last,
        )?;
    }

    println!("\n-- after evict(0): drop everything not used this round --");
    typst::comemo::evict(0);
    let after = rss_kb();
    println!(
        "RSS: {} MB (reclaimed {} KB from cache)",
        after / 1024,
        last as i64 - after as i64
    );

    Ok(())
}
