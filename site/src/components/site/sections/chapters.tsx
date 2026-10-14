"use client";

import { useI18n } from "@/i18n/locale-context";
import { MediaSlot, Reveal } from "../ui";
import { NotesDemo } from "../demo/notes-demo";
import { TodoDemo } from "../demo/todo-demo";
import { ShopDemo } from "../demo/shop-demo";
import { CustomStickerDemo } from "../demo/custom-sticker-demo";
import { HandwritingDemo } from "../demo/handwriting-demo";
import { TransparentDemo } from "../demo/transparent-demo";
import { TextStyleDemo } from "../demo/text-style-demo";
import { MemoriesDemo } from "../demo/memories-demo";
import { PomodoroDemo } from "../demo/pomodoro-demo";
import { SnapDemo } from "../demo/snap-demo";
import { cn } from "@/lib/utils";

const DEMOS: Record<
  string,
  { demo: React.ReactNode; ratio: string; minH: number }
> = {
  notes: { demo: <NotesDemo />, ratio: "8 / 5.2", minH: 300 },
  todo: { demo: <TodoDemo />, ratio: "8 / 5.2", minH: 300 },
  shop: { demo: <ShopDemo />, ratio: "8 / 5.6", minH: 330 },
  custom: { demo: <CustomStickerDemo />, ratio: "8 / 4.8", minH: 270 },
  handwriting: { demo: <HandwritingDemo />, ratio: "8 / 5.2", minH: 300 },
  transparent: { demo: <TransparentDemo />, ratio: "8 / 4.6", minH: 260 },
  text: { demo: <TextStyleDemo />, ratio: "8 / 5.2", minH: 300 },
  memories: { demo: <MemoriesDemo />, ratio: "8 / 4.8", minH: 300 },
  pomodoro: { demo: <PomodoroDemo />, ratio: "8 / 5.6", minH: 370 },
  snap: { demo: <SnapDemo />, ratio: "8 / 5.2", minH: 300 },
};

export function Chapters() {
  const { dict } = useI18n();

  return (
    <section className="mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 sm:py-24">
      <div className="flex flex-col gap-20 sm:gap-28">
        {dict.chapters.map((ch, i) => {
          const media = DEMOS[ch.media];
          const flipped = i % 2 === 1;
          return (
            <article key={ch.id} id={ch.id} className="scroll-mt-24">
              <div className="grid items-center gap-8 lg:grid-cols-2 lg:gap-14">
                {/* media */}
                <Reveal
                  className={cn(flipped && "lg:order-2")}
                  y={32}
                >
                  <div className="relative">
                    <MediaSlot
                      id={ch.media}
                      ratio={media?.ratio ?? "4 / 3"}
                      minH={media?.minH ?? 300}
                    >
                      {media?.demo}
                    </MediaSlot>
                    {/* chapter tab */}
                    <span className="absolute -top-3 start-5 z-10 rotate-[-2deg] rounded-md border border-line bg-paper px-2.5 py-1 font-display text-sm font-semibold text-ink-soft shadow-note">
                      {ch.eyebrow}
                    </span>
                  </div>
                </Reveal>

                {/* copy */}
                <Reveal
                  delay={0.1}
                  className={cn("flex flex-col gap-4", flipped && "lg:order-1")}
                >
                  <span className="flex items-center gap-3 text-[0.72rem] font-bold uppercase tracking-[0.22em] text-ink-mute">
                    <span className="font-display text-xl tracking-normal text-pin">
                      {ch.eyebrow}
                    </span>
                    <span className="h-px w-8 bg-line" aria-hidden />
                  </span>
                  <h3 className="text-balance font-display text-2xl font-semibold tracking-tight text-ink sm:text-[1.8rem]">
                    {ch.title}
                  </h3>
                  <p className="text-pretty max-w-xl text-[0.92rem] leading-relaxed text-ink-soft">
                    {ch.desc}
                  </p>
                  <ul className="mt-1 flex flex-col gap-2.5">
                    {ch.bullets.map((b, bi) => (
                      <li key={bi} className="flex items-start gap-2.5 text-[0.86rem] leading-relaxed text-ink-soft">
                        <span
                          className="mt-[7px] inline-block size-[7px] shrink-0 rounded-full border-[1.5px] border-pin/70 bg-pin/15"
                          aria-hidden
                        />
                        {b}
                      </li>
                    ))}
                  </ul>
                </Reveal>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
