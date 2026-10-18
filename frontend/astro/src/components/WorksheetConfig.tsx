import { useEffect, useState } from "react";
import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
} from "@/components/ui/accordion";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import {
    InputGroup,
    InputGroupAddon,
    InputGroupButton,
    InputGroupInput,
} from "@/components/ui/input-group";
import { Shuffle } from "lucide-react";
import {
    DIVISION_STYLE_KINDS,
    FORMATS,
    effectiveDivisionStyle,
    type DivisionStyle,
    type WorksheetConfig,
} from "@/lib/api";
import { localeDef, type Locale } from "@/i18n/locales";
import { levelsFor, defaultLevel, type Level } from "@/lib/levels";
import type { Names } from "@/lib/useNames";
import { fmt, type IslandStrings, type LevelStrings } from "@/i18n";
import { StippleBg } from "@/components/StippleBg";

export function WorksheetConfigPanel({
    cfg,
    onChange,
    names,
    onNamesChange,
    ui,
    levelStrings,
    locale,
}: {
    cfg: WorksheetConfig;
    onChange: (cfg: WorksheetConfig) => void;
    names: Names;
    onNamesChange: (patch: Partial<Names>) => void;
    ui: IslandStrings;
    levelStrings: LevelStrings;
    locale: Locale;
}) {
    function patch<K extends keyof WorksheetConfig>(
        key: K,
        value: WorksheetConfig[K],
    ) {
        onChange({ ...cfg, [key]: value });
    }

    return (
        <Card className="stippled w-full">
            <StippleBg />
            {/*<CardHeader>
        <CardTitle>Worksheet</CardTitle>
      </CardHeader>*/}
            <CardContent className="space-y-4">
                <KindSpecific
                    cfg={cfg}
                    onChange={onChange}
                    ui={ui}
                    levelStrings={levelStrings}
                />

                {showsDivisionStyle(cfg) && (
                    <DivisionStylePicker
                        kind={cfg.kind}
                        value={effectiveDivisionStyle(cfg, locale)}
                        sign={localeDef(locale).divideSign}
                        ui={ui}
                        onChange={(v) => patch("division", v)}
                    />
                )}

                <div className="pt-2 border-t space-y-4">
                    <Field label={ui.seed}>
                        <InputGroup>
                            <InputGroupInput
                                type="number"
                                placeholder={ui.seedPlaceholder}
                                value={cfg.seed ?? ""}
                                onChange={(e) =>
                                    patch(
                                        "seed",
                                        e.target.value === ""
                                            ? undefined
                                            : Number(e.target.value),
                                    )
                                }
                            />
                            <InputGroupAddon align="inline-end">
                                <InputGroupButton
                                    type="button"
                                    aria-label={ui.shuffleLabel}
                                    title={ui.shuffleTitle}
                                    onClick={() =>
                                        // 6-digit seed: deterministic for sharing, short enough
                                        // to fit the filename slug without looking noisy.
                                        patch(
                                            "seed",
                                            Math.floor(
                                                Math.random() * 1_000_000,
                                            ),
                                        )
                                    }
                                >
                                    <Shuffle />
                                </InputGroupButton>
                            </InputGroupAddon>
                        </InputGroup>
                    </Field>

                    <div className="flex items-center justify-between">
                        <Label htmlFor="solve_first">{ui.solveFirst}</Label>
                        <Switch
                            id="solve_first"
                            checked={cfg.solve_first ?? false}
                            onCheckedChange={(v) => patch("solve_first", v)}
                        />
                    </div>

                    <div className="flex items-center justify-between">
                        <Label htmlFor="include_answers">
                            {ui.answerKey}{" "}
                            <span className="text-xs text-muted-foreground">
                                {ui.pdfOnly}
                            </span>
                        </Label>
                        <Switch
                            id="include_answers"
                            checked={cfg.include_answers ?? false}
                            onCheckedChange={(v) => {
                                // Turning on the answer key forces PDF — PNG/SVG are
                                // single-image formats and can't carry a second page.
                                if (v && cfg.format !== "pdf") {
                                    onChange({
                                        ...cfg,
                                        include_answers: true,
                                        format: "pdf",
                                    });
                                } else {
                                    patch("include_answers", v);
                                }
                            }}
                        />
                    </div>

                    <PersonalizeSection
                        names={names}
                        onNamesChange={onNamesChange}
                        ui={ui}
                    />

                    <Field label={ui.format}>
                        <div className="flex gap-2">
                            {FORMATS.map((f) => (
                                <Button
                                    key={f}
                                    type="button"
                                    variant={
                                        cfg.format === f ? "default" : "outline"
                                    }
                                    size="sm"
                                    // PNG / SVG are single-image; grey them out when the
                                    // user has the answer-key toggle on.
                                    disabled={
                                        !!cfg.include_answers && f !== "pdf"
                                    }
                                    onClick={() => patch("format", f)}
                                    className="flex-1 uppercase"
                                >
                                    {f}
                                </Button>
                            ))}
                        </div>
                    </Field>
                </div>
            </CardContent>
        </Card>
    );
}

