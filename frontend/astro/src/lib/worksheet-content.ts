// Loads the per-worksheet help text out of the `worksheets` content
// collection and hands it back keyed by kind, the shape the pages want.
//
// The collection lives in markdown (`src/content/worksheets/<bcp47>/<kind>.md`)
// so the prose can be edited without touching TypeScript; the schema in
// `src/content.config.ts` validates it, and `WORKSHEET_SECTIONS` in
// `worksheet-info.ts` stays in code because homepage grouping is layout,
// not copy.

import { getCollection } from "astro:content";
import { WORKSHEET_KINDS, type WorksheetKind } from "./api";
import { LOCALES, localeDef, type Locale } from "@/i18n/locales";

export type WorksheetInfo = {
    /** Title in the page's language, matches the server's `title()` output. */
    title: string;
    /** One-sentence description of what the worksheet drills. */
    summary: string;
    /** What the student should already be comfortable with. */
    prerequisites: string[];
    /** Skills gained with mastery of this worksheet. */
    learning: string[];
};

type AllInfo = Record<Locale, Record<WorksheetKind, WorksheetInfo>>;

/**
 * Every locale's help text, keyed by locale then kind.
 *
 * Throws when any locale is missing a file for a kind, when a file
 * exists for an unknown kind, or when a folder doesn't belong to any
 * locale. All are build-time failures on purpose: the alternative is a
 * worksheet page that renders with empty "Know before you start" and
 * "the student can" sections, which looks like a styling bug rather
 * than missing content.
 */
async function loadAll(): Promise<AllInfo> {
    const entries = await getCollection("worksheets");
    const folders = new Map<string, Map<string, WorksheetInfo>>();
    for (const e of entries) {
        const [folder, kind, ...rest] = e.id.split("/");
        if (!kind || rest.length > 0) {
            throw new Error(
                `src/content/worksheets/${e.id}: expected <language>/<kind>.md`,
            );
        }
        if (!folders.has(folder)) folders.set(folder, new Map());
        folders.get(folder)!.set(kind, e.data);
    }

    // Astro lowercases collection ids, so a region-tagged folder like
    // `nb-NO/` arrives as `nb-no`; compare tags case-insensitively.
    const known = new Set<string>(LOCALES.map((l) => l.bcp47.toLowerCase()));
    const strayFolders = [...folders.keys()].filter((f) => !known.has(f));
    if (strayFolders.length > 0) {
        throw new Error(
            `src/content/worksheets has folders for unknown languages: ${strayFolders.join(", ")}`,
        );
    }

    const all = {} as AllInfo;
    const problems: string[] = [];
    for (const { id, bcp47 } of LOCALES) {
        const byKind = folders.get(bcp47.toLowerCase()) ?? new Map();
        const unknown = [...byKind.keys()].filter(
            (k) => !(WORKSHEET_KINDS as readonly string[]).includes(k),
        );
        if (unknown.length > 0) {
            problems.push(
                `${bcp47}/ has files for unknown kinds: ${unknown.join(", ")}`,
            );
        }
        const info = {} as Record<WorksheetKind, WorksheetInfo>;
        const missing: string[] = [];
        for (const kind of WORKSHEET_KINDS) {
            const data = byKind.get(kind);
            if (!data) missing.push(kind);
            else info[kind] = data;
        }
        if (missing.length > 0) {
            problems.push(
                `${bcp47}/ is missing help text for: ${missing.join(", ")}`,
            );
        }
        all[id] = info;
    }
    if (problems.length > 0) {
        throw new Error(`src/content/worksheets: ${problems.join("; ")}`);
    }
    return all;
}

/** One locale's help text, keyed by kind. */
export async function getWorksheetInfo(
    locale: Locale,
): Promise<Record<WorksheetKind, WorksheetInfo>> {
    return (await loadAll())[localeDef(locale).id];
}
