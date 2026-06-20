import { atom } from "nanostores";
import type { AccentColor, AppSettings, ThemeMode } from "$lib/contracts/settings";

export const themeStore = atom<ThemeMode>("system");
export const accentStore = atom<AccentColor>("#0064ff");

export function applyTheme(theme: ThemeMode): void {
  themeStore.set(theme);
  const root = document.documentElement;
  root.dataset.theme = theme;
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.toggle("dark", prefersDark);
  }
}

export function applyAccentColor(accentColor: AccentColor): void {
  accentStore.set(accentColor);
  const root = document.documentElement;
  root.style.setProperty("--accent", accentColor);
  root.style.setProperty(
    "--accent-soft",
    `color-mix(in srgb, ${accentColor} 13%, transparent)`,
  );
  root.style.setProperty(
    "--accent-strong",
    `color-mix(in srgb, ${accentColor} 22%, transparent)`,
  );
}

export function applyAppearance(settings: Pick<AppSettings, "theme" | "accentColor">): void {
  applyTheme(settings.theme);
  applyAccentColor(settings.accentColor);
}
