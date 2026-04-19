import { Printer } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetState } from "@/lib/useWorksheet";

/**
 * Print the currently-loaded worksheet bytes (not the whole web page) by
 * mounting a hidden iframe pointed at the blob URL and calling
 * `contentWindow.print()`. The browser's print dialog then uses the PDF's
 * own page size (A4) for PDF output, or the image's dimensions for
 * PNG/SVG. No server round-trip — we reuse the blob the preview already
 * fetched.
 */
export function PrintButton({ state }: { state: WorksheetState }) {
  const ready = state.status === "ready";

  const onPrint = () => {
    if (!ready) return;
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
      // PDF viewers occasionally need a tick to finish setting up
      // before print() is acceptable. One frame is enough in practice.
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
    // Clean up once the print dialog has had time to appear and close.
    // `afterprint` doesn't fire reliably for iframe print() in all
    // browsers, so a generous timeout is the simplest correct answer.
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
