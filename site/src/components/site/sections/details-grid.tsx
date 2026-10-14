"use client";

import { Magnet, Star, ArrowLeftRight, Undo2, MousePointerClick, Languages } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading } from "../ui";
import type { NavigateFn } from "../header";

const ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  "detail-snap": Magnet,
  "detail-starred": Star,
  "detail-export": ArrowLeftRight,
  "detail-undo": Undo2,
  "detail-context": MousePointerClick,
  "detail-langs": Languages,
};

export function DetailsGrid({ navigate }: { navigate: NavigateFn }) {
  const { dict } = useI18n();

  return (
    <section className="bg-paper/50 py-16 sm:py-24">
      <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
        <Reveal>
          <SectionHeading
            eyebrow={dict.details.eyebrow}
            title={dict.details.title}
            sub={dict.details.sub}
          />
        </Reveal>

        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {dict.details.cards.map((card, i) => {
            const Icon = ICONS[card.id] ?? Star;
            return (
              <Reveal key={card.id} delay={Math.min(i * 0.06, 0.3)} y={20}>
                <button
                  type="button"
                  onClick={() => navigate("home", card.id)}
                  className="paper-grain group flex h-full w-full flex-col items-start gap-3 rounded-xl border border-line bg-card p-5 text-start shadow-note transition-all duration-300 hover:-translate-y-1 hover:shadow-lift sm:p-6"
                >
                  <span className="inline-flex size-10 items-center justify-center rounded-lg bg-paper-deep text-ink-soft transition-colors group-hover:bg-note-butter/60 group-hover:text-ink">
                    <Icon className="size-5" />
                  </span>
                  <span className="font-display text-[1.05rem] font-semibold text-ink">
                    {card.title}
                  </span>
                  <span className="text-pretty text-[0.82rem] leading-relaxed text-ink-soft">
                    {card.desc}
                  </span>
                </button>
              </Reveal>
            );
          })}
        </div>
      </div>
    </section>
  );
}
