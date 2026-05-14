/*
 * Paint a sparse two-tone speckle into a <canvas>. Used by the
 * Stippled.astro / StippleBg.tsx components — see those for the
 * lifecycle wrapping (ResizeObserver + astro lifecycle hooks /
 * React useEffect).
 *
 * Dot grid: one device pixel per cell, density ~4%. Half the dots
 * are translucent black, half match the graph-paper grid hue (217)
 * so the texture ties into the page background. Cards have white
 * bg; the result reads as a faint paper tooth.
 */
export function paintStipple(canvas: HTMLCanvasElement): void {
    const rect = canvas.getBoundingClientRect();
    if (!rect.width || !rect.height) return;
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(rect.width * dpr);
    const h = Math.round(rect.height * dpr);
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const size = dpr;
    const density = 0.04;
    ctx.clearRect(0, 0, w, h);
    const black = "rgba(0, 0, 0, 0.07)";
    const blue = "hsla(217, 70%, 50%, 0.1)";
    for (let y = 0; y < h; y += size) {
        for (let x = 0; x < w; x += size) {
            if (Math.random() < density) {
                ctx.fillStyle = Math.random() < 0.5 ? black : blue;
                ctx.fillRect(x, y, size, size);
            }
        }
    }
}
