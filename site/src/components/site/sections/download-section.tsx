"use client";

import { Download, Check, Monitor, WifiOff, FileLock2, Scale } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading } from "../ui";
import { LogoMark } from "../logo";
import { GithubMark } from "../github-link";
import { siteConfig } from "@/lib/site-config";

const REQ_ICONS = [Monitor, null, WifiOff, FileLock2, Scale];

export function DownloadSection({ onDownload }: { onDownload: () => void }) {
  const { dict } = useI18n();

  return (
    <section id="download" className="scroll-mt-20 py-16 sm:py-24">
      <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
        <Reveal>
          <SectionHeading
            eyebrow={dict.download.eyebrow}
            title={dict.download.title}
            sub={dict.download.sub}
          />
        </Reveal>

        <Reveal delay={0.08}>
          <div className="paper-grain relative mx-auto max-w-3xl overflow-hidden rounded-2xl border border-line bg-card p-8 shadow-soft sm:p-10">
            {/* floating deco stickers */}
            <span className="absolute -right-6 -top-6 rotate-12 opacity-90" aria-hidden>
              <LogoMark size={84} />
            </span>

            <div className="relative flex flex-col items-start gap-6">
              <div className="flex flex-col items-start gap-2">
                <Button
                  size="lg"
                  onClick={onDownload}
                  className="h-14 rounded-full bg-ink px-8 text-base font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift active:translate-y-0"
                >
                  <Download className="size-5" />
                  <span className="ml-2">{dict.download.button}</span>
                  <span className="ml-2 rounded-full bg-paper/20 px-2.5 py-0.5 text-[0.7rem] font-semibold">
                    {dict.download.size}
                  </span>
                </Button>
                <p className="text-[0.78rem] text-ink-mute">{dict.download.pending}</p>
              </div>

              <div className="w-full border-t border-line/80 pt-6">
                <h3 className="mb-3 text-[0.72rem] font-bold uppercase tracking-[0.18em] text-ink-mute">
                  {dict.download.reqTitle}
                </h3>
                <ul className="grid gap-2.5 sm:grid-cols-2">
                  {dict.download.reqs.map((req, i) => {
                    const Icon = REQ_ICONS[i] ?? Check;
                    return (
                      <li
                        key={i}
                        className="flex items-start gap-2.5 rounded-lg bg-paper/70 px-3.5 py-2.5 text-[0.82rem] leading-relaxed text-ink-soft"
                      >
                        {Icon ? (
                          <Icon className="mt-0.5 size-4 shrink-0 text-pin" aria-hidden />
                        ) : (
                          <Check className="mt-0.5 size-4 shrink-0 text-pin" aria-hidden />
                        )}
                        {req}
                      </li>
                    );
                  })}
                </ul>
              </div>

              <p
                id="license"
                className="flex flex-wrap items-center gap-x-2 gap-y-1 border-t border-line/80 pt-5 text-[0.78rem] text-ink-mute"
              >
                <Scale className="size-4 text-ink-soft" aria-hidden />
                {dict.download.license}
                <span className="text-ink-mute/60">·</span>
                <a
                  href={siteConfig.githubUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 underline decoration-line underline-offset-4 transition-colors hover:text-ink"
                >
                  <GithubMark className="size-3.5" />
                  {dict.download.sourceOnGithub}
                </a>
              </p>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
