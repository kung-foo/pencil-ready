import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetKind } from "@/lib/api";
import type { WorksheetState } from "@/lib/useWorksheet";
import type { IslandStrings } from "@/i18n";

export function DownloadButton({
    state,
    kind,
    ui,
}: {
    state: WorksheetState;
    kind: WorksheetKind;
    ui: IslandStrings;
}) {
    if (state.status === "ready") {
        const format = state.filename.split(".").pop() ?? "pdf";
        return (
            <Button asChild className="w-full">
                <a
                    href={state.blobUrl}
                    download={state.filename}
                    onClick={() =>
                        window.umami?.track("download", { kind, format })
                    }
                >
                    <Download className="size-4" /> {ui.download}
                </a>
            </Button>
        );
    }
    return (
        <Button disabled className="w-full">
            <Download className="size-4" />{" "}
            {state.status === "error" ? ui.download : ui.preparing}
        </Button>
    );
}
