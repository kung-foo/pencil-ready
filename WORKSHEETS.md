# Worksheet Types

## Constraints

- **Max digits per operand**: 5
- **Max problems per page**: 16
- **Max operands**: 4 (addition only; subtraction and multiplication are always 2)

## Shared parameters

All worksheet types accept these (CLI flag names shown; HTTP query
params are identical except `problems` replaces `--problems` and
`include_answers` uses an underscore).

- `digits: list[DigitRange]` — digit count per operand, comma-separated
  (e.g. `2,2`, `3-5,2`). Length determines operand count. Default
  varies per type.
- `problems: u32` — number of problems on the page (1–16 for
  non-drill; up to 40 for drills). Default 12 (or per-type).
- `cols: u32` — columns in the grid. Default varies per type.
- `pages: u32` — number of pages. Each page has `problems` unique
  problems. `pages > 1` requires PDF output.
- `paper: string` — paper size passed to typst. Default `a4`.
- `seed: u64?` — optional random seed for reproducible output. Used
  in the filename slug and the PDF-title when set.
- `solve_first: bool` — render the first problem as a worked example
  so the student can see the method. Each type defines its own idea
  of "solved" (filled-in answer, partial products, x = N, etc.).
- `include_answers: bool` — append an answer-key section after the
  problem page(s). One answer page per problem page. PDF-only.
  Answers are compact: just the final numeric answer, not the full
  worked steps.
- `locale: us | no` — regional symbol defaults for horizontal layouts.
  US: `×` / `÷`; Norway: `·` / `:`.
- `symbol: string?` — explicit operator override (typst expression,
  e.g. `sym.colon`). Wins over `locale`.
- `debug: bool` — draw red/blue debug borders on problem boxes and
  grid cells.

## Addition

Supports 2 or more operands via `--digits` (e.g. `--digits 2,2,2` for
three addends).

- `carry: none | any | force | ripple` (default: any)
  - **none** — no column sum reaches the radix (no carrying). Easiest.
  - **any** — no constraint, random mix.
  - **force** — every problem has at least one carry.
  - **ripple** — every problem has a carry that chains through 2+
    consecutive columns. Hardest.
  - Carry detection works for 2 or 3+ operands (column sums can
    exceed 20 with 3 addends).
- `binary: bool` — render in base 2. The `digits` values are
  reinterpreted as **bit counts** per operand, and the rendered
  numbers appear in binary with leading-zero padding. Pair with
  `--digits 4,4` or similar. Carry rules use radix 2. Operand range
  is `[0, 2^d − 1]` — the high bit isn't forced, so `carry=none` is
  feasible at any bit width.

When the unique problem space is smaller than the requested count
(e.g. 2-bit binary + `carry=none` has only 9 unique solutions),
`pencil-ready-core` resamples from the valid set to fill the page —
students get some repetition rather than a short worksheet.

## Subtraction

Always 2 operands. First operand is always >= second (no negative results).

- `borrow: none | no-across-zero | any | force | ripple` (default: any)
  - **none** — no column requires borrowing. Easiest.
  - **no-across-zero** — borrowing OK, but never across a 0 digit (e.g. 403 - 157 is rejected because the borrow from ones must ripple past the 0 in tens).
  - **any** — no constraint, random mix.
  - **force** — every problem has at least one borrow.
  - **ripple** — every problem has a borrow that chains through 2+ consecutive columns. Hardest.

## Multiplication

Always 2 operands. No operation-specific parameters yet.

## Simple Division

Tests times-table recall. Divisor (2-9) × quotient (1 to max_quotient). Always divides evenly.

```
  16
÷  4
────
```

- `max_quotient: u32` (2-12, default: 10)
  - Caps how high the answer can be. Tests knowledge of tables up to N.
  - Dividend range is a consequence: divisor (2-9) × quotient (1-max_quotient).
  - e.g. max_quotient=5 → dividends 2-45. max_quotient=12 → dividends 2-108.

### Problem generation

- Pick divisor (2-9), pick quotient (1 to max_quotient)
- Compute dividend = divisor × quotient

## Long Division

Practices the long division algorithm with larger numbers. Divisor is always 1 digit (2-9).

```
      ____
  4 │ 16
```

- `digits: u32` (2-4, default: 2) — dividend digit count
- `remainder: bool` (default: false)
  - When false, all problems divide evenly.
  - When true, problems may have a remainder.

### Problem generation

- **No remainder**: pick divisor (2-9), pick quotient such that dividend = divisor × quotient stays in digit range.
- **With remainder**: pick divisor (2-9), pick dividend randomly in digit range.

## Multiplication Drill

Horizontal times-table recall. Tests rapid recall of specific multiplication tables.

