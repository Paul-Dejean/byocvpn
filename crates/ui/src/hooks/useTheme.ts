import { useCallback, useEffect, useState } from "react";
import { Theme } from "../types/theme";

export const THEME_STORAGE_KEY = "byocvpn-theme";

const LIGHT_MEDIA_QUERY = "(prefers-color-scheme: light)";

function readStoredTheme(): Theme | null {
  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  return stored === Theme.LIGHT || stored === Theme.DARK ? stored : null;
}

function readSystemTheme(): Theme {
  return window.matchMedia(LIGHT_MEDIA_QUERY).matches ? Theme.LIGHT : Theme.DARK;
}

function readInitialTheme(): Theme {
  return readStoredTheme() ?? readSystemTheme();
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(readInitialTheme);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    const mediaQuery = window.matchMedia(LIGHT_MEDIA_QUERY);
    const handleSystemChange = (event: MediaQueryListEvent) => {
      if (readStoredTheme() === null) {
        setThemeState(event.matches ? Theme.LIGHT : Theme.DARK);
      }
    };
    mediaQuery.addEventListener("change", handleSystemChange);
    return () => mediaQuery.removeEventListener("change", handleSystemChange);
  }, []);

  const setTheme = useCallback((next: Theme) => {
    localStorage.setItem(THEME_STORAGE_KEY, next);
    setThemeState(next);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((current) => {
      const next = current === Theme.DARK ? Theme.LIGHT : Theme.DARK;
      localStorage.setItem(THEME_STORAGE_KEY, next);
      return next;
    });
  }, []);

  return { theme, setTheme, toggleTheme };
}
