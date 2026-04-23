//! Visual regression testing tool for Pencil Ready components.
//!
//! Stories are self-contained .typ files under `stories/`. Each gets compiled
//! to a PNG; baselines are committed to `stories/baseline/`; diffs go in
//! `stories/diff/`. A change to shared typst (e.g. font in shared.typ)
//! invalidates all baselines — which is the point.
//!
//! Workflow:
//!   cargo run -p pencil-ready-stories -- generate    # regenerate stories/current/
//!   cargo run -p pencil-ready-stories -- diff        # compare current vs baseline
//!   cargo run -p pencil-ready-stories -- approve     # promote current to baseline

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use image::{GenericImageView, ImageBuffer, Rgba};
use pencil_ready_core::{Fonts, OutputFormat, compile_typst};

#[derive(Parser)]
#[command(name = "stories", about = "Visual regression tool for Pencil Ready")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate PNGs for every story file into stories/current/
    Generate,
    /// Diff stories/current/ against stories/baseline/ (writes stories/diff/)
    Diff,
    /// Generate + diff in one step. Exits non-zero on any change.
    Check,
    /// Promote stories/current/ to stories/baseline/
    Approve {
        /// Specific story name (without .typ). If omitted, approves all changed.
        story: Option<String>,
    },
    /// Directional pixel-diff two arbitrary PNGs; write the diff image to <out>.
    /// Exits non-zero when the images differ.
    DiffFiles {
        baseline: PathBuf,
        current: PathBuf,
        out: PathBuf,
    },
}

fn project_root() -> Result<PathBuf> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .context("no parent")?
        .canonicalize()
        .context("canonicalize root")
}

fn list_stories(root: &Path) -> Result<Vec<(String, PathBuf)>> {
    let dir = root.join("stories");
    let mut stories = Vec::new();
    for entry in std::fs::read_dir(&dir).context("reading stories dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "typ") {
            let name = path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            stories.push((name, path));
        }
    }
    stories.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(stories)
}

fn cmd_generate(root: &Path) -> Result<()> {
    let stories = list_stories(root)?;
    let out_dir = root.join("stories/current");
    std::fs::create_dir_all(&out_dir)?;

    let fonts = Fonts::load(root).context("loading fonts from <root>/fonts")?;

    for (name, path) in &stories {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let bytes = compile_typst(&source, OutputFormat::Png, root, &fonts)
            .with_context(|| format!("compiling {name}"))?;
        let out = out_dir.join(format!("{name}.png"));
        std::fs::write(&out, &bytes)?;
        println!("✓ {name}");
    }
    println!("wrote {} stories to {}", stories.len(), out_dir.display());
    Ok(())
}

fn cmd_diff(root: &Path) -> Result<bool> {
    let stories = list_stories(root)?;
    let current_dir = root.join("stories/current");
    let baseline_dir = root.join("stories/baseline");
    let diff_dir = root.join("stories/diff");
    std::fs::create_dir_all(&diff_dir)?;

    let mut any_changed = false;

    for (name, _) in &stories {
        let current = current_dir.join(format!("{name}.png"));
        let baseline = baseline_dir.join(format!("{name}.png"));
        let diff = diff_dir.join(format!("{name}.png"));

        if !current.exists() {
            println!("? {name}: no current PNG (run `generate` first)");
            any_changed = true;
            continue;
        }
        if !baseline.exists() {
            println!("+ {name}: NEW (no baseline yet)");
            any_changed = true;
            continue;
        }

        match directional_diff(&baseline, &current, &diff)? {
            None => {
                // Scrub any stale diff from a prior failing run so
                // stories/diff/ only contains still-failing cases.
                let _ = std::fs::remove_file(&diff);
                println!("✓ {name}");
            }
            Some((removed, added)) => {
                println!(
                    "✗ {name}: {removed} removed, {added} added pixels → {}",
                    diff.display()
                );
                any_changed = true;
            }
        }
    }

    Ok(any_changed)
}

