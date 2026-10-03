import { useState } from "react";
import { Moon, Sun } from "lucide-react";
import { Theme } from "../../types/theme";
import { Toggle } from "../primitives/Toggle";
import { SettingsRow } from "./SettingsRow";

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
    <SettingsRow
      icon={isLight ? <Sun size={16} /> : <Moon size={16} />}
      title="Light mode"
      description="Use a light color scheme instead of dark"
      control={
        <Toggle checked={isLight} onChange={toggleTheme} ariaLabel="Toggle light mode" />
      }
    />
  );
}
