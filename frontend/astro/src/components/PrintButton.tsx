import { Printer } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetState } from "@/lib/useWorksheet";

// iOS/iPadOS (all browsers, since every browser on iOS uses WebKit) does
// not honor iframe.contentWindow.print() — it either no-ops or falls
// through to printing the whole page. Detect it and route those users
// through the native PDF viewer in a new tab, where Share → Print works
// correctly. Modern iPad reports "MacIntel" from platform; the
// maxTouchPoints check disambiguates.
function isIOS() {
    if (typeof navigator === "undefined") return false;
    return (
        /iPad|iPhone|iPod/.test(navigator.userAgent) ||
        (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1)
    );
}

/**
 * Print the currently-loaded worksheet bytes (not the whole web page). On
 * desktop browsers a hidden iframe pointed at the blob URL handles it
 * cleanly — the print dialog uses the PDF's native A4 page size. On iOS
 * we instead pop the blob into a new tab; iOS's PDF viewer + share sheet
 * is the user's only practical path to AirPrint.
 */
export function PrintButton({ state }: { state: WorksheetState }) {
    const ready = state.status === "ready";

    const onPrint = () => {
        if (!ready) return;
        if (isIOS()) {
            window.open(state.blobUrl, "_blank");
            return;
        }
        const iframe = document.createElement("iframe");
        Object.assign(iframe.style, {
            position: "fixed",
            right: "0",
            bottom: "0",
            width: "0",
            height: "0",
            border: "0",
        });
        iframe.src = state.blobUrl;
        iframe.onload = () => {
            requestAnimationFrame(() => {
                try {
                    iframe.contentWindow?.focus();
                    iframe.contentWindow?.print();
                } catch {
                    // Some browsers throw on cross-document access; ignore.
                }
            });
        };
        document.body.appendChild(iframe);
        // `afterprint` doesn't fire reliably for iframe print() in all
        // browsers; a generous timeout is the simplest correct cleanup.
        setTimeout(() => iframe.remove(), 60_000);
    };

    return (
        <Button
            type="button"
            variant="outline"
            className="w-full"
            onClick={onPrint}
            disabled={!ready}
        >
            <Printer className="size-4" /> Print
        </Button>
    );
}
