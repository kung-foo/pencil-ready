// The markdown copy of each worksheet page, served at
// `/worksheets/<kind>.md` for LLMs and agents, following the llms.txt
// convention of a clean `.md` beside each HTML page. The HTML page is
// mostly a React configurator, so this is the readable version: the help
// copy plus how to fetch the PDF straight from the API.
//
// English only. Every locale's page links here, and a section at the end
// lists the languages and the `locale` value that prints each one.
//
// Parameter tables come from `src/data/openapi.json`, a checked-in copy
// of the server's spec. The `openapi_snapshot` test in
// crates/server/src/main.rs fails when it drifts from the handlers;
// refresh it with
// `UPDATE_OPENAPI=1 cargo test -p pencil-ready-server openapi_snapshot`.

import spec from "@/data/openapi.json";
import {
    LOCALES,
    SITE_ORIGIN,
    absoluteUrl,
    localizePath,
} from "@/i18n/locales";
import { DIVISION_STYLE_KINDS, type WorksheetKind } from "./api";
import { tokenText, toPlainText, type Piece } from "./math-text";
import { getWorksheetInfo } from "./worksheet-content";
import { linkWorksheetList } from "./worksheet-links";

type Schema = {
    type?: string;
    enum?: string[];
    minimum?: number;
    maximum?: number;
    description?: string;
    $ref?: string;
};
type Param = {
    name: string;
    description?: string;
    schema: Schema;
    example?: unknown;
};

const SCHEMAS = spec.components.schemas as Record<string, Schema>;
const PATHS = spec.paths as Record<string, { get: { parameters: Param[] } }>;

/** Shared params worth documenting for every sheet, in table order.
 * Left out: `debug`, `symbol`, `qr` and `share_url`, which are for the
 * site's own previews and printing, not for someone fetching a sheet,
 * and `cols`: each sheet's default grid is laid out to fill the page. */
const COMMON = [
    "format",
    "seed",
    "problems",
    "paper",
    "locale",
    "solve_first",
    "include_answers",
    "student_name",
] as const;

/** Kinds whose problem-type switches all default to off; the server
 * rejects a request that turns none on. `example` goes in the example
 * URLs so they work. */
const REQUIRES_ONE_OF: Partial<
    Record<WorksheetKind, { switches: string[]; example: string }>
> = {
    "algebra-one-step": {
        switches: ["add", "subtract", "multiply", "divide"],
        example: "add=true",
    },
    "algebra-square-root": {
        switches: ["squares", "roots"],
        example: "squares=true",
    },
};

/** Every shared param: the ones on every endpoint. */
const SHARED = new Set(
    Object.values(PATHS)
        .map((p) => p.get.parameters.map((q) => q.name))
        .reduce((a, b) => a.filter((n) => b.includes(n))),
);

function params(kind: WorksheetKind): Param[] {
    const path = PATHS[`/api/worksheets/${kind}`];
    if (!path) throw new Error(`openapi.json has no endpoint for ${kind}`);
    return path.get.parameters;
}

function resolve(s: Schema): Schema {
    return s.$ref ? SCHEMAS[s.$ref.split("/").pop()!] : s;
}

function values(p: Param): string {
    const s = resolve(p.schema);
    if (s.enum) return s.enum.map((v) => `\`${v}\``).join(", ");
    if (s.type === "boolean") return "`true`, `false`";
    if (s.type === "integer") {
        const range =
            s.minimum !== undefined && s.maximum !== undefined
                ? ` ${s.minimum}–${s.maximum}`
                : "";
        return `integer${range}`;
    }
    return p.example !== undefined && p.example !== null
        ? `string, e.g. \`${p.example}\``
        : "string";
}

/** An enum's own description says what each value means; append it. */
function description(p: Param): string {
    const s = resolve(p.schema);
    const extra = s !== p.schema ? s.description : undefined;
    return [p.description, extra].filter(Boolean).join(" ");
}

function cell(s: string): string {
    return s.replace(/\s*\n\s*/g, " ").replace(/\|/g, "\\|");
}

function table(ps: Param[]): string {
    return [
        "| Parameter | Values | Description |",
        "|---|---|---|",
        ...ps.map(
            (p) =>
                `| \`${p.name}\` | ${cell(values(p))} | ${cell(description(p))} |`,
        ),
    ].join("\n");
}

/** Prose pieces to markdown; worksheet mentions link to their `.md`. */
function piecesToMarkdown(pieces: Piece[]): string {
    return pieces
        .map((p) =>
            p.type === "text" && "href" in p && p.href
                ? `[${p.text}](${absoluteUrl(p.href.replace(/\/$/, ".md"))})`
                : tokenText(p),
        )
        .join("");
}

/** The markdown copy's path; the same for every locale's page. */
export function markdownPath(kind: WorksheetKind): string {
    return `/worksheets/${kind}.md`;
}

export async function buildWorksheetMarkdown(
    kind: WorksheetKind,
): Promise<string> {
    const info = (await getWorksheetInfo("en"))[kind];
    const page = absoluteUrl(`/worksheets/${kind}/`);
    const endpoint = `${SITE_ORIGIN}/api/worksheets/${kind}`;

    const all = params(kind);
    const own = all.filter((p) => !SHARED.has(p.name));
    const commonNames: string[] = [...COMMON];
    if (DIVISION_STYLE_KINDS.includes(kind)) commonNames.push("division");
    const common = commonNames.map((n) => all.find((p) => p.name === n)!);

    const required = REQUIRES_ONE_OF[kind];
    const q = (rest: string) =>
        [required?.example, rest].filter(Boolean).join("&");
    const url = (rest = "") => (q(rest) ? `${endpoint}?${q(rest)}` : endpoint);

    const sections = [
        `# ${info.title}`,
        `> ${toPlainText(info.summary)}`,
        `A free printable math worksheet from Pencil Ready. Set it up and print it at ${page}.`,
        "## Know before you start",
        linkWorksheetList(info.prerequisites, kind, "en")
            .map((pieces) => `- ${piecesToMarkdown(pieces)}`)
            .join("\n"),
        "## With mastery, the student can",
        info.learning.map((l) => `- ${toPlainText(l)}`).join("\n"),
        "## Generate the PDF",
        `\`GET ${endpoint}\` returns the worksheet as a PDF. Every option is a query parameter; leave one out to get its default. The same \`seed\` with the same options always gives the same worksheet.`,
        ...(required
            ? [
                  `At least one of ${required.switches.map((s) => `\`${s}\``).join(", ")} must be \`true\`; they all default to \`false\`, and a request with none on is rejected with HTTP 400.`,
              ]
            : []),
        "### Examples",
        [
            `- Default settings, new problems on every request: ${url()}`,
            `- Seed 42, with an answer key: ${url("seed=42&include_answers=true")}`,
        ].join("\n"),
        ...(own.length ? ["### Options for this worksheet", table(own)] : []),
        "### Options for every worksheet",
        table(common),
        `Full API reference (OpenAPI): ${SITE_ORIGIN}/openapi.json`,
        "## Languages",
        "Worksheets print in each language below. Choose one with the `locale` query parameter, which sets the printed instructions and number notation. Each language also has its own version of the worksheet page.",
        LOCALES.map(
            (l) =>
                `- ${l.endonym} (\`${l.bcp47}\`): \`locale=${l.worksheetLocale}\`, page ${absoluteUrl(localizePath(`/worksheets/${kind}/`, l.id))}`,
        ).join("\n"),
    ];
    return sections.join("\n\n") + "\n";
}