/** The division-layout toggle applies to kinds that print a division —
 * not mean's open-workspace level, which prints no division at all. */
function showsDivisionStyle(cfg: WorksheetConfig): boolean {
    if (!DIVISION_STYLE_KINDS.includes(cfg.kind)) return false;
    if (cfg.kind === "mean") {
        const level = Number(cfg.level ?? defaultLevel("mean"));
        return levelsFor("mean")?.[level - 1]?.supportsScaffold !== false;
    }
    return true;
}

/** Bracket vs one-line division. Each option previews the notation in
 * the locale's division sign, since the names alone ("bracket") don't
 * say much to a parent. The value shown is the locale's default until
 * the user picks one; picking writes `?division=` to the URL. */
function DivisionStylePicker({
    kind,
    value,
    sign,
    ui,
    onChange,
}: {
    kind: WorksheetConfig["kind"];
    value: DivisionStyle;
    sign: string;
    ui: IslandStrings;
    onChange: (v: DivisionStyle) => void;
}) {
    const simple = kind === "simple-divide";
    const options: {
        style: DivisionStyle;
        label: string;
        preview: React.ReactNode;
    }[] = [
        {
            style: "equation",
            label: ui.divisionOneLine,
            preview: simple ? `56 ${sign} 7 =` : `852 ${sign} 4 =`,
        },
        {
            style: "bracket",
            label: simple ? ui.divisionStacked : ui.divisionBracket,
            preview: simple ? (
                <span className="inline-flex flex-col items-end leading-tight">
                    <span>56</span>
                    <span className="border-b border-current">÷ 7</span>
                </span>
            ) : (
                // Bracket drawn as one SVG path (curve into overline)
                // over the dividend's box, so the bar always meets the
                // curve and spans the digits — a `⟌` glyph or a CSS
                // border can't guarantee either across fonts/browsers.
                // `preserveAspectRatio="none"` stretches the path to the
                // box; `non-scaling-stroke` keeps the line weight even.
                <span className="inline-flex items-end">
                    <span>4</span>
                    <span className="relative ml-[0.15em] pl-[0.5em] pr-[0.15em] pt-[0.2em]">
                        <svg
                            className="absolute inset-0 h-full w-full overflow-visible"
                            viewBox="0 0 100 100"
                            preserveAspectRatio="none"
                            aria-hidden="true"
                        >
                            <path
                                d="M 2 100 Q 16 55 2 6 L 100 6"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="1.5"
                                vectorEffect="non-scaling-stroke"
                            />
                        </svg>
                        852
                    </span>
                </span>
            ),
        },
    ];
    return (
        <Field label={ui.divisionLayout}>
            <div
                className="flex gap-2"
                role="group"
                aria-label={ui.divisionLayout}
            >
                {options.map((o) => {
                    const active = o.style === value;
                    return (
                        <button
                            key={o.style}
                            type="button"
                            aria-pressed={active}
                            onClick={() => onChange(o.style)}
                            className={
                                "flex flex-1 cursor-pointer flex-col items-center justify-center gap-1 rounded-md border px-2 py-2 text-xs transition-colors " +
                                (active
                                    ? "border-primary bg-[color-mix(in_srgb,var(--primary)_10%,var(--card))] text-primary"
                                    : "border-input bg-background hover:bg-muted")
                            }
                        >
                            <span className="font-mono text-sm text-foreground">
                                {o.preview}
                            </span>
                            <span>{o.label}</span>
                        </button>
                    );
                })}
            </div>
        </Field>
    );
}

