import { BookOpen, GraduationCap } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { WorksheetKind } from "@/lib/api";
import { WORKSHEET_INFO } from "@/lib/worksheet-info";

export function WorksheetHelp({ kind }: { kind: WorksheetKind }) {
  const info = WORKSHEET_INFO[kind];

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>About this worksheet</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground italic">{info.summary}</p>

        <Section icon={<BookOpen className="size-4" />} label="Before you start">
          {info.prerequisites}
        </Section>

        <Section icon={<GraduationCap className="size-4" />} label="With mastery, the student learns">
          {info.learning}
        </Section>
      </CardContent>
    </Card>
  );
}

function Section({
  icon,
  label,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  children: string[];
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-sm font-semibold">
        <span className="text-muted-foreground">{icon}</span>
        <span>{label}</span>
      </div>
      <ul className="text-sm space-y-1.5 pl-6 list-disc marker:text-muted-foreground">
        {children.map((item, i) => (
          <li key={i}>{item}</li>
        ))}
      </ul>
    </div>
  );
}
