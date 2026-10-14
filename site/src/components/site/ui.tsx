"use client";

import { asset } from "@/lib/site-config";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion, useReducedMotion } from "framer-motion";
import { Image as ImageIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { screenshotPath } from "@/lib/site-config";
import { useI18n } from "@/i18n/locale-context";

/* ------------------------------------------------------------------ */
/* Scroll reveal                                                       */
/* ------------------------------------------------------------------ */

export function Reveal({
  children,
  delay = 0,
  y = 26,
  className,
}: {
  children: ReactNode;
  delay?: number;
  y?: number;
  className?: string;
}) {
  const reduce = useReducedMotion();
  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: reduce ? 0 : y }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-60px" }}
      transition={{ duration: 0.65, delay, ease: [0.22, 0.8, 0.28, 1] }}
    >
      {children}
    </motion.div>
  );
}

/* ------------------------------------------------------------------ */
/* Section heading                                                     */
/* ------------------------------------------------------------------ */

export function SectionHeading({
  eyebrow,
  title,
  sub,
  align = "center",
  className,
}: {
  eyebrow?: string;
  title: string;
  sub?: string;
  align?: "center" | "start";
  className?: string;
}) {
  return (
    <div
      className={cn(
        "mb-10 flex flex-col gap-3 sm:mb-14",
        align === "center" ? "items-center text-center" : "items-start text-start",
        className
      )}
    >
      {eyebrow && (
        <span className="inline-flex items-center gap-2 text-[0.72rem] font-bold uppercase tracking-[0.22em] text-ink-mute">
          <span className="inline-block h-[2px] w-5 rounded-full bg-pin/70" aria-hidden />
          {eyebrow}
          <span className="inline-block h-[2px] w-5 rounded-full bg-pin/70" aria-hidden />
        </span>
      )}
      <h2 className="text-balance font-display text-3xl font-semibold tracking-tight text-ink sm:text-4xl">
        {title}
      </h2>
      {sub && (
        <p className="text-pretty max-w-2xl text-[0.95rem] leading-relaxed text-ink-soft sm:text-base">
          {sub}
        </p>
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Paper decorations                                                   */
/* ------------------------------------------------------------------ */

export function PinHead({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={cn("absolute -translate-x-1/2", className)}
      width="22"
      height="26"
      aria-hidden
    >
      <rect x="11.2" y="9" width="1.6" height="12" rx="0.8" fill="#8d8177" />
      <ellipse cx="12" cy="20.4" rx="2.2" ry="0.9" fill="#3a302a" opacity="0.18" />
      <circle cx="12" cy="7.5" r="6.4" fill="#e24234" />
      <circle cx="10.1" cy="5.7" r="1.8" fill="#ffd9d2" opacity="0.9" />
      <circle cx="12" cy="7.5" r="6.4" fill="url(#pingrad)" opacity="0" />
    </svg>
  );
}

const TAPE_PATTERNS = ["", "tape-stripes", "tape-dots", "tape-cross"] as const;

export function WashiTape({
  color = "var(--note-rose)",
  pattern = 0,
  className,
  width = 88,
  rotate = -6,
}: {
  color?: string;
  pattern?: number;
  className?: string;
  width?: number;
  rotate?: number;
}) {
  const zig =
    "polygon(0 6%, 4% 0, 8% 7%, 12% 1%, 16% 6%, 20% 0, 24% 7%, 28% 1%, 32% 6%, 36% 0, 40% 7%, 44% 1%, 48% 6%, 52% 0, 56% 7%, 60% 1%, 64% 6%, 68% 0, 72% 7%, 76% 1%, 80% 6%, 84% 0, 88% 7%, 92% 1%, 96% 6%, 100% 0, 100% 100%, 96% 94%, 92% 100%, 88% 93%, 84% 99%, 80% 94%, 76% 100%, 72% 93%, 68% 99%, 64% 94%, 60% 100%, 56% 93%, 52% 99%, 48% 94%, 44% 100%, 40% 93%, 36% 99%, 32% 94%, 28% 100%, 24% 93%, 20% 99%, 16% 94%, 12% 100%, 8% 93%, 4% 99%, 0 94%)";
  return (
    <div
      aria-hidden
      className={cn("pointer-events-none absolute", TAPE_PATTERNS[pattern], className)}
      style={{
        width,
        height: 24,
        background: color,
        opacity: 0.88,
        clipPath: zig,
        transform: `rotate(${rotate}deg)`,
        boxShadow: "0 1px 2px rgb(58 48 42 / 0.12)",
      }}
    />
  );
}

export function PaperClip({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 44"
      width="18"
      height="34"
      aria-hidden
      className={cn("absolute", className)}
    >
      <path
        d="M12 2v26a5 5 0 0 0 10 0V12a3.5 3.5 0 0 0-7 0v22a5.5 5.5 0 0 1-11 0V16"
        fill="none"
        stroke="#a89d90"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
      <path
        d="M12 2v26a5 5 0 0 0 10 0V12a3.5 3.5 0 0 0-7 0v22a5.5 5.5 0 0 1-11 0V16"
        fill="none"
        stroke="#fdfbf7"
        strokeWidth="0.9"
        strokeLinecap="round"
        opacity="0.7"
        transform="translate(-0.6 -0.4)"
      />
    </svg>
  );
}

/* ------------------------------------------------------------------ */
/* Sticky note                                                         */
/* ------------------------------------------------------------------ */

export const NOTE_COLORS = {
  butter: "var(--note-butter)",
  peach: "var(--note-peach)",
  sky: "var(--note-sky)",
  mint: "var(--note-mint)",
  tangerine: "var(--note-tangerine)",
  lavender: "var(--note-lavender)",
  white: "var(--note-white)",
  kraft: "var(--note-kraft)",
  rose: "var(--note-rose)",
  mist: "var(--note-mist)",
  sage: "var(--note-sage)",
  oat: "var(--note-oat)",
  grape: "var(--note-grape)",
  milktea: "var(--note-milktea)",
  mustard: "var(--note-mustard)",
  warmgray: "var(--note-warmgray)",
} as const;

export type NoteColor = keyof typeof NOTE_COLORS;

const TEXTURE_CLASS: Record<string, string> = {
  lined: "note-texture-lined",
  grid: "note-texture-grid",
  dots: "note-texture-dots",
};

export function NotePaper({
  color = "butter",
  texture,
  className,
  children,
  lift = false,
  style,
}: {
  color?: NoteColor;
  texture?: "lined" | "grid" | "dots" | "none";
  className?: string;
  children?: ReactNode;
  lift?: boolean;
  style?: React.CSSProperties;
}) {
  return (
    <div
      className={cn(
        "paper-grain relative rounded-[5px] transition-shadow duration-300",
        lift ? "shadow-lift" : "shadow-note",
        TEXTURE_CLASS[texture ?? "none"],
        className
      )}
      style={{ background: NOTE_COLORS[color], ...style }}
    >
      {children}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Die-cut sticker (AI image with skeleton fade-in)                    */
/* ------------------------------------------------------------------ */

const STICKER_SOURCES = {
  cat: asset("img/stickers/cat.webp"),
  shiba: asset("img/stickers/shiba.webp"),
  boba: asset("img/stickers/boba.webp"),
  strawberry: asset("img/stickers/strawberry.webp"),
  donut: asset("img/stickers/donut.webp"),
  riceball: asset("img/stickers/riceball.webp"),
  cactus: asset("img/stickers/cactus.webp"),
  monstera: asset("img/stickers/monstera.webp"),
  sunflower: asset("img/stickers/sunflower.webp"),
  sakura: asset("img/stickers/sakura.webp"),
  rainbow: asset("img/stickers/rainbow.webp"),
  moon: asset("img/stickers/moon.webp"),
  corgi: asset("img/stickers/corgi.webp"),
  luckycat: asset("img/stickers/luckycat.webp"),
  ingot: asset("img/stickers/ingot.webp"),
  fu: asset("img/stickers/fu.webp"),
  zhaocai: asset("img/stickers/zhaocai.webp"),
  daruma: asset("img/stickers/daruma.webp"),
  nazar: asset("img/stickers/nazar.webp"),
  pothos: asset("img/stickers/pothos.webp"),
  bonsai: asset("img/stickers/bonsai.webp"),
  calico: asset("img/stickers/calico.webp"),
  frenchie: asset("img/stickers/frenchie.webp"),
  scooter: asset("img/stickers/scooter.webp"),
  "scooter-mint": asset("img/stickers/scooter-mint.webp"),
  bento: asset("img/stickers/bento.webp"),
  drumstick: asset("img/stickers/drumstick.webp"),
} as const;

/** A mixed dozen for the shop demo shelf: a few from every set. */
export const SHOP_DEMO_STICKERS: StickerName[] = ["cat", "shiba", "boba", "luckycat", "fu", "daruma", "pothos", "monstera", "calico", "frenchie", "scooter", "sakura"];

export type StickerName = keyof typeof STICKER_SOURCES;

export const STICKER_NAMES = Object.keys(STICKER_SOURCES) as StickerName[];

export function StickerImg({
  name,
  size = 76,
  className,
  alt,
  wiggle = true,
}: {
  name: StickerName;
  size?: number;
  className?: string;
  alt?: string;
  wiggle?: boolean;
}) {
  const [state, setState] = useState<"loading" | "ok" | "fail">("loading");
  const imgRef = useRef<HTMLImageElement | null>(null);

  /* the image may finish (or fail) before hydration — catch that */
  useEffect(() => {
    const img = imgRef.current;
    if (!img || !img.complete) return;
    const ok = img.naturalWidth > 0;
    const t = window.setTimeout(() => setState(ok ? "ok" : "fail"), 0);
    return () => window.clearTimeout(t);
  }, []);

  return (
    <span
      className={cn(
        "group relative inline-flex shrink-0 items-center justify-center drop-shadow-[0_3px_5px_rgb(60_40_20/0.2)]",
        wiggle && "transition-transform duration-300 hover:-rotate-6 hover:scale-105",
        className
      )}
      style={{ width: size, height: size }}
    >
      {state === "loading" && (
        <span className="shimmer absolute inset-0 rounded-[42%]" aria-hidden />
      )}
      {state === "fail" && (
        <span
          className="absolute inset-0 rounded-[42%] border-2 border-dashed border-ink-mute/30"
          aria-hidden
        />
      )}
      <img
        ref={imgRef}
        src={STICKER_SOURCES[name]}
        alt={alt ?? `${name} sticker`}
        width={size}
        height={size}
        loading="lazy"
        draggable={false}
        onLoad={() => setState("ok")}
        onError={() => setState("fail")}
        className={cn(
          "size-full object-contain transition-opacity duration-500",
          state === "ok" ? "opacity-100" : "opacity-0"
        )}
      />
    </span>
  );
}

/* ------------------------------------------------------------------ */
/* Media slots (real capture replaces the demo when the file exists)   */
/* ------------------------------------------------------------------ */

export function MediaSlot({
  id,
  ratio = "4 / 3",
  children,
  className,
  minH = 300,
}: {
  id: string;
  ratio?: string;
  children?: ReactNode;
  className?: string;
  file?: string;
  minH?: number;
}) {
  const { dict } = useI18n();
  const [state, setState] = useState<"loading" | "ready" | "missing">("loading");
  /** with a real screenshot: show it, and let the visitor switch to the hand-built live demo */
  const [demo, setDemo] = useState(false);
  const path = screenshotPath(id);
  const probeRef = useRef<HTMLImageElement | null>(null);

  /* the probe may resolve before the effect runs — catch that */
  useEffect(() => {
    const img = probeRef.current;
    if (!img || !img.complete) return;
    const ok = img.naturalWidth > 0;
    const t = window.setTimeout(() => setState(ok ? "ready" : "missing"), 0);
    return () => window.clearTimeout(t);
  }, []);

  const showDemo = state === "missing" || (state === "ready" && demo);
  const pill =
    "absolute end-3 top-3 z-10 inline-flex items-center gap-1.5 rounded-full border border-line bg-background/85 px-2.5 py-1 text-[0.65rem] font-bold uppercase tracking-[0.12em] text-ink-mute backdrop-blur-sm";

  return (
    <div
      className={cn("paper-grain relative overflow-hidden rounded-xl border border-line bg-paper-deep/50", className)}
      style={state === "ready" && !demo ? { aspectRatio: ratio } : { minHeight: minH }}
    >
      {state === "loading" && <div className="shimmer absolute inset-0" aria-hidden />}
      {state === "ready" && !demo && (
        <>
          <img src={path} alt={dict.common.screenshotSlot} className="absolute inset-0 size-full object-cover" />
          <div className="absolute inset-0 shadow-[inset_0_0_0_1px_rgb(58_48_42/0.08)]" />
          {children && (
            <button type="button" onClick={() => setDemo(true)} className={cn(pill, "cursor-pointer transition hover:text-ink")}>
              <span className="relative flex size-[6px]">
                <span className="absolute inline-flex size-full animate-ping rounded-full bg-pin opacity-60" />
                <span className="relative inline-flex size-[6px] rounded-full bg-pin" />
              </span>
              {dict.common.tryIt}
            </button>
          )}
        </>
      )}
      {showDemo && (
        <>
          {state === "ready" ? (
            <button type="button" onClick={() => setDemo(false)} className={cn(pill, "cursor-pointer transition hover:text-ink")}>
              <ImageIcon className="size-3" />
              {dict.common.showScreenshot}
            </button>
          ) : (
            <span className={pill}>
              <span className="relative inline-flex size-[6px] rounded-full bg-pin" />
              {dict.common.liveDemo}
            </span>
          )}
          {children}
        </>
      )}
      {/* silent probe */}
      <img
        ref={probeRef}
        src={path}
        alt=""
        aria-hidden
        className="hidden"
        onLoad={() => setState("ready")}
        onError={() => setState("missing")}
      />
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Typewriter-ish scramble text (for the password note demo)           */
/* ------------------------------------------------------------------ */

export function useScramble(
  target: string,
  active: boolean,
  duration = 900
): string {
  const [text, setText] = useState<string | null>(null);
  const raf = useRef<number | null>(null);

  useEffect(() => {
    if (!active) return;
    const chars = "ABCDEF ghijkl MNOPQR stuvwx 01234 56789 !?*#%&";
    const start = performance.now();
    const tick = (now: number) => {
      const p = Math.min(1, (now - start) / duration);
      const keep = Math.floor(target.length * p);
      let out = target.slice(0, keep);
      for (let i = keep; i < target.length; i++) {
        out +=
          target[i] === "\n"
            ? "\n"
            : chars[Math.floor(Math.random() * chars.length)];
      }
      setText(out);
      if (p < 1) raf.current = requestAnimationFrame(tick);
    };
    raf.current = requestAnimationFrame(tick);
    return () => {
      if (raf.current) cancelAnimationFrame(raf.current);
      setText(null);
    };
  }, [target, active, duration]);

  return active ? (text ?? target) : target;
}