function PersonalizeSection({
    names,
    onNamesChange,
    ui,
}: {
    names: Names;
    onNamesChange: (patch: Partial<Names>) => void;
    ui: IslandStrings;
}) {
    const summary = names.student ?? "";

    return (
        <Accordion type="single" collapsible>
            <AccordionItem value="personalize" className="border-b-0">
                <AccordionTrigger className="py-2">
                    <span className="grid flex-1 grid-cols-[auto_minmax(0,1fr)] items-baseline gap-2">
                        <span>{ui.personalize}</span>
                        {summary && (
                            <span className="truncate text-xs font-normal text-muted-foreground">
                                {summary}
                            </span>
                        )}
                    </span>
                </AccordionTrigger>
                <AccordionContent className="space-y-3">
                    <SubField label={ui.studentName}>
                        <DeferredNameInput
                            value={names.student ?? ""}
                            placeholder={ui.studentNamePlaceholder}
                            onCommit={(v) => onNamesChange({ student: v })}
                        />
                    </SubField>
                </AccordionContent>
            </AccordionItem>
        </Accordion>
    );
}

function Field({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div className="space-y-1.5">
            <Label>{label}</Label>
            {children}
        </div>
    );
}

/** Labeled section with indented children — for grouping a small list of
 * related controls (e.g. four op toggles) under a single heading. */
function FieldGroup({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div className="space-y-2">
            <Label>{label}</Label>
            <div className="ml-4 space-y-2">{children}</div>
        </div>
    );
}

/** Single-thumb slider with the same defer-on-commit behavior as
 * `RangeSliderField`. Used for scalar params like simple-divide's
 * `max_quotient`. */
function SliderField({
    label,
    min,
    max,
    value,
    onCommit,
}: {
    label: string;
    min: number;
    max: number;
    value: number;
    onCommit: (next: number) => void;
}) {
    const [draft, setDraft] = useState(value);
    useEffect(() => setDraft(value), [value]);

    return (
        <div className="space-y-2">
            <div className="flex items-baseline justify-between">
                <Label>{label}</Label>
                <span className="text-sm tabular-nums text-muted-foreground">
                    {draft}
                </span>
            </div>
            <Slider
                min={min}
                max={max}
                step={1}
                value={[draft]}
                onValueChange={(v) => setDraft(v[0])}
                onValueCommit={(v) => onCommit(v[0])}
            />
        </div>
    );
}

/** Two-thumb range slider that keeps a local draft while dragging and only
 * fires `onCommit` on release (Radix's `onValueCommit`) — same intent as
 * `DeferredNameInput`: avoid hammering the worksheet API on every tick. */
function RangeSliderField({
    label,
    min,
    max,
    value,
    onCommit,
}: {
    label: string;
    min: number;
    max: number;
    value: [number, number];
    onCommit: (next: [number, number]) => void;
}) {
    const [draft, setDraft] = useState<[number, number]>(value);
    // Re-sync when the source of truth changes externally (e.g. URL parse on
    // hydration, or sibling control reset).
    useEffect(() => setDraft(value), [value[0], value[1]]);

    return (
        <div className="space-y-2">
            <div className="flex items-baseline justify-between">
                <Label>{label}</Label>
                <span className="text-sm tabular-nums text-muted-foreground">
                    {draft[0]}
                    {draft[0] === draft[1] ? "" : `–${draft[1]}`}
                </span>
            </div>
            <Slider
                min={min}
                max={max}
                step={1}
                value={draft}
                onValueChange={(v) =>
                    setDraft([v[0], v[1]] as [number, number])
                }
                onValueCommit={(v) =>
                    onCommit([v[0], v[1]] as [number, number])
                }
            />
        </div>
    );
}

