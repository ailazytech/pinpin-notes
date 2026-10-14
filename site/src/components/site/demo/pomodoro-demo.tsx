"use client";

import { useEffect, useRef, useState } from "react";
import { Pause, Play, RotateCcw, Minus, Plus } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { StickerImg } from "../ui";
import { cn } from "@/lib/utils";

type StyleIdx = 0 | 1 | 2 | 3 | 4 | 5;

export function PomodoroDemo() {
  const { dict } = useI18n();
  const d = dict.demos.pomodoro;

  const [style, setStyle] = useState<StyleIdx>(0);
  const [minutes, setMinutes] = useState(25);
  const [left, setLeft] = useState(25 * 60);
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState(0);
  const [flash, setFlash] = useState(false);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    if (!running) return;
    timer.current = window.setInterval(() => {
      setLeft((l) => {
        if (l <= 1) {
          setRunning(false);
          setDone((n) => n + 1);
          setFlash(true);
          window.setTimeout(() => setFlash(false), 900);
          return minutes * 60;
        }
        return l - 1;
      });
    }, 1000);
    return () => {
      if (timer.current) window.clearInterval(timer.current);
    };
  }, [running, minutes]);

  const mm = Math.floor(left / 60)
    .toString()
    .padStart(2, "0");
  const ss = (left % 60).toString().padStart(2, "0");
  const progress = minutes > 0 ? left / (minutes * 60) : 0;

  const setMin = (m: number) => {
    const clamped = Math.min(180, Math.max(1, m));
    setMinutes(clamped);
    setLeft(clamped * 60);
    setRunning(false);
  };

  return (
    <div className="flex size-full flex-col items-center justify-center gap-3 p-4">
      {/* style chips */}
      <div className="flex flex-wrap items-center justify-center gap-1.5">
        {d.styles.map((name, i) => (
          <button
            key={name}
            type="button"
            onClick={() => setStyle(i as StyleIdx)}
            aria-pressed={style === i}
            className={cn(
              "rounded-full border px-2.5 py-1 text-[0.64rem] font-bold transition-all",
              style === i
                ? "border-pin/50 bg-pin/10 text-pin"
                : "border-line bg-paper text-ink-mute hover:text-ink-soft"
            )}
          >
            {name}
          </button>
        ))}
      </div>

      {/* the timekeeper — still frame, moving digits */}
      <div className={cn("relative flex h-[132px] w-full items-center justify-center", flash && "animate-wiggle")}>
        <Face style={style} mm={mm} ss={ss} progress={progress} running={running} />
      </div>

      {/* controls */}
      <div className="flex flex-wrap items-center justify-center gap-2">
        <button
          type="button"
          onClick={() => setRunning((r) => !r)}
          className="inline-flex h-8 items-center gap-1.5 rounded-full bg-ink px-4 text-[0.72rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
        >
          {running ? <Pause className="size-3.5" /> : <Play className="size-3.5" />}
          {running ? d.pause : d.start}
        </button>
        <button
          type="button"
          onClick={() => setLeft(minutes * 60)}
          className="inline-flex h-8 items-center gap-1.5 rounded-full border border-line bg-paper px-3.5 text-[0.72rem] font-bold text-ink-soft shadow-note transition-all hover:-translate-y-0.5 hover:text-ink"
        >
          <RotateCcw className="size-3.5" />
          {d.reset}
        </button>
        <span className="inline-flex h-8 items-center gap-1 rounded-full border border-line bg-paper px-1.5 shadow-note">
          <button
            type="button"
            onClick={() => setMin(minutes - 1)}
            disabled={running}
            aria-label="-1"
            className="grid size-6 place-items-center rounded-full text-ink-mute hover:text-ink disabled:opacity-40"
          >
            <Minus className="size-3" />
          </button>
          <span className="tabular w-[46px] text-center font-mono text-[0.72rem] font-bold text-ink">
            {minutes} {d.minutes}
          </span>
          <button
            type="button"
            onClick={() => setMin(minutes + 1)}
            disabled={running}
            aria-label="+1"
            className="grid size-6 place-items-center rounded-full text-ink-mute hover:text-ink disabled:opacity-40"
          >
            <Plus className="size-3" />
          </button>
        </span>
      </div>

      {/* finished tomatoes */}
      <p className="flex items-center gap-1.5 text-[0.64rem] font-semibold text-ink-mute">
        {[0, 1, 2, 3, 4].map((i) => (
          <span
            key={i}
            className={cn(
              "size-[7px] rounded-full transition-all",
              i < done ? "bg-pin shadow-[0_0_6px_rgb(226_66_52/0.6)]" : "bg-ink/15"
            )}
          />
        ))}
        <span className="ml-1 tabular">
          {done} {d.done}
        </span>
      </p>
    </div>
  );
}

