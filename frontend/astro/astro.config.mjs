import crypto from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, fontProviders } from "astro/config";
import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import { optimize as svgoOptimize } from "svgo";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Content-hashed SVG optimizer.
//
// Astro's bundled `svgoOptimizer()` doesn't pass file path info into
// SVGO's `prefixIds`, so all SVGs get the same `prefix__a, b, c…`
// namespace. That breaks the homepage's worksheet thumbs: typst
// hashes glyph IDs by content, identical glyphs (e.g. the digit "2")
// across files share IDs, and when 4 thumbs inline into the same
// document `<use href="#g…">` resolves to whichever was defined first
// — making subtract/multiply/divide all render as addition.
//
// Here we keep on-disk SVGs as raw typst output and namespace IDs at
// bundle time using a sha1 of the SVG contents. Identical SVGs hash
// the same (and their IDs are already identical, so no collision);
// distinct SVGs get distinct prefixes.
const thumbSvgOptimizer = () => ({
    name: "thumb-svg-optimizer",
    optimize: (contents) => {
        const prefix =
            "s" +
            crypto
                .createHash("sha1")
                .update(contents)
                .digest("hex")
                .slice(0, 8);
        const { data } = svgoOptimize(contents, {
            multipass: true,
            plugins: [
                "preset-default",
                { name: "prefixIds", params: { prefix, delim: "-" } },
                "removeXMLNS",
                { name: "removeXlink", params: { includeLegacy: true } },
            ],
        });
        return data;
    },
});

// https://astro.build/config
export default defineConfig({
  // Deployed standalone — lives at the root of its origin. The Rust
  // server selects this app via `--framework astro`; no base prefix,
  // no /astro mount.
  site: "https://pencilready.com",
  trailingSlash: "ignore",
  output: "static",
  compressHTML: false,
  integrations: [
    react(),
    // Emits sitemap-index.xml + sitemap-0.xml at build time listing
    // every generated route (/, /worksheets/add/, ...). Search engines
    // use these to discover URLs without crawling.
    sitemap(),
  ],
  // Optimize SVG component imports at production-build time. See
  // `thumbSvgOptimizer` above for the content-hash-prefixed twist.
  experimental: {
    svgOptimizer: thumbSvgOptimizer(),
  },
  // Opt every anchor into hover-triggered prefetch. ClientRouter picks
  // it up and fetches the destination HTML when a link is hovered/
  // focused, so the real navigation is instant. Prefetches are in-idle
  // <link rel="prefetch"> requests and deduplicated automatically.
  prefetch: {
    prefetchAll: true,
    defaultStrategy: "hover",
  },
  // Astro Fonts API: pulls woff2 files from Google at build time,
  // self-hosts them under dist/_astro/ with content-hashed filenames,
  // and emits @font-face rules with computed metric overrides
  // (size-adjust / ascent-override / etc.) calibrated against each
  // declared fallback stack. The metric overrides minimize layout
  // shift when the webfont swaps in. Weights match what the UI
  // actually renders — 400 (body), 500 (font-medium), 600
  // (font-semibold) for Roboto Slab; 600 only for Crimson Text
  // (used in the headers card on worksheet pages).
  fonts: [
    {
      provider: fontProviders.google(),
      name: "Roboto Slab",
      cssVariable: "--font-roboto-slab",
      weights: [400, 500, 600],
      styles: ["normal"],
      subsets: ["latin"],
      fallbacks: ["Georgia", "serif"],
    },
    {
      provider: fontProviders.google(),
      name: "Crimson Text",
      cssVariable: "--font-crimson-text",
      weights: [600],
      styles: ["normal"],
      subsets: ["latin"],
      fallbacks: ["Georgia", "serif"],
    },
  ],
  vite: {
    plugins: [tailwindcss()],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "src"),
      },
    },
    server: {
      // Dev-only proxy: /api/* to the Rust backend so the island's
      // useWorksheet fetches land same-origin from the browser's POV.
      proxy: {
        "/api": "http://127.0.0.1:8080",
        "/umami": "http://127.0.0.1:8080",
      },
    },
  },
});