```
7 × 3 = ___     4 × 9 = ___
```

Layout is horizontal with 2-3 columns, many problems per page. Uses the shared `--symbol` override for locale (e.g. `sym.dot.c` for Norwegian `·`).

- `multiplicand: list[range]` — which table(s) to drill, comma-separated. Supports ranges.
  - `--multiplicand 2` → the 2s table: 2×1, 2×2, ... 2×10
  - `--multiplicand 2,3` → the 2s and 3s tables mixed together
  - `--multiplicand 1-10` → all tables 1-10 (default)
  - `--multiplicand 1-12` → full extended tables
- `multiplier: range` — the range of the other factor (default: 1-10)
  - `--multiplier 1-12` → test up to ×12

### Commutative deduplication

If `2 × 7` is in the problem set, `7 × 2` is excluded. This avoids testing the same fact twice. The multiplicand is always shown on the left.

### Problem generation

- Enumerate all pairs: for each multiplicand M in the selected tables, pair with each multiplier N in the multiplier range
- Deduplicate: if both (M, N) and (N, M) would appear (because N is also a selected multiplicand), keep only the one where M ≤ N
- Shuffle the resulting list
- If the total exceeds `num_problems`, take a random subset
- If the total is less than `num_problems`, use all of them (don't pad with duplicates)

### Constraints

- Max problems per page: higher than other worksheets — up to 40 for horizontal layout
- Default columns: 2 (horizontal problems are wider)
- Multiplicand range: 1-12
- Multiplier range: 1-12

## Division Drill

The same facts as the multiplication drill, rendered in reverse so the
student recalls the *quotient* given the product.

```
56 ÷ 7 = ___     24 ÷ 3 = ___
```

Uses the same horizontal layout and locale-sensitive symbol (÷ in US,
`:` in Norway). Problems are enumerated from a times-table range
(divisor × quotient = dividend) and rendered as `dividend ÷ divisor`.

### Parameters

- `divisor: list[range]` — which divisors to drill (default: `2-10`).
- `max_quotient: range` — range of the quotient (default: `2-10`).
- `count: u32` — number of problems (0 = all enumerated pairs).

### Problem generation

- Enumerate all `(divisor, quotient)` pairs in range; compute
  `dividend = divisor × quotient`.
- Division is *not* commutative, so no dedup step — `12 ÷ 3` and
  `12 ÷ 4` are both meaningful.
- Shuffle; take `count` or all.

## Fraction Multiplication

Whole number times a proper fraction with a clean integer answer. Teaches
the two-step procedure: multiply across, then simplify. The fraction is
rendered with `math.frac` (numerator stacked over denominator).

```
        7    210           4    80
30  ×  ── = ───        20 × ─ = ──
       10    10              5   5
          =  21                = 16
```

Two-row step-by-step layout per problem with vertically aligned `=` signs.
Row 1 holds the multiply-across intermediate fraction (numerator = `whole
× num`, denominator unchanged). Row 2 holds the simplified integer. No
pre-drawn answer blanks — the student writes directly on the line.

### Parameters

- `denominators: list[u32]` — allowed denominators (default: `2,3,4,5,10`)
- `min_whole: u32` / `max_whole: u32` — range for the whole number (default: `2-20`)
- `unit_only: bool` — if true, numerator is always 1; if false, numerator is
  randomly chosen from `1..denominator-1` (default: `false`)
- `solve_first: bool` — render the first problem filled in as a worked
  example showing the multiply-across intermediate and the simplified
  integer. Default: `false`.

Always 3 columns. The number of problems controls how much vertical
breathing room each problem gets (more problems = less space between rows).

### Layout

- Two-row grid per problem. Row 1 shows `whole × num/den =` on the left
  and an empty slot on the right (or the intermediate fraction if solved).
  Row 2 shows `=` and an empty slot (or the integer answer if solved).
- The `=` signs align vertically across both rows.
- A fixed-width slot is reserved to the right of each `=` so solved and
  unsolved problems share the same bounding box.
- Row gutter is `problem-line-height` (1.3em), matching the stack
  spacing used by vertical add/multiply problems.

### Constraints

- Whole-number final answer only: `whole × numerator` must be divisible by
  `denominator`. The generator picks compatible combinations.
- Denominators are limited to `2-12` (5th-grade scope)
- Whole number is at most 2 digits

### Locale

Same locale rules as the multiplication drill — `--locale us` shows `×`,
`--locale no` shows `·`. The fraction itself doesn't change between locales.

### Problem generation

1. Pick a denominator from the allowed set.
2. Pick a numerator: `1` if `unit_only`, else random in `1..denominator-1`.
3. Pick a whole number `w` such that `w × num` is divisible by `den`
   (stride = `den / gcd(num, den)`), landing within `whole_range`.
4. Enumerate all valid `(whole, num, den)` triples, shuffle, and take
   `num_problems`.

### Layout component

```typst
horizontal-fraction-problem((30, 7, 10), opts: (operator: [#sym.times]))
```

The equation is rendered in math mode with Fira Math pinned as the math
font — the default math font inherits the outer text's letter-spacing and
breaks multi-digit numerators/denominators (e.g. "10") into "1 0".

## Algebra: Two-Step

Solve a two-step linear equation of the form `ax + b = c` for `x`. Teaches
the isolate-variable procedure: subtract the constant from both sides, then
divide by the coefficient.

```
(4 × x) + 5 = 21          5 + (6 × x) = 29
    (4 × x) = ___             (6 × x) = ___
          x = ___                   x = ___
```

Three-row step-by-step layout with vertically aligned `=` signs. Row 1 is
the equation as given; rows 2 and 3 are the student's work. The variable
`x` is rendered in STIX Two Text italic — classical serif LaTeX-style
variable — so it's visually distinct from the sans-serif digits and
instantly readable as a variable rather than a letter.

Explicit multiplication is grouped in parentheses `(4 × x)` to emphasize
that the coefficient and variable form a single quantity. Implicit form
(`4x`) drops both the operator and the parens.

### Parameters

- `a_range: range` — coefficient range (default: `2-10`). Covers the
  times-table facts the student already knows.
- `b_range: range` — constant range (default: `1-30`).
- `x_range: range` — allowed answer range (default: `0-20`). `x = 0` and
  `x = 1` are deliberately included — the student should understand that
  `5x + 10 = 10` means `x = 0`.
- `implicit: bool` — if true, render coefficient–variable as `4x`
  (juxtaposition, no parens). If false (default), render with explicit
  operator inside parens: `(4 · x)` (Norway) or `(4 × x)` (US).
- `mix_forms: bool` — if true, randomly vary the equation form between
  `ax + b = c` and `b + ax = c`. Rows 2/3 stay canonical (`ax = ...`,
  `x = ...`) and the `=` signs still align. Default: `true`.
- `solve_first: bool` — render the first problem filled in as a worked
  example showing the intermediate `ax = c - b` and final `x = answer`.
  Default: `false`. Same convention as fraction multiplication.

### Layout

- Three-row grid per problem, `=` column shared across all rows.
- Right-align row 1 so the `=` sits in a consistent column; rows 2 and 3
  indent to keep `ax =` and `x =` in the canonical positions.
- Fixed-width slot reserved to the right of `=` on each row so solved and
  unsolved problems occupy the same bounding box and the worksheet grid
  stays aligned.
- 2 columns on the page — the equations are wide (especially with
  parens), and three-row problems need vertical breathing room.

### Constraints

- Whole-integer answer only. `c - b` must be divisible by `a`.
- `c = a*x + b` is a derived value; not directly configurable.
- `a ≥ 2` (coefficient of 1 collapses to a one-step problem).
- `x ≥ 0` (no negatives at 5th-grade level).

### Equation forms

Three forms mix when `mix_forms` is on:

| Form | Example | Work row 1 intermediate |
|---|---|---|
| Canonical plus   | `(4 · x) + 5 = 21`  | `4 · x = c - b` |
| Const-first plus | `5 + (4 · x) = 21`  | `4 · x = c - b` |
| Canonical minus  | `(4 · x) - 3 = 17`  | `4 · x = c + b` |

The canonical-minus form is only emitted for triples where `a · x ≥ b`,
so `c` stays non-negative. The RHS-flipped form (`21 = 4x + 5`) is
deferred — it complicates equals-column alignment with canonical work
rows.

### Locale

Unlike the drills and fraction multiplication, algebra always uses `·`
regardless of locale. Reason: `×` looks too much like the variable `x`
once variables are introduced. This matches the US pre-algebra convention
(elementary `×` → pre-algebra `·` → algebra implicit). `--symbol` still
overrides.

### Problem generation

1. Enumerate all valid `(a, b, x)` triples in their ranges.
2. Compute `c = a*x + b`.
3. Randomly assign a form (canonical vs. const-first) if `mix_forms` is on.
4. Shuffle; take `num_problems`.
5. Deduplicate so the same `(a, b, x, form)` never repeats.

### Layout component

The problem renders as a 3-row grid:

```typst
two-step-problem((a: 4, b: 5, x: 4, form: "canonical"), debug: false, solved: false)
```

The component owns the form-to-row-1 rendering; rows 2 and 3 are always
`ax = ...` and `x = ...` regardless of form.
