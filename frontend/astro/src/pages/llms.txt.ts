// `/llms.txt` (llmstxt.org): a short map of the site for LLMs and
// agents. Each worksheet links to its markdown copy
// (pages/worksheets/[kind].md.ts), which carries the options.
import type { APIRoute } from "astro";
import { t } from "@/i18n";
import { LOCALES, SITE_ORIGIN, absoluteUrl } from "@/i18n/locales";
import { toPlainText } from "@/lib/math-text";
import { getWorksheetInfo } from "@/lib/worksheet-content";
import { WORKSHEET_SECTIONS } from "@/lib/worksheet-info";
import { markdownPath } from "@/lib/worksheet-markdown";

export const GET: APIRoute = async () => {
    const ui = t("en");
    const info = await getWorksheetInfo("en");
    const languages = LOCALES.map(
        (l) => `${l.endonym} (\`locale=${l.worksheetLocale}\`)`,
    ).join(", ");

    const lines = [
        "# Pencil Ready",
        "",
        `> ${ui.about.subtitle} Every worksheet is a PDF generated on request. No login, no ads.`,
        "",
        "- Each request prints new problems; pass `seed` to get the same sheet again.",
        "- `include_answers=true` appends an answer key.",
        "- Paper: A4 or US Letter (`paper`).",
        `- Languages: ${languages}.`,
        "",
        "## Get a PDF",
        "",
        `\`GET ${SITE_ORIGIN}/api/worksheets/<kind>\` returns the PDF. Each worksheet below links to a page listing its options.`,
        "",
        ...WORKSHEET_SECTIONS.flatMap((section) => [
            `## ${ui.home.sections[section.id]}`,
            "",
            ...section.kinds.map(
                (kind) =>
                    `- [${info[kind].title}](${absoluteUrl(markdownPath(kind))}): ${toPlainText(info[kind].summary)}`,
            ),
            "",
        ]),
        "## Optional",
        "",
        `- [OpenAPI spec](${SITE_ORIGIN}/openapi.json): every endpoint and parameter`,
        `- [About](${absoluteUrl("/about/")}): why the site exists and who makes it`,
    ];
    return new Response(lines.join("\n") + "\n", {
        headers: { "Content-Type": "text/plain; charset=utf-8" },
    });
};
