//! Typst World implementation + compilation.
//!
//! The `World` trait is typst's dependency injection interface — think of it
//! like a Go interface that the compiler calls to resolve files, fonts, etc.
//! We implement it once here, and typst uses it during compilation.
//!
//! In Go terms: typst defines `type World interface { ... }` and we provide
//! a concrete struct that satisfies it.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

use crate::OutputFormat;

/// Pre-loaded fonts, shared across all `MathWorld` instances in a process.
///
/// Loading ~20 fonts from disk takes tens of milliseconds and allocates
/// tens of MB; doing it per request is expensive and — worse — doubles
/// peak memory under concurrency. Load once at process start, clone the
/// `Arc`s per compile.
#[derive(Clone)]
pub struct Fonts {
    book: Arc<LazyHash<FontBook>>,
    faces: Arc<[Font]>,
}

impl Fonts {
    /// Scan `<root>/fonts/` and parse every `.ttf`/`.otf`. Call once.
    pub fn load(root: &Path) -> Result<Self> {
        let (book, faces) = load_fonts(root)?;
        Ok(Self {
            book: Arc::new(LazyHash::new(book)),
            faces: faces.into(),
        })
    }
}

/// Our World implementation. Holds everything typst needs in memory.
///
/// In Go you'd embed the fields and implement methods on the struct.
/// Same idea here.
///
/// `LazyHash<T>` is a typst wrapper that caches the hash of its contents.
/// The World trait requires it for library() and book() return types —
/// it's an optimization so typst doesn't re-hash these on every access.
struct MathWorld {
    /// The main .typ source (generated in memory, not from a file).
    main_source: Source,
    /// Map of file paths to their contents — for lib/*.typ and assets/*.
    /// Like map[string][]byte in Go.
    files: HashMap<FileId, Bytes>,
    /// All source files (including imports from lib/).
    sources: HashMap<FileId, Source>,
    /// Shared, pre-loaded fonts — cheap to clone per world.
    fonts: Fonts,
    /// The standard typst library (built-in functions).
    library: LazyHash<Library>,
}

impl MathWorld {
    /// Build a new World from the generated .typ source, the project
    /// root (for `lib/*.typ` + `assets/*`), and pre-loaded fonts.
    fn new(typ_source: &str, root: &Path, fonts: Fonts) -> Result<Self> {
        // Create the "main" source file with a virtual path.
        // Typst uses virtual paths (not real filesystem paths) internally.
        let main_id = FileId::new(None, VirtualPath::new("/main.typ"));
        let main_source = Source::new(main_id, typ_source.to_string());

        let mut files = HashMap::new();
        let mut sources = HashMap::new();

        // Load lib/*.typ files — these are the imports the template uses.
        load_typ_sources(root, "lib", &mut sources)?;

        // Load assets (like rainbow-heart.svg) as raw bytes.
        load_binary_files(root, "assets", &mut files)?;

        Ok(Self {
            main_source,
            files,
            sources,
            fonts,
            library: LazyHash::new(Library::default()),
        })
    }
}

// --- World trait implementation ---
//
// `impl Trait for Struct` is Rust's way of implementing an interface.
// In Go: `func (w *MathWorld) Source(id FileId) (Source, error)`
// In Rust: `fn source(&self, id: FileId) -> FileResult<Source>`

impl typst::World for MathWorld {
    /// Returns the standard typst library (built-in functions like `grid`, `text`, etc.)
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Font metadata index — typst searches this to resolve font names.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts.book
    }

    /// Which file is the "main" entrypoint.
    fn main(&self) -> FileId {
        self.main_source.id()
    }

    /// Resolve a file ID to its parsed source (for .typ files).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_source.id() {
            Ok(self.main_source.clone())
        } else {
            self.sources
                .get(&id)
                .cloned()
                .ok_or(FileError::NotFound(id.vpath().as_rooted_path().to_path_buf()))
        }
    }

    /// Resolve a file ID to raw bytes (for images, data files, etc.)
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files
            .get(&id)
            .cloned()
            .ok_or(FileError::NotFound(id.vpath().as_rooted_path().to_path_buf()))
    }

    /// Load a font by its index in the font book.
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.faces.get(index).cloned()
    }

    /// Current date — used by typst's `datetime.today()`.
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None // worksheets don't need the current date
    }
}

// --- File loading helpers ---

/// Recursively load all .typ files from a subdirectory into the sources map.
fn load_typ_sources(
    root: &Path,
    subdir: &str,
    sources: &mut HashMap<FileId, Source>,
) -> Result<()> {
    let dir = root.join(subdir);
    if !dir.exists() {
        return Ok(());
    }
    visit_typ_sources(root, &dir, sources)
}

