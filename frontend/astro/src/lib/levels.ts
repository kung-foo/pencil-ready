// Concept-labeled levels for worksheet kinds.
//
// A level is a curated preset of raw worksheet params. Its short
// concept-flavored label and example problem are copy, so they live in
// the dictionaries (`levels` in src/i18n/ui/*.ts, same order and count —
// checked at build time). The configurator shows the levels as a
// vertical picker; selecting one expands into raw query params on the
// API URL while the browser URL stores just `?level=N` (1-based index)
// for shareability.
//
// Using a numeric index across all kinds keeps URLs consistent: `level=1`
// always means "the first concept" of whichever worksheet you're on.
// Levels are sequential — the kid is expected to work through all of
// them — so labels avoid "easy/medium/hard" or "advanced" framing.
//
// To add levels to a new worksheet kind: add an entry to `levels.json`,
// its labels under `levels` in every dictionary, and use the
// `<LevelPicker>` UI in WorksheetConfig. Any `problems`/`cols` must fit
// one page on every paper — the server test
// `levels_fit_one_page_on_every_paper` checks each preset.

import type { DivisionStyle, WorksheetKind } from "./api";
import levelsJson from "./levels.json";

export type Level = {
    /** Why the preset is what it is (JSON has no comments). */
    note?: string;
    /** Raw API query params this level expands to. */
    params: Record<string, string | number | boolean>;
    /** Overrides merged over `params` for a division layout — the
     * equation form's cells are wider, so it fits fewer columns. */
    styleParams?: Partial<
        Record<DivisionStyle, Record<string, string | number | boolean>>
    >;
    /** Whether this level prints a work scaffold the user can toggle.
     * Omitted means yes. Set false for levels whose layout has no
     * scaffold to print, so the configurator doesn't offer a control
     * that does nothing — mean's open-workspace levels, for instance
     * (see `mean::uses_scaffold_layout` in crates/core/src/mean.rs). */
    supportsScaffold?: boolean;
};

/** Per-kind level definitions, from `levels.json` (shared with the
 * server's paper-fit test). The 1-based array position is the level id
 * (URL: `?level=1`); the first entry is the default. */
const { _comment: _, ...levelData } = levelsJson;
export const WORKSHEET_LEVELS = levelData as Partial<
    Record<WorksheetKind, readonly Level[]>
>;

export function levelsFor(kind: WorksheetKind): readonly Level[] | undefined {
    return WORKSHEET_LEVELS[kind];
}

/** Default level for a kind that uses the level system. Always "1"
 * (first entry), as a string to match the URL serialization. */
export function defaultLevel(kind: WorksheetKind): string | undefined {
    return WORKSHEET_LEVELS[kind] ? "1" : undefined;
}

/** Look up a level's expanded raw params by 1-based id (e.g. "2"),
 * with any overrides for the division layout it renders in. */
export function levelParams(
    kind: WorksheetKind,
    value: string | undefined,
    style?: DivisionStyle,
): Record<string, string | number | boolean> | undefined {
    if (!value) return undefined;
    const idx = Number(value);
    if (!Number.isInteger(idx) || idx < 1) return undefined;
    const level = WORKSHEET_LEVELS[kind]?.[idx - 1];
    if (!level) return undefined;
    return { ...level.params, ...(style && level.styleParams?.[style]) };
}
