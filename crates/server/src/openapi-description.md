Generate printable math worksheets as PDF, PNG or SVG.

Every endpoint is a `GET` that returns the file directly, so a worksheet
is just a URL: `/api/worksheets/add?digits=3,3&carry=force&seed=42`.

**Conventions**

- All parameters are optional; omitted ones take the per-worksheet default.
- **Ranges** are `N` (exactly N) or `N-M` (inclusive), e.g. `digits=2-4`.
- **Lists** are comma-separated, e.g. `denominators=2,3,4` or
  `digits=3,2` (one entry per operand).
- **Seeds** make output reproducible: the same parameters and `seed`
  produce the same problems. Without a seed, each request is random.
- Problem count and layout are identical on A4 and US Letter.
- Errors return `400` with a plain-text message.