function Face({
  style,
  mm,
  ss,
  progress,
  running,
}: {
  style: StyleIdx;
  mm: string;
  ss: string;
  progress: number;
  running: boolean;
}) {
  switch (style) {
    case 0:
      return <TomatoFace mm={mm} ss={ss} progress={progress} />;
    case 1:
      return <HourglassFace mm={mm} ss={ss} progress={progress} running={running} />;
    case 2:
      return <LedFace mm={mm} ss={ss} progress={progress} running={running} />;
    case 3:
      return <AlarmFace mm={mm} ss={ss} progress={progress} />;
    case 4:
      return <AnimalFace name="cat" mm={mm} ss={ss} progress={progress} />;
    case 5:
      return <AnimalFace name="shiba" mm={mm} ss={ss} progress={progress} />;
  }
}

function TimeText({ mm, ss }: { mm: string; ss: string }) {
  return (
    <span className="tabular font-display text-[22px] font-semibold tracking-tight text-ink">
      {mm}
      <span className="text-ink/50">:</span>
      {ss}
    </span>
  );
}

/* ---------------- tomato ---------------- */

function TomatoFace({ mm, ss, progress }: { mm: string; ss: string; progress: number }) {
  const r = 30;
  const c = 2 * Math.PI * r;
  return (
    <div className="flex items-center gap-5">
      <svg viewBox="0 0 76 78" width="76" height="78" aria-hidden>
        <ellipse cx="38" cy="74" rx="20" ry="3" fill="#3a302a" opacity="0.12" />
        <circle cx="38" cy="44" r="30" fill="#e24234" />
        <circle cx="29" cy="35" r="9" fill="#f06a5a" opacity="0.8" />
        <path d="M38 16c-3.2-6-9-8.6-9-8.6s2.9 6.6 9 8.6z" fill="#7ca156" />
        <path d="M38 16c3.2-6 9-8.6 9-8.6s-2.9 6.6-9 8.6z" fill="#8fb268" />
        <rect x="36.9" y="9.6" width="2.2" height="7" rx="1.1" fill="#5d7a41" />
        <circle
          cx="38"
          cy="44"
          r={r}
          fill="none"
          stroke="#fdfbf7"
          strokeWidth="4.6"
          strokeLinecap="round"
          strokeDasharray={`${c * progress} ${c}`}
          transform="rotate(-90 38 44)"
        />
      </svg>
      <div className="flex flex-col gap-1">
        <TimeText mm={mm} ss={ss} />
        <span className="flex gap-1">
          {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => (
            <span
              key={i}
              className={cn(
                "size-[4px] rounded-full",
                i / 8 < progress ? "bg-pin/80" : "bg-ink/15"
              )}
            />
          ))}
        </span>
      </div>
    </div>
  );
}

/* ---------------- hourglass ---------------- */

function HourglassFace({
  mm,
  ss,
  progress,
  running,
}: {
  mm: string;
  ss: string;
  progress: number;
  running: boolean;
}) {
  const p = 1 - progress; // elapsed
  const topH = 34 * progress;
  const botH = 34 * p;
  return (
    <div className="flex items-center gap-5">
      <svg viewBox="0 0 64 84" width="58" height="76" aria-hidden>
        <ellipse cx="32" cy="80" rx="20" ry="3" fill="#3a302a" opacity="0.12" />
        <rect x="10" y="4" width="44" height="6" rx="3" fill="#a9845a" />
        <rect x="10" y="74" width="44" height="6" rx="3" fill="#a9845a" />
        <path
          d="M16 10 C16 30 26 34 30 42 C26 50 16 54 16 74 M48 10 C48 30 38 34 34 42 C38 50 48 54 48 74"
          fill="none"
          stroke="#cfe0e8"
          strokeWidth="2.6"
          opacity="0.9"
        />
        {/* top sand */}
        <polygon points={`20,${12 + 30 - topH * 0.88} 44,${12 + 30 - topH * 0.88} 32,42`} fill="#e8b84b" opacity="0.95" />
        {/* falling stream */}
        {running && progress > 0.02 && (
          <rect x="31" y="42" width="2" height="30" fill="#e8b84b" opacity="0.85" />
        )}
        {/* bottom sand */}
        <polygon points={`20,74 44,74 32,${74 - botH}`} fill="#e8b84b" opacity="0.95" />
      </svg>
      <div className="flex flex-col gap-1">
        <TimeText mm={mm} ss={ss} />
        <span className="text-[0.6rem] font-semibold uppercase tracking-wider text-ink-mute">
          sand
        </span>
      </div>
    </div>
  );
}

/* ---------------- LED ---------------- */

