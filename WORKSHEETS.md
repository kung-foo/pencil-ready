# Worksheet Types

## Constraints

- **Max digits per operand**: 5
- **Max problems per page**: 16
- **Max operands**: 4 (addition only; subtraction and multiplication are always 2)

## Shared parameters

All worksheet types accept these:

- `digits: list[u32]` — digit count per operand (1-5 each), comma-separated. Length determines the number of operands. (default: 2,2)
  - `2,2` → two 2-digit operands (e.g. 34 + 57)
  - `2,2,2` → three 2-digit operands (e.g. 21 + 34 + 45)
  - `3,2` → 3-digit and 2-digit (e.g. 342 - 56)
  - `3,1` → 3-digit and 1-digit (e.g. 456 × 7)
- `num_problems: u32` — number of problems on the page (1-16, default: 12, must be divisible by cols)
- `cols: u32` — columns in the grid (default: 4)
- `font: string` — font name (default: "Cascadia Code")
- `paper: string` — paper size, passed to typst (default: "a4", also: "us-letter", "a3", etc.)
- `seed: u64?` — optional random seed for reproducible worksheets
- `debug: bool` — show red borders on boxes, blue borders on grid cells

## Addition

Supports 2 or more operands via `--digits` (e.g. `--digits 2,2,2` for three addends).

- `carry: none | any | force | ripple` (default: any)
  - **none** — no column sum reaches 10 (no carrying). Easiest.
  - **any** — no constraint, random mix.
  - **force** — every problem has at least one carry.
  - **ripple** — every problem has a carry that chains through 2+ consecutive columns. Hardest.
  - Carry detection works correctly for 2 or 3+ operands (column sums can exceed 20 with 3 addends).

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