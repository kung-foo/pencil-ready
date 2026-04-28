import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetKind } from "@/lib/api";
import type { WorksheetState } from "@/lib/useWorksheet";

export function DownloadButton({
  state,
  kind,
}: {
  state: WorksheetState;
  kind: WorksheetKind;
}) {
  if (state.status === "ready") {
    const format = state.filename.split(".").pop() ?? "pdf";
    return (
      <Button asChild className="w-full">
        <a
          href={state.blobUrl}
          download={state.filename}
          onClick={() => window.umami?.track("download", { kind, format })}
        >
          <Download className="size-4" /> Download
        </a>
      </Button>
    );
  }
  return (
    <Button disabled className="w-full">
      <Download className="size-4" />{" "}
      {state.status === "error" ? "Download" : "Preparing…"}
    </Button>
  );
}
