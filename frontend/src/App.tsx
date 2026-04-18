import { useEffect, useState } from "react";
import {
  BrowserRouter,
  Navigate,
  Route,
  Routes,
  useNavigate,
  useParams,
  useSearchParams,
} from "react-router-dom";
import { AppFooter } from "@/components/AppFooter";
import { DownloadButton } from "@/components/DownloadButton";
import { Preview } from "@/components/Preview";
// ThemeSwitcher hidden for now — keep the import/component for easy re-enable.
// import { ThemeSwitcher } from "@/components/ThemeSwitcher";
import { WorksheetConfigPanel } from "@/components/WorksheetConfig";
import { WorksheetHelp } from "@/components/WorksheetHelp";
import {
  configToSearchParams,
  parseConfig,
  worksheetUrl,
  type WorksheetConfig,
  type WorksheetKind,
  WORKSHEET_KINDS,
} from "@/lib/api";
import { applyTheme, loadTheme, saveTheme } from "@/lib/themes";
import { useWorksheet } from "@/lib/useWorksheet";

export default function App() {
  // Theme state retained so the saved theme still applies on load. Setter
  // is unused while the switcher is hidden — will come back when we
  // re-expose the control.
  const [theme] = useState<string>(() => loadTheme());

  useEffect(() => {
    applyTheme(theme);
    saveTheme(theme);
  }, [theme]);

  return (
    <BrowserRouter>
      <div className="h-screen flex flex-col">
        <header className="border-b px-6 py-3">
          <h1
            className="text-lg font-semibold"
            style={{ fontFamily: "var(--font-display)" }}
          >
            Pencil Ready
          </h1>
          <p className="text-xs text-muted-foreground">Worksheet configurator</p>
        </header>

        <Routes>
          <Route path="/" element={<Navigate to="/worksheets/add" replace />} />
          <Route path="/worksheets/:kind" element={<WorksheetPage />} />
          <Route path="*" element={<Navigate to="/worksheets/add" replace />} />
        </Routes>

        <AppFooter />
      </div>
    </BrowserRouter>
  );
}

function WorksheetPage() {
  const { kind: kindParam } = useParams<{ kind: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();

  // Guard the URL kind against the allowed list. An invalid kind bounces
  // the user to the default — /worksheets/add.
  const kind = (WORKSHEET_KINDS as readonly string[]).includes(kindParam ?? "")
    ? (kindParam as WorksheetKind)
    : null;
  if (!kind) return <Navigate to="/worksheets/add" replace />;

  const cfg = parseConfig(kind, searchParams);
  const url = worksheetUrl(cfg);
  const state = useWorksheet(url);

  const onChange = (next: WorksheetConfig) => {
    const params = configToSearchParams(next);
    if (next.kind !== kind) {
      // Type change: path changes too. Carry over any shared query params
      // that still apply — the parser drops anything irrelevant.
      const qs = params.toString();
      navigate(`/worksheets/${next.kind}${qs ? `?${qs}` : ""}`);
    } else {
      setSearchParams(params, { replace: false });
    }
  };

  return (
    <div className="flex-1 grid grid-cols-[320px_1fr_320px] gap-4 p-4 min-h-0">
      <aside className="overflow-auto pr-1 space-y-3">
        <WorksheetConfigPanel cfg={cfg} onChange={onChange} />
        <DownloadButton state={state} />
      </aside>
      <main className="min-h-0">
        <Preview cfg={cfg} state={state} />
      </main>
      <aside className="overflow-auto pl-1">
        <WorksheetHelp kind={cfg.kind} />
      </aside>
    </div>
  );
}
