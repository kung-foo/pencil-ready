// Turns worksheet names mentioned in prose into links to those pages.
//
// Prerequisites are, by their nature, descriptions of *other* worksheets
// ("Column addition with carrying", "Long division by a single-digit
// divisor"), so a reader who isn't ready for this sheet yet should be one
// click from the one they need. Rather than hand-marking every string,
// a curated alias table is matched against the text — that way a new
// worksheet only needs its aliases added here, not edits scattered
// through `worksheet-info.ts`.
//
// Deliberately conservative: only phrases that unambiguously name one
// worksheet are listed. "fraction" is absent because it could mean any of
// three fraction sheets, and a link to an arbitrary one is worse than no
// link at all.

import type { WorksheetKind } from "./api";

/** A phrase and the worksheet it names. Order is irrelevant — matching
 * always prefers the longest phrase at a given position, so "Column
 * addition" beats "addition" and "Long division" beats "division". */
const ALIASES: ReadonlyArray<{ phrase: string; kind: WorksheetKind }> = [
    // Multi-word names first for readability; length ordering is applied
    // at match time regardless.
    { phrase: "multiplication drill", kind: "mult-drill" },
    { phrase: "division drill", kind: "div-drill" },
    { phrase: "column addition", kind: "add" },
    { phrase: "column subtraction", kind: "subtract" },
    { phrase: "column multiplication", kind: "multiply" },
    { phrase: "decimal addition", kind: "decimal-add" },
    { phrase: "decimal subtraction", kind: "decimal-subtract" },
    { phrase: "decimal multiplication", kind: "decimal-multiply" },
    { phrase: "simple division", kind: "simple-divide" },
    { phrase: "long division", kind: "long-divide" },
    { phrase: "division with remainder", kind: "long-divide" },
    { phrase: "order of operations", kind: "order-of-ops" },
    { phrase: "equivalent fractions", kind: "fraction-equiv" },
    // Times tables are what the drills exist for.
    { phrase: "times tables", kind: "mult-drill" },
    { phrase: "times table", kind: "mult-drill" },
    { phrase: "times-table", kind: "mult-drill" },
    // Algebra: the step count is the distinguishing word.
    { phrase: "one-step", kind: "algebra-one-step" },
    { phrase: "two-step", kind: "algebra-two-step" },
    { phrase: "square root", kind: "algebra-square-root" },
    { phrase: "square roots", kind: "algebra-square-root" },
    // Bare operation names resolve to the canonical sheet for that
    // operation. "division" goes to the times-table sheet rather than
    // long division: it's the one a student blocked on this prerequisite
    // would start with.
    { phrase: "addition", kind: "add" },
    { phrase: "subtraction", kind: "subtract" },
    { phrase: "multiplication", kind: "multiply" },
    { phrase: "division", kind: "simple-divide" },
    { phrase: "mean", kind: "mean" },
];

/** One run of prose, optionally a link. */
export type Segment = { text: string; href?: string };

const byLengthDesc = [...ALIASES].sort(
    (a, b) => b.phrase.length - a.phrase.length,
);

/** Escape a phrase for use in a regex. Aliases are hand-written, but a
 * hyphen or dot slipping in shouldn't silently change the pattern. */
function escapeRe(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// One alternation of every alias, longest first, with word boundaries so
// "addition" doesn't match inside "additional". `\b` sits outside the
// group because some aliases end in "-step" and `\b` after a hyphenated
// word still behaves.
const MATCHER = new RegExp(
    `\\b(${byLengthDesc.map((a) => escapeRe(a.phrase)).join("|")})\\b`,
    "gi",
);

/**
 * Split a list of prose strings into segments, linking the first mention
 * of each worksheet.
 *
 * `self` is the page being rendered; mentions of it stay plain text — a
 * link to the page you're already on is noise. Each worksheet is linked
 * at most once across the *whole list*, not once per string: several
 * bullets often name the same operation, and four underlined words that
 * all lead to the same place read as clutter rather than navigation.
 */
export function linkWorksheetList(
    texts: readonly string[],
    self: WorksheetKind,
): Segment[][] {
    const linked = new Set<WorksheetKind>();
    return texts.map((text) => {
        const segments: Segment[] = [];
        let last = 0;

        for (const m of text.matchAll(MATCHER)) {
            const alias = byLengthDesc.find(
                (a) => a.phrase === m[0].toLowerCase(),
            );
            if (!alias || alias.kind === self || linked.has(alias.kind)) {
                continue;
            }

            const start = m.index ?? 0;
            if (start > last) segments.push({ text: text.slice(last, start) });
            segments.push({ text: m[0], href: `/worksheets/${alias.kind}/` });
            linked.add(alias.kind);
            last = start + m[0].length;
        }

        if (last < text.length) segments.push({ text: text.slice(last) });
        return segments;
    });
}
