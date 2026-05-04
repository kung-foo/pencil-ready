// Concept-labeled levels for worksheet kinds.
//
// A level is a curated preset of raw worksheet params, tagged with a
// short concept-flavored label and an example problem. The configurator
// shows the levels as a vertical picker; selecting one expands into raw
// query params on the API URL while the browser URL stores just
// `?level=N` (1-based index) for shareability.
//
// Using a numeric index across all kinds keeps URLs consistent: `level=1`
// always means "the first concept" of whichever worksheet you're on.
// Levels are sequential — the kid is expected to work through all of
// them — so labels avoid "easy/medium/hard" or "advanced" framing.
//
// To add levels to a new worksheet kind: add an entry under
// `WORKSHEET_LEVELS` and use the `<LevelPicker>` UI in WorksheetConfig.

import type { WorksheetKind } from "./api";

export type Level = {
    /** Short label shown on the picker button. */
    label: string;
    /** Concrete example problem (e.g. "2.5 × 3"). Optional but helps
     * a parent recognize what the level produces. */
    example?: string;
    /** Raw API query params this level expands to. */
    params: Record<string, string | number | boolean>;
};

/** Per-kind level definitions. The 1-based array position is the level
 * id (URL: `?level=1`); the first entry is the default. */
export const WORKSHEET_LEVELS: Partial<Record<WorksheetKind, readonly Level[]>> = {
    add: [
        {
            label: "2-digit, no carry",
            example: "23 + 45",
            params: { digits: "2,2", carry: "none" },
        },
        {
            label: "2-digit with carry",
            example: "47 + 38",
            // `force` so every problem actually exercises carrying — `any`
            // would let plain no-carry sums slip in and dilute the practice.
            params: { digits: "2,2", carry: "force" },
        },
        {
            label: "3-digit, mixed",
            example: "346 + 278",
            params: { digits: "3,3", carry: "any" },
        },
    ],
    subtract: [
        {
            label: "2-digit, no borrow",
            example: "67 − 24",
            params: { digits: "2,2", borrow: "none" },
        },
        {
            label: "2-digit with borrow",
            example: "73 − 48",
            // `force` so the kid actually practices borrowing every problem.
            params: { digits: "2,2", borrow: "force" },
        },
        {
            label: "3-digit, mixed",
            example: "624 − 287",
            // `any` lets borrow-across-zero cases appear naturally —
            // a soft intro before level 4 forces ripple every time.
            params: { digits: "3,3", borrow: "any" },
        },
        {
            label: "3-digit, ripple borrow",
            example: "403 − 178",
            // `ripple` guarantees every problem chains the borrow through
            // 2+ columns (e.g. across a 0). This is the case that
            // typically trips kids up the most.
            params: { digits: "3,3", borrow: "ripple" },
        },
    ],
    "decimal-add": [
        {
            label: "Tenths",
            example: "12.5 + 34.7",
            params: { digits: "2,2", decimal_places: 1 },
        },
        {
            label: "Hundredths",
            example: "12.34 + 56.78",
            params: { digits: "2,2", decimal_places: 2 },
        },
    ],
    "decimal-subtract": [
        {
            label: "Tenths",
            example: "34.7 − 12.5",
            params: { digits: "2,2", decimal_places: 1 },
        },
        {
            label: "Hundredths",
            example: "56.78 − 12.34",
            params: { digits: "2,2", decimal_places: 2 },
        },
    ],
    "long-divide": [
        {
            label: "2-digit dividends",
            example: "84 ÷ 6",
            // Cell ~3.8 × 6cm: 4 cols × 3 rows = 12 cells.
            params: { digits: "2", remainder: false, problems: 12, cols: 4 },
        },
        {
            label: "3-digit dividends",
            example: "432 ÷ 4",
            // Cell ~3.8 × 8cm: 4 cols × 3 rows = 12 cells.
            params: { digits: "3", remainder: false, problems: 12, cols: 4 },
        },
        {
            label: "4-digit, with remainders",
            example: "5432 ÷ 7",
            // Remainder mode widens the cell to ~5.6 × 10cm to fit the
            // `Q r N` answer; 3 cols × 2 rows = 6 cells per page.
            params: { digits: "4", remainder: true, problems: 6, cols: 3 },
        },
    ],
    "algebra-two-step": [
        {
            label: "Smaller values",
            example: "4x + 7 = 31",
            params: {
                a_range: "2-9",
                b_range: "1-20",
                x_range: "0-15",
                mix_forms: true,
            },
        },
        {
            label: "Larger values",
            example: "8x + 37 = 109",
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
            label: "Plus & minus",
            example: "x + 7 = 12",
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
            label: "Times & divide",
            example: "5 · x = 30",
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
            label: "All four",
            example: "x + 47 = 73",
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
            label: "Tenths × whole",
            example: "2.5 × 3",
            params: {
                digits: "1-2",
                decimal_places: 1,
                multiplier_min: 2,
                multiplier_max: 9,
                bottom_decimal_places: 0,
            },
        },
        {
            label: "Hundredths × whole",
            example: "1.23 × 4",
            params: {
                digits: "1-2",
                decimal_places: 2,
                multiplier_min: 2,
                multiplier_max: 9,
                bottom_decimal_places: 0,
            },
        },
        {
            label: "Decimal × decimal",
            example: "2.5 × 0.3",
            params: {
                digits: "1",
                decimal_places: 1,
                multiplier_min: 1,
                multiplier_max: 9,
                bottom_decimal_places: 1,
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

/** Look up a level's expanded raw params by 1-based id (e.g. "2"). */
export function levelParams(
    kind: WorksheetKind,
    value: string | undefined,
): Record<string, string | number | boolean> | undefined {
    if (!value) return undefined;
    const idx = Number(value);
    if (!Number.isInteger(idx) || idx < 1) return undefined;
    return WORKSHEET_LEVELS[kind]?.[idx - 1]?.params;
}
