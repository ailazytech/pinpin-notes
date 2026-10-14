"use client";

import { useState } from "react";
import { Minus, Plus } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, WashiTape } from "../ui";
import { cn } from "@/lib/utils";

/** Chapter 7 media: A− / A+ live text sizing on a note. */
export function TextStyleDemo() {
  const { dict } = useI18n();
  const d = dict.demos.text;
  const [scale, setScale] = useState(1.6);
  const [fontIdx, setFontIdx] = useState(0);

  const MIN = 1;
  const MAX = 3.4;

  const fontStacks = [
    "var(--font-caveat), 'Segoe Print', cursive",
    "'Comic Sans MS', 'Chalkboard SE', cursive",
    "Georgia, 'Times New Roman', serif",
    "Impact, 'Arial Black', sans-serif",
  ];

  return (
    <div className="flex size-full flex-col items-center justify-center gap-3 p-4">
      <div className="flex w-full max-w-[360px] items-center justify-center gap-2">
        <button
          type="button"
          onClick={() => setScale((s) => Math.max(MIN, +(s - 0.35).toFixed(2)))}
          disabled={scale <= MIN}
          aria-label={d.smaller}
          className="grid size-8 place-items-center rounded-full border border-line bg-paper text-ink-soft shadow-note transition-all hover:-translate-y-0.5 hover:text-ink disabled:opacity-40 disabled:hover:translate-y-0"
        >
          <Minus className="size-4" />
        </button>
        <span className="tabular rounded-full border border-line bg-paper px-3 py-1 font-mono text-[0.72rem] font-bold text-ink-soft">
          ×{scale.toFixed(2)}
        </span>
        <button
          type="button"
          onClick={() => setScale((s) => Math.min(MAX, +(s + 0.35).toFixed(2)))}
          disabled={scale >= MAX}
          aria-label={d.larger}
          className="grid size-8 place-items-center rounded-full border border-line bg-paper text-ink-soft shadow-note transition-all hover:-translate-y-0.5 hover:text-ink disabled:opacity-40 disabled:hover:translate-y-0"
        >
          <Plus className="size-4" />
        </button>
      </div>

      <NotePaper
        color="butter"
        className="flex w-full max-w-[360px] items-center justify-center overflow-hidden px-4 py-5"
        style={{ minHeight: 90 + scale * 26 }}
      >
        <WashiTape color="var(--note-lavender)" pattern={0} className="left-[-12px] top-[-9px]" rotate={-5} width={58} />
        <span
          className="text-center leading-tight text-ink transition-all duration-300"
          style={{
            fontSize: `${scale * 1.05}rem`,
            fontFamily: fontStacks[fontIdx],
            fontWeight: fontIdx === 3 ? 700 : 500,
          }}
        >
          {d.sample}
        </span>
      </NotePaper>

      <div className="flex items-center gap-1.5">
        {d.fonts.map((f, i) => (
          <button
            key={f}
            type="button"
            onClick={() => setFontIdx(i)}
            aria-pressed={fontIdx === i}
            className={cn(
              "rounded-full border px-2.5 py-1 text-[0.64rem] font-semibold transition-all",
              fontIdx === i
                ? "border-pin/50 bg-pin/10 text-pin"
                : "border-line bg-paper text-ink-mute hover:text-ink-soft"
            )}
          >
            {f}
          </button>
        ))}
      </div>
      <p className="text-[0.64rem] text-ink-mute">{d.note}</p>
    </div>
  );
}
