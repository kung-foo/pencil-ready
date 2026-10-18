import { useEffect, useState } from "react";
import { PAPER_EVENT, getPaper, isPaper, type Paper } from "./paper";

/** Current paper (see `paper.ts`), kept in sync with changes made from
 * the language menu. */
export function usePaper(): Paper {
    const [paper, setPaper] = useState<Paper>(getPaper);
    useEffect(() => {
        const onChange = (e: Event) => {
            const next = (e as CustomEvent<Paper>).detail;
            if (isPaper(next)) setPaper(next);
        };
        window.addEventListener(PAPER_EVENT, onChange);
        return () => window.removeEventListener(PAPER_EVENT, onChange);
    }, []);
    return paper;
}
