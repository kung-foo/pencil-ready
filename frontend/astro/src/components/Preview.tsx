import type { WorksheetConfig } from "@/lib/api";
import type { WorksheetState } from "@/lib/useWorksheet";
import type { IslandStrings } from "@/i18n";
import { paperAspect, type Paper } from "@/lib/paper";

/**
 * Pure renderer — state is owned by the parent (WorksheetPage) so the
 * DownloadButton in the sidebar can share the same blob URL. The A4 shell
 * is always rendered so loading / error / content states all occupy the
 * same box — no layout shift when the document arrives.
 */
export function Preview({
    cfg,
    state,
    ui,
    paper,
}: {
    cfg: WorksheetConfig;
    state: WorksheetState;
    ui: IslandStrings;
    paper: Paper;
}) {
    const aspect = paperAspect(paper);
    return (
        <div
            className="w-full h-full flex items-start justify-center"
            style={{ containerType: "size" }}
        >
            {/* data-print-area flags this subtree as the only thing to print
          when the browser falls back to page-level printing (iOS Share
          → Print, desktop Cmd/Ctrl+P). CSS in index.css does the rest. */}
            <div
                data-print-area
                className="bg-white rounded-sm ring-1 ring-black/5 shadow-[0_4px_16px_rgba(0,0,0,0.08),0_1px_3px_rgba(0,0,0,0.06)] overflow-hidden"
                style={{
                    // Width is the smaller of (container width) and the width the
                    // page would need to fill the container height — whichever binds
                    // first. Height derives from aspect-ratio (A4 or US Letter).
                    width: `min(100cqw, calc(100cqh * ${aspect}))`,
                    aspectRatio: String(aspect),
                }}
            >
                <Body cfg={cfg} state={state} ui={ui} />
            </div>
        </div>
    );
}

function Body({
    cfg,
    state,
    ui,
}: {
    cfg: WorksheetConfig;
    state: WorksheetState;
    ui: IslandStrings;
}) {
    if (state.status === "loading") {
        return (
            <div className="w-full h-full flex items-center justify-center text-muted-foreground text-sm">
                {ui.rendering}
            </div>
        );
    }
    if (state.status === "error") {
        return (
            <div className="w-full h-full flex items-center justify-center p-8 text-center">
                <div className="max-w-xs space-y-2">
                    <p className="text-sm font-medium text-foreground">
                        {state.message}
                    </p>
                    <p className="text-xs text-muted-foreground">
                        {ui.errorHint}
                    </p>
                </div>
            </div>
        );
    }

    if (cfg.format === "pdf") {
        // Viewer params:
        //   toolbar=0 / navpanes=0  (Chrome/Edge/Acrobat) — hide in-viewer chrome.
        //   view=FitH / zoom=page-width — fit page to the iframe's width. FitH
        //     is the Adobe spec value; zoom=page-width is pdf.js's extension.
        const pdfSrc = `${state.blobUrl}#toolbar=0&navpanes=0&view=FitH&zoom=page-width`;
        return (
            <iframe
                src={pdfSrc}
                title={ui.previewTitle}
                className="w-full h-full border-0"
            />
        );
    }
    // alt="" — the preview mirrors what the form configured, so the image is
    // decorative for a11y purposes. Empty alt also prevents the text from
    // flashing in the top-left while the blob decodes.
    return (
        <img
            src={state.blobUrl}
            alt=""
            className="w-full h-full object-contain"
        />
    );
}
