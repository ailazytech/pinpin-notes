"use client";

import { Download, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/i18n/locale-context";
import { HeroScene } from "../demo/hero-scene";
import { Reveal } from "../ui";
import type { NavigateFn } from "../header";

export function Hero({
  navigate,
  onDownload,
}: {
  navigate: NavigateFn;
  onDownload: () => void;
}) {
  const { dict } = useI18n();

  return (
    <section className="relative overflow-hidden">
      {/* faint dotted paper backdrop */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0"
        style={{
          backgroundImage:
            "radial-gradient(rgb(58 48 42 / 0.055) 1px, transparent 1.4px)",
          backgroundSize: "26px 26px",
          maskImage:
            "linear-gradient(to bottom, black 0%, black 62%, transparent 100%)",
          WebkitMaskImage:
            "linear-gradient(to bottom, black 0%, black 62%, transparent 100%)",
        }}
      />

      <div className="relative mx-auto grid w-full max-w-6xl items-center gap-12 px-4 pb-16 pt-12 sm:px-6 sm:pt-16 lg:grid-cols-[1.02fr_1fr] lg:gap-10 lg:pb-24">
        {/* copy */}
        <div className="flex flex-col items-start gap-6">
          <Reveal>
            <span className="inline-flex items-center gap-2 rounded-full border border-line bg-paper px-3.5 py-1.5 text-[0.72rem] font-bold uppercase tracking-[0.14em] text-ink-soft shadow-note">
              <Sparkles className="size-3.5 text-pin" />
              {dict.hero.badge}
            </span>
          </Reveal>

          <Reveal delay={0.08}>
            <h1 className="text-balance font-display text-[2.6rem] font-semibold leading-[1.06] tracking-tight text-ink sm:text-6xl">
              {dict.hero.titleA}{" "}
              <span className="relative inline-block">
                <span className="relative z-10 italic">{dict.hero.titleB}</span>
                <span
                  aria-hidden
                  className="absolute inset-x-[-4px] bottom-[0.06em] z-0 h-[0.34em] rounded-[3px] bg-note-butter/70"
                />
              </span>
            </h1>
          </Reveal>

          <Reveal delay={0.12}>
            <p className="-mt-2 font-hand text-[1.6rem] leading-tight text-pin sm:text-[1.85rem]">{dict.hero.tagline}</p>
          </Reveal>

          <Reveal delay={0.16}>
            <p className="text-pretty max-w-xl text-[1rem] leading-relaxed text-ink-soft sm:text-[1.08rem]">
              {dict.hero.sub}
            </p>
          </Reveal>

          <Reveal delay={0.24} className="flex flex-col items-start gap-3">
            <div className="flex flex-wrap items-center gap-3">
              <Button
                size="lg"
                onClick={onDownload}
                className="h-12 rounded-full bg-ink px-7 text-[0.95rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift active:translate-y-0"
              >
                <Download className="size-[18px]" />
                <span className="ml-1.5">{dict.hero.download}</span>
              </Button>
              <Button
                size="lg"
                variant="ghost"
                onClick={() => navigate("home", "features")}
                className="h-12 rounded-full px-6 text-[0.95rem] font-bold text-ink-soft hover:bg-paper-deep hover:text-ink"
              >
                {dict.hero.explore}
              </Button>
            </div>
            <p className="text-[0.78rem] text-ink-mute">
              {dict.hero.downloadNote} · {dict.common.comingSoon}
            </p>
          </Reveal>

          <Reveal delay={0.32}>
            <dl className="mt-2 grid w-full max-w-lg grid-cols-2 gap-x-8 gap-y-4 sm:grid-cols-4 sm:gap-x-5">
              {dict.hero.stats.map((s) => (
                <div key={s.label} className="flex flex-col gap-0.5">
                  <dt className="order-2 text-[0.7rem] font-semibold uppercase tracking-[0.12em] text-ink-mute">
                    {s.label}
                  </dt>
                  <dd className="order-1 font-display text-2xl font-semibold tracking-tight text-ink">
                    {s.value}
                  </dd>
                </div>
              ))}
            </dl>
          </Reveal>
        </div>

        {/* scene */}
        <Reveal delay={0.2} y={36}>
          <HeroScene />
        </Reveal>
      </div>
    </section>
  );
}
