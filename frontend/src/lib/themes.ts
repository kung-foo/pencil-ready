/** Catalogue of visual themes for the configurator UI. */

export type Theme = {
  id: string;
  name: string;
  hint: string;
};

export const THEMES: Theme[] = [
  { id: "default", name: "Default", hint: "Clean, neutral grays" },
  { id: "paper", name: "Paper", hint: "Warm cream, slab serif" },
  { id: "chalkboard", name: "Chalkboard", hint: "Dark green, chalk white" },
  { id: "graph-paper", name: "Graph Paper", hint: "Faint blue grid" },
  { id: "blueprint", name: "Blueprint", hint: "Navy + cyan, monospace" },
  { id: "terminal", name: "Terminal", hint: "Black + amber phosphor" },
  { id: "scholar", name: "Scholar", hint: "Sepia, classical serif" },
  { id: "playground", name: "Playground", hint: "Soft pastels, rounded" },
  { id: "norsk", name: "Norsk", hint: "Norwegian flag palette" },
  { id: "minimalist", name: "Minimalist", hint: "Hairline borders, high whitespace" },
];

const THEME_STORAGE_KEY = "pencil-ready:theme";

const DEFAULT_THEME = "graph-paper";

export function loadTheme(): string {
  if (typeof window === "undefined") return DEFAULT_THEME;
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  if (stored && THEMES.some((t) => t.id === stored)) return stored;
  return DEFAULT_THEME;
}

export function saveTheme(id: string) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(THEME_STORAGE_KEY, id);
}

export function applyTheme(id: string) {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = id;
}
