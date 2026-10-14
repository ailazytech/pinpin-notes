"use client";

import {
  StickyNote,
  ListTodo,
  Sticker,
  Scissors,
  PenTool,
  Ghost,
  Type,
  Camera,
  Timer,
  LockKeyhole,
  Star,
  Magnet,
  PanelBottom,
  ArrowLeftRight,
  Languages,
  Leaf,
} from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, WashiTape } from "../ui";
import { cn } from "@/lib/utils";
import type { NavigateFn } from "../header";

const ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  notes: StickyNote,
  todo: ListTodo,
  shop: Sticker,
  custom: Scissors,
  handwriting: PenTool,
  transparent: Ghost,
  text: Type,
  memories: Camera,
  pomodoro: Timer,
  "detail-password": LockKeyhole,
  "detail-starred": Star,
  snap: Magnet,
  "detail-context": PanelBottom,
  "detail-export": ArrowLeftRight,
  langs: Languages,
  power: Leaf,
};

/** a few cards carry a tape or pin so the grid feels hand-arranged */
const DECOR: Record<string, "tape" | "pin"> = {
  notes: "tape",
  shop: "pin",
  memories: "pin",
  langs: "tape",
};

export function FeaturesOverview({ navigate }: { navigate: NavigateFn }) {
  const { dict } = useI18n();

  return (
    <section id="features" className="scroll-mt-20 bg-paper/50 py-16 sm:py-24">
      <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
        <Reveal>
          <SectionHeading
            eyebrow={dict.overview.eyebrow}
            title={dict.overview.title}
            sub={dict.overview.sub}
          />
        </Reveal>

        <div className="grid grid-cols-2 gap-3.5 sm:grid-cols-3 lg:grid-cols-4 sm:gap-4">
          {dict.overview.cards.map((card, i) => {
            const Icon = ICONS[card.id] ?? StickyNote;
            const decor = DECOR[card.id];
            return (
              <Reveal key={card.id} delay={Math.min(i * 0.04, 0.4)} y={18}>
                <button
                  type="button"
                  onClick={() => navigate("home", card.id)}
                  aria-label={dict.nav.gotoSection + ": " + card.title}
                  className={cn(
                    "paper-grain group relative flex h-full w-full flex-col items-start gap-3 rounded-xl border border-line bg-card p-4 text-start shadow-note transition-all duration-300",
                    "hover:-translate-y-1 hover:-rotate-[0.7deg] hover:shadow-lift active:translate-y-0 sm:p-5"
                  )}
                >
                  {decor === "tape" && (
                    <WashiTape
                      color="var(--note-sage)"
                      pattern={2}
                      className="left-[-10px] top-[-8px]"
                      rotate={-4}
                      width={52}
                    />
                  )}
                  {decor === "pin" && (
                    <span className="absolute right-3 top-3">
                      <span className="inline-block size-2.5 rounded-full bg-pin shadow-sm" aria-hidden />
                    </span>
                  )}
                  <span className="inline-flex size-10 items-center justify-center rounded-lg bg-paper-deep text-ink-soft transition-colors group-hover:bg-note-butter/60 group-hover:text-ink">
                    <Icon className="size-5" />
                  </span>
                  <span className="flex flex-col gap-1">
                    <span className="text-[0.92rem] font-bold leading-tight text-ink">
                      {card.title}
                    </span>
                    <span className="text-[0.75rem] leading-snug text-ink-mute">
                      {card.blurb}
                    </span>
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
