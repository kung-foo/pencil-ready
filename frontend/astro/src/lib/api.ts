// Shape-for-shape mirror of the pencil-ready-server API. Keep in sync with
// /openapi.json — which is now the source of truth for param names.
//
// Enums are declared as `const` arrays of string literals and the TypeScript
// types are derived via `(typeof X)[number]`, so each enum lives in exactly
// one place and can be iterated for select-option rendering.
//
// Some kinds use the concept-level system (`lib/levels.ts`) instead of
// exposing raw params: the configurator emits `?level=...` to the
// browser URL, and `worksheetUrl()` expands it back to the raw params
// the server expects.

import { levelParams } from "./levels";

export const FORMATS = ["pdf", "png", "svg"] as const;
export type Format = (typeof FORMATS)[number];

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
    "algebra-square-root",
    "decimal-add",
    "decimal-subtract",
    "decimal-multiply",
] as const;
export type WorksheetKind = (typeof WORKSHEET_KINDS)[number];

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
    | {
          kind: "add";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | {
          kind: "subtract";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | { kind: "multiply"; digits?: string }
    | { kind: "simple-divide"; max_quotient?: number }
    | {
          kind: "long-divide";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
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
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | {
          kind: "algebra-one-step";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | {
          kind: "algebra-square-root";
          b_range?: string;
          squares?: boolean;
          roots?: boolean;
      }
    | {
          kind: "decimal-add";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | {
          kind: "decimal-subtract";
          /** Concept-level preset id (see `lib/levels.ts`). */
          level?: string;
      }
    | {
          kind: "decimal-multiply";
          /** Concept-level preset id (see `lib/levels.ts`). Expanded to
           * raw multiplier/digit params at API-fetch time. */
          level?: string;
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
            return { ...shared, kind, level: s("level") };
        case "subtract":
            return { ...shared, kind, level: s("level") };
        case "multiply":
            return { ...shared, kind, digits: s("digits") };
        case "simple-divide":
            return { ...shared, kind, max_quotient: n("max_quotient") };
        case "long-divide":
            return { ...shared, kind, level: s("level") };
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
            return { ...shared, kind, level: s("level") };
        case "algebra-one-step":
            return { ...shared, kind, level: s("level") };
        case "algebra-square-root":
            return {
                ...shared,
                kind,
                b_range: s("b_range"),
                squares: b("squares"),
                roots: b("roots"),
            };
        case "decimal-add":
        case "decimal-subtract":
            return { ...shared, kind, level: s("level") };
        case "decimal-multiply":
            return {
                ...shared,
                kind,
                level: s("level"),
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
 * without polluting the shareable browser URL.
 *
 * For kinds that use the level system (`lib/levels.ts`), the `level=...`
 * query param is replaced with the level's raw params (digits,
 * decimal_places, etc.) so the server — which knows nothing about
 * levels — receives the expanded form. The browser URL still shows just
 * `?level=...` (set by `configToSearchParams`).
 *
 * When `cfg.seed` is set, a `share_url=` param is appended pointing back
 * at the *frontend* route (with the original `level=...` form, not the
 * expanded one). The server validates it against an allowed-origin list
 * and embeds it as a QR code in the bottom-right of every page. */
export function worksheetUrl(
    cfg: WorksheetConfig,
    names?: { student?: string },
): string {
    const qs = configToSearchParams(cfg);
    // Snapshot the frontend-form qs (with `level=…`) before the API
    // expansion below mutates it. This is what the QR points to so a
    // scan lands on the same configurator state, not the expanded
    // raw-params form (which `parseConfig` for level kinds ignores).
    const shareSearch = new URLSearchParams(qs);
    const levelValue = qs.get("level");
    if (levelValue) {
        const expanded = levelParams(cfg.kind, levelValue);
        if (expanded) {
            qs.delete("level");
            for (const [k, v] of Object.entries(expanded)) {
                qs.set(k, String(v));
            }
        }
    }
    if (names?.student) qs.set("student_name", names.student);
    // QR rendering is gated server-side by `qr=true`. Disabled by
    // default until a UI toggle lands — when it does, the block below
    // should gate on the toggle. The server defaults `qr` to false, so
    // simply not emitting the params is enough to suppress rendering;
    // we keep the snapshot/build logic ready for the UI to enable.
    const QR_ENABLED = false;
    if (QR_ENABLED && typeof window !== "undefined" && cfg.seed !== undefined) {
        const shareTail = shareSearch.toString();
        const shareUrl = `${window.location.origin}/worksheets/${cfg.kind}/${shareTail ? `?${shareTail}` : ""}`;
        qs.set("share_url", shareUrl);
        qs.set("qr", "true");
    }
    const s = qs.toString();
    return `/api/worksheets/${cfg.kind}${s ? `?${s}` : ""}`;
}
