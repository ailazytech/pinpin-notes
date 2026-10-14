"use client";

import { asset } from "@/lib/site-config";
import { useEffect, useRef, useState } from "react";
import { Clapperboard, Play } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading } from "../ui";
import { siteConfig } from "@/lib/site-config";

export function VideoSection() {
  const { dict } = useI18n();
  const [state, setState] = useState<"loading" | "ready" | "missing">("loading");
  const probeRef = useRef<HTMLVideoElement | null>(null);

  /* the video may resolve before hydration — catch that */
  useEffect(() => {
    const v = probeRef.current;
    if (!v) return;
    const failed = !!v.error;
    const ok = !failed && v.readyState >= 1;
    if (!failed && !ok) return;
    const t = window.setTimeout(
      () => setState(failed ? "missing" : "ready"),
      0
    );
    return () => window.clearTimeout(t);
  }, []);

  return (
    <section className="mx-auto w-full max-w-6xl px-4 py-16 sm:px-6 sm:py-20">
      <Reveal>
        <SectionHeading
          eyebrow={dict.video.eyebrow}
          title={dict.video.title}
          sub={dict.video.sub}
        />
      </Reveal>

      <Reveal delay={0.1}>
        <div className="paper-grain relative mx-auto aspect-video w-full max-w-3xl overflow-hidden rounded-2xl border border-line bg-paper-deep/60 shadow-lift">
          {state === "loading" && <div className="shimmer absolute inset-0" aria-hidden />}

          {state === "ready" && (
            <video
              src={asset(siteConfig.media.videoSrc)}
              controls
              playsInline
              preload="metadata"
              className="absolute inset-0 size-full object-cover"
              onError={() => setState("missing")}
            />
          )}

          {state === "missing" && (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-5 p-6 text-center">
              {/* playful little film strip */}
              <div className="flex items-center gap-2 opacity-70">
                {[0, 1, 2, 3, 4].map((i) => (
                  <span
                    key={i}
                    className="animate-float size-2.5 rounded-full bg-ink-mute/60"
                    style={{ animationDelay: `${i * 0.35}s` }}
                  />
                ))}
              </div>
              <div className="group flex size-20 items-center justify-center rounded-full border-2 border-dashed border-ink-mute/50 bg-paper transition-transform duration-300 group-hover:scale-105">
                <Play className="size-8 translate-x-[2px] text-ink-soft" />
              </div>
              <div className="flex flex-col gap-1.5">
                <p className="flex items-center gap-2 font-display text-lg font-semibold text-ink">
                  <Clapperboard className="size-5 text-pin" />
                  {dict.video.makingTitle}
                </p>
                <p className="max-w-md text-[0.83rem] leading-relaxed text-ink-soft">
                  {dict.video.makingDesc}
                </p>
              </div>
              <code className="rounded-md border border-line bg-paper px-2.5 py-1 font-mono text-[0.7rem] text-ink-mute">
                {siteConfig.media.videoSrc}
              </code>
            </div>
          )}

          {/* silent probe */}
          <video
            ref={probeRef}
            src={asset(siteConfig.media.videoSrc)}
            preload="metadata"
            className="hidden"
            aria-hidden
            onLoadedMetadata={() => setState("ready")}
            onError={() => setState("missing")}
          />
        </div>
      </Reveal>
    </section>
  );
}
