export const defaultTheme = "dark";
export const themeStorageKey = "kanban-theme";

export const themes = ["dark", "light"] as const;

export type Theme = (typeof themes)[number];

const themeColors: Record<Theme, string> = {
  dark: "#10111c",
  light: "#f9f9ff",
};

export function readThemePreference(storage: Storage): Theme {
  const savedTheme = storage.getItem(themeStorageKey);
  return isTheme(savedTheme) ? savedTheme : defaultTheme;
}

export function saveThemePreference(storage: Storage, theme: Theme) {
  storage.setItem(themeStorageKey, theme);
}

export function applyTheme(document: Document, theme: Theme) {
  const root = document.documentElement;
  root.classList.toggle("dark", theme === "dark");
  root.classList.toggle("light", theme === "light");
  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute("content", themeColors[theme]);
}

export function initialiseTheme(document: Document, storage: Storage): Theme {
  const theme = readThemePreference(storage);
  applyTheme(document, theme);
  return theme;
}

export function isTheme(value: string | null): value is Theme {
  return themes.some((theme) => theme === value);
}
