"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Menu, Download, X } from "lucide-react";
import { Sheet, SheetContent, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { LogoMark, LogoWord } from "./logo";
import { LanguageSwitcher } from "./language-switcher";
import { ThemeToggle } from "./theme-toggle";
import { GithubLink } from "./github-link";
import { useI18n } from "@/i18n/locale-context";
import type { View } from "./shell-types";
import { cn } from "@/lib/utils";

export type NavigateFn = (view: View, scrollId?: string) => void;

export function Header({
  view,
  navigate,
  onDownload,
}: {
  view: View;
  navigate: NavigateFn;
  onDownload: () => void;
}) {
  const { dict } = useI18n();
  const [menuOpen, setMenuOpen] = useState(false);

  const links: { key: string; label: string; view: View; scrollId?: string }[] = [
    { key: "home", label: dict.nav.home, view: "home" },
    { key: "features", label: dict.nav.features, view: "home", scrollId: "features" },
    { key: "custom", label: dict.nav.custom, view: "custom" },
    { key: "order", label: dict.nav.order, view: "order" },
    { key: "qa", label: dict.nav.qa, view: "qa" },
    { key: "donate", label: dict.nav.donate, view: "donate" },
  ];

  /* on the home page the features link lights up once its section reaches the header */
  const [onFeatures, setOnFeatures] = useState(false);
  useEffect(() => {
    if (view !== "home") return;
    let frame = 0;
    const check = () => {
      frame = 0;
      const el = document.getElementById("features");
      setOnFeatures(!!el && el.getBoundingClientRect().top < 140);
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(check);
    };
    check();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      window.removeEventListener("scroll", onScroll);
      cancelAnimationFrame(frame);
    };
  }, [view]);

  const activeKey = view === "home" ? (onFeatures ? "features" : "home") : view;
  const isActive = (l: (typeof links)[number]) => l.key === activeKey;

  const go = (l: (typeof links)[number]) => {
    /* move the underline right away; the scroll position catches up */
    if (l.view === "home" && view === "home") setOnFeatures(l.key === "features");
    navigate(l.view, l.scrollId);
  };

  return (
    <header className="sticky top-0 z-40 border-b border-line/70 bg-background/85 backdrop-blur-md">
      <div className="mx-auto flex h-16 w-full max-w-6xl items-center justify-between gap-3 px-4 sm:px-6">
        {/* logo lockup */}
        <button
          type="button"
          onClick={() => navigate("home")}
          className="group flex items-center gap-2.5 outline-none"
          aria-label={dict.nav.home}
        >
          <span className="transition-transform duration-500 group-hover:-rotate-6">
            <LogoMark size={34} />
          </span>
          <LogoWord className="hidden sm:inline-block" />
        </button>

        {/* desktop nav */}
        <nav className="hidden items-center gap-1 md:flex" aria-label="Primary">
          {links.map((l) => (
            <button
              key={l.key}
              type="button"
              onClick={() => go(l)}
              aria-current={isActive(l) ? "page" : undefined}
              className={cn(
                "relative rounded-full px-3.5 py-2 text-[0.9rem] font-semibold transition-colors outline-none",
                isActive(l) ? "text-ink" : "text-ink-soft hover:bg-paper-deep/70 hover:text-ink"
              )}
            >
              {l.label}
              {isActive(l) && <InkUnderline key={activeKey} />}
            </button>
          ))}
        </nav>

        {/* actions */}
        <div className="flex items-center gap-1">
          <LanguageSwitcher />
          <ThemeToggle />
          <GithubLink />
          <Button
            size="sm"
            onClick={onDownload}
            className="ml-1 hidden h-9 rounded-full bg-ink px-4 font-semibold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift sm:inline-flex"
          >
            <Download className="size-4" />
            <span className="ml-1.5">{dict.nav.download}</span>
          </Button>

          {/* mobile menu */}
          <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="size-9 rounded-full text-ink-soft hover:bg-paper-deep md:hidden"
                aria-label={dict.nav.openMenu}
              >
                <Menu className="size-5" />
              </Button>
            </SheetTrigger>
            <SheetContent
              side="right"
              className="w-[290px] border-line bg-background p-0"
            >
              <div className="flex h-full flex-col">
                <div className="flex items-center justify-between border-b border-line/70 px-5 py-4">
                  <div className="flex items-center gap-2">
                    <LogoMark size={30} />
                    <LogoWord />
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="size-8 rounded-full text-ink-soft"
                    onClick={() => setMenuOpen(false)}
                    aria-label={dict.nav.closeMenu}
                  >
                    <X className="size-4" />
                  </Button>
                </div>
                <nav className="flex flex-col gap-1 p-4" aria-label="Mobile">
                  {links.map((l) => (
                    <button
                      key={l.key}
                      type="button"
                      onClick={() => {
                        setMenuOpen(false);
                        go(l);
                      }}
                      className={cn(
                        "rounded-xl px-4 py-3 text-start text-base font-semibold transition-colors",
                        isActive(l)
                          ? "bg-paper-deep text-ink"
                          : "text-ink-soft hover:bg-paper-deep/60 hover:text-ink"
                      )}
                    >
                      {l.label}
                    </button>
                  ))}
                  <Button
                    onClick={() => {
                      setMenuOpen(false);
                      onDownload();
                    }}
                    className="mt-3 h-11 rounded-full bg-ink font-semibold text-paper shadow-note"
                  >
                    <Download className="size-4" />
                    <span className="ml-1.5">{dict.nav.download}</span>
                  </Button>
                </nav>
                <div className="mt-auto flex items-center justify-between border-t border-line/70 px-5 py-4">
                  <SheetTitle className="sr-only">{dict.nav.openMenu}</SheetTitle>
                  <LanguageSwitcher />
                  <ThemeToggle />
                </div>
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}

/** Red-pen underline: slides to the active link, then draws itself again. */
function InkUnderline() {
  return (
    <motion.span
      layoutId="nav-ink"
      aria-hidden
      className="pointer-events-none absolute inset-x-3 bottom-[1px] h-[7px] text-[#e24234] dark:text-[#ef6a5c]"
      transition={{ type: "spring", stiffness: 420, damping: 34 }}
    >
      <svg viewBox="0 0 44 7" preserveAspectRatio="none" className="size-full overflow-visible">
        <motion.path
          d="M2 4.5c7-2.6 14-3 20-2.2 6 .8 12 2.4 20 1.4"
          fill="none"
          stroke="currentColor"
          strokeWidth={2.4}
          strokeLinecap="round"
          vectorEffect="non-scaling-stroke"
          initial={{ pathLength: 0, opacity: 0.4 }}
          animate={{ pathLength: 1, opacity: 0.88 }}
          transition={{ duration: 0.42, delay: 0.12, ease: [0.4, 0, 0.2, 1] }}
        />
      </svg>
    </motion.span>
  );
}