function LedFace({
  mm,
  ss,
  progress,
  running,
}: {
  mm: string;
  ss: string;
  progress: number;
  running: boolean;
}) {
  return (
    <div className="flex flex-col items-center gap-2.5">
      <div className="flex items-center gap-2 rounded-xl bg-[#241d18] px-5 py-3 shadow-note ring-1 ring-white/10">
        <span className="led-glow tabular font-mono text-[26px] font-bold tracking-[0.08em] text-[#ff5a47]">
          {mm}
        </span>
        <span
          className={cn(
            "led-glow font-mono text-[26px] font-bold text-[#ff5a47]",
            running && "animate-blink"
          )}
        >
          :
        </span>
        <span className="led-glow tabular font-mono text-[26px] font-bold tracking-[0.08em] text-[#ff5a47]">
          {ss}
        </span>
      </div>
      <span className="flex gap-[5px]">
        {[0, 1, 2, 3, 4, 5, 6, 7, 8, 9].map((i) => (
          <span
            key={i}
            className={cn(
              "h-[7px] w-[7px] rounded-full transition-all duration-500",
              i / 10 < progress
                ? "bg-[#ff5a47] shadow-[0_0_5px_rgb(255_90_71/0.8)]"
                : "bg-white/15"
            )}
          />
        ))}
      </span>
    </div>
  );
}

/* ---------------- alarm clock ---------------- */

function buildTicks() {
  const ticks: React.ReactNode[] = [];
  for (let i = 0; i < 12; i++) {
    const a = (i * Math.PI) / 6;
    const x1 = 50 + 34 * Math.sin(a);
    const y1 = 58 - 34 * Math.cos(a);
    const x2 = 50 + 38 * Math.sin(a);
    const y2 = 58 - 38 * Math.cos(a);
    ticks.push(
      <line
        key={i}
        x1={x1}
        y1={y1}
        x2={x2}
        y2={y2}
        stroke="#3a302a"
        strokeWidth={i % 3 === 0 ? 2.4 : 1.2}
        opacity="0.55"
      />
    );
  }
  return ticks;
}

function AlarmFace({ mm, ss, progress }: { mm: string; ss: string; progress: number }) {
  const R = 40;
  const angle = progress * 2 * Math.PI;
  const large = progress > 0.5;
  const x = 50 + R * Math.sin(angle);
  const y = 50 - R * Math.cos(angle);
  const sweep = large ? 1 : 0;
  const sec = parseInt(ss, 10);
  const ticks = buildTicks();
  return (
    <div className="flex items-center gap-5">
      <svg viewBox="0 0 100 108" width="86" height="93" aria-hidden>
        <ellipse cx="50" cy="102" rx="26" ry="3.5" fill="#3a302a" opacity="0.12" />
        {/* bells */}
        <circle cx="26" cy="12" r="10" fill="none" stroke="#a9845a" strokeWidth="4.5" />
        <circle cx="74" cy="12" r="10" fill="none" stroke="#a9845a" strokeWidth="4.5" />
        <line x1="33" y1="19" x2="42" y2="27" stroke="#a9845a" strokeWidth="4" strokeLinecap="round" />
        <line x1="67" y1="19" x2="58" y2="27" stroke="#a9845a" strokeWidth="4" strokeLinecap="round" />
        {/* face */}
        <circle cx="50" cy="58" r="42" fill="#fdfbf7" stroke="#a9845a" strokeWidth="4.5" />
        {/* remaining sector */}
        <path
          d={`M50 58 L50 18 A${R} ${R} 0 ${sweep} ${x} ${y} Z`}
          fill="#e24234"
          opacity="0.16"
        />
        {/* ticks */}
        {ticks}
        {/* second hand */}
        <line
          x1="50"
          y1="58"
          x2={50 + 30 * Math.sin((sec * Math.PI) / 30)}
          y2={58 - 30 * Math.cos((sec * Math.PI) / 30)}
          stroke="#e24234"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
        <circle cx="50" cy="58" r="3" fill="#3a302a" />
      </svg>
      <div className="flex flex-col gap-1">
        <TimeText mm={mm} ss={ss} />
        <span className="text-[0.6rem] font-semibold uppercase tracking-wider text-ink-mute">
          remaining
        </span>
      </div>
    </div>
  );
}

/* ---------------- cat / shiba ---------------- */

function AnimalFace({
  name,
  mm,
  ss,
  progress,
}: {
  name: "cat" | "shiba";
  mm: string;
  ss: string;
  progress: number;
}) {
  const R = 15;
  const c = 2 * Math.PI * R;
  return (
    <div className="flex items-center gap-5">
      <div className="relative">
        <StickerImg name={name} size={92} alt={`${name} holding a clock`} wiggle={false} />
        <svg viewBox="0 0 44 44" width="44" height="44" className="absolute -bottom-2 -right-3" aria-hidden>
          <circle cx="22" cy="22" r="20" fill="#fdfbf7" stroke="#3a302a" strokeWidth="2.5" />
          <circle
            cx="22"
            cy="22"
            r={R}
            fill="none"
            stroke="#e24234"
            strokeWidth="3"
            strokeLinecap="round"
            strokeDasharray={`${c * progress} ${c}`}
            transform="rotate(-90 22 22)"
          />
          <line x1="22" y1="22" x2="22" y2="12" stroke="#3a302a" strokeWidth="1.8" strokeLinecap="round" />
          <line x1="22" y1="22" x2="29" y2="25" stroke="#3a302a" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
      </div>
      <TimeText mm={mm} ss={ss} />
    </div>
  );
}
