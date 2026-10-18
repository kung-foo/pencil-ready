import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetKind } from "@/lib/api";
import type { WorksheetState } from "@/lib/useWorksheet";
import type { IslandStrings } from "@/i18n";
import type { Locale } from "@/i18n/locales";

export function DownloadButton({
    state,
    kind,
    locale,
    ui,
}: {
    state: WorksheetState;
    kind: WorksheetKind;
    locale: Locale;
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
                        window.umami?.track("download", {
                            kind,
                            format,
                            locale,
                        })
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
