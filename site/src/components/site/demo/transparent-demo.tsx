"use client";

import { asset } from "@/lib/site-config";
import { useEffect, useRef, useState } from "react";
import { Eye, Pencil, GripHorizontal } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";

/** Chapter 6 media: transparent slogan on the wallpaper. */
export function TransparentDemo() {
  const { dict } = useI18n();
  const [hover, setHover] = useState(false);
  const d = dict.demos.transparent;
  const [wallLoaded, setWallLoaded] = useState(false);
  const wallRef = useRef<HTMLImageElement | null>(null);

  useEffect(() => {
    const img = wallRef.current;
    if (!img || !img.complete) return;
    const t = window.setTimeout(() => setWallLoaded(true), 0);
    return () => window.clearTimeout(t);
  }, []);

  return (
    <div className="flex size-full flex-col items-center justify-center gap-3 overflow-hidden p-0">
      <div
        className="relative flex size-full items-center justify-center"
        onMouseEnter={() => setHover(true)}
        onMouseLeave={() => setHover(false)}
        onFocus={() => setHover(true)}
        onBlur={() => setHover(false)}
      >
        <div
          className="absolute inset-0"
          style={{
            background: "linear-gradient(135deg,#f6efe2,#e9dcc4)",
          }}
        >
          {!wallLoaded && <div className="shimmer absolute inset-0" aria-hidden />}
          <img
            ref={wallRef}
            src={asset("img/wallpaper-light.webp")}
            alt=""
            aria-hidden
            className="absolute inset-0 size-full object-cover"
            style={{ opacity: wallLoaded ? 1 : 0 }}
            onLoad={() => setWallLoaded(true)}
          />
        </div>

        {/* the transparent note */}
        <div
          className="group relative cursor-grab px-6 py-4 active:cursor-grabbing"
          role="button"
          tabIndex={0}
          aria-label={d.hint}
        >
          <div
            className="absolute inset-0 rounded-lg border-2 border-dashed border-white/70 transition-opacity duration-300"
            style={{ opacity: hover ? 1 : 0 }}
          >
            <div className="absolute -top-3 left-1/2 flex -translate-x-1/2 gap-1 transition-opacity duration-300" style={{ opacity: hover ? 1 : 0 }}>
              <FrameBtn>
                <GripHorizontal className="size-3" />
              </FrameBtn>
              <FrameBtn>
                <Pencil className="size-3" />
              </FrameBtn>
              <FrameBtn>
                <Eye className="size-3" />
              </FrameBtn>
            </div>
          </div>
          <p className="relative text-center font-display text-[clamp(1.3rem,4.2vw,2.3rem)] font-semibold italic leading-tight text-[#3a302a] mix-blend-multiply">
            {d.motto}
          </p>
          <svg viewBox="0 0 120 10" className="mx-auto mt-1 w-[70%]" aria-hidden>
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
      </div>
      <p className="bg-background/70 py-0.5 text-[0.66rem] text-ink-mute backdrop-blur-[2px]">
        {d.hint}
      </p>
    </div>
  );
}

function FrameBtn({ children }: { children: React.ReactNode }) {
  return (
    <span className="grid size-6 place-items-center rounded-md border border-dashed border-white/70 bg-white/85 text-[#3a302a] shadow-note">
      {children}
    </span>
  );
}
