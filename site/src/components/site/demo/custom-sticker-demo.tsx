"use client";

import { motion } from "framer-motion";
import { ArrowRight, ClipboardPaste, Sparkles } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { StickerImg } from "../ui";

/** Chapter 4 media: clipboard image → die-cut sticker. */
export function CustomStickerDemo() {
  const { dict } = useI18n();
  const d = dict.demos.custom;

  return (
    <div className="flex size-full flex-col items-center justify-center gap-4 p-4">
      <div className="flex items-center justify-center gap-3 sm:gap-5">
        {/* paste card */}
        <div className="flex flex-col items-center gap-2">
          <div className="group flex h-[104px] w-[104px] flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed border-ink-mute/45 bg-paper transition-colors hover:border-pin/60 sm:h-[118px] sm:w-[118px]">
            <span className="grid size-10 place-items-center rounded-lg bg-paper-deep text-ink-soft transition-colors group-hover:bg-pin/10 group-hover:text-pin">
              <ClipboardPaste className="size-5" />
            </span>
            <span className="flex flex-col items-center gap-0.5 text-center">
              <span className="text-[0.7rem] font-bold leading-none text-ink-soft">
                Ctrl+V
              </span>
              <span className="text-[0.62rem] leading-tight text-ink-mute">
                {d.paste}
              </span>
            </span>
          </div>
          <span className="text-[0.6rem] text-ink-mute">{d.drop}</span>
        </div>

        {/* arrow */}
        <motion.span
          animate={{ x: [0, 5, 0] }}
          transition={{ duration: 1.6, repeat: Infinity, ease: "easeInOut" }}
          className="flex flex-col items-center"
        >
          <ArrowRight className="size-5 text-pin" />
          <span className="mt-0.5 text-[0.58rem] font-semibold uppercase tracking-wider text-ink-mute">
            {d.arrow}
          </span>
        </motion.span>

        {/* die-cut result */}
        <div className="flex flex-col items-center gap-2">
          <motion.div
            animate={{ rotate: [0, -4, 3, 0] }}
            transition={{ duration: 5, repeat: Infinity, ease: "easeInOut" }}
            className="relative"
          >
            <span className="absolute -right-2 -top-2 z-10 grid size-6 place-items-center rounded-full bg-pin text-white shadow-note">
              <Sparkles className="size-3.5" />
            </span>
            <StickerImg name="boba" size={110} alt={d.result} className="sm:!h-[124px] sm:!w-[124px]" />
          </motion.div>
          <span className="rounded-full bg-pin/10 px-2.5 py-0.5 text-[0.64rem] font-bold text-pin">
            {d.result}
          </span>
        </div>
      </div>
      <p className="text-[0.66rem] text-ink-mute">{d.keepRes}</p>
    </div>
  );
}
