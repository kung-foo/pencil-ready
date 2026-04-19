import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    // Proxy API calls to the local Rust server so the browser avoids CORS.
    // Run `cargo run --release --bin pencil-ready-server` alongside `pnpm dev`.
    proxy: {
      "/api": "http://127.0.0.1:8080",
    },
  },
});
