//! Visual regression testing tool for Pencil Ready components.
//!
//! Stories are self-contained .typ files under `stories/`. Each gets compiled
//! to a PNG; baselines are committed to `stories/baseline/`; diffs go in
//! `stories/diff/`. A change to shared typst (e.g. font in shared.typ)
//! invalidates all baselines — which is the point.
//!
//! `diff` also rebuilds `stories/unapproved/`: the current render of every
//! story that is new or changed, and nothing else — the review list.
//!
//! Workflow:
//!   cargo run -p pencil-ready-stories -- generate    # regenerate stories/current/
//!   cargo run -p pencil-ready-stories -- diff        # compare current vs baseline
//!   cargo run -p pencil-ready-stories -- approve     # promote stories/unapproved/ to baseline

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
    /// Diff stories/current/ against stories/baseline/ (writes stories/diff/
    /// and rebuilds stories/unapproved/ with the new and changed renders)
    Diff,
    /// Generate + diff + lint in one step. Exits non-zero on any change
    /// or any lint failure.
    Check,
    /// Flag ink outside (or on) each story's red debug box, for every
    /// story (`check` only lints new and changed ones).
    Lint,
    /// Promote stories/unapproved/ to stories/baseline/
    Approve {
        /// Specific story name (without .typ). If omitted, approves every
        /// story in stories/unapproved/.
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
    // Rebuilt every run, so it only ever holds what still needs review.
    let unapproved_dir = root.join("stories/unapproved");
    let _ = std::fs::remove_dir_all(&unapproved_dir);
    std::fs::create_dir_all(&unapproved_dir)?;
    let unapprove = |name: &str, current: &Path| -> Result<()> {
        std::fs::copy(current, unapproved_dir.join(format!("{name}.png")))?;
        Ok(())
    };

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
            unapprove(name, &current)?;
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
                unapprove(name, &current)?;
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
    let unapproved_dir = root.join("stories/unapproved");
    std::fs::create_dir_all(&baseline_dir)?;

    let names: Vec<String> = match story {
        Some(s) => {
            if !stories.iter().any(|(n, _)| n == s) {
                bail!("unknown story: {s}");
            }
            vec![s.to_string()]
        }
        // Only what `diff` flagged as new or changed.
        None => {
            let mut names: Vec<String> = std::fs::read_dir(&unapproved_dir)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter_map(|e| {
                            e.file_name().to_str()?.strip_suffix(".png").map(str::to_string)
                        })
                        .collect()
                })
                .unwrap_or_default();
            names.sort();
            if names.is_empty() {
                println!("nothing to approve (stories/unapproved/ is empty; run `check` first)");
            }
            names
        }
    };

    for name in &names {
        let current = current_dir.join(format!("{name}.png"));
        let baseline = baseline_dir.join(format!("{name}.png"));
        if !current.exists() {
            bail!("no current PNG for {name} (run `generate` first)");
        }
        std::fs::copy(&current, &baseline)?;
        let _ = std::fs::remove_file(unapproved_dir.join(format!("{name}.png")));
        println!("approved {name}");
    }
    Ok(())
}

