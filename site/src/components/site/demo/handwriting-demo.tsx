"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead } from "../ui";
import { cn } from "@/lib/utils";

type Pen = 0 | 1 | 2 | 3;

const PEN_STYLE: Record<
  Pen,
  { color: string; width: number; opacity: number; dash?: string; blur?: number; cap: "round" | "butt"; blend?: string }
> = {
  0: { color: "#3a302a", width: 3, opacity: 0.95, cap: "round" },
  1: { color: "#ffd23f", width: 11, opacity: 0.5, cap: "butt" },
  2: { color: "#9a9187", width: 3.6, opacity: 0.9, dash: "0.5 4", blur: 0.4, cap: "round" },
  3: { color: "#e07b39", width: 4.4, opacity: 0.92, dash: "7 1.6", cap: "round" },
};

/** Chapter 5 media: one word, four pen tempers. */
export function HandwritingDemo() {
  const { dict } = useI18n();
  const [pen, setPen] = useState<Pen>(0);
  const d = dict.demos.handwriting;

  const flour = "M14 8 C 60 34, 120 -14, 180 22 S 300 4, 346 18";

  return (
    <div className="flex size-full flex-col items-center justify-center gap-4 p-4">
      {/* pen chips */}
      <div className="flex items-center gap-2">
        {d.pens.map((name, i) => {
          const s = PEN_STYLE[i as Pen];
          return (
            <button
              key={name}
              type="button"
              onClick={() => setPen(i as Pen)}
              aria-pressed={pen === i}
              className={cn(
                "flex flex-col items-center gap-1.5 rounded-xl border px-3 py-2 transition-all",
                pen === i
                  ? "border-pin/50 bg-paper shadow-note"
                  : "border-transparent opacity-60 hover:opacity-100"
              )}
            >
              <svg viewBox="0 0 44 12" width="44" height="12" aria-hidden>
                <path
                  d="M2 8 C 10 2, 18 11, 26 5 S 38 9, 42 4"
                  fill="none"
                  stroke={s.color}
                  strokeWidth={Math.min(s.width, 6)}
                  strokeLinecap={s.cap}
                  strokeDasharray={s.dash}
                  opacity={s.opacity}
                />
              </svg>
              <span className="text-[0.6rem] font-bold uppercase tracking-wide text-ink-soft">
                {name}
              </span>
            </button>
          );
        })}
      </div>

      <NotePaper color="white" texture="dots" className="w-full max-w-[330px] px-6 py-5 pt-6">
        <PinHead className="left-1/2 top-[-8px]" />
        <div className="flex flex-col items-center gap-1">
          <motion.span
            key={pen}
            initial={{ opacity: 0.4 }}
            animate={{ opacity: 1 }}
            className="font-hand text-[38px] leading-none text-ink"
          >
            {d.word}
          </motion.span>
          <svg viewBox="0 0 360 26" className="w-full" aria-hidden>
            <motion.path
              key={`flour-${pen}`}
              d={flour}
              fill="none"
              stroke={PEN_STYLE[pen].color}
              strokeWidth={PEN_STYLE[pen].width}
              strokeLinecap={PEN_STYLE[pen].cap}
              strokeDasharray={PEN_STYLE[pen].dash}
              opacity={PEN_STYLE[pen].opacity}
              filter={PEN_STYLE[pen].blur ? `blur(${PEN_STYLE[pen].blur}px)` : undefined}
              initial={{ pathLength: 0 }}
              animate={{ pathLength: 1 }}
              transition={{ duration: 0.9, ease: "easeInOut" }}
              style={
                pen === 1 ? { mixBlendMode: "multiply" } : undefined
              }
            />
          </svg>
        </div>
      </NotePaper>
      <p className="text-[0.66rem] text-ink-mute">{d.caption}</p>
    </div>
  );
}
