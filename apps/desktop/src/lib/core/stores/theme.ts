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

export function applyInterfaceFont(interfaceFont: string): void {
  const root = document.documentElement;
  const font = {
    system: '"Segoe UI Variable", "Segoe UI", system-ui, sans-serif',
    inter: '"Inter", "Segoe UI", system-ui, sans-serif',
    roboto: '"Roboto", "Segoe UI", system-ui, sans-serif',
    "public-sans": '"Public Sans", "Segoe UI", system-ui, sans-serif',
    "jetbrains-mono": '"JetBrains Mono", "Cascadia Mono", monospace',
  }[interfaceFont] ?? '"Segoe UI Variable", "Segoe UI", system-ui, sans-serif';
  root.style.setProperty("--font-ui", font);
}

export function applyAppearance(
  settings: Pick<AppSettings, "theme" | "accentColor"> & Partial<Pick<AppSettings, "interfaceFont">>,
): void {
  applyTheme(settings.theme);
  applyAccentColor(settings.accentColor);
  applyInterfaceFont(settings.interfaceFont ?? "system");
}
