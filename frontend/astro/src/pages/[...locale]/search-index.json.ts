// `/search-index.json` and `/no/search-index.json`: the header search's
// index for each locale (see lib/search-index.ts).
import type { APIRoute } from "astro";
import { localeFromParam, localeStaticParams } from "@/i18n/locales";
import { buildSearchIndex } from "@/lib/search-index";

export function getStaticPaths() {
    return localeStaticParams().map((params) => ({ params }));
}

export const GET: APIRoute = async ({ params }) => {
    const docs = await buildSearchIndex(localeFromParam(params.locale));
    return new Response(JSON.stringify(docs), {
        headers: { "Content-Type": "application/json" },
    });
};
