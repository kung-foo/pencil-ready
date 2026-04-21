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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Shuffle } from "lucide-react";
import {
  BORROW_MODES,
  BORROW_MODE_LABELS,
  CARRY_MODES,
  CARRY_MODE_LABELS,
  FORMATS,
  type WorksheetConfig,
} from "@/lib/api";
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
    <Card className="w-full">
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
                    e.target.value === "" ? undefined : Number(e.target.value),
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
                    patch("seed", Math.floor(Math.random() * 1_000_000))
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
              <span className="text-xs text-muted-foreground">(PDF only)</span>
            </Label>
            <Switch
              id="include_answers"
              checked={cfg.include_answers ?? false}
              onCheckedChange={(v) => {
                // Turning on the answer key forces PDF — PNG/SVG are
                // single-image formats and can't carry a second page.
                if (v && cfg.format !== "pdf") {
                  onChange({ ...cfg, include_answers: true, format: "pdf" });
                } else {
                  patch("include_answers", v);
                }
              }}
            />
          </div>

          <PersonalizeSection names={names} onNamesChange={onNamesChange} />

          <Field label="Format">
            <div className="flex gap-2">
              {FORMATS.map((f) => (
                <Button
                  key={f}
                  type="button"
                  variant={cfg.format === f ? "default" : "outline"}
                  size="sm"
                  // PNG / SVG are single-image; grey them out when the
                  // user has the answer-key toggle on.
                  disabled={!!cfg.include_answers && f !== "pdf"}
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
    case "add":
      return (
        <div className="space-y-4">
          <Field label="Digits (e.g. 2,2 or 2-4,2-4)">
            <Input
              value={cfg.digits ?? ""}
              placeholder="2,2"
              onChange={(e) => patch("digits", e.target.value)}
            />
          </Field>
          <Field label="Carrying">
            <Select
              value={cfg.carry ?? "any"}
              onValueChange={(v) => patch("carry", v)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CARRY_MODES.map((m) => (
                  <SelectItem key={m} value={m}>
                    {CARRY_MODE_LABELS[m]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
          <div className="flex items-center justify-between">
            <Label htmlFor="binary">Binary (base 2)</Label>
            <Switch
              id="binary"
              checked={cfg.binary ?? false}
              onCheckedChange={(v) => patch("binary", v)}
            />
          </div>
        </div>
      );

    case "subtract":
      return (
        <div className="space-y-4">
          <Field label="Digits">
            <Input
              value={cfg.digits ?? ""}
              placeholder="2,2"
              onChange={(e) => patch("digits", e.target.value)}
            />
          </Field>
          <Field label="Borrowing">
            <Select
              value={cfg.borrow ?? "any"}
              onValueChange={(v) => patch("borrow", v)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {BORROW_MODES.map((m) => (
                  <SelectItem key={m} value={m}>
                    {BORROW_MODE_LABELS[m]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
        </div>
      );

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
        <Field label="Max quotient (2–12)">
          <Input
            type="number"
            placeholder="10"
            value={cfg.max_quotient ?? ""}
            onChange={(e) =>
              patch(
                "max_quotient",
                e.target.value === "" ? undefined : Number(e.target.value),
              )
            }
          />
        </Field>
      );

    case "long-divide":
      return (
        <div className="space-y-4">
          <Field label="Dividend digits">
            <Input
              value={cfg.digits ?? ""}
              placeholder="3"
              onChange={(e) => patch("digits", e.target.value)}
            />
          </Field>
          <div className="flex items-center justify-between">
            <Label htmlFor="remainder">Allow remainders</Label>
            <Switch
              id="remainder"
              checked={cfg.remainder ?? false}
              onCheckedChange={(v) => patch("remainder", v)}
            />
          </div>
        </div>
      );

    case "mult-drill":
      return (
        <div className="space-y-4">
          <Field label="Multiplicand (e.g. 2,3 or 1-10)">
            <Input
              value={cfg.multiplicand ?? ""}
              placeholder="1-10"
              onChange={(e) => patch("multiplicand", e.target.value)}
            />
          </Field>
          <Field label="Multiplier">
            <Input
              value={cfg.multiplier ?? ""}
              placeholder="1-10"
              onChange={(e) => patch("multiplier", e.target.value)}
            />
          </Field>
        </div>
      );

    case "div-drill":
      return (
        <div className="space-y-4">
          <Field label="Divisor">
            <Input
              value={cfg.divisor ?? ""}
              placeholder="2-10"
              onChange={(e) => patch("divisor", e.target.value)}
            />
          </Field>
          <Field label="Max quotient">
            <Input
              value={cfg.max_quotient ?? ""}
              placeholder="2-10"
              onChange={(e) => patch("max_quotient", e.target.value)}
            />
          </Field>
        </div>
      );

    case "fraction-mult":
      return (
        <div className="space-y-4">
          <Field label="Denominators">
            <Input
              value={cfg.denominators ?? ""}
              placeholder="2,3,4,5,10"
              onChange={(e) => patch("denominators", e.target.value)}
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

    case "algebra-two-step":
      return (
        <div className="space-y-4">
          <Field label="Coefficient range (a)">
            <Input
              value={cfg.a_range ?? ""}
              placeholder="2-10"
              onChange={(e) => patch("a_range", e.target.value)}
            />
          </Field>
          <Field label="Constant range (b)">
            <Input
              value={cfg.b_range ?? ""}
              placeholder="1-30"
              onChange={(e) => patch("b_range", e.target.value)}
            />
          </Field>
          <Field label="Answer range (x)">
            <Input
              value={cfg.x_range ?? ""}
              placeholder="0-20"
              onChange={(e) => patch("x_range", e.target.value)}
            />
          </Field>
          <div className="flex items-center justify-between">
            <Label htmlFor="implicit">Implicit form (4x)</Label>
            <Switch
              id="implicit"
              checked={cfg.implicit ?? false}
              onCheckedChange={(v) => patch("implicit", v)}
            />
          </div>
        </div>
      );
  }
}
