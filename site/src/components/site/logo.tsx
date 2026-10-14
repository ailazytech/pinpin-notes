import { motion, useReducedMotion } from "framer-motion";
import { asset } from "@/lib/site-config";
import { cn } from "@/lib/utils";

/**
 * The PinPin mark: a smiling butter-yellow sticky note with mint washi tape, a red pin
 * and a die-cut thumbs-up sticker (the same artwork as the app icon).
 * When `play` is true it drops in, squashes on landing and settles with a tiny
 * damped swing — the signature loading animation.
 */
export function LogoMark({
  size = 96,
  play = false,
  delay = 0,
  className,
}: {
  size?: number;
  play?: boolean;
  delay?: number;
  className?: string;
}) {
  const reduce = useReducedMotion();
  const animated = play && !reduce;
  const src = asset(size > 96 ? "icon-512.png" : "icon-192.png");

  return (
    <span
      className={cn("relative inline-block select-none", className)}
      style={{ width: size, height: size }}
      role="img"
      aria-label="PinPin Notes logo"
    >
      {/* ground shadow */}
      <motion.span
        aria-hidden
        className="absolute rounded-[50%] bg-ink"
        style={{ left: "22%", right: "22%", bottom: "-2%", height: "7%", filter: "blur(3px)" }}
        initial={animated ? { opacity: 0, scale: 0.5 } : false}
        animate={animated ? { opacity: 0.16, scale: 1 } : { opacity: 0.12 }}
        transition={{ delay: delay + 0.18, duration: 0.3 }}
      />
      <motion.img
        src={src}
        alt=""
        draggable={false}
        className="absolute inset-0 size-full"
        style={{ originY: 1 }}
        initial={animated ? { y: -size * 0.45, opacity: 0, scaleX: 0.92, scaleY: 1.08, rotate: -6 } : false}
        animate={
          animated
            ? {
                y: [-size * 0.45, 0, 0, 0],
                opacity: [0, 1, 1, 1],
                scaleX: [0.92, 1.08, 0.98, 1],
                scaleY: [1.08, 0.9, 1.02, 1],
                rotate: [-6, 2, -1, 0],
              }
            : undefined
        }
        transition={animated ? { delay, duration: 0.7, times: [0, 0.4, 0.7, 1], ease: "easeOut" } : undefined}
      />
    </span>
  );
}

export function LogoWord({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "font-display text-[1.35rem] font-semibold leading-none tracking-tight text-ink",
        className
      )}
    >
      PinPin
      <span className="ml-[2px] inline-block size-[6px] translate-y-[-2px] rounded-full bg-pin align-baseline" aria-hidden />
    </span>
  );
}
