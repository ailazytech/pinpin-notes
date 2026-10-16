"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { LOCALE_META, STORAGE_KEY, isLocale, localeFromSlug, localePath, type Locale } from "./config";
import { en, type Dictionary } from "./locales/en";
import { zhTW } from "./locales/zh-TW";
import { zhCN } from "./locales/zh-CN";
import { ja } from "./locales/ja";
import { ko } from "./locales/ko";
import { ar } from "./locales/ar";

const DICTS: Record<Locale, Dictionary> = {
  en,
  "zh-TW": zhTW,
  "zh-CN": zhCN,
  ja,
  ko,
  ar,
};

type I18nContextValue = {
  locale: Locale;
  dict: Dictionary;
  dir: "ltr" | "rtl";
  setLocale: (locale: Locale) => void;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({
  initialLocale,
  children,
}: {
  initialLocale: Locale;
  children: ReactNode;
}) {
  const [locale, setLocaleState] = useState<Locale>(initialLocale);

  const setLocale = useCallback((next: Locale) => {
    if (!isLocale(next)) return;
    setLocaleState(next);
    try {
      window.localStorage.setItem(STORAGE_KEY, next);
      // Cookie so the server can render the right locale on the next visit.
      document.cookie = `${STORAGE_KEY}=${next};path=/;max-age=31536000;samesite=lax`;
    } catch {
      /* storage unavailable — keep going */
    }
    // Keep the address on the language's own path (/ja/…), so a copied link opens in this language.
    // Production only: the Cloudflare middleware maps /ja/… back onto the real files; the local dev server doesn't.
    const { pathname, hash } = window.location;
    const first = pathname.split("/")[1] ?? "";
    if (import.meta.env.PROD && (pathname === "/" || pathname === "/index.html" || localeFromSlug(first))) {
      window.history.replaceState(window.history.state, "", localePath(next) + hash);
    }
  }, []);

  useEffect(() => {
    const meta = LOCALE_META[locale];
    document.documentElement.lang = locale;
    document.documentElement.dir = meta.dir;
  }, [locale]);

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      dict: DICTS[locale],
      dir: LOCALE_META[locale].dir,
      setLocale,
    }),
    [locale, setLocale]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used inside <I18nProvider>");
  return ctx;
}
