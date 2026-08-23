import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

// Per-worksheet help text, one markdown file per kind under
// `src/content/worksheets/`. The filename must match the kind slug in
// `WORKSHEET_KINDS` — `lib/worksheet-content.ts` checks that every kind
// has a file at build time, so a new worksheet fails the build rather
// than shipping a page with no explanation on it.
//
// Everything lives in frontmatter because it is structured: the page
// renders the summary, the prerequisites, and the learning goals into
// three separate styled sections, and `lib/worksheet-links.ts` matches
// worksheet names inside the prerequisite strings. The markdown body is
// unused for now and available if a kind ever wants longer prose.
//
// House style, enforced by review rather than by the schema:
//
//   summary        One sentence, imperative, verb first. What the
//                  student does, plus the constraint that defines the
//                  sheet. Aim for 10-20 words.
//   prerequisites  Exactly three noun phrases naming a skill, so they
//                  read as a list under "Know before you start".
//                  Name other worksheets by their real names — the
//                  linker turns them into links.
//   learning       Exactly three bare verb phrases, each completing
//                  "With mastery, the student can ...".
//
// Three of each keeps the cards the same height across the site and
// stops one worksheet reading as better documented than its neighbour.
const worksheets = defineCollection({
    loader: glob({ pattern: "*.md", base: "./src/content/worksheets" }),
    schema: z.object({
        /** Plain-English title. Sentence case. Matches the server's `title()`. */
        title: z.string().min(1),
        /** One-sentence description of what the worksheet drills. */
        summary: z.string().min(1),
        /** What the student should already be comfortable with. */
        prerequisites: z.array(z.string().min(1)).length(3),
        /** What mastery of this worksheet gives them. */
        learning: z.array(z.string().min(1)).length(3),
    }),
});

export const collections = { worksheets };
