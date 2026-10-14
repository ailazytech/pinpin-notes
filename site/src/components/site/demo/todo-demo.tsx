"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { MousePointerClick } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead } from "../ui";

/** Interactive strike-through todo list — click a line. */
export function TodoDemo() {
  const { dict } = useI18n();
  const [struck, setStruck] = useState<boolean[]>(
    () => dict.demos.todo.lines.map(() => false)
  );

  const toggle = (i: number) =>
    setStruck((s) => s.map((v, idx) => (idx === i ? !v : v)));

  return (
    <div className="flex size-full flex-col items-center justify-center gap-4 p-4">
      <NotePaper color="butter" texture="lined" className="w-full max-w-[300px] px-5 py-5 pt-6">
        <PinHead className="left-1/2 top-[-8px]" />
        <ul className="space-y-[10px]">
          {dict.demos.todo.lines.map((line, i) => (
            <li key={i}>
              <button
                type="button"
                onClick={() => toggle(i)}
                className="group relative flex w-full items-center gap-2.5 text-start outline-none"
                aria-pressed={struck[i]}
              >
                <span
                  className={`inline-flex size-[15px] shrink-0 items-center justify-center rounded-full border-[1.7px] transition-colors ${
                    struck[i] ? "border-pin/70 bg-pin/15" : "border-ink/50"
                  }`}
                >
                  {struck[i] && (
                    <svg viewBox="0 0 10 10" className="size-[9px]">
                      <path
                        d="M1.5 5.5 L4 8 L8.8 1.8"
                        fill="none"
                        stroke="#e24234"
                        strokeWidth="1.9"
                        strokeLinecap="round"
                      />
                    </svg>
                  )}
                </span>
                <span className="relative">
                  <span
                    className={`font-hand text-[17px] leading-[24px] transition-colors duration-300 ${
                      struck[i] ? "text-ink/40" : "text-ink/90"
                    }`}
                  >
                    {line}
                  </span>
                  {/* the red pen */}
                  <motion.svg
                    viewBox="0 0 110 10"
                    preserveAspectRatio="none"
                    aria-hidden
                    className="pointer-events-none absolute inset-x-[-4px] top-1/2 h-[10px] w-[calc(100%+8px)] -translate-y-1/2"
                  >
                    <motion.path
                      d="M2 6.2 C 18 3, 30 8.4, 46 5 S 72 2.6, 88 6.4 S 102 4, 108 5.2"
                      fill="none"
                      stroke="#e24234"
                      strokeWidth="2.6"
                      strokeLinecap="round"
                      initial={false}
                      animate={{ pathLength: struck[i] ? 1 : 0, opacity: struck[i] ? 1 : 0 }}
                      transition={{ duration: 0.45, ease: [0.65, 0, 0.35, 1] }}
                    />
                  </motion.svg>
                </span>
              </button>
            </li>
          ))}
        </ul>
      </NotePaper>
      <p className="flex items-center gap-1.5 text-[0.7rem] font-semibold text-ink-mute">
        <MousePointerClick className="size-3.5" />
        {dict.demos.todo.strikeHint}
      </p>
    </div>
  );
}
