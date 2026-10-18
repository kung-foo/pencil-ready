// Paper size preference: A4 or US Letter. Per device, like the student
// name (`useNames`), not part of the worksheet URL — pagination uses the
// area both papers share (`common_content_area_cm` in crates/core), so
// the same URL, seed or QR is the same worksheet on either paper and
// the recipient's printer decides.
//
// Set from the language menu (a vanilla script) and read by the React
// island, which re-renders when it changes: `setPaper` broadcasts a
// `pr:paper` event on window. No React here, so the menu's script stays
// framework-free — the island's hook is in `usePaper.ts`.

export const PAPERS = [
    // `id` is the server's `paper=` value; `aspect` is width / height.
    { id: "a4", aspect: 210 / 297 },
    { id: "us-letter", aspect: 8.5 / 11 },
] as const;
export type Paper = (typeof PAPERS)[number]["id"];

/** Server default; omitted from API requests. */
export const DEFAULT_PAPER: Paper = "a4";
const KEY = "pr.paper";
export const PAPER_EVENT = "pr:paper";

export function isPaper(v: unknown): v is Paper {
    return PAPERS.some((p) => p.id === v);
}

export function getPaper(): Paper {
    if (typeof window === "undefined") return DEFAULT_PAPER;
    try {
        const v = window.localStorage.getItem(KEY);
        return isPaper(v) ? v : DEFAULT_PAPER;
    } catch {
        return DEFAULT_PAPER;
    }
}

export function setPaper(paper: Paper): void {
    try {
        window.localStorage.setItem(KEY, paper);
    } catch {
        // Privacy modes / quota: the choice still applies to this page.
    }
    window.dispatchEvent(
        new CustomEvent<Paper>(PAPER_EVENT, { detail: paper }),
    );
}

export function paperAspect(paper: Paper): number {
    return PAPERS.find((p) => p.id === paper)?.aspect ?? PAPERS[0].aspect;
}
