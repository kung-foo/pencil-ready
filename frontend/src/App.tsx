import { useMemo, useState } from "react";
import { Preview } from "@/components/Preview";
import { WorksheetConfigPanel } from "@/components/WorksheetConfig";
import { worksheetUrl, type WorksheetConfig } from "@/lib/api";

const DEFAULT_CONFIG: WorksheetConfig = {
  kind: "add",
  format: "pdf",
  seed: 42,
};

export default function App() {
  const [cfg, setCfg] = useState<WorksheetConfig>(DEFAULT_CONFIG);
  const url = useMemo(() => worksheetUrl(cfg), [cfg]);

  return (
    <div className="h-screen flex flex-col">
      <header className="border-b px-6 py-3">
        <h1 className="text-lg font-semibold">Pencil Ready</h1>
        <p className="text-xs text-muted-foreground">Worksheet configurator</p>
      </header>
      <div className="flex-1 grid grid-cols-[320px_1fr] gap-4 p-4 min-h-0">
        <aside className="overflow-auto">
          <WorksheetConfigPanel cfg={cfg} onChange={setCfg} />
        </aside>
        <main className="min-h-0">
          <Preview cfg={cfg} url={url} />
        </main>
      </div>
    </div>
  );
}
