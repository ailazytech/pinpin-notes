"use client";

import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead, WashiTape, PaperClip, NOTE_COLORS, type NoteColor } from "../ui";

const SWATCHES: NoteColor[] = [
  "butter", "peach", "sky", "mint", "tangerine", "lavender", "white", "kraft",
  "rose", "mist", "sage", "oat", "grape", "milktea", "mustard", "warmgray",
];

const TEXTURE_CHIPS = [
  { label: "sticky", texture: "none" },
  { label: "grid", texture: "grid" },
  { label: "lined", texture: "lined" },
  { label: "dots", texture: "dots" },
] as const;

/** Chapter 1 media: paper colors & textures. */
export function NotesDemo() {
  const { dict } = useI18n();

  return (
    <div className="flex size-full flex-col justify-center gap-4 p-4 sm:p-5">
      <div className="flex items-start justify-center gap-3 sm:gap-4">
        <NotePaper color="butter" texture="lined" className="h-[86px] w-[72px] shrink-0 -rotate-3 px-2.5 py-2 pt-4">
          <PinHead className="left-1/2 top-[-7px]" />
          <div className="space-y-[5px]">
            <div className="h-[3px] w-full rounded bg-ink/40" />
            <div className="h-[3px] w-3/4 rounded bg-ink/25" />
            <div className="h-[3px] w-5/6 rounded bg-ink/15" />
          </div>
        </NotePaper>

        <NotePaper color="sky" texture="grid" className="relative h-[86px] w-[72px] shrink-0 rotate-2 px-2.5 py-2">
          <WashiTape color="var(--note-mustard)" pattern={1} className="left-1/2 top-[-10px] -translate-x-1/2" rotate={-3} width={54} />
          <span className="font-hand text-[13px] leading-tight text-ink/75">
            09:30<br />stand-up
          </span>
        </NotePaper>

        <NotePaper color="kraft" texture="dots" className="h-[86px] w-[72px] shrink-0 -rotate-2 px-2.5 py-2 pt-4">
          <PaperClip className="left-[22%] top-[-13px]" />
          <span className="font-hand text-[13px] leading-tight text-[#4a3a28]/80">
            kraft<br />fiber
          </span>
        </NotePaper>

        <NotePaper color="grape" className="h-[86px] w-[72px] shrink-0 rotate-3 px-2.5 py-2">
          <span className="font-hand text-[13px] leading-tight text-ink/75">plain</span>
        </NotePaper>
      </div>

      {/* 17 colors — the last one is transparent */}
      <div className="flex flex-col gap-1.5">
        <span className="text-[0.66rem] font-bold uppercase tracking-[0.14em] text-ink-mute">
          {dict.demos.notes.colorsLabel}
        </span>
        <div className="flex flex-wrap items-center gap-1.5">
          {SWATCHES.map((c) => (
            <span
              key={c}
              title={c}
              className="size-[15px] rounded-[4px] border border-ink/10 shadow-sm transition-transform hover:scale-125"
              style={{ background: NOTE_COLORS[c] }}
            />
          ))}
          <span
            title={dict.demos.notes.transparent}
            className="grid size-[15px] place-items-center rounded-[4px] border border-dashed border-ink-mute/60"
          >
            <span className="size-[5px] rounded-full bg-ink/20" />
          </span>
        </div>
      </div>

      <div className="flex flex-col gap-1.5">
        <span className="text-[0.66rem] font-bold uppercase tracking-[0.14em] text-ink-mute">
          {dict.demos.notes.texturesLabel}
        </span>
        <div className="flex flex-wrap items-center gap-2">
          {TEXTURE_CHIPS.map((t) => (
            <NotePaper
              key={t.label}
              color="white"
              texture={t.texture as "grid" | "lined" | "dots" | "none"}
              className="flex h-8 w-[74px] items-center justify-center text-[0.62rem] font-semibold uppercase tracking-wide text-ink-mute"
            >
              {t.label}
            </NotePaper>
          ))}
          <span className="text-[0.66rem] text-ink-mute">+ washi · kraft fiber</span>
        </div>
      </div>
    </div>
  );
}