function csvToRange(
    csv: string | undefined,
    fallback: [number, number],
): [number, number] {
    if (!csv) return fallback;
    const nums = csv
        .split(",")
        .map((s) => Number(s.trim()))
        .filter((n) => Number.isFinite(n) && n > 0);
    if (nums.length === 0) return fallback;
    return [Math.min(...nums), Math.max(...nums)];
}

/** Expand a [lo, hi] slider range to a CSV of denominators, dropping
 * primes ≥ 7 from the interior. Those primes almost never reduce against
 * numerators ≤ ~20 — keeping them just dilutes the worksheet with
 * already-reduced fractions. The slider's chosen endpoints are always
 * preserved so the label and the emitted set agree. */
function expandDenominators(lo: number, hi: number): string {
    const out: number[] = [];
    for (let n = lo; n <= hi; n++) {
        if (n !== lo && n !== hi && n >= 7 && isPrime(n)) continue;
        out.push(n);
    }
    return out.join(",");
}

function isPrime(n: number): boolean {
    if (n < 2) return false;
    if (n % 2 === 0) return n === 2;
    for (let d = 3; d * d <= n; d += 2) {
        if (n % d === 0) return false;
    }
    return true;
}

/** Parse a `parse_digit_range`-style string ("N" or "N-M") into [lo, hi]. */
function dashToRange(
    s: string | undefined,
    fallback: [number, number],
): [number, number] {
    if (!s) return fallback;
    const m = s.trim().match(/^(\d+)(?:-(\d+))?$/);
    if (!m) return fallback;
    const lo = Number(m[1]);
    const hi = m[2] === undefined ? lo : Number(m[2]);
    return lo <= hi ? [lo, hi] : [hi, lo];
}

function rangeToDash(lo: number, hi: number): string {
    return lo === hi ? String(lo) : `${lo}-${hi}`;
}

/** Input that keeps its own draft while focused and only fires `onCommit`
 * on blur — keeps keystrokes from hammering the worksheet API. Also
 * commits on Enter so the preview refreshes without tabbing away. */
function DeferredNameInput({
    value,
    placeholder,
    onCommit,
}: {
    value: string;
    placeholder?: string;
    onCommit: (next: string) => void;
}) {
    const [draft, setDraft] = useState(value);
    // Sync when the source of truth changes externally (e.g. another input
    // clears localStorage, or initial hydration lands after first render).
    useEffect(() => setDraft(value), [value]);

    const commit = () => {
        const trimmed = draft.trim();
        if (trimmed !== value) onCommit(trimmed);
    };

    return (
        <Input
            value={draft}
            placeholder={placeholder}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => {
                if (e.key === "Enter") {
                    e.preventDefault();
                    commit();
                }
            }}
        />
    );
}

/** Smaller-label variant for nested "child" fields (e.g. inside an accordion). */
function SubField({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div className="space-y-1">
            <Label className="text-xs text-muted-foreground">{label}</Label>
            {children}
        </div>
    );
}

