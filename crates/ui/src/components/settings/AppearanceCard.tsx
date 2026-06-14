import { useState } from "react";
import { Theme } from "../../types/theme";
import { Toggle } from "../primitives/Toggle";

const THEME_STORAGE_KEY = "byocvpn-theme";

function readActiveTheme(): Theme {
  return document.documentElement.dataset.theme === Theme.LIGHT
    ? Theme.LIGHT
    : Theme.DARK;
}

function applyTheme(theme: Theme) {
  localStorage.setItem(THEME_STORAGE_KEY, theme);
  document.documentElement.dataset.theme = theme;
}

export function AppearanceCard() {
  const [theme, setTheme] = useState<Theme>(readActiveTheme);
  const isLight = theme === Theme.LIGHT;

  function toggleTheme() {
    const nextTheme = isLight ? Theme.DARK : Theme.LIGHT;
    applyTheme(nextTheme);
    setTheme(nextTheme);
  }

  return (
    <div className="py-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 min-w-0">
          <div className="w-12 h-12 rounded-xl bg-blue-900/50 flex items-center justify-center flex-shrink-0">
            {isLight ? <SunIcon /> : <MoonIcon />}
          </div>
          <div className="min-w-0">
            <h3 className="font-semibold text-primary">Light mode</h3>
            <p className="text-sm text-gray-400 mt-0.5">
              Use a light color scheme instead of dark
            </p>
          </div>
        </div>

        <Toggle checked={isLight} onChange={toggleTheme} ariaLabel="Toggle light mode" />
      </div>
    </div>
  );
}

function MoonIcon() {
  return (
    <svg className="w-5 h-5 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"
      />
    </svg>
  );
}

function SunIcon() {
  return (
    <svg className="w-5 h-5 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M12 3v2m0 14v2m9-9h-2M5 12H3m15.36 6.36l-1.42-1.42M7.05 7.05L5.64 5.64m12.72 0l-1.42 1.42M7.05 16.95l-1.41 1.41M16 12a4 4 0 11-8 0 4 4 0 018 0z"
      />
    </svg>
  );
}
