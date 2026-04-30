// Shape-for-shape mirror of the pencil-ready-server API. Keep in sync with
// /openapi.json — which is now the source of truth for param names.
//
// Enums are declared as `const` arrays of string literals and the TypeScript
// types are derived via `(typeof X)[number]`, so each enum lives in exactly
// one place and can be iterated for select-option rendering.

export const FORMATS = ["pdf", "png", "svg"] as const;
export type Format = (typeof FORMATS)[number];

export const CARRY_MODES = ["none", "any", "force", "ripple"] as const;
export type CarryMode = (typeof CARRY_MODES)[number];

export const BORROW_MODES = [
    "none",
    "no-across-zero",
    "any",
    "force",
    "ripple",
] as const;
export type BorrowMode = (typeof BORROW_MODES)[number];

export const WORKSHEET_KINDS = [
    "add",
    "subtract",
    "multiply",
    "simple-divide",
    "long-divide",
    "mult-drill",
    "div-drill",
    "fraction-mult",
    "fraction-simplify",
    "fraction-equiv",
    "algebra-one-step",
    "algebra-two-step",
] as const;
export type WorksheetKind = (typeof WORKSHEET_KINDS)[number];

/** Nicely-phrased labels for dropdowns. Keys mirror the array constants. */
export const CARRY_MODE_LABELS: Record<CarryMode, string> = {
    none: "None",
    any: "Any",
    force: "Force",
    ripple: "Ripple (multi-column)",
};

export const BORROW_MODE_LABELS: Record<BorrowMode, string> = {
    none: "None",
    "no-across-zero": "No across zero",
    any: "Any",
    force: "Force",
    ripple: "Ripple (multi-column)",
};

export type SharedConfig = {
    format: Format;
    seed?: number;
    solve_first?: boolean;
    /** Append an answer-key page showing just the final answers. PDF only. */
    include_answers?: boolean;
    problems?: number;
    cols?: number;
};

// Per-kind config (only the distinguishing params; rest fall back to API defaults).
export type KindConfig =
    | { kind: "add"; digits?: string; carry?: CarryMode; binary?: boolean }
    | { kind: "subtract"; digits?: string; borrow?: BorrowMode }
    | { kind: "multiply"; digits?: string }
    | { kind: "simple-divide"; max_quotient?: number }
    | { kind: "long-divide"; digits?: string; remainder?: boolean }
    | {
          kind: "mult-drill";
          multiplicand?: string;
          multiplier?: string;
          count?: number;
      }
    | {
          kind: "div-drill";
          divisor?: string;
          max_quotient?: string;
          count?: number;
      }
    | {
          kind: "fraction-mult";
          denominators?: string;
          min_whole?: number;
          max_whole?: number;
          unit_only?: boolean;
      }
    | {
          kind: "fraction-simplify";
          denominators?: string;
          max_numerator?: number;
          proper_only?: boolean;
          include_whole?: boolean;
      }
    | {
          kind: "fraction-equiv";
          denominators?: string;
          scale?: string;
          missing?: "any" | "left-num" | "left-den" | "right-num" | "right-den";
          proper_only?: boolean;
      }
    | {
          kind: "algebra-two-step";
          a_range?: string;
          b_range?: string;
          x_range?: string;
          implicit?: boolean;
          mix_forms?: boolean;
      }
    | {
          kind: "algebra-one-step";
          a_range?: string;
          b_range?: string;
          x_range?: string;
          add?: boolean;
          subtract?: boolean;
          multiply?: boolean;
          divide?: boolean;
      };

export type WorksheetConfig = SharedConfig & KindConfig;

/** Serialize a config to query-string form (omits the `kind` — that's a route param). */
export function configToSearchParams(cfg: WorksheetConfig): URLSearchParams {
    const q = new URLSearchParams();
    const { kind: _kind, ...rest } = cfg;
    for (const [k, v] of Object.entries(rest)) {
        if (v === undefined || v === "" || v === null || v === false) continue;
        q.set(k, String(v));
    }
    return q;
}