/// Ink that escapes a story's debug box: pixels of text or lines lying
/// outside every outermost red box, or touching one's border. A
/// component's bounding rect must contain what it draws (SPEC.md →
/// Layout rules → Bounding box); this catches a denominator hanging
/// below the box or working drawn beside it. Returns the number of such
/// pixels and their bounding rectangle, or `None` when the story draws
/// no red box.
fn bbox_lint(path: &Path) -> Result<Option<(usize, (u32, u32, u32, u32))>> {
    let img = image::open(path).context("read png")?.to_rgba8();
    let (w, h) = img.dimensions();
    let px = |x: u32, y: u32| img.get_pixel(x, y).0;
    // The debug stroke is `red` (#ff4136) and its anti-aliased edge.
    let is_red = |p: [u8; 4]| p[0] > 180 && p[0] as i32 - p[1] as i32 > 60 && p[0] as i32 - p[2] as i32 > 60;
    // Ink: dark in every channel, so neither red nor blue debug strokes.
    let is_ink = |p: [u8; 4]| p[3] > 0 && p[0] < 170 && p[1] < 170 && p[2] < 170;

    // Red boxes: bounding rects of 8-connected red regions.
    let mut seen = vec![false; (w * h) as usize];
    let mut boxes: Vec<(u32, u32, u32, u32)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if seen[(y * w + x) as usize] || !is_red(px(x, y)) {
                continue;
            }
            let (mut x0, mut y0, mut x1, mut y1, mut n) = (x, y, x, y, 0usize);
            let mut stack = vec![(x, y)];
            seen[(y * w + x) as usize] = true;
            while let Some((cx, cy)) = stack.pop() {
                n += 1;
                (x0, y0, x1, y1) = (x0.min(cx), y0.min(cy), x1.max(cx), y1.max(cy));
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let i = (ny as u32 * w + nx as u32) as usize;
                        if !seen[i] && is_red(px(nx as u32, ny as u32)) {
                            seen[i] = true;
                            stack.push((nx as u32, ny as u32));
                        }
                    }
                }
            }
            // Ignore specks; a box outline is long.
            if n > 50 {
                boxes.push((x0, y0, x1, y1));
            }
        }
    }
    if boxes.is_empty() {
        return Ok(None);
    }
    let inside = |a: &(u32, u32, u32, u32), b: &(u32, u32, u32, u32)| {
        a != b && a.0 >= b.0 && a.1 >= b.1 && a.2 <= b.2 && a.3 <= b.3
    };
    let outer: Vec<_> = boxes.iter().filter(|a| !boxes.iter().any(|b| inside(a, b))).copied().collect();

    // Border (1pt at 300 ppi ≈ 4 px) plus a pixel of anti-aliasing: ink
    // this close to an edge is touching it.
    let m = 6;
    // Judge connected pieces of ink, not single pixels: a rule (a line
    // at most `thin` px thick) may run to the box edge by design — the
    // answer line of a column sum spans the box — so it only fails if it
    // leaves the box. Anything else (a glyph, a fraction bar's digits)
    // must sit clear of the border.
    let thin = 8;
    let mut ink_seen = vec![false; (w * h) as usize];
    let mut count = 0;
    let mut ext = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if ink_seen[(y * w + x) as usize] || !is_ink(px(x, y)) {
                continue;
            }
            let (mut x0, mut y0, mut x1, mut y1, mut n) = (x, y, x, y, 0usize);
            let mut stack = vec![(x, y)];
            ink_seen[(y * w + x) as usize] = true;
            while let Some((cx, cy)) = stack.pop() {
                n += 1;
                (x0, y0, x1, y1) = (x0.min(cx), y0.min(cy), x1.max(cx), y1.max(cy));
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let i = (ny as u32 * w + nx as u32) as usize;
                        if !ink_seen[i] && is_ink(px(nx as u32, ny as u32)) {
                            ink_seen[i] = true;
                            stack.push((nx as u32, ny as u32));
                        }
                    }
                }
            }
            let clear = outer.iter().any(|&(bx0, by0, bx1, by1)| {
                x0 > bx0 + m && x1 + m < bx1 && y0 > by0 + m && y1 + m < by1
            });
            let rule = (y1 - y0 < thin || x1 - x0 < thin)
                && outer.iter().any(|&(bx0, by0, bx1, by1)| {
                    x0 >= bx0 && x1 <= bx1 && y0 >= by0 && y1 <= by1
                });
            if !clear && !rule {
                count += n;
                ext = (ext.0.min(x0), ext.1.min(y0), ext.2.max(x1), ext.3.max(y1));
            }
        }
    }
    Ok(Some((count, ext)))
}

