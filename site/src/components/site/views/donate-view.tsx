"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { Coffee, CupSoda, Heart, Megaphone, Star, Sticker, Sun } from "lucide-react";
import { toast } from "sonner";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, StickerImg, NotePaper, WashiTape, PinHead } from "../ui";
import { siteConfig } from "@/lib/site-config";
import { cn } from "@/lib/utils";

const TIER_ICONS = [Sticker, CupSoda, Coffee, Sun];

const kofi = siteConfig.donate.kofi;

export function DonateView() {
  const { dict } = useI18n();
  const d = dict.donate;
  const [custom, setCustom] = useState("");

  // Every button (tiers, custom amount, the method card) goes to Ko-fi.
  const thank = () => {
    if (kofi) {
      window.open(kofi, "_blank");
      return;
    }
    toast(dict.common.linkPending, { description: "Ko-fi" });
  };

  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-12 sm:px-6 sm:py-16">
      {/* hero */}
      <div className="grid items-center gap-8 lg:grid-cols-[1.2fr_0.8fr]">
        <Reveal>
          <div className="flex flex-col items-start gap-5">
            <span className="inline-flex items-center gap-2 rounded-full border border-line bg-paper px-3.5 py-1.5 text-[0.72rem] font-bold uppercase tracking-[0.14em] text-ink-soft shadow-note">
              <Heart className="size-3.5 fill-pin text-pin" />
              {d.eyebrow}
            </span>
            <h1 className="text-balance font-display text-4xl font-semibold tracking-tight text-ink sm:text-5xl">
              {d.title}
            </h1>
            <p className="text-pretty max-w-xl text-[0.98rem] leading-relaxed text-ink-soft">
              {d.sub}
            </p>
          </div>
        </Reveal>
        <Reveal delay={0.15}>
          <motion.div
            animate={{ y: [0, -8, 0], rotate: [-2, 0, -2] }}
            transition={{ duration: 6, repeat: Infinity, ease: "easeInOut" }}
            className="relative mx-auto w-fit"
          >
            <StickerImg name="boba" size={170} alt="a cup of bubble tea" />
            <span className="absolute -right-3 -top-2 rotate-12">
              <StickerImg name="strawberry" size={58} alt="strawberry" />
            </span>
            <span className="absolute -bottom-2 -left-4 -rotate-12">
              <StickerImg name="donut" size={64} alt="donut" />
            </span>
          </motion.div>
        </Reveal>
      </div>

      {/* tiers */}
      <div className="mt-16">
        <Reveal>
          <SectionHeading title={d.tiersTitle} align="start" className="mb-8" />
        </Reveal>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {d.tiers.map((tier, i) => {
            const Icon = TIER_ICONS[i] ?? CupSoda;
            return (
              <Reveal key={tier.name} delay={i * 0.07} y={22}>
                <motion.button
                  type="button"
                  whileHover={{ y: -6, rotate: i % 2 === 0 ? -1 : 1 }}
                  onClick={thank}
                  className={cn(
                    "paper-grain relative flex h-full w-full flex-col items-start gap-3 rounded-xl border p-5 text-start shadow-note transition-shadow hover:shadow-lift sm:p-6",
                    i === 1 ? "border-pin/40 bg-pin/[0.06]" : "border-line bg-card"
                  )}
                >
                  {i === 1 && (
                    <span className="absolute -top-2.5 end-4 rotate-2 rounded-full bg-pin px-2.5 py-0.5 text-[0.6rem] font-bold uppercase tracking-wider text-white shadow-note">
                      {d.hearts}
                    </span>
                  )}
                  <span className="grid size-11 place-items-center rounded-xl bg-paper-deep text-ink-soft">
                    <Icon className="size-5" />
                  </span>
                  <span className="font-display text-2xl font-semibold tracking-tight text-ink">
                    {tier.amount}
                  </span>
                  <span className="text-[0.9rem] font-bold text-ink">{tier.name}</span>
                  <span className="text-[0.78rem] leading-relaxed text-ink-soft">
                    {tier.desc}
                  </span>
                </motion.button>
              </Reveal>
            );
          })}
        </div>

        {/* custom amount */}
        <Reveal delay={0.1}>
          <div className="mt-6 flex flex-col items-start gap-3 rounded-xl border border-dashed border-ink-mute/40 bg-paper/60 p-5 sm:flex-row sm:items-center sm:justify-between">
            <label htmlFor="custom-amount" className="text-[0.88rem] font-semibold text-ink-soft">
              {d.customLabel}
            </label>
            <div className="flex items-center gap-2">
              <span className="text-[0.8rem] font-bold text-ink-mute">{d.customUnit}</span>
              <input
                id="custom-amount"
                inputMode="decimal"
                value={custom}
                onChange={(e) => setCustom(e.target.value.replace(/[^\d.]/g, ""))}
                placeholder={d.customPlaceholder}
                className="h-10 w-32 rounded-full border border-line bg-card px-4 text-center text-[0.9rem] font-semibold text-ink shadow-inner outline-none placeholder:font-normal placeholder:text-ink-mute/60 focus:border-pin/50"
              />
              <button
                type="button"
                onClick={thank}
                className="inline-flex h-10 items-center rounded-full bg-ink px-5 text-[0.82rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
              >
                <Heart className="mr-1.5 size-4 fill-paper" />
                {d.eyebrow}
              </button>
            </div>
          </div>
        </Reveal>
      </div>

      {/* payment methods */}
      <div className="mt-16">
        <Reveal>
          <SectionHeading title={d.methodsTitle} align="start" className="mb-6" />
        </Reveal>
        <Reveal delay={0.06}>
          <div className="flex flex-col gap-3">
            <div className="grid gap-3 sm:max-w-sm">
              {d.methods.map((m) => (
                <button
                  key={m.name}
                  type="button"
                  onClick={thank}
                  className="group flex items-center justify-between rounded-xl border border-line bg-card px-5 py-4 shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
                >
                  <span className="flex items-center gap-3">
                    <span className="grid size-9 place-items-center rounded-full bg-paper-deep text-ink-soft transition-colors group-hover:bg-pin/10 group-hover:text-pin">
                      <Heart className="size-4" />
                    </span>
                    <span className="text-[0.92rem] font-bold text-ink">{m.name}</span>
                  </span>
                  {!kofi && (
                    <span className="rounded-full border border-dashed border-ink-mute/40 px-2.5 py-0.5 text-[0.64rem] font-semibold text-ink-mute">
                      {m.note}
                    </span>
                  )}
                </button>
              ))}
            </div>
            {!kofi && <p className="text-[0.75rem] text-ink-mute">{d.methodsPending}</p>}
          </div>
        </Reveal>
      </div>

      {/* other ways */}
      <div className="mt-16">
        <Reveal>
          <SectionHeading title={d.otherTitle} align="start" className="mb-6" />
        </Reveal>
        <div className="grid gap-4 sm:grid-cols-2">
          {d.others.map((o, i) => (
            <Reveal key={o.title} delay={i * 0.06} y={18}>
              <div className="paper-grain flex h-full items-start gap-4 rounded-xl border border-line bg-card p-5 shadow-note">
                <span className="grid size-10 shrink-0 place-items-center rounded-lg bg-paper-deep text-ink-soft">
                  {i === 0 ? (
                    <Star className="size-5" />
                  ) : i === 1 ? (
                    <Megaphone className="size-5" />
                  ) : (
                    <Heart className="size-5" />
                  )}
                </span>
                <div className="flex flex-col gap-1">
                  <span className="text-[0.92rem] font-bold text-ink">{o.title}</span>
                  <span className="text-[0.8rem] leading-relaxed text-ink-soft">{o.desc}</span>
                </div>
              </div>
            </Reveal>
          ))}
        </div>
      </div>

      {/* thank-you note */}
      <Reveal delay={0.1}>
        <div className="mt-16 flex justify-center">
          <NotePaper color="butter" className="relative max-w-md -rotate-1 px-8 py-7">
            <PinHead className="left-1/2 top-[-9px]" />
            <WashiTape color="var(--note-mist)" pattern={2} className="right-[-14px] top-[10px]" rotate={86} width={60} />
            <p className="text-center font-hand text-2xl leading-snug text-ink/90">
              {d.thanks}
            </p>
          </NotePaper>
        </div>
      </Reveal>
    </div>
  );
}