fn visit_typ_sources(
    root: &Path,
    dir: &Path,
    sources: &mut HashMap<FileId, Source>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir).context("reading lib dir")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_typ_sources(root, &path, sources)?;
        } else if path.extension().is_some_and(|e| e == "typ") {
            // Build the virtual path relative to root: "/lib/problems/vertical.typ"
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let vpath = format!("/{}", rel.to_string_lossy());
            let id = FileId::new(None, VirtualPath::new(&vpath));
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            sources.insert(id, Source::new(id, text));
        }
    }
    Ok(())
}

/// Load all files from a subdirectory as raw bytes.
fn load_binary_files(
    root: &Path,
    subdir: &str,
    files: &mut HashMap<FileId, Bytes>,
) -> Result<()> {
    let dir = root.join(subdir);
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&dir).context("reading assets dir")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let vpath = format!(
                "/{}/{}",
                subdir,
                path.file_name().unwrap().to_string_lossy()
            );
            let id = FileId::new(None, VirtualPath::new(&vpath));
            let data = std::fs::read(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            files.insert(id, Bytes::new(data));
        }
    }
    Ok(())
}

/// Scan fonts/ directory and load all .ttf/.otf files.
///
/// Returns a FontBook (metadata index) + Vec<Font> (loaded font data).
/// The indices must match: font_book entry 0 corresponds to fonts[0].
fn load_fonts(root: &Path) -> Result<(FontBook, Vec<Font>)> {
    let mut book = FontBook::new();
    let mut fonts = Vec::new();

    let font_dir = root.join("fonts");
    if !font_dir.exists() {
        return Ok((book, fonts));
    }

    // Walk the fonts directory recursively.
    // `walkdir` would be cleaner but we'll keep deps minimal.
    visit_fonts(&font_dir, &mut book, &mut fonts)?;

    Ok((book, fonts))
}

/// Recursively walk a directory loading font files.
fn visit_fonts(dir: &Path, book: &mut FontBook, fonts: &mut Vec<Font>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_fonts(&path, book, fonts)?;
        } else if path
            .extension()
            .is_some_and(|e| e == "ttf" || e == "otf" || e == "ttc")
        {
            let data = std::fs::read(&path)
                .with_context(|| format!("reading font {}", path.display()))?;
            let bytes = Bytes::new(data);

            // A single font file can contain multiple faces (e.g., .ttc collections).
            // We iterate all of them.
            for font in Font::iter(bytes) {
                // font.info() returns &FontInfo in typst 0.14.
                // We always have info for successfully-parsed fonts.
                book.push(font.info().clone());
                fonts.push(font);
            }
        }
    }
    Ok(())
}

// --- Compilation + export ---

/// Compile a .typ source string and export to the requested format.
/// This is the "inner" function that lib.rs's generate() calls.
pub fn compile_and_export(
    typ_source: &str,
    format: OutputFormat,
    root: &Path,
    fonts: &Fonts,
) -> Result<Vec<u8>> {
    let world = MathWorld::new(typ_source, root, fonts.clone())?;

    // typst::compile returns a Warned<SourceResult<Document>>.
    // .output is the SourceResult, which is Result<Document, EcoVec<Diagnostic>>.
    // We convert the diagnostics to anyhow::Error for our error type.
    let document = typst::compile(&world)
        .output
        .map_err(|diagnostics| {
            let messages: Vec<String> = diagnostics
                .iter()
                .map(|d| d.message.to_string())
                .collect();
            anyhow::anyhow!("typst compilation failed:\n{}", messages.join("\n"))
        })?;

    match format {
        OutputFormat::Pdf => {
            let options = typst_pdf::PdfOptions::default();
            let pdf = typst_pdf::pdf(&document, &options)
                .map_err(|diagnostics| {
                    let messages: Vec<String> = diagnostics
                        .iter()
                        .map(|d| d.message.to_string())
                        .collect();
                    anyhow::anyhow!("PDF export failed:\n{}", messages.join("\n"))
                })?;
            Ok(pdf)
        }
        OutputFormat::Svg => {
            // SVG export is per-page. Worksheets are single-page.
            let page = document
                .pages
                .first()
                .ok_or_else(|| anyhow::anyhow!("document has no pages"))?;
            let svg = typst_svg::svg(page);
            Ok(svg.into_bytes())
        }
        OutputFormat::Png => {
            // Render at 300 PPI. Typst uses "points" (72 per inch),
            // so 300/72 ≈ 4.17 pixels per point.
            let pixel_per_pt = 300.0 / 72.0;
            let page = document
                .pages
                .first()
                .ok_or_else(|| anyhow::anyhow!("document has no pages"))?;
            let pixmap = typst_render::render(page, pixel_per_pt);
            let png = pixmap
                .encode_png()
                .map_err(|e| anyhow::anyhow!("PNG encoding failed: {e}"))?;
            Ok(png)
        }
    }
}
