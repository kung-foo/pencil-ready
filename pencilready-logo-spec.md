# Pencil Ready — brand mark spec

A pixelated horizontal #2 pencil, in two primary uses:

1. **Favicon / logo mark** — standalone pixel pencil, 16 × 3 grid
2. **Header underline** — extended variant that stretches to match the "Pencil Ready" wordmark width

---

## Palette

| Role          | Hex       | Notes                                          |
|---------------|-----------|------------------------------------------------|
| Eraser pink   | `#E88E8A` | Dusty coral, Ticonderoga-like                  |
| Ferrule green | `#3E8948` | Forest green                                   |
| Barrel yellow | `#FDB600` | Goldenrod — canonical #2 pencil shade          |
| Graphite      | `#1A1A1A` | Near-black tip                                 |
| Wood tan      | `#D9A066` | Tapered-tip variant only (V3, not used in site)|

---

## Canonical column layout (16 cols)

```
col:  0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15
      P  P  G  Y  G  Y  Y  Y  Y  Y  Y  Y  Y  Y  Y  K
```

- cols 0–1: eraser (pink)
- cols 2–5: ferrule — alternating green/yellow, `GYGY`
- cols 6–14: yellow barrel (9 cols; merges visually with col 5 → 10 continuous yellow cols)
- col 15: graphite tip (black)

---

## Favicon / logo mark (V1, 16 × 3)

The primary mark. Scales cleanly from 16 × 3 px up to arbitrarily large sizes. Because the shape is inherently horizontal (16:3), a square favicon slot will have whitespace above and below — that's expected.

```svg
<svg viewBox="0 0 16 3" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">
  <rect x="0"  y="0" width="2"  height="3" fill="#E88E8A"/>
  <rect x="2"  y="0" width="1"  height="3" fill="#3E8948"/>
  <rect x="3"  y="0" width="1"  height="3" fill="#FDB600"/>
  <rect x="4"  y="0" width="1"  height="3" fill="#3E8948"/>
  <rect x="5"  y="0" width="10" height="3" fill="#FDB600"/>
  <rect x="15" y="0" width="1"  height="3" fill="#1A1A1A"/>
</svg>
```

`shape-rendering="crispEdges"` keeps pixel boundaries sharp at any scale. No `preserveAspectRatio` needed — default preserves the 16:3 aspect.

---

## Header underline

Sits beneath the serif "Pencil Ready" wordmark in the site header. The eraser, ferrule bands, and tip keep a fixed aspect ratio; the yellow barrel stretches horizontally to absorb whatever width the wordmark renders at.

### The SVG

```svg
<svg preserveAspectRatio="none" viewBox="0 0 288 12" shape-rendering="crispEdges"
     xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
  <rect x="0"   y="0" width="12"  height="12" fill="#E88E8A"/>
  <rect x="12"  y="0" width="6"   height="12" fill="#3E8948"/>
  <rect x="18"  y="0" width="6"   height="12" fill="#FDB600"/>
  <rect x="24"  y="0" width="6"   height="12" fill="#3E8948"/>
  <rect x="30"  y="0" width="252" height="12" fill="#FDB600"/>
  <rect x="282" y="0" width="6"   height="12" fill="#1A1A1A"/>
</svg>
```

### Usage

Wrap the `<h1>` and the SVG in an `inline-flex` column with `align-items: stretch`. The wrapper sizes to the title's natural width; the SVG inherits that width via `stretch`, and `preserveAspectRatio="none"` lets it scale horizontally without fighting the layout.

```html
<div style="display: inline-flex; flex-direction: column; align-items: stretch;">
  <h1>Pencil Ready</h1>
  <svg preserveAspectRatio="none" viewBox="0 0 288 12" shape-rendering="crispEdges"
       style="height: 12px; margin-top: 12px;" aria-hidden="true">
    <rect x="0"   y="0" width="12"  height="12" fill="#E88E8A"/>
    <rect x="12"  y="0" width="6"   height="12" fill="#3E8948"/>
    <rect x="18"  y="0" width="6"   height="12" fill="#FDB600"/>
    <rect x="24"  y="0" width="6"   height="12" fill="#3E8948"/>
    <rect x="30"  y="0" width="252" height="12" fill="#FDB600"/>
    <rect x="282" y="0" width="6"   height="12" fill="#1A1A1A"/>
  </svg>
</div>
```

Swap fonts, change the title text, tweak the font size — the underline follows.

### How the stretching works

The viewBox numbers (`0 0 288 12`) are an arbitrary coordinate space. Because `preserveAspectRatio="none"`, only the *proportions* between rects matter:

| Element                | viewBox width | Fraction of total |
|------------------------|--------------:|------------------:|
| Eraser                 |           12  |           4.17 %  |
| Green ferrule band ×2  |            6  |           2.08 %  |
| Yellow ferrule band ×1 |            6  |           2.08 %  |
| Barrel                 |          252  |          87.50 %  |
| Tip                    |            6  |           2.08 %  |

Height stays fixed at 12 px via the CSS `height`. When the wrapper's width changes, only the barrel stretches; the eraser stays a 12 × 12 square (at default width), the ferrule bands and tip stay 1:2 tall rectangles.

---

## Other variants (explored, not in use)

Kept here for reference in case you want to bring one back.

### V2 — 16 × 5 rectangular

Chunkier body, same column plan as V1. Reads better than V1 at very small sizes; worse ratio for a 16×16 favicon slot since the shape is more square-ish but still horizontal.

```svg
<svg viewBox="0 0 16 5" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">
  <rect x="0"  y="0" width="2"  height="5" fill="#E88E8A"/>
  <rect x="2"  y="0" width="1"  height="5" fill="#3E8948"/>
  <rect x="3"  y="0" width="1"  height="5" fill="#FDB600"/>
  <rect x="4"  y="0" width="1"  height="5" fill="#3E8948"/>
  <rect x="5"  y="0" width="10" height="5" fill="#FDB600"/>
  <rect x="15" y="0" width="1"  height="5" fill="#1A1A1A"/>
</svg>
```

### V3 — 16 × 5 with tapered tip

Most pencil-realistic. Wood transition + pointed graphite. Fragile at 1× because the sharpest part of the tip lives in a single pixel — better at larger sizes.

```svg
<svg viewBox="0 0 16 5" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">
  <rect x="0"  y="0" width="2" height="5" fill="#E88E8A"/>
  <rect x="2"  y="0" width="1" height="5" fill="#3E8948"/>
  <rect x="3"  y="0" width="1" height="5" fill="#FDB600"/>
  <rect x="4"  y="0" width="1" height="5" fill="#3E8948"/>
  <rect x="5"  y="0" width="8" height="5" fill="#FDB600"/>
  <rect x="13" y="0" width="1" height="5" fill="#D9A066"/>
  <rect x="14" y="1" width="1" height="3" fill="#1A1A1A"/>
  <rect x="15" y="2" width="1" height="1" fill="#1A1A1A"/>
</svg>
```

---

Project: [pencilready.com](https://pencilready.com) — free printable math worksheets, Oslo.
