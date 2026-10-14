"use client";

import { useI18n } from "@/i18n/locale-context";
import { StickerImg, SHOP_DEMO_STICKERS, WashiTape } from "../ui";

/** Chapter 3 media: the Sticker Shop shelf. */
export function ShopDemo() {
  const { dict } = useI18n();
  const d = dict.demos.shop;

  return (
    <div className="flex size-full flex-col justify-center gap-3 p-4 sm:p-5">
      {/* counters */}
      <div className="flex flex-wrap items-center justify-center gap-1.5 text-[0.68rem] font-bold text-ink-soft">
        <Count text={`8 ${d.pins}`} />
        <Count text={`7 ${d.clips}`} />
        <Count text={`8 ${d.tapes}`} />
        <Count text={`28 ${d.stickers}`} />
        <span className="rounded-full bg-pin/10 px-2 py-0.5 text-[0.62rem] font-bold uppercase tracking-wider text-pin">
          {d.free}
        </span>
      </div>

      {/* pins / clips / tapes strip */}
      <div className="flex items-end justify-center gap-4 py-1">
        <span className="flex flex-col items-center gap-1">
          <svg viewBox="0 0 24 26" width="18" height="20" aria-hidden>
            <rect x="11.2" y="9" width="1.6" height="12" rx="0.8" fill="#8d8177" />
            <circle cx="12" cy="7.5" r="6.2" fill="#e24234" />
            <circle cx="10.2" cy="5.8" r="1.7" fill="#ffd9d2" opacity="0.9" />
          </svg>
        </span>
        <span className="flex flex-col items-center gap-1">
          <svg viewBox="0 0 24 26" width="18" height="20" aria-hidden>
            <rect x="11.2" y="9" width="1.6" height="12" rx="0.8" fill="#8d8177" />
            <circle cx="12" cy="7.5" r="6.2" fill="#5b7fa6" />
            <circle cx="10.2" cy="5.8" r="1.7" fill="#d5e2ee" opacity="0.9" />
          </svg>
        </span>
        <span className="flex flex-col items-center gap-1">
          <svg viewBox="0 0 24 44" width="15" height="27" aria-hidden>
            <path
              d="M12 2v26a5 5 0 0 0 10 0V12a3.5 3.5 0 0 0-7 0v22a5.5 5.5 0 0 1-11 0V16"
              fill="none"
              stroke="#a89d90"
              strokeWidth="2.4"
              strokeLinecap="round"
            />
          </svg>
        </span>
        <WashiTapeStatic color="var(--note-rose)" pattern={0} />
        <WashiTapeStatic color="var(--note-sage)" pattern={1} />
        <WashiTapeStatic color="var(--note-sky)" pattern={2} />
      </div>

      {/* shelf */}
      <div className="relative rounded-lg border border-line/80 bg-paper/80 p-2.5">
        <span className="absolute left-3 top-1.5 text-[0.6rem] font-bold uppercase tracking-[0.18em] text-ink-mute">
          {d.shelf}
        </span>
        <div className="grid grid-cols-6 gap-1.5 pt-4 sm:gap-2">
          {SHOP_DEMO_STICKERS.map((name) => (
            <div key={name} className="flex items-center justify-center">
              <StickerImg name={name} size={44} className="sm:hidden" />
              <StickerImg name={name} size={54} className="hidden sm:inline-flex" />
            </div>
          ))}
        </div>
        <div className="mt-2 h-[5px] rounded-full bg-gradient-to-b from-line to-transparent" />
      </div>
    </div>
  );
}

function Count({ text }: { text: string }) {
  return (
    <span className="rounded-full border border-line bg-paper px-2.5 py-0.5">
      {text}
    </span>
  );
}

function WashiTapeStatic({ color, pattern }: { color: string; pattern: number }) {
  return (
    <span className="inline-block" aria-hidden>
      <WashiTape
        color={color}
        pattern={pattern}
        width={46}
        rotate={0}
        className="relative"
      />
    </span>
  );
}
