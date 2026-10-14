export const LOCALES = ["en", "zh-TW", "zh-CN", "ja", "ko", "ar"] as const;

export type Locale = (typeof LOCALES)[number];

export const LOCALE_META: Record<
  Locale,
  { label: string; english: string; dir: "ltr" | "rtl" }
> = {
  en: { label: "English", english: "English", dir: "ltr" },
  "zh-TW": { label: "繁體中文", english: "Traditional Chinese", dir: "ltr" },
  "zh-CN": { label: "简体中文", english: "Simplified Chinese", dir: "ltr" },
  ja: { label: "日本語", english: "Japanese", dir: "ltr" },
  ko: { label: "한국어", english: "Korean", dir: "ltr" },
  ar: { label: "العربية", english: "Arabic", dir: "rtl" },
};

export function isLocale(value: string | null | undefined): value is Locale {
  return !!value && (LOCALES as readonly string[]).includes(value);
}

/** Map a browser language tag to one of our supported locales. */
export function matchBrowserLocale(tag: string | undefined | null): Locale | null {
  if (!tag) return null;
  const t = tag.toLowerCase();
  if (t.startsWith("zh")) {
    return /tw|hk|hant/i.test(tag) ? "zh-TW" : "zh-CN";
  }
  if (t.startsWith("en")) return "en";
  if (t.startsWith("ja")) return "ja";
  if (t.startsWith("ko")) return "ko";
  if (t.startsWith("ar")) return "ar";
  return null;
}

export const STORAGE_KEY = "pinpin-site-locale";
