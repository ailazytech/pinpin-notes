"use client";

import { asset } from "@/lib/site-config";
import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { Lock, LockOpen, KeyRound } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead, WashiTape, useScramble } from "../ui";

/** Chapter 8 media: a polaroid and a password note in disguise. */
export function MemoriesDemo() {
  useI18n();
  return (
    <div className="flex size-full flex-col items-center justify-center gap-4 p-4 sm:flex-row sm:gap-6">
      <Polaroid />
      <PasswordNote />
    </div>
  );
}

function Polaroid() {
  const [loaded, setLoaded] = useState(false);
  const photoRef = useRef<HTMLImageElement | null>(null);

  useEffect(() => {
    const img = photoRef.current;
    if (!img || !img.complete) return;
    const t = window.setTimeout(() => setLoaded(true), 0);
    return () => window.clearTimeout(t);
  }, []);

  return (
    <motion.div
      whileHover={{ rotate: 0, scale: 1.03 }}
      className="relative w-[150px] shrink-0 -rotate-2 rounded-[4px] bg-white p-[7px] pb-[26px] shadow-note sm:w-[168px]"
    >
      <WashiTape color="var(--note-mist)" pattern={1} className="left-1/2 top-[-11px] -translate-x-1/2" rotate={-3} width={62} />
      <div className="relative aspect-[6/5] overflow-hidden bg-paper-deep">
        {!loaded && <div className="shimmer absolute inset-0" aria-hidden />}
        <img
          ref={photoRef}
          src={asset("img/photos/mochi.webp")}
          alt="cozy desk polaroid"
          className="absolute inset-0 size-full object-cover transition-opacity duration-500"
          style={{ opacity: loaded ? 1 : 0 }}
          onLoad={() => setLoaded(true)}
        />
      </div>
      <p className="absolute inset-x-0 bottom-[5px] text-center font-hand text-[15px] text-[#4a4238]">
        my cozy desk
      </p>
    </motion.div>
  );
}

function PasswordNote() {
  const { dict } = useI18n();
  const d = dict.demos.password;
  const [state, setState] = useState<"locked" | "scrambling" | "open">("locked");
  const [input, setInput] = useState("");
  const [shake, setShake] = useState(0);

  const decoyText = d.decoy.join(" · ");
  const display = useScramble(
    state === "open" ? d.secret : decoyText,
    state === "scrambling",
    850
  );

  const unlock = () => {
    if (!input.trim()) {
      setShake((s) => s + 1);
      return;
    }
    setState("scrambling");
    window.setTimeout(() => setState("open"), 900);
  };

  const lock = () => {
    setState("scrambling");
    window.setTimeout(() => setState("locked"), 900);
  };

  return (
    <div className="flex w-full max-w-[220px] flex-col items-center gap-2.5">
      <motion.div
        key={shake}
        animate={shake ? { x: [0, -7, 7, -5, 5, 0] } : { x: 0 }}
        transition={{ duration: 0.45 }}
        className="w-full"
      >
        <NotePaper
          color={state === "open" ? "grape" : "sage"}
          className="w-full px-4 py-3.5 pt-4"
        >
          <PinHead className="left-1/2 top-[-8px]" />
          <p className="flex items-center gap-1.5 text-[0.66rem] font-bold uppercase tracking-[0.14em] text-ink/60">
            {state === "open" ? (
              <LockOpen className="size-3" />
            ) : (
              <Lock className="size-3" />
            )}
            {d.title}
          </p>
          <p className="mt-1 min-h-[42px] font-hand text-[16px] leading-snug text-ink/85">
            {display}
          </p>
        </NotePaper>
      </motion.div>

      {state === "locked" ? (
        <div className="flex items-center gap-1.5">
          <label className="sr-only" htmlFor="demo-password">
            {d.inputLabel}
          </label>
          <div className="relative">
            <KeyRound className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-ink-mute" />
            <input
              id="demo-password"
              type="password"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && unlock()}
              placeholder={d.hint}
              className="h-9 w-[132px] rounded-full border border-line bg-paper pl-8 pr-3 text-[0.78rem] text-ink shadow-inner outline-none placeholder:text-ink-mute/70 focus:border-pin/50"
            />
          </div>
          <button
            type="button"
            onClick={unlock}
            className="grid h-9 shrink-0 place-items-center rounded-full bg-ink px-3.5 text-[0.72rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
          >
            {d.unlock}
          </button>
        </div>
      ) : (
        <button
          type="button"
          onClick={lock}
          disabled={state === "scrambling"}
          className="rounded-full border border-line bg-paper px-4 py-1.5 text-[0.72rem] font-bold text-ink-soft shadow-note transition-all hover:-translate-y-0.5 hover:text-ink disabled:opacity-50"
        >
          {d.lock}
        </button>
      )}
    </div>
  );
}
