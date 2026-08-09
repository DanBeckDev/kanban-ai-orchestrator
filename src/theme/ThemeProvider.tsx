import {
  createContext,
  type ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

import {
  applyTheme,
  readThemePreference,
  saveThemePreference,
  type Theme,
} from "./theme-preference";

type ThemeContextValue = Readonly<{
  theme: Theme;
  setTheme: (theme: Theme) => void;
}>;

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

type ThemeProviderProps = Readonly<{
  children: ReactNode;
}>;

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(() =>
    readThemePreference(window.localStorage),
  );

  useEffect(() => {
    applyTheme(document, theme);
    saveThemePreference(window.localStorage, theme);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error("useTheme must be used within ThemeProvider.");
  }

  return context;
}