/** Per-kind fields. Only the distinguishing knobs; others fall back to server defaults. */
function KindSpecific({
    cfg,
    onChange,
    ui,
    levelStrings,
}: {
    cfg: WorksheetConfig;
    onChange: (cfg: WorksheetConfig) => void;
    ui: IslandStrings;
    levelStrings: LevelStrings;
}) {
    // Key/value typed loosely: TS can't distribute `keyof` across the kind
    // union well enough to infer per-variant keys. The switch narrowing below
    // keeps each call site honest.
    const patch = (key: string, value: unknown) =>
        onChange({ ...cfg, [key]: value } as WorksheetConfig);

    switch (cfg.kind) {
        case "multiply":
            return (
                <Field label={ui.digits}>
                    <Input
                        value={cfg.digits ?? ""}
                        placeholder="2,2"
                        onChange={(e) => patch("digits", e.target.value)}
                    />
                </Field>
            );

        case "simple-divide":
            return (
                <SliderField
                    label={ui.maxQuotient}
                    min={2}
                    max={12}
                    value={cfg.max_quotient ?? 10}
                    onCommit={(v) => patch("max_quotient", v)}
                />
            );

        case "mult-drill": {
            const [mLo, mHi] = dashToRange(cfg.multiplier, [1, 10]);
            return (
                <div className="space-y-4">
                    <Field label={ui.multiplicand}>
                        <Input
                            value={cfg.multiplicand ?? ""}
                            placeholder="1-10"
                            onChange={(e) =>
                                patch("multiplicand", e.target.value)
                            }
                        />
                    </Field>
                    <RangeSliderField
                        label={ui.multiplier}
                        min={1}
                        max={12}
                        value={[mLo, mHi]}
                        onCommit={([lo, hi]) =>
                            patch("multiplier", rangeToDash(lo, hi))
                        }
                    />
                </div>
            );
        }

        case "div-drill": {
            const [qLo, qHi] = dashToRange(cfg.max_quotient, [2, 9]);
            return (
                <div className="space-y-4">
                    <Field label={ui.divisor}>
                        <Input
                            value={cfg.divisor ?? ""}
                            placeholder="2-9"
                            onChange={(e) => patch("divisor", e.target.value)}
                        />
                    </Field>
                    <RangeSliderField
                        label={ui.maxQuotient}
                        min={2}
                        max={12}
                        value={[qLo, qHi]}
                        onCommit={([lo, hi]) =>
                            patch("max_quotient", rangeToDash(lo, hi))
                        }
                    />
                </div>
            );
        }

        case "fraction-mult":
            return (
                <div className="space-y-4">
                    <Field label={ui.denominators}>
                        <Input
                            value={cfg.denominators ?? ""}
                            placeholder="2,3,4,5,10"
                            onChange={(e) =>
                                patch("denominators", e.target.value)
                            }
                        />
                    </Field>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="unit_only">{ui.unitOnly}</Label>
                        <Switch
                            id="unit_only"
                            checked={cfg.unit_only ?? false}
                            onCheckedChange={(v) => patch("unit_only", v)}
                        />
                    </div>
                </div>
            );

        case "fraction-simplify": {
            const [denMin, denMax] = csvToRange(cfg.denominators, [2, 12]);
            return (
                <div className="space-y-4">
                    <RangeSliderField
                        label={ui.denominators}
                        min={2}
                        max={20}
                        value={[denMin, denMax]}
                        onCommit={([lo, hi]) =>
                            patch("denominators", expandDenominators(lo, hi))
                        }
                    />
                    <SliderField
                        label={ui.maxNumerator}
                        min={5}
                        max={30}
                        value={cfg.max_numerator ?? 20}
                        onCommit={(v) => patch("max_numerator", v)}
                    />
                    <div className="flex items-center justify-between">
                        <Label htmlFor="proper_only">{ui.properOnly}</Label>
                        <Switch
                            id="proper_only"
                            checked={cfg.proper_only ?? false}
                            onCheckedChange={(v) => patch("proper_only", v)}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="include_whole">{ui.allowWhole}</Label>
                        <Switch
                            id="include_whole"
                            checked={cfg.include_whole ?? false}
                            onCheckedChange={(v) => patch("include_whole", v)}
                        />
                    </div>
                </div>
            );
        }

        case "add":
        case "subtract":
        case "long-divide":
        case "algebra-two-step":
        case "algebra-one-step":
        case "decimal-add":
        case "decimal-subtract":
        case "decimal-multiply":
        case "order-of-ops": {
            const levels = levelsFor(cfg.kind) ?? [];
            const current = cfg.level ?? defaultLevel(cfg.kind);
            return (
                <LevelPicker
                    levels={levels}
                    copy={levelStrings[cfg.kind]}
                    ui={ui}
                    value={current}
                    onChange={(v) => patch("level", v)}
                />
            );
        }

        case "mean": {
            const levels = levelsFor(cfg.kind) ?? [];
            const current = cfg.level ?? defaultLevel(cfg.kind);
            // The scaffold toggle only applies to levels that actually
            // print one. `cfg.scaffold` is deliberately left alone when
            // hidden, so switching to a workspace level and back keeps
            // the earlier choice.
            const showScaffold =
                levels[Number(current) - 1]?.supportsScaffold !== false;
            return (
                <div className="space-y-4">
                    <LevelPicker
                        levels={levels}
                        copy={levelStrings.mean}
                        ui={ui}
                        value={current}
                        onChange={(v) => patch("level", v)}
                    />
                    {showScaffold && (
                        <FieldGroup label={ui.workingOut}>
                            <OpToggle
                                id="mean-scaffold"
                                label={ui.printStack}
                                checked={cfg.scaffold ?? false}
                                onChange={(v) => patch("scaffold", v)}
                            />
                        </FieldGroup>
                    )}
                </div>
            );
        }

        case "algebra-square-root": {
            const onSquares = cfg.squares ?? false;
            const onRoots = cfg.roots ?? false;
            // Lock the last enabled toggle so users can't switch both off
            // (server validation rejects that).
            const enabledCount = Number(onSquares) + Number(onRoots);
            const lockToggle = (currentlyOn: boolean) =>
                enabledCount === 1 && currentlyOn;
            return (
                <div className="space-y-4">
                    <FieldGroup label={ui.families}>
                        <OpToggle
                            id="op-squares"
                            label={ui.squares}
                            checked={onSquares}
                            disabled={lockToggle(onSquares)}
                            onChange={(v) => patch("squares", v)}
                        />
                        <OpToggle
                            id="op-roots"
                            label={ui.roots}
                            checked={onRoots}
                            disabled={lockToggle(onRoots)}
                            onChange={(v) => patch("roots", v)}
                        />
                    </FieldGroup>
                </div>
            );
        }
    }
}

