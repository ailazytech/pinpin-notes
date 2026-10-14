"use client";

import { FileJson, FolderOpen, Save, Wand2 } from "lucide-react";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, NotePaper } from "../ui";
import { LOCALES, LOCALE_META } from "@/i18n/config";

const STEP_ICONS = [FolderOpen, FileJson, Save, Wand2];

export function LanguagesSection() {
  const { dict, locale } = useI18n();

  return (
    <section id="langs" className="scroll-mt-20 bg-paper/50 py-16 sm:py-24">
      <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
        <Reveal>
          <SectionHeading
            eyebrow={dict.langs.eyebrow}
            title={dict.langs.title}
            sub={dict.langs.desc}
          />
        </Reveal>

        <div className="grid gap-6 lg:grid-cols-2">
          {/* language chips + RTL demo */}
          <Reveal className="flex flex-col gap-5">
            <div className="paper-grain rounded-2xl border border-line bg-card p-6 shadow-soft">
              <div className="flex flex-wrap gap-2.5">
                {LOCALES.map((code) => (
                  <span
                    key={code}
                    className={`inline-flex items-center gap-2 rounded-full border px-3.5 py-1.5 text-[0.82rem] font-semibold ${
                      code === locale
                        ? "border-pin/40 bg-pin/10 text-ink"
                        : "border-line bg-paper text-ink-soft"
                    }`}
                    dir={LOCALE_META[code].dir}
                  >
                    {LOCALE_META[code].label}
                    {LOCALE_META[code].dir === "rtl" && (
                      <span className="rounded bg-paper-deep px-1.5 py-0.5 text-[0.6rem] font-bold tracking-wide text-ink-mute">
                        RTL
                      </span>
                    )}
                  </span>
                ))}
              </div>

              {/* RTL note demo */}
              <div className="mt-6">
                <p className="mb-2.5 text-[0.72rem] font-bold uppercase tracking-[0.16em] text-ink-mute">
                  {dict.langs.rtlTitle}
                </p>
                <NotePaper color="oat" className="px-5 py-4">
                  <p dir="rtl" className="text-end font-semibold leading-relaxed text-ink/85">
                    {dict.langs.rtlSample}
                  </p>
                  <p className="mt-2 text-[0.72rem] text-ink-mute">
                    {dict.langs.lineDirection}
                  </p>
                </NotePaper>
              </div>
            </div>
          </Reveal>

          {/* custom language file */}
          <Reveal delay={0.1} className="flex flex-col gap-5">
            <div className="paper-grain rounded-2xl border border-line bg-card p-6 shadow-soft">
              <h3 className="font-display text-lg font-semibold text-ink">
                {dict.langs.customTitle}
              </h3>
              <p className="mt-1.5 text-[0.85rem] leading-relaxed text-ink-soft">
                {dict.langs.customSub}
              </p>

              <div className="mt-4 overflow-hidden rounded-lg border border-line bg-[#211c17] shadow-note">
                <div className="flex items-center gap-1.5 border-b border-white/10 px-3.5 py-2">
                  <span className="size-2.5 rounded-full bg-[#e24234]" />
                  <span className="size-2.5 rounded-full bg-[#e8b84b]" />
                  <span className="size-2.5 rounded-full bg-[#8fb268]" />
                  <span className="ms-2 font-mono text-[0.68rem] text-white/60">
                    {dict.langs.jsonTitle}
                  </span>
                </div>
                <pre className="overflow-x-auto p-4 font-mono text-[0.72rem] leading-relaxed text-[#f0e9dc]">
                  {dict.langs.jsonLines.join("\n")}
                </pre>
              </div>

              <ol className="mt-4 flex flex-col gap-2">
                {dict.langs.customSteps.map((step, i) => {
                  const Icon = STEP_ICONS[i % STEP_ICONS.length];
                  return (
                    <li key={i} className="flex items-center gap-2.5 text-[0.82rem] leading-relaxed text-ink-soft">
                      <span className="grid size-6 shrink-0 place-items-center rounded-full bg-paper-deep text-ink-soft">
                        <Icon className="size-3.5" />
                      </span>
                      {step}
                    </li>
                  );
                })}
              </ol>
            </div>
          </Reveal>
        </div>
      </div>
    </section>
  );
}