/// Lint current renders; prints failures, returns whether any. With
/// `only_unapproved`, just the stories in stories/unapproved/ (new or
/// changed), so `check` can gate what's up for approval without failing
/// on older stories that predate the lint.
fn cmd_lint(root: &Path, only_unapproved: bool) -> Result<bool> {
    let mut any = false;
    for (name, _) in list_stories(root)? {
        let current = root.join("stories/current").join(format!("{name}.png"));
        if !current.exists() {
            continue;
        }
        if only_unapproved && !root.join("stories/unapproved").join(format!("{name}.png")).exists() {
            continue;
        }
        if let Some((n, (x0, y0, x1, y1))) = bbox_lint(&current)? {
            if n > 0 {
                println!("! {name}: {n} ink pixels outside or on the red box (x {x0}-{x1}, y {y0}-{y1})");
                any = true;
            }
        }
    }
    Ok(any)
}

/// Directional diff with canvas-size awareness.
///
/// Inside the overlap (each image centered on the union canvas):
///   - baseline-only darker → RED ("removed")
///   - current-only darker  → GREEN ("added")
///   - shared content       → dim gray
///
/// Outside the overlap (one image has pixels there, the other doesn't):
///   - baseline-only area  → ORANGE  ("canvas shrank" — area lost from current)
///   - current-only area   → BLUE    ("canvas grew"   — area new in current)
///
/// Returns None if no meaningful change.
fn directional_diff(
    baseline: &Path,
    current: &Path,
    out: &Path,
) -> Result<Option<(u32, u32)>> {
    let base = image::open(baseline).context("read baseline")?;
    let curr = image::open(current).context("read current")?;

    let (bw, bh) = base.dimensions();
    let (cw, ch) = curr.dimensions();

    // Union canvas; place each input centered. When dims match this
    // collapses to the original same-canvas comparison.
    let uw = bw.max(cw);
    let uh = bh.max(ch);
    let base_off_x = (uw - bw) / 2;
    let base_off_y = (uh - bh) / 2;
    let curr_off_x = (uw - cw) / 2;
    let curr_off_y = (uh - ch) / 2;

    let mut out_img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(uw, uh);
    let mut n_removed = 0u32;
    let mut n_added = 0u32;

    // Coords inside the source image, or None if (x, y) on the union
    // canvas falls outside that image's centered placement.
    let map = |x: u32, y: u32, off_x: u32, off_y: u32, w: u32, h: u32| {
        let sx = x.checked_sub(off_x)?;
        let sy = y.checked_sub(off_y)?;
        if sx < w && sy < h { Some((sx, sy)) } else { None }
    };

    for y in 0..uh {
        for x in 0..uw {
            let in_base = map(x, y, base_off_x, base_off_y, bw, bh);
            let in_curr = map(x, y, curr_off_x, curr_off_y, cw, ch);

            match (in_base, in_curr) {
                (Some((bx, by)), Some((cx, cy))) => {
                    let da = darkness(&base.get_pixel(bx, by));
                    let db = darkness(&curr.get_pixel(cx, cy));

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
                (Some(_), None) => {
                    // Canvas shrank: baseline had this region, current
                    // doesn't. Warm orange band along the lost edge.
                    out_img.put_pixel(x, y, Rgba([255, 165, 0, 255]));
                    n_removed += 1;
                }
                (None, Some(_)) => {
                    // Canvas grew: current has this region, baseline
                    // didn't. Cool blue band along the new edge.
                    out_img.put_pixel(x, y, Rgba([0, 165, 255, 255]));
                    n_added += 1;
                }
                (None, None) => {
                    // Unreachable: every (x, y) on the union canvas
                    // is inside at least one source. White as a safe
                    // fallback if the geometry ever becomes off.
                    out_img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
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
            let lint = cmd_lint(&root, true)?;
            if changed || lint {
                std::process::exit(1);
            }
        }
        Command::Lint => {
            if cmd_lint(&root, false)? {
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
