"use client";

import { asset } from "@/lib/site-config";
import { useEffect, useRef, useState } from "react";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead, WashiTape, PaperClip, StickerImg } from "../ui";

/**
 * A hand-built “desktop” scene: wallpaper + pinned notes + washi tape +
 * stickers + a transparent slogan. Doubles as the hero art until real
 * screenshots land.
 */
export function HeroScene() {
  const { dict } = useI18n();
  const [wallLoaded, setWallLoaded] = useState(false);
  const wallRef = useRef<HTMLImageElement | null>(null);

  /* the wallpaper may finish (or fail) before hydration */
  useEffect(() => {
    const img = wallRef.current;
    if (!img || !img.complete) return;
    const t = window.setTimeout(() => setWallLoaded(true), 0);
    return () => window.clearTimeout(t);
  }, []);

  /* notes drop in after the intro curtain; plain CSS so it never depends on JS timing */
  const drop = (i: number) => ({ animationDelay: `${1.7 + i * 0.16}s` });

  return (
    <div className="relative">
      <div className="paper-grain relative aspect-[4/3.1] overflow-hidden rounded-2xl border border-line shadow-lift">
        {/* wallpaper */}
        <div
          className="absolute inset-0 bg-paper"
          style={{
            background:
              "linear-gradient(135deg, #f6efe2 0%, #efe5d2 45%, #e9dcc4 100%)",
          }}
        >
          {!wallLoaded && <div className="shimmer absolute inset-0" aria-hidden />}
          <img
            ref={wallRef}
            src={asset("img/wallpaper-light.webp")}
            alt=""
            aria-hidden
            className="absolute inset-0 size-full object-cover transition-opacity duration-700"
            style={{ opacity: wallLoaded ? 1 : 0 }}
            onLoad={() => setWallLoaded(true)}
          />
        </div>

        {/* note 1 — todo list with a struck line */}
        <div style={drop(0)} className="pb-drop group absolute left-[6%] top-[10%] w-[42%] max-w-[210px] -rotate-3 transition-transform duration-300 hover:rotate-0 hover:scale-[1.03]"
        >
          <NotePaper color="butter" texture="lined" className="px-4 py-5 pt-6">
            <PinHead className="left-1/2 top-[-8px]" />
            <ul className="space-y-[7px] font-hand text-[15px] leading-[22px] text-ink/90 sm:text-[17px]">
              {dict.hero.note1.split("\n").map((line, i) => (
                <li key={i} className="relative flex items-center gap-2">
                  <span className="inline-flex size-[13px] shrink-0 items-center justify-center rounded-full border-[1.6px] border-ink/60" />
                  <span className={i === 0 ? "text-ink/45" : ""}>{line}</span>
                  {i === 0 && (
                    <svg
                      viewBox="0 0 40 8"
                      aria-hidden
                      className="pointer-events-none absolute left-[21px] top-1/2 w-[82px] -translate-y-1/2"
                    >
                      <path
                        d="M1 4.5c7-2.6 14-3 20-2.2 6 .8 12 2.4 18 1.4"
                        fill="none"
                        stroke="#e24234"
                        strokeWidth="2.2"
                        strokeLinecap="round"
                      />
                    </svg>
                  )}
                </li>
              ))}
            </ul>
          </NotePaper>
        </div>

        {/* note 2 — pomodoro */}
        <div style={drop(1)} className="pb-drop group absolute right-[7%] top-[6%] w-[36%] max-w-[175px] rotate-[4deg] transition-transform duration-300 hover:rotate-1 hover:scale-[1.03]"
        >
          <NotePaper color="rose" texture="grid" className="flex items-center gap-3 px-4 py-4">
            <WashiTape color="var(--note-sage)" pattern={1} className="left-[-14px] top-[-9px]" rotate={-38} width={64} />
            <TomatoMini />
            <div className="flex flex-col">
              <span className="font-hand text-[15px] leading-tight text-ink/80">focus</span>
              <span className="tabular font-display text-[19px] font-semibold tracking-tight text-ink">
                25:00
              </span>
              <span className="mt-0.5 flex gap-1">
                {[0, 1, 2].map((d) => (
                  <span key={d} className="size-[5px] rounded-full bg-pin/85" />
                ))}
              </span>
            </div>
          </NotePaper>
        </div>

        {/* note 3 — clipped reminder */}
        <div style={drop(2)} className="pb-drop group absolute bottom-[14%] left-[14%] w-[34%] max-w-[165px] rotate-[2.5deg] transition-transform duration-300 hover:rotate-0 hover:scale-[1.03]"
        >
          <NotePaper color="mint" className="px-4 py-4 pt-5">
            <PaperClip className="left-[16%] top-[-15px]" />
            <p className="font-hand text-[16px] leading-snug text-ink/90">
              {dict.hero.note3.split("\n").map((line, i) => (
                <span key={i}>
                  {line}
                  <br />
                </span>
              ))}
            </p>
          </NotePaper>
        </div>

        {/* transparent slogan */}
        <div style={drop(3)} className="pb-drop absolute bottom-[7%] right-[8%] max-w-[46%] text-end"
        >
          <p className="whitespace-pre-line font-display text-[clamp(1.05rem,2.6vw,1.7rem)] font-semibold italic leading-tight text-[#3a302a] mix-blend-multiply">
            {dict.hero.slogan}
          </p>
          <svg viewBox="0 0 120 10" className="ms-auto mt-1 w-full max-w-[150px]" aria-hidden>
            <path
              d="M2 6c18-3.5 36-4.5 54-3 18 1.5 36 4 62 2"
              fill="none"
              stroke="#e24234"
              strokeWidth="2.6"
              strokeLinecap="round"
              opacity="0.9"
            />
          </svg>
        </div>

        {/* stickers */}
        <div style={drop(4)} className="pb-drop absolute right-[30%] top-[38%]">
          <div className="animate-float" style={{ animationDelay: "0.6s" }}>
            <StickerImg name="boba" size={54} alt="boba sticker" />
          </div>
        </div>
        <div style={drop(5)} className="pb-drop absolute bottom-[26%] right-[38%]">
          <div className="animate-float" style={{ animationDelay: "1.4s" }}>
            <StickerImg name="cat" size={62} alt="cat sticker" />
          </div>
        </div>
        <div style={drop(6)} className="pb-drop absolute left-[52%] top-[54%]">
          <div className="animate-float" style={{ animationDelay: "2.1s" }}>
            <StickerImg name="sakura" size={44} alt="sakura sticker" />
          </div>
        </div>
      </div>

      <p className="mt-3 text-center font-hand text-base text-ink-mute">
        {dict.hero.sceneCaption}
      </p>
    </div>
  );
}

function TomatoMini() {
  return (
    <svg viewBox="0 0 44 46" width="42" height="44" aria-hidden className="shrink-0">
      <ellipse cx="22" cy="43" rx="13" ry="2.4" fill="#3a302a" opacity="0.14" />
      <circle cx="22" cy="27" r="17" fill="#e24234" />
      <circle cx="16.5" cy="21.5" r="5" fill="#f06a5a" opacity="0.85" />
      <path d="M22 10c-1.8-3.4-5-5-5-5s1.6 3.8 5 5z" fill="#7ca156" />
      <path d="M22 10c1.8-3.4 5-5 5-5s-1.6 3.8-5 5z" fill="#8fb268" />
      <rect x="21.1" y="6.5" width="1.8" height="4.5" rx="0.9" fill="#5d7a41" />
      <circle
        cx="22"
        cy="27"
        r="13.5"
        fill="none"
        stroke="#fdfbf7"
        strokeWidth="2.6"
        strokeDasharray="56 29"
        strokeLinecap="round"
        transform="rotate(-90 22 27)"
        opacity="0.95"
      />
    </svg>
  );
}