fn cmd_approve(root: &Path, story: Option<&str>) -> Result<()> {
    let stories = list_stories(root)?;
    let current_dir = root.join("stories/current");
    let baseline_dir = root.join("stories/baseline");
    std::fs::create_dir_all(&baseline_dir)?;

    let names: Vec<&str> = match story {
        Some(s) => {
            if !stories.iter().any(|(n, _)| n == s) {
                bail!("unknown story: {s}");
            }
            vec![s]
        }
        None => stories.iter().map(|(n, _)| n.as_str()).collect(),
    };

    for name in &names {
        let current = current_dir.join(format!("{name}.png"));
        let baseline = baseline_dir.join(format!("{name}.png"));
        if !current.exists() {
            bail!("no current PNG for {name} (run `generate` first)");
        }
        std::fs::copy(&current, &baseline)?;
        println!("approved {name}");
    }
    Ok(())
}

/// Directional diff: baseline-only pixels in RED, current-only in GREEN,
/// shared content dimmed to gray. Returns None if no meaningful change.
fn directional_diff(
    baseline: &Path,
    current: &Path,
    out: &Path,
) -> Result<Option<(u32, u32)>> {
    let base = image::open(baseline).context("read baseline")?;
    let curr = image::open(current).context("read current")?;

    if base.dimensions() != curr.dimensions() {
        // Different dimensions = definitely changed. Write a simple marker image
        // and report the mismatch.
        let (w, h) = curr.dimensions();
        let mut marker: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
        for px in marker.pixels_mut() {
            *px = Rgba([255, 200, 0, 255]); // amber: dimensions differ
        }
        marker.save(out)?;
        return Ok(Some((u32::MAX, u32::MAX)));
    }

    let (w, h) = base.dimensions();
    let mut out_img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    let mut n_removed = 0u32;
    let mut n_added = 0u32;

    for y in 0..h {
        for x in 0..w {
            let da = darkness(&base.get_pixel(x, y));
            let db = darkness(&curr.get_pixel(x, y));

            let common = da.min(db);
            let removed = da.saturating_sub(db);
            let added = db.saturating_sub(da);

            let dim = common / 2;
            let r = 255u8.saturating_sub(dim).saturating_sub(added);
            let g = 255u8.saturating_sub(dim).saturating_sub(removed);
            let b = 255u8
                .saturating_sub(dim)
                .saturating_sub(removed)
                .saturating_sub(added);
            out_img.put_pixel(x, y, Rgba([r, g, b, 255]));

            if removed > 8 {
                n_removed += 1;
            }
            if added > 8 {
                n_added += 1;
            }
        }
    }

    if n_removed == 0 && n_added == 0 {
        // Clean — caller decides whether to remove any stale diff at
        // this path. cmd_diff's stories/diff/<name>.png is a managed
        // cache so it scrubs; diff-files takes an arbitrary user-
        // supplied path and leaves it alone.
        Ok(None)
    } else {
        out_img.save(out)?;
        Ok(Some((n_removed, n_added)))
    }
}

fn darkness(px: &Rgba<u8>) -> u8 {
    let [r, g, b, a] = px.0;
    let lum = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
    let composited = (lum * a as u32 + 255 * (255 - a as u32)) / 255;
    255u8.saturating_sub(composited as u8)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = project_root()?;

    match cli.command {
        Command::Generate => cmd_generate(&root)?,
        Command::Diff => {
            let changed = cmd_diff(&root)?;
            if changed {
                std::process::exit(1);
            }
        }
        Command::Check => {
            cmd_generate(&root)?;
            let changed = cmd_diff(&root)?;
            if changed {
                std::process::exit(1);
            }
        }
        Command::Approve { story } => cmd_approve(&root, story.as_deref())?,
        Command::DiffFiles { baseline, current, out } => {
            match directional_diff(&baseline, &current, &out)? {
                None => println!("✓ identical"),
                Some((removed, added)) => {
                    println!("✗ {removed} removed, {added} added pixels → {}", out.display());
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
