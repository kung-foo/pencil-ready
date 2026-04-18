// Shape-for-shape mirror of the pencil-ready-server API. Keep in sync with
// /openapi.json — which is now the source of truth for param names.

export type WorksheetKind =
  | "add"
  | "subtract"
  | "multiply"
  | "simple-divide"
  | "long-divide"
  | "mult-drill"
  | "div-drill"
  | "fraction-mult"
  | "algebra-two-step";

export type Format = "pdf" | "png" | "svg";

export type SharedConfig = {
  format: Format;
  seed?: number;
  solve_first?: boolean;
  problems?: number;
  cols?: number;
};

// Per-kind config (only the distinguishing params; rest fall back to API defaults).
export type KindConfig =
  | { kind: "add"; digits?: string; carry?: "none" | "any" | "force" | "ripple"; binary?: boolean }
  | { kind: "subtract"; digits?: string; borrow?: "none" | "no-across-zero" | "any" | "force" | "ripple" }
  | { kind: "multiply"; digits?: string }
  | { kind: "simple-divide"; max_quotient?: number }
  | { kind: "long-divide"; digits?: string; remainder?: boolean }
  | { kind: "mult-drill"; multiplicand?: string; multiplier?: string; count?: number }
  | { kind: "div-drill"; divisor?: string; max_quotient?: string; count?: number }
  | { kind: "fraction-mult"; denominators?: string; min_whole?: number; max_whole?: number; unit_only?: boolean }
  | { kind: "algebra-two-step"; a_range?: string; b_range?: string; x_range?: string; implicit?: boolean; mix_forms?: boolean };

export type WorksheetConfig = SharedConfig & KindConfig;

/** Build the URL the server serves for a given config — used as iframe/img src. */
export function worksheetUrl(cfg: WorksheetConfig): string {
  const q = new URLSearchParams();
  const { kind, ...rest } = cfg;
  for (const [k, v] of Object.entries(rest)) {
    if (v === undefined || v === "" || v === null) continue;
    q.set(k, String(v));
  }
  const qs = q.toString();
  return `/api/worksheets/${kind}${qs ? `?${qs}` : ""}`;
}
