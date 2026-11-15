// The header search's index: one JSON file per locale, built at build
// time by pages/[...locale]/search-index.json.ts and loaded into
// MiniSearch the first time the search panel is used
// (components/WorksheetSearch.astro).
//
// Short keys keep the file small. Every field is a plain string, so the
// client hands the documents straight to MiniSearch:
//
//   id  worksheet kind              u  page URL in this locale
//   t   title                       p  topic name
//   s   summary                     l  learning goals, one per line
//   a   alias phrases, one per line (lib/worksheet-links.ts)
//   x   the title in the other locales, one per line
//
// Prerequisites stay out on purpose: they describe *other* worksheets,
// so matching them would surface the wrong sheet.

import { t } from "@/i18n";
import { LOCALES, localizePath, type Locale } from "@/i18n/locales";
import { toPlainText } from "./math-text";
import { getWorksheetInfo } from "./worksheet-content";
import { WORKSHEET_SECTIONS } from "./worksheet-info";
import { worksheetAliases } from "./worksheet-links";

export type SearchDoc = {
    id: string;
    u: string;
    t: string;
    p: string;
    s: string;
    l: string;
    a: string;
    x: string;
};

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
