// Inline math markup for worksheet help copy (`content/worksheets/**`).
//
//   `+`        operator key — exactly one operator in backticks
//   $ax + b$   math. With an operator or relation inside it's an
//              equation key (one box, never wraps); without, it's an
//              unboxed variable or single term ($x$, $x²$, $√x$).
//
// Inside `$…$`, single Latin letters are variables (italic), operators
// keep the spaces they were authored with, and digits, parentheses, `²`,
// `√` and decimal commas stay upright text.
//
// Pure and locale-agnostic: both `en/` and `nb/` content go through the
// same parser and the same build-time checks (`lib/worksheet-content.ts`).
// `:` is an operator because Norwegian writes division with it; in plain
// prose it's an ordinary colon, so the raw-operator check only flags it
// between digits.

/** Operators and relations that become keys / `.math-op`. Use U+00B7
 * `·` for multiplication dots, never U+22C5. */
export const MATH_OPERATORS = ["+", "−", "×", "÷", "·", "=", "±", ":"] as const;
const OPERATOR_CHARS = new Set<string>(MATH_OPERATORS);

/** Longest equation (plain text) the validation accepts. Equation keys
 * never wrap, so they have to fit the narrowest card (≈ 260px of text
 * at a 320px viewport). */
export const MAX_EQUATION_LENGTH = 20;

export type MathPart = { kind: "var" | "op" | "text"; text: string };

export type MathToken =
    | { type: "text"; text: string }
    | { type: "key"; op: string }
    | { type: "eq"; parts: MathPart[] }
    | { type: "term"; parts: MathPart[] };

export class MathTextError extends Error {}

function mathParts(src: string): MathPart[] {
    const parts: MathPart[] = [];
    // Operators with their authored surrounding spaces, single letters,
    // and runs of everything else.
    const re = /(\s*[+−×÷·=±:]\s*)|([A-Za-z])|([^A-Za-z+−×÷·=±:\s]+|\s+)/gy;
    for (const m of src.matchAll(re)) {
        if (m[1] !== undefined) parts.push({ kind: "op", text: m[1] });
        else if (m[2] !== undefined) parts.push({ kind: "var", text: m[2] });
        else parts.push({ kind: "text", text: m[3] });
    }
    return parts;
}

/**
 * Split a content string into text and math tokens. Throws
 * `MathTextError` on malformed markup — an unbalanced `$` or backtick, a
 * backtick token that isn't exactly one operator, or an ASCII `-` used
 * as a minus inside math.
 */
export function parseMathText(s: string): MathToken[] {
    const tokens: MathToken[] = [];
    let text = "";
    const flush = () => {
        if (text) tokens.push({ type: "text", text });
        text = "";
    };
    for (let i = 0; i < s.length; i++) {
        const c = s[i];
        if (c !== "`" && c !== "$") {
            text += c;
            continue;
        }
        const end = s.indexOf(c, i + 1);
        if (end === -1) {
            throw new MathTextError(`unbalanced ${c} in: ${s}`);
        }
        const inner = s.slice(i + 1, end);
        flush();
        if (c === "`") {
            if (!OPERATOR_CHARS.has(inner)) {
                throw new MathTextError(
                    `\`${inner}\` is not a single operator (${MATH_OPERATORS.join(" ")}) in: ${s}`,
                );
            }
            tokens.push({ type: "key", op: inner });
        } else {
            if (!inner.trim()) throw new MathTextError(`empty $…$ in: ${s}`);
            if (inner.includes("-")) {
                throw new MathTextError(
                    `ASCII "-" inside $${inner}$ — use the minus sign "−" in: ${s}`,
                );
            }
            const parts = mathParts(inner);
            const isEquation = parts.some((p) => p.kind === "op");
            tokens.push({ type: isEquation ? "eq" : "term", parts });
        }
        i = end;
    }
    flush();
    return tokens;
}

function tokenText(t: MathToken): string {
    switch (t.type) {
        case "text":
            return t.text;
        case "key":
            return t.op;
        case "eq":
        case "term":
            return t.parts.map((p) => p.text).join("");
    }
}

/** The string with markup removed and the real characters kept — for
 * meta descriptions, JSON-LD and anywhere else that can't be styled. */
export function toPlainText(s: string): string {
    return parseMathText(s).map(tokenText).join("");
}

/**
 * Content checks beyond well-formed markup; returns problems (empty when
 * fine). Operators must be marked up — an unmarked `×` would render as
 * plain body text — and equations must stay short enough not to overflow
 * a phone-width card.
 */
export function mathTextProblems(s: string): string[] {
    let tokens: MathToken[];
    try {
        tokens = parseMathText(s);
    } catch (e) {
        return [(e as Error).message];
    }
    const problems: string[] = [];
    for (const t of tokens) {
        if (t.type === "text") {
            const raw = t.text.match(/[+−×÷·=±]|\d\s*:\s*\d/);
            if (raw) {
                problems.push(
                    `unmarked operator "${raw[0]}" (wrap it in \`…\` or $…$) in: ${s}`,
                );
            }
        } else if (t.type === "eq") {
            const len = tokenText(t).length;
            if (len > MAX_EQUATION_LENGTH) {
                problems.push(
                    `equation "${tokenText(t)}" is ${len} characters (max ${MAX_EQUATION_LENGTH}) in: ${s}`,
                );
            }
        }
    }
    return problems;
}

/** A rendered piece: a math token, or text that may carry a link. */
export type Piece = MathToken | { type: "text"; text: string; href?: string };

/** A run of pieces; `nowrap` runs must not break inside. */
export type Run = { nowrap: boolean; pieces: Piece[] };

const LEADING_GLUE = /[(]$/;
const TRAILING_GLUE = /^[,.;:)]+/;

/**
 * Group pieces so punctuation directly next to a key stays on its line:
 * a preceding `(` and following `, . ; : )` join the key in a nowrap
 * run, so `+,` moves as one unit instead of orphaning the comma.
 */
export function glueKeys(pieces: Piece[]): Run[] {
    const runs: Run[] = [];
    const isMath = (p: Piece | undefined) => !!p && p.type !== "text";
    let i = 0;
    while (i < pieces.length) {
        const p = pieces[i];
        if (!isMath(p)) {
            runs.push({ nowrap: false, pieces: [p] });
            i++;
            continue;
        }
        const glued: Piece[] = [p];
        // Steal a trailing "(" from the preceding plain text.
        const prev = runs[runs.length - 1];
        const prevPiece = prev?.pieces[prev.pieces.length - 1];
        if (
            prev &&
            !prev.nowrap &&
            prevPiece?.type === "text" &&
            !("href" in prevPiece && prevPiece.href) &&
            LEADING_GLUE.test(prevPiece.text)
        ) {
            prevPiece.text = prevPiece.text.slice(0, -1);
            glued.unshift({ type: "text", text: "(" });
        }
        // And leading punctuation from the following plain text.
        const next = pieces[i + 1];
        if (next?.type === "text" && !("href" in next && next.href)) {
            const m = next.text.match(TRAILING_GLUE);
            if (m) {
                glued.push({ type: "text", text: m[0] });
                pieces[i + 1] = {
                    type: "text",
                    text: next.text.slice(m[0].length),
                };
            }
        }
        runs.push({ nowrap: true, pieces: glued });
        i++;
    }
    return runs.filter((r) =>
        r.pieces.some((p) => p.type !== "text" || p.text !== ""),
    );
}
