import { useEffect, useState } from "react";
import { LogoMark } from "./logo";
import { useI18n } from "@/i18n/locale-context";

export type CurtainPhase = "hidden" | "cover" | "hold" | "reveal";

/**
 * Full-screen sticky note used as the intro loader and the page-transition
 * curtain: it slaps onto the screen, its bottom-right corner lifts a little,
 * then it is peeled off towards the top-left to show the next page.
 *
 * The peel is pure CSS (see `.pb-peel` in globals.css): a frame rotated 45°
 * clips the note to the side of the fold line, and a mirrored copy of the
 * note (its back side) is the flap. `--fold` moves the fold line.
 */
export function Curtain({
  phase,
  intro,
  occasion,
}: {
  phase: CurtainPhase;
  intro: boolean;
  occasion: number;
}) {
  const { dict } = useI18n();
  const [lineIdx, setLineIdx] = useState(0);

  useEffect(() => {
    if (!(intro && phase === "hold")) return;
    const t = setInterval(
      () => setLineIdx((i) => (i + 1) % dict.curtain.loadingLines.length),
      1150
    );
    return () => clearInterval(t);
  }, [intro, phase, dict.curtain.loadingLines.length]);

  // the logo stays on the note while it is peeled off
  const covered = phase !== "hidden";

  return (
    <div
      aria-hidden={phase === "hidden"}
      data-phase={phase}
      className="pb-peel pointer-events-none fixed inset-0 z-[80]"
    >
      {/* key: restart the slap animation on every page change */}
      <div key={occasion} className="pb-peel-stage">
        <div className="pb-peel-frame">
          <div className="pb-peel-clip">
            <div className="pb-peel-origin">
              <div className="pb-peel-note paper-grain">
                <div className="pb-peel-glue" />
                {covered && (
                  <div className="pb-peel-content">
                    <LogoMark size={112} play delay={intro ? 0.3 : 0.12} />
                    {intro && (
                      <div className="flex flex-col items-center gap-3">
                        <span className="font-hand text-xl text-ink-soft">
                          {dict.curtain.loadingLines[lineIdx % dict.curtain.loadingLines.length]}
                        </span>
                        <div className="shimmer h-[5px] w-40 rounded-full" />
                      </div>
                    )}
                  </div>
                )}
              </div>
              <div className="pb-peel-flap">
                <div className="pb-peel-back" />
              </div>
            </div>
          </div>
        </div>
      </div>
      <span className="sr-only">{dict.curtain.loading}</span>
    </div>
  );
}