/** Parse search params into the shape each kind expects. Invalid/missing → defaults. */
export function parseConfig(
    kind: WorksheetKind,
    sp: URLSearchParams,
): WorksheetConfig {
    const s = (k: string) => sp.get(k) ?? undefined;
    const n = (k: string) => {
        const v = sp.get(k);
        return v === null || v === "" ? undefined : Number(v);
    };
    const b = (k: string) => (sp.has(k) ? sp.get(k) !== "false" : undefined);

    const format = (FORMATS as readonly string[]).includes(s("format") ?? "")
        ? (s("format") as Format)
        : "pdf";

    const shared: SharedConfig = {
        format,
        seed: n("seed"),
        solve_first: b("solve_first"),
        include_answers: b("include_answers"),
        problems: n("problems"),
        cols: n("cols"),
    };

    switch (kind) {
        case "add":
            return {
                ...shared,
                kind,
                digits: s("digits"),
                carry: asEnum(s("carry"), CARRY_MODES),
                binary: b("binary"),
            };
        case "subtract":
            return {
                ...shared,
                kind,
                digits: s("digits"),
                borrow: asEnum(s("borrow"), BORROW_MODES),
            };
        case "multiply":
            return { ...shared, kind, digits: s("digits") };
        case "simple-divide":
            return { ...shared, kind, max_quotient: n("max_quotient") };
        case "long-divide":
            return {
                ...shared,
                kind,
                digits: s("digits"),
                remainder: b("remainder"),
            };
        case "mult-drill":
            return {
                ...shared,
                kind,
                multiplicand: s("multiplicand"),
                multiplier: s("multiplier"),
                count: n("count"),
            };
        case "div-drill":
            return {
                ...shared,
                kind,
                divisor: s("divisor"),
                max_quotient: s("max_quotient"),
                count: n("count"),
            };
        case "fraction-mult":
            return {
                ...shared,
                kind,
                denominators: s("denominators"),
                min_whole: n("min_whole"),
                max_whole: n("max_whole"),
                unit_only: b("unit_only"),
            };
        case "fraction-simplify":
            return {
                ...shared,
                kind,
                denominators: s("denominators"),
                max_numerator: n("max_numerator"),
                proper_only: b("proper_only"),
                include_whole: b("include_whole"),
            };
        case "fraction-equiv":
            return {
                ...shared,
                kind,
                denominators: s("denominators"),
                scale: s("scale"),
                missing: asEnum(s("missing"), [
                    "any",
                    "left-num",
                    "left-den",
                    "right-num",
                    "right-den",
                ] as const),
                proper_only: b("proper_only"),
            };
        case "algebra-two-step":
            return {
                ...shared,
                kind,
                a_range: s("a_range"),
                b_range: s("b_range"),
                x_range: s("x_range"),
                implicit: b("implicit"),
                mix_forms: b("mix_forms"),
            };
        case "algebra-one-step":
            return {
                ...shared,
                kind,
                a_range: s("a_range"),
                b_range: s("b_range"),
                x_range: s("x_range"),
                add: b("add"),
                subtract: b("subtract"),
                multiply: b("multiply"),
                divide: b("divide"),
            };
    }
}

function asEnum<T extends readonly string[]>(
    value: string | undefined,
    allowed: T,
): T[number] | undefined {
    return value && (allowed as readonly string[]).includes(value)
        ? (value as T[number])
        : undefined;
}

/** Build the API URL the server serves for a given config. Names are
 * appended here (not in `configToSearchParams`) so they reach the server
 * without polluting the shareable browser URL. */
export function worksheetUrl(
    cfg: WorksheetConfig,
    names?: { student?: string },
): string {
    const qs = configToSearchParams(cfg);
    if (names?.student) qs.set("student_name", names.student);
    const s = qs.toString();
    return `/api/worksheets/${cfg.kind}${s ? `?${s}` : ""}`;
}
