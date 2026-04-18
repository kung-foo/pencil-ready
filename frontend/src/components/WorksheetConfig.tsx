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
import type { Format, WorksheetConfig, WorksheetKind } from "@/lib/api";

const KIND_OPTIONS: { value: WorksheetKind; label: string }[] = [
  { value: "add", label: "Addition" },
  { value: "subtract", label: "Subtraction" },
  { value: "multiply", label: "Multiplication" },
  { value: "simple-divide", label: "Division" },
  { value: "long-divide", label: "Long division" },
  { value: "mult-drill", label: "Multiplication drill" },
  { value: "div-drill", label: "Division drill" },
  { value: "fraction-mult", label: "Fraction multiplication" },
  { value: "algebra-two-step", label: "Two-step equations" },
];

export function WorksheetConfigPanel({
  cfg,
  onChange,
}: {
  cfg: WorksheetConfig;
  onChange: (cfg: WorksheetConfig) => void;
}) {
  function patch<K extends keyof WorksheetConfig>(
    key: K,
    value: WorksheetConfig[K],
  ) {
    onChange({ ...cfg, [key]: value });
  }

  function changeKind(kind: WorksheetKind) {
    // Reset kind-specific fields when switching types; shared fields persist.
    const { format, seed, solve_first, problems, cols } = cfg;
    onChange({
      kind,
      format,
      seed,
      solve_first,
      problems,
      cols,
    } as WorksheetConfig);
  }

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Worksheet</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Field label="Type">
          <Select
            value={cfg.kind}
            onValueChange={(v) => changeKind(v as WorksheetKind)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {KIND_OPTIONS.map((o) => (
                <SelectItem key={o.value} value={o.value}>
                  {o.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>

        <KindSpecific cfg={cfg} onChange={onChange} />

        <div className="pt-2 border-t space-y-4">
          <Field label="Seed">
            <Input
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
          </Field>

          <div className="flex items-center justify-between">
            <Label htmlFor="solve_first">Solve first problem</Label>
            <Switch
              id="solve_first"
              checked={cfg.solve_first ?? false}
              onCheckedChange={(v) => patch("solve_first", v)}
            />
          </div>

          <Field label="Format">
            <div className="flex gap-2">
              {(["pdf", "png", "svg"] as Format[]).map((f) => (
                <Button
                  key={f}
                  type="button"
                  variant={cfg.format === f ? "default" : "outline"}
                  size="sm"
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
              onValueChange={(v) =>
                patch("carry", v as NonNullable<typeof cfg.carry>)
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">None</SelectItem>
                <SelectItem value="any">Any</SelectItem>
                <SelectItem value="force">Force</SelectItem>
                <SelectItem value="ripple">Ripple (multi-column)</SelectItem>
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
              onValueChange={(v) =>
                patch("borrow", v as NonNullable<typeof cfg.borrow>)
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">None</SelectItem>
                <SelectItem value="no-across-zero">No across zero</SelectItem>
                <SelectItem value="any">Any</SelectItem>
                <SelectItem value="force">Force</SelectItem>
                <SelectItem value="ripple">Ripple (multi-column)</SelectItem>
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
