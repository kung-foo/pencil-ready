import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

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
  // Opt every anchor into hover-triggered prefetch. ClientRouter picks
  // it up and fetches the destination HTML when a link is hovered/
  // focused, so the real navigation is instant. Prefetches are in-idle
  // <link rel="prefetch"> requests and deduplicated automatically.
  prefetch: {
    prefetchAll: true,
    defaultStrategy: "hover",
  },
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
      },
    },
  },
});
