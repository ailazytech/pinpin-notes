import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "next-themes";
import { MotionGlobalConfig } from "framer-motion";
import "@fontsource-variable/fraunces/full.css";
import "@fontsource-variable/nunito";
import "@fontsource-variable/caveat";
import "./styles/globals.css";
import { Toaster } from "@/components/ui/sonner";
import { SiteShell } from "@/components/site/site-shell";
import { STORAGE_KEY, isLocale, matchBrowserLocale, type Locale } from "@/i18n/config";

/** Language: ?lang= → what the visitor picked last time → the browser's language → English. */
function initialLocale(): Locale {
  const fromUrl = new URLSearchParams(window.location.search).get("lang");
  if (isLocale(fromUrl)) return fromUrl;
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (isLocale(saved)) return saved;
  } catch {
    /* storage blocked */
  }
  for (const tag of navigator.languages ?? [navigator.language]) {
    const matched = matchBrowserLocale(tag);
    if (matched) return matched;
  }
  return "en";
}

/* ?static: jump framer animations to their end state (headless screenshots, OG images) */
if (new URLSearchParams(window.location.search).has("static")) MotionGlobalConfig.skipAnimations = true;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider attribute="class" defaultTheme="light" enableSystem={false} disableTransitionOnChange>
      <SiteShell initialLocale={initialLocale()} />
      <Toaster position="bottom-center" />
    </ThemeProvider>
  </StrictMode>,
);
