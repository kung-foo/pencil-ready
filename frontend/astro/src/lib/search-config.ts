// MiniSearch field settings for the header search panel
// (components/WorksheetSearch.astro). They live in a .ts module rather
// than the component's <script> because `tsc` checks .ts files but not
// .astro scripts: with the tables typed against SearchDoc, adding,
// renaming or removing an index field fails typechecking until these
// match. Type-only imports, so none of the build-time index code ships.

import type { SearchDoc, SearchField } from "./search-index";

/** Weight of a match in each searchable field. As tested on the real
 * index: title > alias > other-language title > topic > summary >
 * learning goal. Also decides which line a result shows. */
export const SEARCH_BOOST: Record<SearchField, number> = {
    t: 10,
    a: 8,
    x: 6,
    p: 5,
    s: 3,
    l: 2,
};

/** Fields kept with each result for its row: the URL plus every
 * searchable field (the matched line can come from any of them). */
export const SEARCH_STORED: (keyof SearchDoc)[] = [
    "u",
    ...(Object.keys(SEARCH_BOOST) as SearchField[]),
];
