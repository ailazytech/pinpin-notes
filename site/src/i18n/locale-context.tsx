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
import { LOCALE_META, STORAGE_KEY, isLocale, type Locale } from "./config";
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
