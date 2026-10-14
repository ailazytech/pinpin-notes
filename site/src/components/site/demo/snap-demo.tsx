"use client";

import { useEffect, useRef, useState } from "react";
import { animate, motion, useMotionValue } from "framer-motion";
import { Magnet, Move } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { NotePaper, PinHead } from "../ui";

/**
 * Chapter 10 media: drag the butter note near its sage neighbor —
 * the guide lights up, and on release it springs into a tidy 10 px gap.
 * (Hold Alt to float free.)
 */
export function SnapDemo() {
  const { dict } = useI18n();
  const d = dict.demos.snap;
  const boxRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<HTMLDivElement>(null);
  const targetRef = useRef<HTMLDivElement>(null);
  const guideRef = useRef<HTMLDivElement>(null);
  const [near, setNear] = useState(false);
  const [snapped, setSnapped] = useState(false);
  const [altHeld, setAltHeld] = useState(false);

  const x = useMotionValue(0);
  const y = useMotionValue(0);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "Alt") setAltHeld(true);
    };
    const up = (e: KeyboardEvent) => {
      if (e.key === "Alt") setAltHeld(false);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, []);

  /* keep the dashed guide inside the 10px gap */
  useEffect(() => {
    const guide = guideRef.current;
    const box = boxRef.current;
    const target = targetRef.current;
    if (!guide || !box || !target) return;
    const b = box.getBoundingClientRect();
    const t = target.getBoundingClientRect();
    guide.style.left = `${t.left - b.left - 11}px`;
    guide.style.top = `${t.top - b.top + 6}px`;
    guide.style.height = `${t.height - 12}px`;
  }, [near, snapped]);

  /** live proximity check while dragging (no value fights — just the guide) */
  const onDrag = () => {
    const drag = dragRef.current?.getBoundingClientRect();
    const target = targetRef.current?.getBoundingClientRect();
    if (!drag || !target) return;
    if (altHeld) {
      setNear(false);
      return;
    }
    const THRESH = 30;
    const nearX =
      Math.abs(target.left - drag.right) <= THRESH ||
      Math.abs(drag.left - target.right) <= THRESH;
    const nearY =
      Math.abs(target.top - drag.top) <= THRESH ||
      Math.abs(target.bottom - drag.bottom) <= THRESH;
    setNear(nearX || nearY);
  };

  /** on release, spring into alignment */
  const onDragEnd = () => {
    setNear(false);
    if (altHeld) return;
    const drag = dragRef.current?.getBoundingClientRect();
    const target = targetRef.current?.getBoundingClientRect();
    if (!drag || !target) return;

    const THRESH = 30;
    const GAP = 10;
    const gapLeft = target.left - drag.right;
    const gapRight = drag.left - target.right;
    const dTop = target.top - drag.top;
    const dBottom = target.bottom - drag.bottom;

    let snapX: number | null = null;
    let snapY: number | null = null;

    if (Math.abs(gapLeft) <= THRESH || Math.abs(gapRight) <= THRESH) {
      snapX =
        Math.abs(gapLeft) <= Math.abs(gapRight)
          ? x.get() + gapLeft - GAP
          : x.get() + GAP - gapRight;
    }
    if (Math.abs(dTop) <= THRESH || Math.abs(dBottom) <= THRESH) {
      snapY =
        Math.abs(dTop) <= Math.abs(dBottom) ? y.get() + dTop : y.get() + dBottom;
    }

    if (snapX !== null || snapY !== null) {
      setSnapped(true);
      animate(x, snapX ?? x.get(), { type: "spring", stiffness: 420, damping: 26 });
      animate(y, snapY ?? y.get(), { type: "spring", stiffness: 420, damping: 26 });
    } else {
      setSnapped(false);
    }
  };

  const guideActive = near || snapped;

  return (
    <div className="flex size-full flex-col items-center justify-center gap-3 p-4">
      <div
        ref={boxRef}
        className="relative h-[196px] w-full max-w-[400px] overflow-hidden rounded-xl border border-line/80 bg-paper/70"
      >
        {/* screen-edge rails */}
        <div className="absolute inset-y-0 left-0 w-[3px] bg-pin/10" aria-hidden />
        <div className="absolute inset-y-0 right-0 w-[3px] bg-pin/10" aria-hidden />

        {/* snap guide inside the gap */}
        <motion.div
          ref={guideRef}
          animate={{ opacity: guideActive ? 1 : 0 }}
          transition={{ duration: 0.18 }}
          className="pointer-events-none absolute z-10"
          aria-hidden
        >
          <div className="h-full border-s-2 border-dashed border-pin/60" />
          <span className="absolute start-[50%] top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-pin px-1.5 py-0.5 text-[0.56rem] font-bold text-white">
            {d.gap}
          </span>
        </motion.div>

        {/* target note */}
        <div ref={targetRef} className="absolute right-[26px] top-[36px]">
          <NotePaper color="sage" className="w-[120px] px-3 py-2.5">
            <span className="font-hand text-[14px] text-ink/75">already here</span>
          </NotePaper>
        </div>

        {/* draggable note */}
        <motion.div
          ref={dragRef}
          style={{ x, y }}
          drag
          dragMomentum={false}
          dragElastic={0}
          dragConstraints={boxRef}
          onDrag={onDrag}
          onDragEnd={onDragEnd}
          whileDrag={{ scale: 1.04, rotate: 1.5 }}
          className="absolute left-[18px] top-[118px] cursor-grab touch-none active:cursor-grabbing"
        >
          <NotePaper color="butter" lift className="w-[120px] px-3 pb-2.5 pt-3.5">
            <PinHead className="left-1/2 top-[-8px]" />
            <span className="flex items-center gap-1 font-hand text-[14px] text-ink/85">
              <Move className="size-3.5 text-ink/40" />
              drag me
            </span>
          </NotePaper>
        </motion.div>
      </div>

      <p className="flex items-center gap-1.5 text-center text-[0.68rem] font-semibold text-ink-mute">
        <Magnet className="size-3.5 shrink-0 text-pin" />
        {d.hint}
        <span className="hidden text-ink-mute/60 sm:inline">· {d.alt}</span>
      </p>
    </div>
  );
}
