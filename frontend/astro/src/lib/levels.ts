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
// To add levels to a new worksheet kind: add an entry under
// `WORKSHEET_LEVELS`, its labels under `levels` in every dictionary, and
// use the `<LevelPicker>` UI in WorksheetConfig.

import type { DivisionStyle, WorksheetKind } from "./api";

export type Level = {
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

/** Per-kind level definitions. The 1-based array position is the level
 * id (URL: `?level=1`); the first entry is the default. */
export const WORKSHEET_LEVELS: Partial<
    Record<WorksheetKind, readonly Level[]>
> = {
    add: [
        {
            params: { digits: "2,2", carry: "none" },
        },
        {
            // `force` so every problem actually exercises carrying — `any`
            // would let plain no-carry sums slip in and dilute the practice.
            params: { digits: "2,2", carry: "force" },
        },
        {
            params: { digits: "3,3", carry: "any" },
        },
    ],
    subtract: [
        {
            params: { digits: "2,2", borrow: "none" },
        },
        {
            // `force` so the kid actually practices borrowing every problem.
            params: { digits: "2,2", borrow: "force" },
        },
        {
            // `any` lets borrow-across-zero cases appear naturally —
            // a soft intro before level 4 forces ripple every time.
            params: { digits: "3,3", borrow: "any" },
        },
        {
            // `ripple` guarantees every problem chains the borrow through
            // 2+ columns (e.g. across a 0). This is the case that
            // typically trips kids up the most.
            params: { digits: "3,3", borrow: "ripple" },
        },
    ],
    "decimal-add": [
        {
            params: { digits: "2,2", decimal_places: 1 },
        },
        {
            params: { digits: "2,2", decimal_places: 2 },
        },
    ],
    "decimal-subtract": [
        {
            params: { digits: "2,2", decimal_places: 1 },
        },
        {
            params: { digits: "2,2", decimal_places: 2 },
        },
    ],
    "long-divide": [
        {
            // Cell ~3.8 × 6cm: 4 cols × 3 rows = 12 cells.
            params: { digits: "2", remainder: false, problems: 12, cols: 4 },
            // Equation cell 5.0 × 3.8cm: 3 cols × 4 rows.
            styleParams: { equation: { cols: 3 } },
        },
        {
            // Cell ~3.8 × 8cm: 4 cols × 3 rows = 12 cells.
            params: { digits: "3", remainder: false, problems: 12, cols: 4 },
            // Equation cell 5.9 × 5.4cm: 3 cols × 4 rows.
            styleParams: { equation: { cols: 3 } },
        },
        {
            // Remainder mode widens the cell to ~5.6 × 10cm to fit the
            // `Q r N` answer; 3 cols × 2 rows = 6 cells per page.
            params: { digits: "4", remainder: true, problems: 6, cols: 3 },
            // Equation cell 8.9 × 6.9cm: 2 cols × 3 rows.
            styleParams: { equation: { cols: 2 } },
        },
    ],
    "algebra-two-step": [
        {
            params: {
                a_range: "2-9",
                b_range: "1-20",
                x_range: "0-15",
                mix_forms: true,
            },
        },
        {
            params: {
                a_range: "2-12",
                b_range: "1-50",
                x_range: "0-30",
                mix_forms: true,
            },
        },
    ],
    "algebra-one-step": [
        {
            params: {
                add: true,
                subtract: true,
                multiply: false,
                divide: false,
                a_range: "2-10",
                b_range: "1-15",
                x_range: "1-15",
            },
        },
        {
            params: {
                add: false,
                subtract: false,
                multiply: true,
                divide: true,
                a_range: "2-9",
                b_range: "1-30",
                x_range: "0-10",
            },
        },
        {
            params: {
                add: true,
                subtract: true,
                multiply: true,
                divide: true,
                a_range: "2-12",
                b_range: "1-50",
                x_range: "0-30",
            },
        },
    ],
    "decimal-multiply": [
        {
            params: {
                digits: "1-2",
                decimal_places: 1,
                multiplier_min: 2,
                multiplier_max: 9,
                bottom_decimal_places: 0,
            },
        },
        {
            params: {
                digits: "1-2",
                decimal_places: 2,
                multiplier_min: 2,
                multiplier_max: 9,
                bottom_decimal_places: 0,
            },
        },
        {
            params: {
                digits: "1",
                decimal_places: 1,
                multiplier_min: 1,
                multiplier_max: 9,
                bottom_decimal_places: 1,
            },
        },
    ],
    "order-of-ops": [
        {
            params: { operations: 2, parens: false },
        },
        {
            params: { operations: 3, parens: false },
        },
        {
            params: { operations: 2, parens: true },
        },
    ],
    mean: [
        // Levels 1-2 mix 3- and 4-value sets on the same page: the
        // divisor changes per problem, so the kid has to count the
        // values instead of reusing one divisor down the column. Both
        // fit 2 x 2 = 4 problems and reserve room for the two-step
        // scaffold, but printing it is opt-in via the "Working out"
        // toggle — the cell footprint is the same either way, so the
        // toggle changes how much help she gets, not the layout.
        {
            params: { count: "3-4", digits: "1-2", problems: 4, cols: 2 },
        },
        {
            params: { count: "3-4", digits: "3", problems: 4, cols: 2 },
        },
        // Level 3 is a longer data set with tenths — the point shifts
        // from "run the two-step procedure" to "handle a real data set".
        // Eight values won't fit a printed addition stack, and long
        // division has no decimal support, so this level uses the
        // open-workspace layout: the data line wraps across a full-width
        // cell and the student lays out the work herself. One column,
        // two problems.
        {
            // Open-workspace layout: no printed stack or bracket, so the
            // "Working out" toggle has nothing to act on.
            supportsScaffold: false,
            params: {
                count: 8,
                digits: 2,
                decimals: true,
                problems: 2,
                cols: 1,
            },
        },
    ],
};

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
