// English UI strings — the reference dictionary. Every other locale's
// dictionary is typed as `UiStrings` (derived from this object), so a
// missing key fails typechecking, and `i18n/index.ts` re-checks the
// key set at build time so `pnpm build` fails too.
//
// Values are plain strings (no functions) so the `island` and `levels`
// subtrees can be passed to the React island as serializable props.
// Placeholders like `{title}` are filled by `fmt()` in `i18n/index.ts`.
//
// Not in here on purpose: the site name "Pencil Ready" and the motto
// "print → solve → sharpen → repeat" stay English in every locale.

export const en = {
    meta: {
        homeTitle: "Pencil Ready — printable math worksheets",
        /** Meta description: 25–160 characters (checked in Layout). */
        homeDescription:
            "Free printable math worksheets for 1st–6th grade: addition, subtraction, multiplication, long division, fractions and equations. Pick a level, print the PDF.",
        aboutTitle: "About — Pencil Ready",
        aboutDescription:
            "Pencil Ready is a free, ad-free math worksheet generator for primary-school students. Configure a worksheet, download as PDF, print.",
        notFoundTitle: "Not found — Pencil Ready",
        notFoundDescription:
            "The page you're looking for doesn't exist. Head back to the worksheets.",
        worksheetTitle: "{title} — Pencil Ready",
    },
    nav: {
        about: "About",
    },
    footer: {
        madeWith: "made with",
        inOslo: "in Oslo",
        licensed: "worksheets licensed",
    },
    home: {
        heading: "Printable math worksheets",
        intro: "Master math the old-fashioned way: with handwritten drills. No need for an iPad or AI. Just print the problems you want to practice and pick up a pencil. If you miss too many, print a new sheet and try again.",
        sections: {
            arithmetic: "Arithmetic",
            drills: "Fact drills",
            longForm: "Long-form",
            fractions: "Fractions",
            decimals: "Decimals",
            statistics: "Statistics",
            preAlgebra: "Pre-algebra",
        },
    },
    worksheet: {
        allWorksheets: "All worksheets",
        breadcrumbLabel: "Breadcrumb",
        aboutThis: "About this worksheet",
        knowBefore: "Know before you start",
        withMastery: "With mastery, the student can",
        /** JSON-LD `LearningResource.name`. */
        resourceName: "{title} worksheet",
    },
    notFound: {
        heading: "Page not found",
        body: "Nothing lives at this URL. Check the link, or head back to pick a worksheet.",
        back: "← Back to worksheets",
        pencilTitle: "Broken pencil",
        pencilDesc: "A pixelated number two pencil snapped in half.",
    },
    about: {
        heading: "About Pencil Ready",
        subtitle: "Free, printable math worksheets for 1st–6th grade.",
        onThisPage: "On this page",
        sections: {
            why: "Why this exists",
            using: "Using the worksheets",
            inside: "What's inside",
            openSource: "Open source",
            next: "What's next",
            credits: "Credits",
        },
        back: "← Back to worksheets",
    },
    selector: {
        heading: "Language",
        /** Button `aria-label`, naming this (the current) language. */
        buttonLabel: "Language and paper: English",
        remember: "We'll remember your choice on this device.",
        close: "Close",
        suggested: "Suggested",
        all: "All languages",
        filter: "Search languages",
        noMatches: "No matching languages",
        paper: "Paper",
        paperA4: "A4",
        paperA4Detail: "210 × 297 mm",
        paperLetter: "US Letter",
        paperLetterDetail: "8.5 × 11 in",
    },
    /** Each language's name in this UI language, keyed by locale id. */
    languageNames: {
        en: "English",
        no: "Norwegian",
    },
    island: {
        rendering: "Rendering…",
        errorNotFound: "This worksheet isn't available right now.",
        errorServer: "The worksheet generator hit an error.",
        errorGeneric: "Couldn't generate this worksheet.",
        errorUnreachable: "Couldn't reach the worksheet generator.",
        errorHint: "Check back in a moment, or refresh the page.",
        previewTitle: "worksheet preview",
        download: "Download",
        preparing: "Preparing…",
        print: "Print",
        seed: "Seed",
        seedPlaceholder: "random",
        shuffleLabel: "Shuffle seed",
        shuffleTitle: "Pick a new random seed",
        solveFirst: "Solve first problem",
        answerKey: "Answer key",
        pdfOnly: "(PDF only)",
        personalize: "Personalize",
        studentName: "Student name",
        studentNamePlaceholder: "e.g. Math Hippo",
        format: "Format",
        digits: "Digits (e.g. 2,2 or 3,2)",
        maxQuotient: "Max quotient",
        multiplicand: "Multiplicand (e.g. 2,3 or 1-10)",
        multiplier: "Multiplier",
        divisor: "Divisor",
        denominators: "Denominators",
        unitOnly: "Unit fractions only",
        maxNumerator: "Max numerator",
        properOnly: "Proper fractions only",
        allowWhole: "Allow whole-number answers",
        workingOut: "Working out",
        printStack: "Print the addition stack",
        families: "Families",
        squares: "Squares (x²)",
        roots: "Square roots (√x)",
        divisionLayout: "Layout",
        divisionOneLine: "One line",
        divisionBracket: "Bracket",
        divisionStacked: "Stacked",
        level: "Level {n}",
        example: "e.g. {example}",
    },
    /**
     * Level picker copy, one entry per level in `lib/levels.ts` (same
     * order, same count — checked at build time). Examples use this
     * locale's notation (decimal separator, × / ÷ glyphs).
     */
    levels: {
        add: [
            { label: "2-digit, no carry", example: "23 + 45" },
            { label: "2-digit with carry", example: "47 + 38" },
            { label: "3-digit, mixed", example: "346 + 278" },
        ],
        subtract: [
            { label: "2-digit, no borrow", example: "67 − 24" },
            { label: "2-digit with borrow", example: "73 − 48" },
            { label: "3-digit, mixed", example: "624 − 287" },
            { label: "3-digit, ripple borrow", example: "403 − 178" },
        ],
        "decimal-add": [
            { label: "Tenths", example: "12.5 + 34.7" },
            { label: "Hundredths", example: "12.34 + 56.78" },
        ],
        "decimal-subtract": [
            { label: "Tenths", example: "34.7 − 12.5" },
            { label: "Hundredths", example: "56.78 − 12.34" },
        ],
        "long-divide": [
            { label: "2-digit dividends", example: "84 ÷ 6" },
            { label: "3-digit dividends", example: "432 ÷ 4" },
            { label: "4-digit, with remainders", example: "5432 ÷ 7" },
        ],
        "algebra-two-step": [
            { label: "Smaller values", example: "4x + 7 = 31" },
            { label: "Larger values", example: "8x + 37 = 109" },
        ],
        "algebra-one-step": [
            { label: "Plus & minus", example: "x + 7 = 12" },
            { label: "Times & divide", example: "5 · x = 30" },
            { label: "All four", example: "x + 47 = 73" },
        ],
        "decimal-multiply": [
            { label: "Tenths × whole", example: "2.5 × 3" },
            { label: "Hundredths × whole", example: "1.23 × 4" },
            { label: "Decimal × decimal", example: "2.5 × 0.3" },
        ],
        "order-of-ops": [
            { label: "Two operations", example: "5 + 6 × 2" },
            { label: "Three operations", example: "5 + 6 × 2 − 3" },
            { label: "Parentheses first", example: "(5 + 6) × 2" },
        ],
        mean: [
            { label: "Small numbers", example: "23, 39, 35, 23" },
            { label: "Three-digit numbers", example: "163, 172, 145" },
            {
                label: "Eight numbers, tenths",
                example: "66.8, 63.0, 62.1, 35.6, …",
            },
        ],
    },
};

export type UiStrings = typeof en;
