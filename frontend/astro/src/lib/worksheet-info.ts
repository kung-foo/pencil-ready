// Homepage grouping only — the per-worksheet help text lives in the
// `worksheets` content collection (see lib/worksheet-content.ts).
import { WORKSHEET_KINDS, type WorksheetKind } from "@/lib/api";

/** Homepage grouping. Ordered by rough curriculum progression — the
 * list order drives render order, and each section's `kinds` array
 * drives the per-section card order. Must cover every kind in
 * `WORKSHEET_KINDS` exactly once (enforced by the homepage's build). */
export const WORKSHEET_SECTIONS: ReadonlyArray<{
    title: string;
    kinds: ReadonlyArray<WorksheetKind>;
}> = [
    {
        title: "Arithmetic",
        kinds: ["add", "subtract", "multiply", "simple-divide"],
    },
    {
        title: "Fact drills",
        kinds: ["mult-drill", "div-drill"],
    },
    {
        title: "Long-form",
        kinds: ["long-divide"],
    },
    {
        title: "Fractions",
        kinds: ["fraction-equiv", "fraction-simplify", "fraction-mult"],
    },
    {
        title: "Decimals",
        kinds: ["decimal-add", "decimal-subtract", "decimal-multiply"],
    },
    {
        title: "Statistics",
        kinds: ["mean"],
    },
    {
        title: "Pre-algebra",
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
