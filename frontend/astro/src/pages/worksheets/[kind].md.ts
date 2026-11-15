// `/worksheets/<kind>.md`: the markdown copy of each worksheet page,
// English only (see lib/worksheet-markdown.ts).
import type { APIRoute } from "astro";
import { WORKSHEET_KINDS, type WorksheetKind } from "@/lib/api";
import { buildWorksheetMarkdown } from "@/lib/worksheet-markdown";

export function getStaticPaths() {
    return WORKSHEET_KINDS.map((kind) => ({ params: { kind } }));
}

export const GET: APIRoute = async ({ params }) => {
    const md = await buildWorksheetMarkdown(params.kind as WorksheetKind);
    return new Response(md, {
        headers: { "Content-Type": "text/markdown; charset=utf-8" },
    });
};
