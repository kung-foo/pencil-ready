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
//
// One alias table per locale, matched against that locale's help text;
// links point at the same locale's pages.

import type { WorksheetKind } from "./api";
import { localizePath, type Locale } from "@/i18n/locales";

type Alias = { phrase: string; kind: WorksheetKind };

/** A phrase and the worksheet it names. Order is irrelevant — matching
 * always prefers the longest phrase at a given position, so "Column
 * addition" beats "addition" and "Long division" beats "division". */
const ALIASES_EN: ReadonlyArray<Alias> = [
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

/** Norwegian aliases — phrases as they appear in `content/worksheets/nb/`.
 * Same conservatism: "brøk" alone is absent for the same reason as
 * "fraction". */
const ALIASES_NB: ReadonlyArray<Alias> = [
    { phrase: "gangetrening", kind: "mult-drill" },
    { phrase: "divisjonstrening", kind: "div-drill" },
    { phrase: "addisjon med mente", kind: "add" },
    { phrase: "subtraksjon med låning", kind: "subtract" },
    { phrase: "oppstilt multiplikasjon", kind: "multiply" },
    { phrase: "addisjon med desimaltall", kind: "decimal-add" },
    { phrase: "subtraksjon med desimaltall", kind: "decimal-subtract" },
    { phrase: "multiplikasjon med desimaltall", kind: "decimal-multiply" },
    { phrase: "lang divisjon", kind: "long-divide" },
    { phrase: "divisjon med rest", kind: "long-divide" },
    { phrase: "regnerekkefølge", kind: "order-of-ops" },
    { phrase: "likeverdige brøker", kind: "fraction-equiv" },
    { phrase: "gangetabellen", kind: "mult-drill" },
    { phrase: "likninger i ett steg", kind: "algebra-one-step" },
    { phrase: "likninger i to steg", kind: "algebra-two-step" },
    { phrase: "kvadratrøtter", kind: "algebra-square-root" },
    { phrase: "kvadratrot", kind: "algebra-square-root" },
    { phrase: "addisjon", kind: "add" },
    { phrase: "subtraksjon", kind: "subtract" },
    { phrase: "multiplikasjon", kind: "multiply" },
    { phrase: "divisjon", kind: "simple-divide" },
    { phrase: "gjennomsnitt", kind: "mean" },
];

const ALIASES: Record<Locale, ReadonlyArray<Alias>> = {
    en: ALIASES_EN,
    no: ALIASES_NB,
};

/** One run of prose, optionally a link. */
export type Segment = { text: string; href?: string };

/** Escape a phrase for use in a regex. Aliases are hand-written, but a
 * hyphen or dot slipping in shouldn't silently change the pattern. */
function escapeRe(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// One alternation of every alias, longest first, bounded so "addition"
// doesn't match inside "additional". The boundaries are Unicode-aware
// lookarounds rather than `\b`, which only knows ASCII letters and would
// treat the "ø" in "regnerekkefølge" as a word break.
function matcherFor(aliases: ReadonlyArray<Alias>) {
    const byLengthDesc = [...aliases].sort(
        (a, b) => b.phrase.length - a.phrase.length,
    );
    const re = new RegExp(
        `(?<![\\p{L}\\p{N}])(${byLengthDesc.map((a) => escapeRe(a.phrase)).join("|")})(?![\\p{L}\\p{N}])`,
        "giu",
    );
    return { byLengthDesc, re };
}

const MATCHERS = Object.fromEntries(
    Object.entries(ALIASES).map(([locale, aliases]) => [
        locale,
        matcherFor(aliases),
    ]),
) as Record<Locale, ReturnType<typeof matcherFor>>;

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
    locale: Locale,
): Segment[][] {
    const { byLengthDesc, re } = MATCHERS[locale];
    const linked = new Set<WorksheetKind>();
    return texts.map((text) => {
        const segments: Segment[] = [];
        let last = 0;

        for (const m of text.matchAll(re)) {
            const alias = byLengthDesc.find(
                (a) => a.phrase === m[0].toLowerCase(),
            );
            if (!alias || alias.kind === self || linked.has(alias.kind)) {
                continue;
            }

            const start = m.index ?? 0;
            if (start > last) segments.push({ text: text.slice(last, start) });
            segments.push({
                text: m[0],
                href: localizePath(`/worksheets/${alias.kind}/`, locale),
            });
            linked.add(alias.kind);
            last = start + m[0].length;
        }

        if (last < text.length) segments.push({ text: text.slice(last) });
        return segments;
    });
}
