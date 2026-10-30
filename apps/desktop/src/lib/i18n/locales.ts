export type Locale = "pt-BR" | "en-US";

export const LOCALES: { value: Locale; label: string }[] = [
  { value: "pt-BR", label: "Português (Brasil)" },
  { value: "en-US", label: "English (US)" },
];

export const DEFAULT_LOCALE: Locale = "pt-BR";
