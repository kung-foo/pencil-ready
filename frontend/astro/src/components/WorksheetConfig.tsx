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
import { FORMATS, type WorksheetConfig } from "@/lib/api";
import { levelsFor, defaultLevel, type Level } from "@/lib/levels";
import type { Names } from "@/lib/useNames";

export function WorksheetConfigPanel({
    cfg,
    onChange,
    names,
    onNamesChange,
}: {
    cfg: WorksheetConfig;
    onChange: (cfg: WorksheetConfig) => void;
    names: Names;
    onNamesChange: (patch: Partial<Names>) => void;
}) {
    function patch<K extends keyof WorksheetConfig>(
        key: K,
        value: WorksheetConfig[K],
    ) {
        onChange({ ...cfg, [key]: value });
    }

    return (
        <Card className="stippled w-full">
            <canvas className="stipple-canvas" />
            {/*<CardHeader>
        <CardTitle>Worksheet</CardTitle>
      </CardHeader>*/}
            <CardContent className="space-y-4">
                <KindSpecific cfg={cfg} onChange={onChange} />

                <div className="pt-2 border-t space-y-4">
                    <Field label="Seed">
                        <InputGroup>
                            <InputGroupInput
                                type="number"
                                placeholder="random"
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
                                    aria-label="Shuffle seed"
                                    title="Pick a new random seed"
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
                        <Label htmlFor="solve_first">Solve first problem</Label>
                        <Switch
                            id="solve_first"
                            checked={cfg.solve_first ?? false}
                            onCheckedChange={(v) => patch("solve_first", v)}
                        />
                    </div>

                    <div className="flex items-center justify-between">
                        <Label htmlFor="include_answers">
                            Answer key{" "}
                            <span className="text-xs text-muted-foreground">
                                (PDF only)
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
                    />

                    <Field label="Format">
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

function PersonalizeSection({
    names,
    onNamesChange,
}: {
    names: Names;
    onNamesChange: (patch: Partial<Names>) => void;
}) {
    const summary = names.student ?? "";

    return (
        <Accordion type="single" collapsible>
            <AccordionItem value="personalize" className="border-b-0">
                <AccordionTrigger className="py-2">
                    <span className="grid flex-1 grid-cols-[auto_minmax(0,1fr)] items-baseline gap-2">
                        <span>Personalize</span>
                        {summary && (
                            <span className="truncate text-xs font-normal text-muted-foreground">
                                {summary}
                            </span>
                        )}
                    </span>
                </AccordionTrigger>
                <AccordionContent className="space-y-3">
                    <SubField label="Student name">
                        <DeferredNameInput
                            value={names.student ?? ""}
                            placeholder="e.g. Math Hippo"
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
}: {
    cfg: WorksheetConfig;
    onChange: (cfg: WorksheetConfig) => void;
}) {
    // Key/value typed loosely: TS can't distribute `keyof` across the kind
    // union well enough to infer per-variant keys. The switch narrowing below
    // keeps each call site honest.
    const patch = (key: string, value: unknown) =>
        onChange({ ...cfg, [key]: value } as WorksheetConfig);

    switch (cfg.kind) {
        case "multiply":
            return (
                <Field label="Digits (e.g. 2,2 or 3,2)">
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
                    label="Max quotient"
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
                    <Field label="Multiplicand (e.g. 2,3 or 1-10)">
                        <Input
                            value={cfg.multiplicand ?? ""}
                            placeholder="1-10"
                            onChange={(e) =>
                                patch("multiplicand", e.target.value)
                            }
                        />
                    </Field>
                    <RangeSliderField
                        label="Multiplier"
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
                    <Field label="Divisor">
                        <Input
                            value={cfg.divisor ?? ""}
                            placeholder="2-9"
                            onChange={(e) => patch("divisor", e.target.value)}
                        />
                    </Field>
                    <RangeSliderField
                        label="Max quotient"
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
                    <Field label="Denominators">
                        <Input
                            value={cfg.denominators ?? ""}
                            placeholder="2,3,4,5,10"
                            onChange={(e) =>
                                patch("denominators", e.target.value)
                            }
                        />
                    </Field>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="unit_only">Unit fractions only</Label>
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
                        label="Denominators"
                        min={2}
                        max={20}
                        value={[denMin, denMax]}
                        onCommit={([lo, hi]) =>
                            patch("denominators", expandDenominators(lo, hi))
                        }
                    />
                    <SliderField
                        label="Max numerator"
                        min={5}
                        max={30}
                        value={cfg.max_numerator ?? 20}
                        onCommit={(v) => patch("max_numerator", v)}
                    />
                    <div className="flex items-center justify-between">
                        <Label htmlFor="proper_only">
                            Proper fractions only
                        </Label>
                        <Switch
                            id="proper_only"
                            checked={cfg.proper_only ?? false}
                            onCheckedChange={(v) => patch("proper_only", v)}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="include_whole">
                            Allow whole-number answers
                        </Label>
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
        case "decimal-multiply": {
            const levels = levelsFor(cfg.kind) ?? [];
            const current = cfg.level ?? defaultLevel(cfg.kind);
            return (
                <LevelPicker
                    levels={levels}
                    value={current}
                    onChange={(v) => patch("level", v)}
                />
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
                    <FieldGroup label="Families">
                        <OpToggle
                            id="op-squares"
                            label="Squares (x²)"
                            checked={onSquares}
                            disabled={lockToggle(onSquares)}
                            onChange={(v) => patch("squares", v)}
                        />
                        <OpToggle
                            id="op-roots"
                            label="Square roots (√x)"
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
    value,
    onChange,
}: {
    levels: readonly Level[];
    value: string | undefined;
    onChange: (next: string) => void;
}) {
    return (
        <div className="flex flex-col gap-2">
            {levels.map((level, idx) => {
                const id = String(idx + 1);
                const active = id === value;
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
                            <span className="font-medium">{level.label}</span>
                            {level.example && (
                                <span className="text-xs text-muted-foreground">
                                    e.g. {level.example}
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
                                Level {idx + 1}
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
