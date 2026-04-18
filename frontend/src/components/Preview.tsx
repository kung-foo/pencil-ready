import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { WorksheetConfig } from "@/lib/api";

export function Preview({ cfg, url }: { cfg: WorksheetConfig; url: string }) {
  const common = "w-full h-full border rounded-md bg-white";

  return (
    <div className="flex flex-col h-full min-h-0">
      <div className="flex items-center justify-between pb-3">
        <div className="text-sm text-muted-foreground font-mono truncate">{url}</div>
        <Button asChild size="sm">
          <a href={url} download>
            <Download className="size-4" /> Download
          </a>
        </Button>
      </div>
      <div className="flex-1 min-h-0">
        {cfg.format === "png" ? (
          <img src={url} alt="worksheet preview" className={common + " object-contain"} />
        ) : (
          // PDF and SVG both render in an iframe — browsers have built-in
          // PDF viewers, and SVG is just a web image.
          <iframe src={url} title="worksheet preview" className={common} />
        )}
      </div>
    </div>
  );
}
