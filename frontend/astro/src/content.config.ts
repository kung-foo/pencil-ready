import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

// Per-worksheet help text, one markdown file per kind per language under
// `src/content/worksheets/<bcp47>/` (`en/add.md`, `nb/add.md`). The
// folder is the locale's language tag from `LOCALES`; the filename must
// match the kind slug in `WORKSHEET_KINDS`. `lib/worksheet-content.ts`
// checks that every kind has a file in every locale at build time, so a
// new worksheet — or a missing translation — fails the build rather than
// shipping a page with no explanation on it.
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
// Norwegian (nb/) follows the same shape in natural school Bokmål:
//
//   title          Matches `title_nb()` in crates/core/src/meta.rs.
//   summary        One sentence, imperative ("Legg sammen …", "Finn …").
//   prerequisites  Three noun phrases ("Gangetabellen opp til 10 · 10.").
//                  Name other worksheets by their Norwegian titles so
//                  the linker picks them up.
//   learning       Three infinitive phrases without "å", each completing
//                  "Når dette sitter, kan eleven ...".
//   Notation       Decimal comma, `·` and `:` for times and divide —
//                  the same glyphs the Norwegian PDFs print. Primary-
//                  school terms: addisjon, subtraksjon, multiplikasjon,
//                  divisjon, brøk, desimaltall, likninger, mente, låne.
//
// Three of each keeps the cards the same height across the site and
// stops one worksheet reading as better documented than its neighbour.
const worksheets = defineCollection({
    loader: glob({ pattern: "**/*.md", base: "./src/content/worksheets" }),
    schema: z.object({
        /** Title in the file's language. Sentence case. Matches the server's `title()`. */
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
