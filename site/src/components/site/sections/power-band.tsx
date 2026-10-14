"use client";

import { BatteryCharging, Cpu, FileBox, WifiOff } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, WashiTape } from "../ui";

const POINT_ICONS = [Cpu, BatteryCharging, BatteryCharging, FileBox];

export function PowerBand() {
  const { dict } = useI18n();

  return (
    <section id="power" className="relative scroll-mt-20 overflow-hidden py-16 sm:py-24">
      {/* dotted backdrop */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0"
        style={{
          backgroundImage:
            "radial-gradient(rgb(58 48 42 / 0.05) 1px, transparent 1.4px)",
          backgroundSize: "24px 24px",
        }}
      />

      <div className="relative mx-auto w-full max-w-6xl px-4 sm:px-6">
        <Reveal>
          <SectionHeading
            eyebrow={dict.power.eyebrow}
            title={dict.power.title}
            sub={dict.power.desc}
          />
        </Reveal>

        <Reveal delay={0.08}>
          <div className="paper-grain relative mx-auto max-w-4xl rounded-2xl border border-line bg-card p-6 shadow-soft sm:p-10">
            <WashiTape
              color="var(--note-sage)"
              pattern={1}
              className="left-1/2 top-[-13px] -translate-x-1/2"
              rotate={-2}
              width={110}
            />

            <dl className="grid grid-cols-2 gap-6 sm:grid-cols-4 sm:gap-4">
              {dict.power.stats.map((s, i) => {
                const Icon = POINT_ICONS[i] ?? WifiOff;
                return (
                  <div key={s.label} className="flex flex-col items-center gap-2 text-center">
                    <span className="grid size-11 place-items-center rounded-full bg-paper-deep text-ink-soft">
                      <Icon className="size-5" />
                    </span>
                    <dd className="font-display text-3xl font-semibold tracking-tight text-ink">
                      {s.value}
                    </dd>
                    <dt className="text-[0.7rem] font-bold uppercase tracking-[0.14em] text-ink-mute">
                      {s.label}
                    </dt>
                  </div>
                );
              })}
            </dl>

            <ul className="mt-8 grid gap-2.5 border-t border-line/80 pt-6 sm:grid-cols-2">
              {dict.power.points.map((p) => (
                <li key={p} className="flex items-start gap-2.5 text-[0.84rem] leading-relaxed text-ink-soft">
                  <WifiOff className="mt-0.5 size-4 shrink-0 text-pin" aria-hidden />
                  {p}
                </li>
              ))}
            </ul>

            <p className="mt-5 text-end font-hand text-base text-ink-mute">
              {dict.power.footnote}
            </p>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
