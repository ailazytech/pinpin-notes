"use client";

import { useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, MessageCircleQuestion, Search } from "lucide-react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, NotePaper, PinHead } from "../ui";
import { cn } from "@/lib/utils";

type Cat = "general" | "notes" | "data" | "timer" | "langs";

const CATS: Cat[] = ["general", "notes", "data", "timer", "langs"];

export function QaView() {
  const { dict } = useI18n();
  const q = dict.qa;
  const [query, setQuery] = useState("");
  const [cat, setCat] = useState<Cat | "all">("all");
  const [openItems, setOpenItems] = useState<string[]>([]);

  const items = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return q.items
      .map((item, idx) => ({ ...item, idx }))
      .filter((item) => (cat === "all" ? true : item.cat === cat))
      .filter((item) =>
        needle
          ? item.q.toLowerCase().includes(needle) ||
            item.a.toLowerCase().includes(needle)
          : true
      );
  }, [q.items, query, cat]);

  const countFor = (c: Cat | "all") =>
    c === "all" ? q.items.length : q.items.filter((i) => i.cat === c).length;

  return (
    <div className="mx-auto w-full max-w-3xl px-4 py-12 sm:px-6 sm:py-16">
      {/* heading */}
      <Reveal>
        <div className="flex flex-col items-center gap-4 text-center">
          <span className="inline-flex items-center gap-2 rounded-full border border-line bg-paper px-3.5 py-1.5 text-[0.72rem] font-bold uppercase tracking-[0.14em] text-ink-soft shadow-note">
            <MessageCircleQuestion className="size-3.5 text-pin" />
            {q.eyebrow}
          </span>
          <h1 className="text-balance font-display text-4xl font-semibold tracking-tight text-ink sm:text-5xl">
            {q.title}
          </h1>
          <p className="text-pretty max-w-xl text-[0.95rem] leading-relaxed text-ink-soft">
            {q.sub}
          </p>
        </div>
      </Reveal>

      {/* search */}
      <Reveal delay={0.08}>
        <div className="relative mt-8">
          <Search className="pointer-events-none absolute start-4 top-1/2 size-[18px] -translate-y-1/2 text-ink-mute" />
          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={q.searchPlaceholder}
            aria-label={q.searchPlaceholder}
            className="h-12 w-full rounded-full border border-line bg-card ps-11 pe-4 text-[0.95rem] text-ink shadow-note outline-none transition-colors placeholder:text-ink-mute/70 focus:border-pin/50"
          />
        </div>
      </Reveal>

      {/* categories */}
      <Reveal delay={0.12}>
        <div className="mt-4 flex flex-wrap justify-center gap-2">
          {(["all", ...CATS] as const).map((c) => (
            <button
              key={c}
              type="button"
              onClick={() => setCat(c)}
              aria-pressed={cat === c}
              className={cn(
                "inline-flex items-center gap-1.5 rounded-full border px-3.5 py-1.5 text-[0.78rem] font-semibold transition-all",
                cat === c
                  ? "border-pin/50 bg-pin/10 text-pin"
                  : "border-line bg-paper text-ink-soft hover:text-ink"
              )}
            >
              {q.categories[c]}
              <span className="tabular text-[0.62rem] opacity-60">{countFor(c)}</span>
            </button>
          ))}
        </div>
      </Reveal>

      {/* list */}
      <Reveal delay={0.16}>
        <div className="mt-8">
          {items.length === 0 ? (
            <NotePaper color="oat" className="relative mx-auto mt-6 max-w-sm -rotate-1 px-6 py-5">
              <PinHead className="left-1/2 top-[-9px]" />
              <p className="text-center font-hand text-xl text-ink/80">{q.noResults}</p>
            </NotePaper>
          ) : (
            <Accordion
              type="multiple"
              className="flex flex-col gap-3"
              value={
                openItems.length > 0
                  ? openItems
                  : items.length === 1
                    ? [`item-${items[0].idx}`]
                    : []
              }
              onValueChange={setOpenItems}
            >
              {items.map((item) => (
                <AccordionItem
                  key={item.idx}
                  value={`item-${item.idx}`}
                  className="paper-grain overflow-hidden rounded-xl border border-line bg-card shadow-note transition-shadow data-[state=open]:shadow-lift"
                >
                  <AccordionTrigger className="px-5 py-4 text-start hover:no-underline [&>svg]:hidden">
                    <span className="flex items-start gap-3">
                      <span className="mt-[3px] inline-block size-[8px] shrink-0 rounded-full border-[1.6px] border-pin/70 bg-pin/15" aria-hidden />
                      <span className="text-[0.95rem] font-bold leading-snug text-ink">
                        {item.q}
                      </span>
                    </span>
                    <AnimatePresence initial={false}>
                      <motion.span
                        className="grid size-7 shrink-0 place-items-center rounded-full bg-paper-deep text-ink-soft"
                        initial={false}
                      >
                        <ChevronDown className="size-4 transition-transform duration-300 [[data-state=open]>&]:rotate-180" />
                      </motion.span>
                    </AnimatePresence>
                  </AccordionTrigger>
                  <AccordionContent className="px-5 pb-5 ps-[38px]">
                    <p className="text-pretty text-[0.86rem] leading-relaxed text-ink-soft">
                      {item.a}
                    </p>
                  </AccordionContent>
                </AccordionItem>
              ))}
            </Accordion>
          )}
        </div>
      </Reveal>
    </div>
  );
}
