# Pencil Ready

![Pencil Ready — free printable math worksheets](frontend/astro/public/og-image-600x315.png)

Free printable math worksheets. Live at [pencilready.com](https://pencilready.com).

Rust workspace that renders worksheets through [Typst](https://typst.app/)
and serves them via an Astro + React frontend. Deterministic from a seed,
so a URL fully describes a worksheet — handy for sharing or re-printing
the same sheet.

## Running locally

Prerequisites: Rust (stable), [pnpm](https://pnpm.io/), and a typst-capable
font set (already under `fonts/`).

```
make serve
```

Builds the Rust server in release, builds the Astro frontend, and serves
both at <http://localhost:8080> — same shape as production.

For a tighter inner loop while editing the frontend:

```
cargo run --bin pencil-ready-server        # API on :8080
cd frontend/astro && pnpm dev              # Astro dev server with HMR
```

The Astro dev server proxies `/api/*` to the Rust backend.

## CLI

Generate a worksheet directly, without the server:

```
cargo run --bin pencil-ready -- multiply --digits 3,2 --seed 42 --output out
```

`cargo run --bin pencil-ready -- --help` lists every worksheet type and
its flags.

## Tests

```
cargo test --workspace --lib    # unit tests
make stories-check              # visual regression (typst → PNG → diff)
```

## Deploy

```
flyctl deploy --remote-only
```

## Further reading

- [`SPEC.md`](SPEC.md) — architecture, layout rules, Typst conventions
- [`WORKSHEETS.md`](WORKSHEETS.md) — reference for each worksheet type
- [`CLAUDE.md`](CLAUDE.md) — repo layout + per-change checklist (also
  used as agent context)
- [`pencilready-logo-spec.md`](pencilready-logo-spec.md) — brand palette
  and pencil mark

## License

MIT — see [`LICENSE`](LICENSE).