/** Vertical concept-level picker. Shows three (or however many) named
 * presets as a column of selectable rows; each row labels a concrete
 * progression step ("Tenths × whole", etc.) plus an example problem.
 * Used by kinds whose raw param matrix is too large to expose
 * directly — see `lib/levels.ts`. */
function LevelPicker({
    levels,
    copy,
    ui,
    value,
    onChange,
}: {
    levels: readonly Level[];
    /** Label + example per level, same order as `levels`. */
    copy: readonly { label: string; example: string }[];
    ui: IslandStrings;
    value: string | undefined;
    onChange: (next: string) => void;
}) {
    return (
        <div className="flex flex-col gap-2">
            {levels.map((level, idx) => {
                const id = String(idx + 1);
                const active = id === value;
                const { label, example } = copy[idx] ?? {
                    label: id,
                    example: "",
                };
                return (
                    <button
                        key={id}
                        type="button"
                        onClick={() => onChange(id)}
                        aria-pressed={active}
                        className={
                            "group flex cursor-pointer overflow-hidden rounded-md border text-sm transition-colors " +
                            (active
                                ? "border-primary bg-[color-mix(in_srgb,var(--primary)_10%,var(--card))] text-primary"
                                : "border-input bg-background hover:bg-muted")
                        }
                    >
                        <div className="flex flex-1 flex-col items-start gap-0.5 px-3 py-2 text-left">
                            <span className="font-medium">{label}</span>
                            {example && (
                                <span className="text-xs text-muted-foreground">
                                    {fmt(ui.example, { example })}
                                </span>
                            )}
                        </div>
                        <div
                            className={
                                "flex items-center justify-center border-l px-1 text-black transition-colors " +
                                (active
                                    ? "border-pencil-no-2 bg-[#fff4d9]"
                                    : "border-[#fde18c] bg-[#fffaee] group-hover:border-pencil-no-2 group-hover:bg-[#fff4d9]")
                            }
                        >
                            <span className="text-[10px] font-semibold uppercase tracking-wider [writing-mode:vertical-rl]">
                                {fmt(ui.level, { n: idx + 1 })}
                            </span>
                        </div>
                    </button>
                );
            })}
        </div>
    );
}

function OpToggle({
    id,
    label,
    checked,
    disabled,
    onChange,
}: {
    id: string;
    label: string;
    checked: boolean;
    disabled?: boolean;
    onChange: (v: boolean) => void;
}) {
    return (
        <div className="flex items-center justify-between">
            <Label htmlFor={id} className="text-xs font-normal">
                {label}
            </Label>
            <Switch
                id={id}
                checked={checked}
                disabled={disabled}
                onCheckedChange={onChange}
            />
        </div>
    );
}
