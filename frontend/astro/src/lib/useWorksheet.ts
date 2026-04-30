import { useEffect, useRef, useState } from "react";

export type WorksheetState =
    | { status: "loading" }
    | { status: "ready"; blobUrl: string; filename: string }
    | { status: "error"; message: string };

const FILENAME_RE = /filename="([^"]+)"/;

function messageForStatus(status: number): string {
    if (status === 404) return "This worksheet isn't available right now.";
    if (status >= 500) return "The worksheet generator hit an error.";
    return "Couldn't generate this worksheet.";
}

/**
 * Fetch the server-generated worksheet bytes as a Blob, expose it as an
 * object URL. Keeps the previous object URL alive until the next one is
 * ready so the in-page preview never flickers. Caller uses `blobUrl` for
 * both the iframe/img src and the `<a href>` of the download button,
 * guaranteeing preview === download bytes.
 */
export function useWorksheet(url: string): WorksheetState {
    const [state, setState] = useState<WorksheetState>({ status: "loading" });
    const currentBlobUrl = useRef<string | null>(null);

    useEffect(() => {
        let cancelled = false;
        setState({ status: "loading" });

        (async () => {
            try {
                const res = await fetch(url);
                if (!res.ok) {
                    // Keep the body for diagnostics but never show it to the
                    // user — when the server is misconfigured or stale it
                    // can return an HTML 404 page, and dumping that into the
                    // preview leaks framing markup into the UI.
                    const body = await res.text().catch(() => "");
                    console.error(
                        `Worksheet fetch failed (${res.status}) for ${url}`,
                        body,
                    );
                    throw new Error(messageForStatus(res.status));
                }
                const cd = res.headers.get("Content-Disposition") ?? "";
                const filename = FILENAME_RE.exec(cd)?.[1] ?? "worksheet";
                const blob = await res.blob();
                if (cancelled) return;

                const next = URL.createObjectURL(blob);
                const prev = currentBlobUrl.current;
                currentBlobUrl.current = next;
                setState({ status: "ready", blobUrl: next, filename });
                if (prev) URL.revokeObjectURL(prev);
            } catch (e) {
                if (cancelled) return;
                console.error("Worksheet fetch error", e);
                setState({
                    status: "error",
                    message:
                        e instanceof Error && e.message
                            ? e.message
                            : "Couldn't reach the worksheet generator.",
                });
            }
        })();

        return () => {
            cancelled = true;
        };
    }, [url]);

    // Revoke the last object URL on unmount.
    useEffect(
        () => () => {
            if (currentBlobUrl.current)
                URL.revokeObjectURL(currentBlobUrl.current);
        },
        [],
    );

    return state;
}
