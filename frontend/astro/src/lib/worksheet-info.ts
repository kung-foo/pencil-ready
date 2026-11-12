// Homepage grouping only — the per-worksheet help text lives in the
// `worksheets` content collection (see lib/worksheet-content.ts).
import { WORKSHEET_KINDS, type WorksheetKind } from "@/lib/api";
import type { UiStrings } from "@/i18n";

/** Homepage grouping. Ordered by rough curriculum progression — the
 * list order drives render order, and each section's `kinds` array
 * drives the per-section card order. Must cover every kind in
 * `WORKSHEET_KINDS` exactly once (enforced by the homepage's build).
 * `id` keys the localized heading in `UiStrings.home.sections`. */
export const WORKSHEET_SECTIONS: ReadonlyArray<{
    id: keyof UiStrings["home"]["sections"];
    kinds: ReadonlyArray<WorksheetKind>;
}> = [
    {
        id: "arithmetic",
        kinds: ["add", "subtract", "multiply", "simple-divide"],
    },
    {
        id: "drills",
        kinds: ["mult-drill", "div-drill"],
    },
    {
        id: "longForm",
        kinds: ["long-divide"],
    },
    {
        // Right before fractions: the GCF simplifies a fraction in one
        // step (and a least common multiple, later, gives a common
        // denominator). Factor pairs first: listing factors is what the
        // GCF sheet does twice; prime factors build on factor pairs.
        id: "factors",
        kinds: ["factor-pairs", "gcf", "prime-factors"],
    },
    {
        id: "fractions",
        kinds: [
            "fraction-equiv",
            "fraction-simplify",
            "fraction-add",
            "fraction-subtract",
            "fraction-mult",
        ],
    },
    {
        id: "decimals",
        kinds: ["decimal-add", "decimal-subtract", "decimal-multiply"],
    },
    {
        id: "percent",
        // Conversion first: percent as hundredths is the idea the
        // percent-of sheet builds on.
        kinds: ["fraction-decimal-percent", "percent-of"],
    },
    {
        id: "statistics",
        kinds: ["mean"],
    },
    {
        id: "preAlgebra",
        // order-of-ops first: precedence + parens are the conceptual
        // prereqs for the one-step / two-step equation work.
        kinds: [
            "order-of-ops",
            "algebra-one-step",
            "algebra-two-step",
            "algebra-square-root",
        ],
    },
];

/** The homepage anchor for a section: kebab-case of its key, English in
 * every locale, so `/#fractions` and `/no/#fractions` both work. Shared
 * by the homepage and the header search so the two can't drift. */
export function sectionAnchor(
    id: (typeof WORKSHEET_SECTIONS)[number]["id"],
): string {
    return id.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
}

// Coverage check: every worksheet kind must appear in exactly one
// section. Runs at module load so a missing / duplicated kind fails
// the Astro build rather than producing an invisibly-broken homepage.
{
    const flat = WORKSHEET_SECTIONS.flatMap((s) => s.kinds);
    const seen = new Set(flat);
    const missing = WORKSHEET_KINDS.filter((k) => !seen.has(k));
    if (missing.length > 0) {
        throw new Error(
            `WORKSHEET_SECTIONS missing kinds: ${missing.join(", ")}`,
        );
    }
    if (flat.length !== seen.size) {
        throw new Error("WORKSHEET_SECTIONS has duplicate kinds");
    }
}
