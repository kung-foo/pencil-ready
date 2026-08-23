// Loads the per-worksheet help text out of the `worksheets` content
// collection and hands it back keyed by kind, the shape the pages want.
//
// The collection lives in markdown (`src/content/worksheets/<kind>.md`)
// so the prose can be edited without touching TypeScript; the schema in
// `src/content.config.ts` validates it, and `WORKSHEET_SECTIONS` in
// `worksheet-info.ts` stays in code because homepage grouping is layout,
// not copy.

import { getCollection } from "astro:content";
import { WORKSHEET_KINDS, type WorksheetKind } from "./api";

export type WorksheetInfo = {
    /** Plain-English title, matches the server's `title()` output. */
    title: string;
    /** One-sentence description of what the worksheet drills. */
    summary: string;
    /** What the student should already be comfortable with. */
    prerequisites: string[];
    /** Skills gained with mastery of this worksheet. */
    learning: string[];
};

/**
 * Every kind's help text, keyed by kind.
 *
 * Throws when a kind has no markdown file, or when a file exists for a
 * kind that isn't in `WORKSHEET_KINDS`. Both are build-time failures on
 * purpose: the alternative is a worksheet page that renders with empty
 * "Know before you start" and "the student can" sections, which looks
 * like a styling bug rather than missing content.
 */
export async function getWorksheetInfo(): Promise<
    Record<WorksheetKind, WorksheetInfo>
> {
    const entries = await getCollection("worksheets");
    const byKind = new Map(entries.map((e) => [e.id, e.data]));

    const unknown = [...byKind.keys()].filter(
        (id) => !(WORKSHEET_KINDS as readonly string[]).includes(id),
    );
    if (unknown.length > 0) {
        throw new Error(
            `src/content/worksheets has files for unknown kinds: ${unknown.join(", ")}`,
        );
    }

    const info = {} as Record<WorksheetKind, WorksheetInfo>;
    const missing: string[] = [];
    for (const kind of WORKSHEET_KINDS) {
        const data = byKind.get(kind);
        if (!data) {
            missing.push(kind);
            continue;
        }
        info[kind] = data;
    }
    if (missing.length > 0) {
        throw new Error(
            `src/content/worksheets is missing help text for: ${missing.join(", ")}`,
        );
    }
    return info;
}
