// The header search's index: one JSON file per locale, built at build
// time by pages/[...locale]/search-index.json.ts and loaded into
// MiniSearch the first time the search panel is used
// (components/WorksheetSearch.astro).
//
// Short keys keep the file small. Every field is a plain string, so the
// client hands the documents straight to MiniSearch. Multi-value fields
// are joined with newlines; the search panel splits them again to show
// the line that matched.
//
// Prerequisites stay out on purpose: they describe *other* worksheets,
// so matching them would surface the wrong sheet.

import { t } from "@/i18n";
import { LOCALES, localizePath, type Locale } from "@/i18n/locales";
import { toPlainText } from "./math-text";
import { getWorksheetInfo } from "./worksheet-content";
import { WORKSHEET_SECTIONS } from "./worksheet-info";
import { worksheetAliases } from "./worksheet-links";

/** One worksheet in the search index. */
export type SearchDoc = {
    /** Worksheet kind (`long-divide`); MiniSearch's document id. */
    id: string;
    /** Page URL in this locale (`/no/worksheets/long-divide/`). */
    u: string;
    /** Title in this locale. */
    t: string;
    /** Topic name in this locale (the homepage section heading). */
    p: string;
    /** Summary, math markup flattened to plain text. */
    s: string;
    /** Learning goals, plain text, one per line. */
    l: string;
    /** Alias phrases from lib/worksheet-links.ts, one per line. */
    a: string;
    /** The title in each other locale, one per line, so a search in one
     * language finds the sheet from the other ("brøk" on the English site). */
    x: string;
};

/** The fields MiniSearch searches: everything but the id and the URL.
 * `SEARCH_BOOST` (lib/search-config.ts) is typed against this. */
export type SearchField = Exclude<keyof SearchDoc, "id" | "u">;

export async function buildSearchIndex(locale: Locale): Promise<SearchDoc[]> {
    const ui = t(locale);
    const own = await getWorksheetInfo(locale);
    const others = await Promise.all(
        LOCALES.filter((l) => l.id !== locale).map((l) =>
            getWorksheetInfo(l.id),
        ),
    );
    const aliases = worksheetAliases(locale);
    return WORKSHEET_SECTIONS.flatMap((section) =>
        section.kinds.map((kind) => ({
            id: kind,
            u: localizePath(`/worksheets/${kind}/`, locale),
            t: own[kind].title,
            p: ui.home.sections[section.id],
            s: toPlainText(own[kind].summary),
            l: own[kind].learning.map(toPlainText).join("\n"),
            a: (aliases.get(kind) ?? []).join("\n"),
            x: others.map((info) => info[kind].title).join("\n"),
        })),
    );
}
