// Single source of truth for the site's locales. Everything that needs
// to know which languages exist — routing, hreflang, the sitemap, the
// language selector, the worksheet API's `locale=` param — reads this
// list, so adding a language is one entry here plus its dictionary and
// content files. Nothing should assume there are exactly two.
//
// `id` is what appears in code and the `lang` cookie; `path` is the URL
// prefix ("" for the default locale, which lives at the root); `bcp47`
// is the language tag for `<html lang>`, hreflang and JSON-LD. Norwegian
// is `no` in URLs but `nb` (Bokmål) as a language tag.
//
// `astro.config.mjs` derives its `i18n` and sitemap blocks from this
// list. The Rust server's `LOCALE_PREFIXES` (root redirect) is the one
// copy kept by hand.

export const LOCALES = [
    {
        id: "en",
        path: "",
        bcp47: "en",
        short: "EN",
        endonym: "English",
        ogLocale: "en_US",
        // Worksheet API locale (`?locale=`): printed language, decimal
        // separator and × / ÷ glyphs. See `Locale` in crates/core.
        worksheetLocale: "us",
        // Default division layout and sign — mirrors the server's
        // `Locale::division_style` / `divide_symbol`. The configurator
        // shows these until the user picks a layout (`?division=`).
        divisionStyle: "bracket",
        divideSign: "÷",
        dir: "ltr",
    },
    {
        id: "no",
        path: "no",
        bcp47: "nb",
        short: "NO",
        endonym: "Norsk",
        ogLocale: "nb_NO",
        worksheetLocale: "no",
        divisionStyle: "equation",
        divideSign: ":",
        dir: "ltr",
    },
] as const;

export type Locale = (typeof LOCALES)[number]["id"];
export type LocaleDef = (typeof LOCALES)[number];

export const DEFAULT_LOCALE: Locale = "en";
export const SITE_ORIGIN = "https://pencilready.com";

/** Name of the cookie the language selector sets (see SPEC.md → Language). */
export const LANG_COOKIE = "lang";

export function isLocale(value: unknown): value is Locale {
    return LOCALES.some((l) => l.id === value);
}

export function localeDef(locale: Locale): LocaleDef {
    return LOCALES.find((l) => l.id === locale) ?? LOCALES[0];
}

/** Locale for a pathname, from its first segment. Unprefixed paths are
 * the default locale. */
export function localeFromPath(pathname: string): Locale {
    const first = pathname.split("/")[1] ?? "";
    const match = LOCALES.find((l) => l.path !== "" && l.path === first);
    return match ? match.id : DEFAULT_LOCALE;
}

export function localeFromUrl(url: URL): Locale {
    return localeFromPath(url.pathname);
}

/** Strip any locale prefix, returning the locale-neutral path
 * (always starts with "/"). `/no/about/` → `/about/`, `/no` → `/`. */
export function stripLocale(pathname: string): string {
    const locale = localeFromPath(pathname);
    const { path } = localeDef(locale);
    if (!path) return pathname || "/";
    const rest = pathname.slice(path.length + 1);
    return rest.startsWith("/") ? rest : `/${rest}`;
}

/**
 * The same page in another locale. Accepts a locale-neutral or an
 * already-localized path, so callers never need to know which they
 * hold. Query strings and hashes are carried over untouched.
 *
 * Pure string work rather than `getRelativeLocaleUrl` from
 * `astro:i18n`: the React island and the selector script call this in
 * the browser, where the Astro virtual module isn't available.
 */
export function localizePath(path: string, locale: Locale): string {
    const m = path.match(/^([^?#]*)(.*)$/);
    const pathname = m?.[1] || "/";
    const tail = m?.[2] ?? "";
    const neutral = stripLocale(pathname);
    const { path: prefix } = localeDef(locale);
    return `${prefix ? `/${prefix}${neutral}` : neutral}${tail}`;
}

/** Absolute URL on the production origin, for canonical / hreflang /
 * JSON-LD. */
export function absoluteUrl(path: string): string {
    return `${SITE_ORIGIN}${path}`;
}

/** One entry per locale: the given page's path in that locale. */
export function alternatePaths(
    path: string,
): { locale: Locale; bcp47: string; path: string }[] {
    return LOCALES.map((l) => ({
        locale: l.id,
        bcp47: l.bcp47,
        path: localizePath(path, l.id),
    }));
}

/** `getStaticPaths` entries for an optional `[...locale]` rest param:
 * `undefined` for the root locale, the path prefix for the rest. */
export function localeStaticParams(): { locale: string | undefined }[] {
    return LOCALES.map((l) => ({ locale: l.path || undefined }));
}

/** Locale for a `[...locale]` route param, as produced by
 * `localeStaticParams`. */
export function localeFromParam(param: string | undefined): Locale {
    if (!param) return DEFAULT_LOCALE;
    return LOCALES.find((l) => l.path === param)?.id ?? DEFAULT_LOCALE;
}
