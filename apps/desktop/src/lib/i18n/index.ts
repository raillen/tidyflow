import { writable, get } from "svelte/store";
import ptBR from "./pt-BR.json";
import enUS from "./en-US.json";
import type { Locale } from "./locales";

export type { Locale } from "./locales";
export { LOCALES, DEFAULT_LOCALE } from "./locales";

const dictionaries: Record<Locale, Record<string, string>> = {
  "pt-BR": ptBR,
  "en-US": enUS,
};

export const localeStore = writable<Locale>("pt-BR");

export function setLocale(locale: Locale): void {
  localeStore.set(locale);
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("tidyflow-locale", locale);
  }
}

export function loadLocale(): Locale {
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem("tidyflow-locale");
    if (stored === "pt-BR" || stored === "en-US") {
      localeStore.set(stored);
      return stored;
    }
  }
  return "pt-BR";
}

/** BCP-47 tag for `Intl` formatters. */
export function currentLocale(): string {
  return get(localeStore) === "en-US" ? "en-US" : "pt-BR";
}

export function t(key: string, params?: Record<string, string | number>): string {
  const locale = get(localeStore);
  const dict = dictionaries[locale] ?? dictionaries["pt-BR"];
  let value = dict[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, String(v));
    }
  }
  return value;
}