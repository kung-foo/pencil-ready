import {
    glueKeys,
    parseMathText,
    type MathPart,
    type Piece,
} from "@/lib/math-text";

/**
 * Worksheet help copy with inline math markup (lib/math-text.ts):
 * operator keys, equation keys and italic variables, styled by the
 * `.math-*` rules in styles/index.css. Plain text renders as text and
 * keys are ordinary text in spans, so screen readers read "plus",
 * "minus" and so on.
 *
 * Used from Astro pages without a `client:` directive, so it renders to
 * static HTML and ships no JS. A React component rather than .astro on
 * purpose: JSX drops formatting whitespace, while an Astro template with
 * `compressHTML` off turns every line break between tags into a visible
 * space (e.g. "addition ," after a link).
 *
 * Pass `text` for a plain content string, or `pieces` when text runs
 * carry links (`linkWorksheetList` parses the math first and only links
 * plain-text pieces, so a link never splits a key).
 */
export function MathText({
    text = "",
    pieces,
    linkClass,
}: {
    text?: string;
    pieces?: Piece[];
    /** Classes for links inside `pieces`. */
    linkClass?: string;
}) {
    // glueKeys trims punctuation off neighbouring text pieces; give it
    // its own copies.
    const runs = glueKeys(
        (pieces ?? parseMathText(text)).map((p) => ({ ...p }) as Piece),
    );
    return (
        <>
            {runs.map((run, i) =>
                run.nowrap ? (
                    <span key={i} className="math-glue">
                        {run.pieces.map(renderPiece(linkClass))}
                    </span>
                ) : (
                    run.pieces.map(renderPiece(linkClass, i))
                ),
            )}
        </>
    );
}

const renderPart = (q: MathPart, j: number) =>
    q.kind === "var" ? (
        <span key={j} className="math-var">
            {q.text}
        </span>
    ) : q.kind === "op" ? (
        <span key={j} className="math-op">
            {q.text}
        </span>
    ) : (
        q.text
    );

const renderPiece =
    (linkClass: string | undefined, run = 0) =>
    (p: Piece, j: number) => {
        const key = `${run}-${j}`;
        switch (p.type) {
            case "text":
                return "href" in p && p.href ? (
                    <a key={key} href={p.href} className={linkClass}>
                        {p.text}
                    </a>
                ) : (
                    p.text
                );
            case "key":
                return (
                    <span key={key} className="math-key" data-op={p.op}>
                        {p.op}
                    </span>
                );
            case "eq":
                return (
                    <span key={key} className="math-eq">
                        {p.parts.map(renderPart)}
                    </span>
                );
            case "term":
                return p.parts.map(renderPart);
        }
    };
