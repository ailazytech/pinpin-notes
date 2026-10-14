import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { motion } from "framer-motion";
import { RotateCcw } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { StickerImg, STICKER_NAMES, type StickerName } from "../ui";
import { cn } from "@/lib/utils";

/** Where a sticker went: page coordinates of its center (when peeled off), rotation in degrees, scale. */
type Placed = { x?: number; y?: number; rot: number; scale: number };
type Board = Record<string, Placed>;

const STORAGE_KEY = "pinpin-site-stickers";
const SIZE = 58;

function loadBoard(): Board {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    const board = raw ? (JSON.parse(raw) as Board) : {};
    // keep stickers on the page if the window got narrower since last time
    const maxX = document.documentElement.scrollWidth - SIZE / 2;
    for (const p of Object.values(board)) if (p.x !== undefined) p.x = Math.min(Math.max(p.x, SIZE / 2), maxX);
    return board;
  } catch {
    return {};
  }
}

/**
 * Slow conveyor of the built-in stickers. Like in the app, they can be peeled off and stuck anywhere:
 * drag one out, click to select, wheel to rotate, Shift+wheel to resize. Everything is remembered in
 * localStorage; "Reset" flies them all back into their slots.
 */
export function StickerMarquee() {
  const { dict } = useI18n();
  const row = [...STICKER_NAMES, ...STICKER_NAMES];
  const [board, setBoard] = useState<Board>({});
  const [selected, setSelected] = useState<string | null>(null);
  const [dragging, setDragging] = useState<string | null>(null);
  const [returning, setReturning] = useState<Record<string, { x: number; y: number }> | null>(null);
  const boardRef = useRef(board);
  boardRef.current = board;

  useEffect(() => setBoard(loadBoard()), []);
  const save = useCallback((next: Board) => {
    setBoard(next);
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      /* storage blocked */
    }
  }, []);

  // click anywhere else: deselect
  useEffect(() => {
    if (!selected) return;
    const off = (e: PointerEvent) => {
      if (!(e.target as HTMLElement).closest?.("[data-sticker]")) setSelected(null);
    };
    window.addEventListener("pointerdown", off);
    return () => window.removeEventListener("pointerdown", off);
  }, [selected]);

  // wheel on the selected sticker: rotate; Shift+wheel: resize (only while selected, so page scrolling still works)
  useEffect(() => {
    if (!selected) return;
    const onWheel = (e: WheelEvent) => {
      const el = (e.target as HTMLElement).closest?.("[data-sticker]") as HTMLElement | null;
      if (!el || el.dataset.sticker !== selected) return;
      e.preventDefault();
      const cur = boardRef.current[selected] ?? { rot: 0, scale: 1 };
      const delta = e.deltaY || e.deltaX;
      const next = e.shiftKey
        ? { ...cur, scale: Math.min(3, Math.max(0.5, cur.scale * (delta > 0 ? 0.92 : 1.08))) }
        : { ...cur, rot: cur.rot + (delta > 0 ? 8 : -8) };
      save({ ...boardRef.current, [selected]: next });
    };
    window.addEventListener("wheel", onWheel, { passive: false });
    return () => window.removeEventListener("wheel", onWheel);
  }, [selected, save]);

  // drag: from the conveyor (peels it off) or an already placed sticker
  const startDrag = (key: string, e: React.PointerEvent) => {
    if (e.button !== 0 || returning) return;
    e.preventDefault();
    const el = e.currentTarget as HTMLElement;
    const r = el.getBoundingClientRect();
    const start = { x: e.clientX, y: e.clientY };
    const cur = boardRef.current[key] ?? { rot: 0, scale: 1 };
    const center = cur.x !== undefined ? { x: cur.x, y: cur.y! } : { x: r.left + r.width / 2 + window.scrollX, y: r.top + r.height / 2 + window.scrollY };
    let moved = false;
    const move = (ev: PointerEvent) => {
      const dx = ev.clientX - start.x, dy = ev.clientY - start.y;
      if (!moved && Math.hypot(dx, dy) < 4) return;
      moved = true;
      setDragging(key);
      save({ ...boardRef.current, [key]: { ...cur, x: center.x + dx, y: center.y + dy } });
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setDragging(null);
      // a click without moving selects it
      setSelected(key);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  // reset: every placed sticker flies back to its slot, then the board is cleared
  const reset = () => {
    setSelected(null);
    const targets: Record<string, { x: number; y: number }> = {};
    for (const key of Object.keys(boardRef.current)) {
      const slot = document.querySelector(`[data-slot="${key}"]`);
      if (!slot) continue;
      const r = slot.getBoundingClientRect();
      targets[key] = { x: r.left + r.width / 2 + window.scrollX, y: r.top + r.height / 2 + window.scrollY };
    }
    setReturning(targets);
    window.setTimeout(() => {
      save({});
      setReturning(null);
    }, 750);
  };

  const changed = Object.keys(board).length > 0;
  const paused = !!selected || !!dragging || !!returning;

  return (
    <div className="border-y border-line/70 bg-paper/60 py-5">
      <div className="mb-3 flex flex-col items-center gap-1 px-4 text-center">
        <p className="text-[0.68rem] font-bold uppercase tracking-[0.24em] text-ink-mute">{dict.marquee.label}</p>
        <p className="flex flex-wrap items-center justify-center gap-x-2 gap-y-1 text-[0.74rem] text-ink-soft">
          <span>{dict.marquee.hint}</span>
          {changed && (
            <button
              type="button"
              onClick={reset}
              className="inline-flex items-center gap-1 rounded-full border border-line bg-paper px-2.5 py-0.5 font-semibold text-ink shadow-note transition-transform hover:-translate-y-0.5"
            >
              <RotateCcw className="size-3" />
              {dict.marquee.reset}
            </button>
          )}
        </p>
      </div>
      <div className="relative overflow-hidden">
        <div className="pointer-events-none absolute inset-y-0 left-0 z-10 w-16 bg-gradient-to-r from-background to-transparent" />
        <div className="pointer-events-none absolute inset-y-0 right-0 z-10 w-16 bg-gradient-to-l from-background to-transparent" />
        <div className={cn("animate-marquee flex w-max items-center gap-7 px-4 py-3", paused && "[animation-play-state:paused]")}>
          {row.map((name, i) => {
            const key = `${name}-${i}`;
            const p = board[key];
            if (p?.x !== undefined) {
              // peeled off: an empty slot stays behind
              return (
                <span
                  key={key}
                  data-slot={key}
                  aria-hidden
                  className="inline-block shrink-0 rounded-[42%] border-2 border-dashed border-ink-mute/30"
                  style={{ width: SIZE, height: SIZE }}
                />
              );
            }
            return (
              <span
                key={key}
                data-slot={key}
                data-sticker={key}
                onPointerDown={(e) => startDrag(key, e)}
                className={cn("inline-block shrink-0 cursor-grab touch-none select-none", selected === key && "rounded-xl outline-2 outline-offset-4 outline-dashed outline-pin/70")}
                style={{ transform: `rotate(${p?.rot ?? 0}deg) scale(${p?.scale ?? 1})`, transition: "transform 0.25s" }}
              >
                <StickerImg name={name} size={SIZE} wiggle={false} />
              </span>
            );
          })}
        </div>
      </div>
      {typeof document !== "undefined" &&
        createPortal(
          <>
            {Object.entries(board).map(([key, p]) => {
              if (p.x === undefined || p.y === undefined) return null;
              const name = key.slice(0, key.lastIndexOf("-")) as StickerName;
              const back = returning?.[key];
              return (
                <motion.span
                  key={key}
                  data-sticker={key}
                  onPointerDown={(e) => startDrag(key, e)}
                  className={cn(
                    "absolute z-[60] inline-block cursor-grab touch-none select-none",
                    dragging === key && "cursor-grabbing drop-shadow-[0_10px_14px_rgb(60_40_20/0.3)]",
                    selected === key && "rounded-xl outline-2 outline-offset-4 outline-dashed outline-pin/70"
                  )}
                  style={{ width: SIZE, height: SIZE, marginLeft: -SIZE / 2, marginTop: -SIZE / 2 }}
                  initial={false}
                  animate={back ? { left: back.x, top: back.y, rotate: 0, scale: 1 } : { left: p.x, top: p.y, rotate: p.rot, scale: p.scale }}
                  transition={back ? { type: "spring", stiffness: 160, damping: 18 } : { duration: dragging === key ? 0 : 0.2 }}
                >
                  <StickerImg name={name} size={SIZE} wiggle={false} />
                </motion.span>
              );
            })}
          </>,
          document.body
        )}
    </div>
  );
}
