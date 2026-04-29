import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetState } from "@/lib/useWorksheet";

export function DownloadButton({ state }: { state: WorksheetState }) {
    if (state.status === "ready") {
        return (
            <Button asChild className="w-full">
                <a href={state.blobUrl} download={state.filename}>
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
