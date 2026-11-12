// Header menus (search, language, ☰) open under the header: a
// full-width sheet on phones, a card aligned to its button on wide
// screens. The styling is `.header-sheet` in styles/index.css; this
// feeds it the header's bottom edge and the card's horizontal position.
//
// The header is sticky and shrinks once the page scrolls, so its edge
// is tracked (ResizeObserver) while a menu is open rather than read once.

export function anchorUnderHeader(
    panel: HTMLElement,
    button: HTMLElement,
    width: number,
): void {
    const header = () => document.querySelector<HTMLElement>("body > header");
    const update = () => {
        const top = (header() ?? button).getBoundingClientRect().bottom;
        const r = button.getBoundingClientRect();
        const left = Math.max(
            8,
            Math.min(r.right - width, window.innerWidth - width - 8),
        );
        panel.style.setProperty("--sheet-top", `${Math.round(top)}px`);
        panel.style.setProperty("--sheet-left", `${Math.round(left)}px`);
        panel.style.setProperty("--sheet-width", `${width}px`);
    };
    const observer = new ResizeObserver(update);
    panel.addEventListener("beforetoggle", (e) => {
        if ((e as ToggleEvent).newState === "open") update();
    });
    panel.addEventListener("toggle", (e) => {
        const h = header();
        if ((e as ToggleEvent).newState === "open") {
            if (h) observer.observe(h);
            window.addEventListener("resize", update);
        } else {
            observer.disconnect();
            window.removeEventListener("resize", update);
        }
    });
}
