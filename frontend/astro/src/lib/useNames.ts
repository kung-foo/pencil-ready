import { useEffect, useState } from "react";

export type Names = { student?: string };

const KEY = "pr.names";

/**
 * Names persist per-device in localStorage. They aren't part of
 * WorksheetConfig because they don't describe the worksheet shape and
 * shouldn't leak into shareable URLs.
 */
export function useNames(): [Names, (patch: Partial<Names>) => void] {
    const [names, setNames] = useState<Names>(() => {
        if (typeof window === "undefined") return {};
        try {
            const raw = window.localStorage.getItem(KEY);
            return raw ? (JSON.parse(raw) as Names) : {};
        } catch {
            return {};
        }
    });

    useEffect(() => {
        if (typeof window === "undefined") return;
        try {
            window.localStorage.setItem(KEY, JSON.stringify(names));
        } catch {
            // localStorage can throw in privacy modes / quota — fine to ignore.
        }
    }, [names]);

    const update = (patch: Partial<Names>) =>
        setNames((prev) => {
            const next = { ...prev, ...patch };
            // Drop empty strings so the server sees them as absent.
            for (const k of Object.keys(next) as (keyof Names)[]) {
                if (!next[k]) delete next[k];
            }
            return next;
        });

    return [names, update];
}
