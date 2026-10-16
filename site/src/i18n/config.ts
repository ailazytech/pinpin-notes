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

/** Each language lives under its own path: / (English), /zh-tw/, /zh-cn/, /ja/, /ko/, /ar/. */
export const LOCALE_SLUG: Record<Locale, string> = {
  en: "",
  "zh-TW": "zh-tw",
  "zh-CN": "zh-cn",
  ja: "ja",
  ko: "ko",
  ar: "ar",
};

/** Short forms people type; the server redirects them to the real path. */
export const SLUG_ALIASES: Record<string, Locale> = { zh: "zh-TW", tw: "zh-TW", "zh-hant": "zh-TW", cn: "zh-CN", "zh-hans": "zh-CN", en: "en" };

/** The language a path's first segment names (real slug or alias), if any. */
export function localeFromSlug(segment: string | undefined): Locale | null {
  const s = (segment ?? "").toLowerCase();
  if (!s) return null;
  const real = (Object.keys(LOCALE_SLUG) as Locale[]).find((l) => LOCALE_SLUG[l] === s);
  return real ?? SLUG_ALIASES[s] ?? null;
}

/** "/" for English, "/ja/" etc. for the rest */
export const localePath = (locale: Locale) => (LOCALE_SLUG[locale] ? `/${LOCALE_SLUG[locale]}/` : "/");
