// Norwegian Bokmål UI strings. Drafted, pending native review — see
// TRANSLATION-REVIEW.md. Typed as `UiStrings`, so a missing key fails
// typechecking (and the build, via the key check in `i18n/index.ts`).
//
// Level examples use Norwegian notation: decimal comma, `·` for
// multiplication, `:` for division, and `;` between listed values
// (a comma would read as a decimal separator).

import type { UiStrings } from "./en";

export const nb: UiStrings = {
    meta: {
        homeTitle: "Pencil Ready — matteark til utskrift",
        homeDescription:
            "Gratis matteark til utskrift for 1.–7. trinn — print → solve → sharpen → repeat. Addisjon, subtraksjon, multiplikasjon, lang divisjon, brøk og likninger — velg vanskelighetsgrad og last ned som PDF.",
        aboutTitle: "Om — Pencil Ready",
        aboutDescription:
            "Pencil Ready lager gratis matteark uten reklame for elever på barnetrinnet. Sett opp et ark, last det ned som PDF og skriv det ut.",
        notFoundTitle: "Fant ikke siden — Pencil Ready",
        notFoundDescription:
            "Siden du leter etter finnes ikke. Gå tilbake til mattearkene.",
        worksheetTitle: "{title} — Pencil Ready",
    },
    nav: {
        about: "Om",
    },
    footer: {
        madeWith: "laget med",
        inOslo: "i Oslo",
        licensed: "arkene er lisensiert",
    },
    home: {
        heading: "Matteark til utskrift",
        intro: "Lær matte på den gammeldagse måten: med blyant og papir. Du trenger verken iPad eller KI. Skriv ut oppgavene du vil øve på, og finn fram blyanten. Bommer du på for mange, skriver du ut et nytt ark og prøver igjen.",
        sections: {
            arithmetic: "Regning",
            drills: "Pugg",
            longForm: "Oppstilling",
            fractions: "Brøk",
            decimals: "Desimaltall",
            statistics: "Statistikk",
            preAlgebra: "Algebra",
        },
    },
    worksheet: {
        allWorksheets: "Alle matteark",
        breadcrumbLabel: "Brødsmulesti",
        aboutThis: "Om dette arket",
        knowBefore: "Dette bør du kunne først",
        withMastery: "Når dette sitter, kan eleven",
        resourceName: "Matteark: {title}",
    },
    notFound: {
        heading: "Fant ikke siden",
        body: "Det finnes ingenting på denne adressen. Sjekk lenken, eller gå tilbake og velg et matteark.",
        back: "← Tilbake til mattearkene",
        pencilTitle: "Knekt blyant",
        pencilDesc: "En pikselert blyant som har knekt på midten.",
    },
    about: {
        heading: "Om Pencil Ready",
        subtitle: "Gratis matteark til utskrift for 1.–7. trinn.",
        onThisPage: "På denne siden",
        sections: {
            why: "Hvorfor dette finnes",
            using: "Bruk av arkene",
            inside: "Hva som ligger bak",
            openSource: "Åpen kildekode",
            next: "Hva kommer",
            credits: "Takk til",
        },
        back: "← Tilbake til mattearkene",
    },
    selector: {
        heading: "Språk",
        buttonLabel: "Språk: norsk",
        remember: "Vi husker valget ditt på denne enheten.",
        close: "Lukk",
        suggested: "Forslag",
        all: "Alle språk",
        filter: "Søk etter språk",
        noMatches: "Fant ingen språk",
    },
    languageNames: {
        en: "Engelsk",
        no: "Norsk",
    },
    island: {
        rendering: "Lager arket …",
        errorNotFound: "Dette arket er ikke tilgjengelig akkurat nå.",
        errorServer: "Noe gikk galt da arket skulle lages.",
        errorGeneric: "Klarte ikke å lage arket.",
        errorUnreachable: "Fikk ikke kontakt med tjenesten som lager arkene.",
        errorHint: "Prøv igjen om litt, eller last inn siden på nytt.",
        previewTitle: "forhåndsvisning av arket",
        download: "Last ned",
        preparing: "Gjør klart …",
        print: "Skriv ut",
        seed: "Frø",
        seedPlaceholder: "tilfeldig",
        shuffleLabel: "Nytt frø",
        shuffleTitle: "Velg et nytt tilfeldig frø",
        solveFirst: "Løs første oppgave",
        answerKey: "Fasit",
        pdfOnly: "(bare PDF)",
        personalize: "Tilpass",
        studentName: "Elevens navn",
        studentNamePlaceholder: "f.eks. Mattehesten",
        format: "Format",
        digits: "Sifre (f.eks. 2,2 eller 3,2)",
        maxQuotient: "Største svar",
        multiplicand: "Første faktor (f.eks. 2,3 eller 1-10)",
        multiplier: "Andre faktor",
        divisor: "Divisor",
        denominators: "Nevnere",
        unitOnly: "Bare stambrøker",
        maxNumerator: "Største teller",
        properOnly: "Bare ekte brøker",
        allowWhole: "Tillat hele tall som svar",
        workingOut: "Utregning",
        printStack: "Skriv ut oppstillingen for addisjon",
        families: "Oppgavetyper",
        squares: "Kvadrattall (x²)",
        roots: "Kvadratrøtter (√x)",
        divisionLayout: "Oppstilling",
        divisionOneLine: "På én linje",
        divisionBracket: "Med bue",
        divisionStacked: "Under hverandre",
        level: "Nivå {n}",
        example: "f.eks. {example}",
    },
    levels: {
        add: [
            { label: "2 sifre, uten mente", example: "23 + 45" },
            { label: "2 sifre, med mente", example: "47 + 38" },
            { label: "3 sifre, blandet", example: "346 + 278" },
        ],
        subtract: [
            { label: "2 sifre, uten låning", example: "67 − 24" },
            { label: "2 sifre, med låning", example: "73 − 48" },
            { label: "3 sifre, blandet", example: "624 − 287" },
            { label: "3 sifre, låning over null", example: "403 − 178" },
        ],
        "decimal-add": [
            { label: "Tideler", example: "12,5 + 34,7" },
            { label: "Hundredeler", example: "12,34 + 56,78" },
        ],
        "decimal-subtract": [
            { label: "Tideler", example: "34,7 − 12,5" },
            { label: "Hundredeler", example: "56,78 − 12,34" },
        ],
        "long-divide": [
            { label: "Tosifret dividend", example: "84 : 6" },
            { label: "Tresifret dividend", example: "432 : 4" },
            { label: "Firesifret, med rest", example: "5432 : 7" },
        ],
        "algebra-two-step": [
            { label: "Mindre tall", example: "4x + 7 = 31" },
            { label: "Større tall", example: "8x + 37 = 109" },
        ],
        "algebra-one-step": [
            { label: "Pluss og minus", example: "x + 7 = 12" },
            { label: "Gange og dele", example: "5 · x = 30" },
            { label: "Alle fire", example: "x + 47 = 73" },
        ],
        "decimal-multiply": [
            { label: "Tideler · helt tall", example: "2,5 · 3" },
            { label: "Hundredeler · helt tall", example: "1,23 · 4" },
            { label: "Desimaltall · desimaltall", example: "2,5 · 0,3" },
        ],
        "order-of-ops": [
            { label: "To regneoperasjoner", example: "5 + 6 · 2" },
            { label: "Tre regneoperasjoner", example: "5 + 6 · 2 − 3" },
            { label: "Parentes først", example: "(5 + 6) · 2" },
        ],
        mean: [
            { label: "Små tall", example: "23; 39; 35; 23" },
            { label: "Tresifrede tall", example: "163; 172; 145" },
            {
                label: "Åtte tall med tideler",
                example: "66,8; 63,0; 62,1; 35,6; …",
            },
        ],
    },
};
