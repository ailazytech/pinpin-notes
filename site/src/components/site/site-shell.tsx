"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { toast } from "sonner";
import { I18nProvider, useI18n } from "@/i18n/locale-context";
import { Curtain, type CurtainPhase } from "./curtain";
import { Header } from "./header";
import { Footer } from "./footer";
import { HomeView } from "./views/home-view";
import { DonateView } from "./views/donate-view";
import { QaView } from "./views/qa-view";
import { CustomView } from "./views/custom-view";
import { OrderView } from "./views/order-view";
import type { View } from "./shell-types";
import { siteConfig } from "@/lib/site-config";
import type { Locale } from "@/i18n/config";

export function viewFromHash(hash: string): View {
  if (hash.startsWith("#/donate")) return "donate";
  if (hash.startsWith("#/qa")) return "qa";
  if (hash.startsWith("#/custom")) return "custom";
  if (hash.startsWith("#/order")) return "order";
  return "home";
}

function viewToHash(view: View): string {
  if (view === "donate") return "#/donate";
  if (view === "qa") return "#/qa";
  if (view === "custom") return "#/custom";
  if (view === "order") return "#/order";
  return "#/";
}

const INTRO_HOLD_MS = 1500;
const COVER_MS = 400;
const HOLD_MS = 560;
const REVEAL_MS = 700;

export function SiteShell({ initialLocale }: { initialLocale: Locale }) {
  return (
    <I18nProvider initialLocale={initialLocale}>
      <ShellInner />
    </I18nProvider>
  );
}

function ShellInner() {
  const { dict } = useI18n();
  const [view, setView] = useState<View>(() =>
    typeof window === "undefined" ? "home" : viewFromHash(window.location.hash)
  );
  const [phase, setPhase] = useState<CurtainPhase>("hold");
  const [intro, setIntro] = useState(true);
  const [occasion, setOccasion] = useState(0);
  const pendingScroll = useRef<string | null>(null);
  const timers = useRef<number[]>([]);

  const clearTimers = () => {
    timers.current.forEach((t) => window.clearTimeout(t));
    timers.current = [];
  };
  const later = (fn: () => void, ms: number) => {
    timers.current.push(window.setTimeout(fn, ms));
  };

  /* ---------- intro ---------- */
  useEffect(() => {
    later(() => setPhase("reveal"), INTRO_HOLD_MS);
    later(() => {
      setPhase("hidden");
      setIntro(false);
    }, INTRO_HOLD_MS + REVEAL_MS);
    return clearTimers;
  }, []);

  /* ---------- hash routing ---------- */
  useEffect(() => {
    const onHash = () => {
      const next = viewFromHash(window.location.hash);
      setView((current) => {
        if (next === current) return current;
        runTransition(next);
        return next;
      });
    };
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  const runTransition = useCallback((next: View) => {
    clearTimers();
    setOccasion((o) => o + 1);
    setPhase("cover");
    later(() => {
      setView(next);
      window.scrollTo({ top: 0, behavior: "auto" });
      setPhase("hold");
    }, COVER_MS);
    later(() => setPhase("reveal"), COVER_MS + HOLD_MS);
    later(() => setPhase("hidden"), COVER_MS + HOLD_MS + REVEAL_MS);
  }, []);

  const navigate = useCallback(
    (target: View, scrollId?: string) => {
      const current = viewFromHash(window.location.hash);
      const targetHash = viewToHash(target);
      const currentHash = window.location.hash || "#/";

      if (target === current) {
        if (scrollId) {
          document.getElementById(scrollId)?.scrollIntoView({ behavior: "smooth" });
        } else {
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
        return;
      }
      if (scrollId) pendingScroll.current = scrollId;
      if (currentHash === targetHash) {
        runTransition(target);
        setView(target);
      } else {
        window.location.hash = targetHash;
      }
    },
    [runTransition]
  );

  /* ---------- pending scroll after view swap ---------- */
  useEffect(() => {
    if (!pendingScroll.current) return;
    const id = pendingScroll.current;
    pendingScroll.current = null;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        document.getElementById(id)?.scrollIntoView({ behavior: "auto" });
      });
    });
  }, [view]);

  const onDownload = useCallback(() => {
    if (siteConfig.downloadUrl) {
      window.open(siteConfig.downloadUrl, "_blank");
      return;
    }
    toast(dict.hero.downloadPending, {
      description: `${dict.common.linkPending} · GitHub Releases`,
    });
  }, [dict]);

  return (
    <div className="flex min-h-screen flex-col bg-background">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-[90] focus:rounded-full focus:bg-ink focus:px-4 focus:py-2 focus:text-sm focus:text-paper"
      >
        {dict.nav.gotoSection}: main
      </a>

      <Header view={view} navigate={navigate} onDownload={onDownload} />

      <main id="main" className="flex-1">
        <motion.div
          key={view}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1, transition: { duration: 0.45, delay: 0.12 } }}
        >
          {view === "home" && <HomeView navigate={navigate} onDownload={onDownload} />}
          {view === "donate" && <DonateView />}
          {view === "qa" && <QaView />}
          {view === "custom" && <CustomView />}
          {view === "order" && <OrderView />}
        </motion.div>
      </main>

      <Footer navigate={navigate} />

      <Curtain phase={phase} intro={intro} occasion={occasion} />
    </div>
  );
}
