import { useMemo, useState } from "react";
import { navigate } from "astro:transitions/client";
import { DownloadButton } from "@/components/DownloadButton";
import { Preview } from "@/components/Preview";
import { PrintButton } from "@/components/PrintButton";
import { WorksheetConfigPanel } from "@/components/WorksheetConfig";
import {
  configToSearchParams,
  parseConfig,
  worksheetUrl,
  type WorksheetConfig,
  type WorksheetKind,
} from "@/lib/api";
import { useNames } from "@/lib/useNames";
import { useWorksheet } from "@/lib/useWorksheet";

/**
 * Interactive worksheet configurator, rendered as an Astro island.
 * The surrounding Astro page carries the static, indexable content
 * (title/summary/prerequisites/learning); this component owns the live
 * state for tweaking the worksheet and previewing it. Same-origin —
 * Astro dev proxies /api to the Rust backend; in prod the Rust server
 * serves both the Astro bundle and the API.
 *
 * Changing the worksheet type in the dropdown hard-navigates to the new
 * /worksheets/{kind}/ route so the page's static content (title,
 * description, prereqs, help) updates to match. Current params are
 * carried over as query string.
 */
/**
 * Per-kind first-visit defaults, applied only when none of a kind's
 * worksheet-shaping keys are present in the URL. Lets the configurator
 * land on a working preview without baking implicit defaults into
 * `parseConfig` (which stays a pure URL → cfg parser). After the user
 * makes any change, the URL carries explicit values and the strict
 * "absence == off" rule takes over.
 */
function applyFirstVisitDefaults(
  cfg: WorksheetConfig,
  search: URLSearchParams,
): WorksheetConfig {
  if (cfg.kind === "algebra-one-step") {
    const noToggleInUrl =
      !search.has("add") &&
      !search.has("subtract") &&
      !search.has("multiply") &&
      !search.has("divide");
    if (noToggleInUrl) {
      return { ...cfg, add: true, subtract: true };
    }
  }
  return cfg;
}

export function WorksheetIsland({ kind }: { kind: WorksheetKind }) {
  const [cfg, setCfg] = useState<WorksheetConfig>(() => {
    // Seed from URL query so deep-links like /worksheets/add/?seed=42&format=svg
    // and type-change redirects that carry params land with the right state.
    const search =
      typeof window === "undefined"
        ? new URLSearchParams()
        : new URLSearchParams(window.location.search);
    return applyFirstVisitDefaults(parseConfig(kind, search), search);
  });
  const [names, patchNames] = useNames();
  const url = useMemo(() => worksheetUrl(cfg, names), [cfg, names]);
  const state = useWorksheet(url);

  const onChange = (next: WorksheetConfig) => {
    if (next.kind !== kind) {
      // Type changed → navigate via Astro's ClientRouter so the
      // per-route static content (title, summary, help) swaps in
      // without a full page reload. Preserve current query params so
      // seed / format / etc. carry across.
      const qs = configToSearchParams(next).toString();
      navigate(`/worksheets/${next.kind}/${qs ? `?${qs}` : ""}`);
      return;
    }
    setCfg(next);
    // Sync URL so the state is shareable and survives reload.
    const qs = configToSearchParams(next).toString();
    const path = `/worksheets/${kind}/${qs ? `?${qs}` : ""}`;
    window.history.replaceState(null, "", path);
  };

  return (
    <div
      className="grid gap-4 md:grid-cols-[320px_1fr]"
      data-theme="graph-paper"
    >
      <div className="space-y-3">
        <WorksheetConfigPanel
          cfg={cfg}
          onChange={onChange}
          names={names}
          onNamesChange={patchNames}
        />
        <DownloadButton state={state} />
        <PrintButton state={state} />
      </div>
      <div className="aspect-[210/297] md:aspect-auto md:min-h-[80vh]">
        <Preview cfg={cfg} state={state} />
      </div>
    </div>
  );
}
