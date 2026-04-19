import type { WorksheetConfig } from "@/lib/api";
import type { WorksheetState } from "@/lib/useWorksheet";

/**
 * Pure renderer — state is owned by the parent (WorksheetPage) so the
 * DownloadButton in the sidebar can share the same blob URL. The A4 shell
 * is always rendered so loading / error / content states all occupy the
 * same box — no layout shift when the document arrives.
 */
export function Preview({
  cfg,
  state,
}: {
  cfg: WorksheetConfig;
  state: WorksheetState;
}) {
  return (
    <div
      className="w-full h-full flex items-start justify-center"
      style={{ containerType: "size" }}
    >
      <div
        className="bg-white rounded-sm ring-1 ring-black/5 shadow-[0_4px_16px_rgba(0,0,0,0.08),0_1px_3px_rgba(0,0,0,0.06)] overflow-hidden"
        style={{
          // Width is the smaller of (container width) and the width an A4
          // page would need to fill the container height — whichever binds
          // first. Height derives from aspect-ratio.
          width: "min(100cqw, calc(100cqh * 210 / 297))",
          aspectRatio: "210 / 297",
        }}
      >
        <Body cfg={cfg} state={state} />
      </div>
    </div>
  );
}

function Body({
  cfg,
  state,
}: {
  cfg: WorksheetConfig;
  state: WorksheetState;
}) {
  if (state.status === "loading") {
    return (
      <div className="w-full h-full flex items-center justify-center text-muted-foreground text-sm">
        Rendering…
      </div>
    );
  }
  if (state.status === "error") {
    return (
      <div className="w-full h-full p-4 font-mono text-sm whitespace-pre-wrap overflow-auto bg-destructive/5 text-destructive">
        {state.message}
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
        title="worksheet preview"
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
