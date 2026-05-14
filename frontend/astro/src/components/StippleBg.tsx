import { useEffect, useRef } from "react";
import { paintStipple } from "@/lib/stipple";

/*
 * Drop inside any React element with `className="stippled"` to give
 * it a faint canvas-stipple background. Self-paints + observes
 * resize via useEffect — no dependency on the global Astro painter.
 * `data-stipple-react` tells that painter to leave this canvas
 * alone (no double observers).
 */
export function StippleBg() {
    const ref = useRef<HTMLCanvasElement>(null);
    useEffect(() => {
        const canvas = ref.current;
        if (!canvas) return;
        paintStipple(canvas);
        const ro = new ResizeObserver(() => paintStipple(canvas));
        ro.observe(canvas);
        return () => ro.disconnect();
    }, []);
    return (
        <canvas
            ref={ref}
            className="stipple-canvas"
            data-stipple-react="1"
        />
    );
}
