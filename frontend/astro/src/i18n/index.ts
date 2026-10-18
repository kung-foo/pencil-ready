// Dictionary lookup. Pages call `t(locale)` at build time; the React
// island receives `t(locale).island` / `.levels` as props instead of
// importing this module, so it never ships more than one dictionary.

import { en, type UiStrings } from "./ui/en";
import { nb } from "./ui/nb";
import { LOCALES, type Locale } from "./locales";
import { WORKSHEET_LEVELS } from "@/lib/levels";

export type { UiStrings };
export type IslandStrings = UiStrings["island"];
export type LevelStrings = UiStrings["levels"];

const DICTIONARIES: Record<Locale, UiStrings> = {
    en,
    no: nb,
};

export function t(locale: Locale): UiStrings {
    return DICTIONARIES[locale];
}

/** Fill `{name}` placeholders. Unknown placeholders are left as-is so a
 * typo shows up on the page instead of silently vanishing. */
export function fmt(
    template: string,
    vars: Record<string, string | number>,
): string {
    return template.replace(/\{(\w+)\}/g, (whole, key: string) =>
        key in vars ? String(vars[key]) : whole,
    );
}

// Build-time guard. The `UiStrings` type already catches a missing key
// in typechecking, but `astro build` doesn't typecheck — so walk every
// dictionary against English at module load and throw on any gap or
// extra key, or on a level list whose length differs. Keeps "a
// translation is missing" a failed build rather than a blank label.
function keyPaths(value: unknown, prefix = ""): string[] {
    if (Array.isArray(value)) {
        return [
            `${prefix}[len=${value.length}]`,
            ...value.flatMap((v, i) => keyPaths(v, `${prefix}[${i}]`)),
        ];
    }
    if (value && typeof value === "object") {
        return Object.entries(value).flatMap(([k, v]) =>
            keyPaths(v, prefix ? `${prefix}.${k}` : k),
        );
    }
    return [prefix];
}

{
    const reference = new Set(keyPaths(en));
    for (const { id } of LOCALES) {
        const keys = new Set(keyPaths(DICTIONARIES[id]));
        const missing = [...reference].filter((k) => !keys.has(k));
        const extra = [...keys].filter((k) => !reference.has(k));
        if (missing.length || extra.length) {
            throw new Error(
                `i18n dictionary "${id}" doesn't match en: ` +
                    [
                        missing.length && `missing ${missing.join(", ")}`,
                        extra.length && `extra ${extra.join(", ")}`,
                    ]
                        .filter(Boolean)
                        .join("; "),
            );
        }
        for (const l of LOCALES) {
            if (!(l.id in DICTIONARIES[id].languageNames)) {
                throw new Error(
                    `i18n dictionary "${id}" has no languageNames.${l.id}`,
                );
            }
        }
    }
}

// Level copy is parallel to `WORKSHEET_LEVELS` (labels here, params
// there). The key check above keeps every locale in step with English;
// this keeps English in step with the level definitions.
{
    const defined = Object.entries(WORKSHEET_LEVELS);
    const copy = en.levels as Record<string, readonly unknown[]>;
    for (const [kind, levels] of defined) {
        if (copy[kind]?.length !== levels?.length) {
            throw new Error(
                `i18n levels.${kind} has ${copy[kind]?.length ?? 0} entries, lib/levels.ts defines ${levels?.length}`,
            );
        }
    }
    const extra = Object.keys(copy).filter((k) => !(k in WORKSHEET_LEVELS));
    if (extra.length > 0) {
        throw new Error(
            `i18n levels has entries for unknown kinds: ${extra.join(", ")}`,
        );
    }
}
