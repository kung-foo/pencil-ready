#!/usr/bin/env node
// Per-page transfer size of the built Astro site.
//
// For every index.html under dist/, collects what a cold page load
// fetches up front — the HTML, stylesheets, module scripts (and their
// static imports, recursively), React island component + renderer,
// preloaded fonts, the favicon and <img> sources — then reports the
// request count and the brotli size of each, which is what the server's
// CompressionLayer sends (woff2/png are already compressed, so they
// count at their on-disk size).
//
// Not counted: anything only known at runtime — the worksheet preview
// from /api/worksheets/*, the Umami script and beacon, dynamic import()s.
//
// Usage: node scripts/site-size.mjs [dist-dir] [--json out.json]

import { readFileSync, readdirSync, existsSync, writeFileSync } from "node:fs";
import { join, relative, dirname, posix } from "node:path";
import { brotliCompressSync, constants } from "node:zlib";

const args = process.argv.slice(2);
const jsonIdx = args.indexOf("--json");
const jsonOut = jsonIdx >= 0 ? args.splice(jsonIdx, 2)[1] : null;
const dist = args[0] ?? "dist";

if (!existsSync(join(dist, "index.html"))) {
    console.error(`no index.html in ${dist} — build the frontend first`);
    process.exit(1);
}

// tower-http's CompressionLevel::Default maps to brotli quality 4.
const BROTLI_QUALITY = 4;
const PRECOMPRESSED = new Set([
    "woff2",
    "woff",
    "png",
    "jpg",
    "jpeg",
    "webp",
    "avif",
    "gif",
]);

const sizeCache = new Map();
function sizes(urlPath) {
    if (sizeCache.has(urlPath)) return sizeCache.get(urlPath);
    const file = join(dist, urlPath);
    if (!existsSync(file)) return null;
    const raw = readFileSync(file);
    const ext = extOf(urlPath);
    const br = PRECOMPRESSED.has(ext)
        ? raw.length
        : brotliCompressSync(raw, {
              params: { [constants.BROTLI_PARAM_QUALITY]: BROTLI_QUALITY },
          }).length;
    const s = {
        raw: raw.length,
        br,
        text: ext === "js" ? raw.toString("utf8") : null,
    };
    sizeCache.set(urlPath, s);
    return s;
}

function extOf(p) {
    const m = p.match(/\.(\w+)$/);
    return m ? m[1].toLowerCase() : "";
}

function category(p) {
    const ext = extOf(p);
    if (ext === "js") return "js";
    if (ext === "css") return "css";
    if (ext === "woff2" || ext === "woff") return "font";
    return "img";
}

const attr = (tag, name) => tag.match(new RegExp(`\\s${name}="([^"]*)"`))?.[1];
const isLocal = (u) => u && u.startsWith("/") && !u.startsWith("//");

function pageAssets(html) {
    const found = new Set();
    for (const [tag] of html.matchAll(
        /<(?:script|link|img|astro-island)\b[^>]*>/g,
    )) {
        const name = tag.match(/^<([\w-]+)/)[1];
        if (name === "script") {
            const src = attr(tag, "src");
            if (isLocal(src)) found.add(src);
        } else if (name === "link") {
            const rel = attr(tag, "rel");
            const href = attr(tag, "href");
            if (
                ["stylesheet", "preload", "modulepreload", "icon"].includes(
                    rel,
                ) &&
                isLocal(href)
            ) {
                found.add(href);
            }
        } else if (name === "img") {
            const src = attr(tag, "src");
            if (isLocal(src)) found.add(src);
        } else {
            for (const a of ["component-url", "renderer-url"]) {
                const u = attr(tag, a);
                if (isLocal(u)) found.add(u);
            }
        }
    }
    // Inline module scripts can import chunks too.
    for (const [, body] of html.matchAll(
        /<script type="module">([\s\S]*?)<\/script>/g,
    )) {
        for (const u of staticImports(body, "/")) found.add(u);
    }
    return found;
}

function staticImports(js, fromDir) {
    const out = [];
    // `import{..}from"./x.js"` and bare `import"./x.js"` — not import().
    for (const [, spec] of js.matchAll(
        /\bimport\s*(?:[\w*{}\s,$]+from\s*)?["']([^"']+)["']/g,
    )) {
        out.push(spec.startsWith("/") ? spec : posix.join(fromDir, spec));
    }
    return out;
}

function walk(dir) {
    return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
        e.isDirectory()
            ? walk(join(dir, e.name))
            : e.name === "index.html"
              ? [join(dir, e.name)]
              : [],
    );
}

const pages = [];
for (const file of walk(dist).sort(
    (a, b) => a.length - b.length || a.localeCompare(b),
)) {
    const dir = relative(dist, dirname(file)).split("\\").join("/");
    const route = dir ? `/${dir}/` : "/";
    const html = readFileSync(file, "utf8");
    const htmlSize = sizes("/" + relative(dist, file));

    const assets = pageAssets(html);
    // Follow static imports through the JS graph.
    const queue = [...assets].filter((u) => extOf(u) === "js");
    while (queue.length) {
        const u = queue.pop();
        const s = sizes(u);
        if (!s?.text) continue;
        for (const dep of staticImports(s.text, posix.dirname(u))) {
            if (!assets.has(dep)) {
                assets.add(dep);
                queue.push(dep);
            }
        }
    }

    const by = { html: htmlSize.br, js: 0, css: 0, font: 0, img: 0 };
    let raw = htmlSize.raw;
    const missing = [];
    for (const u of assets) {
        const s = sizes(u);
        if (!s) {
            missing.push(u);
            continue;
        }
        by[category(u)] += s.br;
        raw += s.raw;
    }
    const total = Object.values(by).reduce((a, b) => a + b, 0);
    pages.push({
        route,
        requests: 1 + assets.size - missing.length,
        ...by,
        total,
        raw,
        missing,
    });
}

const kb = (n) => (n / 1024).toFixed(1);
const cols = ["requests", "html", "js", "css", "font", "img", "total", "raw"];
const width = Math.max(...pages.map((p) => p.route.length), 5);
console.log(
    "page".padEnd(width) + "  " + cols.map((c) => c.padStart(8)).join(""),
);
for (const p of pages) {
    console.log(
        p.route.padEnd(width) +
            "  " +
            cols
                .map((c) =>
                    (c === "requests" ? String(p[c]) : kb(p[c])).padStart(8),
                )
                .join(""),
    );
    for (const m of p.missing) console.log(`  ! missing: ${m}`);
}
console.log(
    `\nsizes in KB; html/js/css/font/img/total are brotli q${BROTLI_QUALITY} (as served), raw is uncompressed.` +
        `\nexcludes runtime fetches: worksheet preview (/api/worksheets/*), Umami.`,
);

if (jsonOut) {
    const out = {
        generated_at: new Date().toISOString(),
        brotli_quality: BROTLI_QUALITY,
        pages: pages.map(({ missing, ...p }) => p),
    };
    writeFileSync(jsonOut, JSON.stringify(out, null, 2) + "\n");
    console.log(`wrote ${jsonOut}`);
}
