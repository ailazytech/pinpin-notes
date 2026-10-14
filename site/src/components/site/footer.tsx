"use client";

import { Heart, Pin } from "lucide-react";
import { LogoMark, LogoWord } from "./logo";
import { useI18n } from "@/i18n/locale-context";
import { LOCALES, LOCALE_META } from "@/i18n/config";
import type { NavigateFn } from "./header";
import { siteConfig } from "@/lib/site-config";

export function Footer({ navigate }: { navigate: NavigateFn }) {
  const { dict, locale, setLocale } = useI18n();
  const year = new Date().getFullYear();

  return (
    <footer className="torn-edge mt-auto bg-paper pt-16">
      <div className="mx-auto w-full max-w-6xl px-4 pb-10 sm:px-6">
        <div className="grid gap-10 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
          {/* brand */}
          <div className="flex flex-col gap-4">
            <div className="flex items-center gap-3">
              <LogoMark size={44} />
              <LogoWord className="text-xl" />
            </div>
            <p className="flex items-center gap-2 font-hand text-lg text-ink-soft">
              <Pin className="size-4 -translate-y-0.5 text-pin" />
              {dict.footer.tagline}
            </p>
            <p className="max-w-xs text-[0.8rem] leading-relaxed text-ink-mute">
              {dict.meta.description}
            </p>
          </div>

          {/* site */}
          <FooterCol title={dict.footer.siteTitle}>
            <FooterBtn onClick={() => navigate("home")}>{dict.footer.links.home}</FooterBtn>
            <FooterBtn onClick={() => navigate("qa")}>{dict.footer.links.qa}</FooterBtn>
            <FooterBtn onClick={() => navigate("custom")}>{dict.footer.links.custom}</FooterBtn>
            <FooterBtn onClick={() => navigate("order")}>{dict.footer.links.order}</FooterBtn>
            <FooterBtn onClick={() => navigate("donate")}>
              {dict.footer.links.donate}
            </FooterBtn>
            <FooterBtn onClick={() => navigate("home", "download")}>
              {dict.footer.links.download}
            </FooterBtn>
          </FooterCol>

          {/* project */}
          <FooterCol title={dict.footer.projectTitle}>
            <FooterBtn
              onClick={() =>
                siteConfig.githubUrl ? window.open(siteConfig.githubUrl, "_blank") : undefined
              }
            >
              {dict.footer.links.source}
            </FooterBtn>
            <FooterBtn onClick={() => navigate("home", "license")}>
              {dict.footer.links.license}
            </FooterBtn>
            <FooterBtn onClick={() => navigate("home", "power")}>
              {dict.power.eyebrow}
            </FooterBtn>
            <FooterBtn onClick={() => navigate("home", "langs")}>
              {dict.langs.eyebrow}
            </FooterBtn>
          </FooterCol>

          {/* languages */}
          <FooterCol title={dict.footer.languageTitle}>
            {LOCALES.map((code) => (
              <FooterBtn
                key={code}
                active={code === locale}
                dir={LOCALE_META[code].dir}
                onClick={() => setLocale(code)}
              >
                {LOCALE_META[code].label}
              </FooterBtn>
            ))}
          </FooterCol>
        </div>

        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t border-line/80 pt-6 text-[0.78rem] text-ink-mute sm:flex-row">
          <p>
            © {year} {dict.footer.copyright}
          </p>
          <p className="flex items-center gap-1.5">
            {dict.footer.crafted}
            <Heart className="size-3.5 fill-pin text-pin" />
            <span className="font-hand text-base text-ink-soft">
              {dict.donate.hearts}
            </span>
          </p>
        </div>
      </div>
    </footer>
  );
}

function FooterCol({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <nav className="flex flex-col gap-1" aria-label={title}>
      <h3 className="mb-2 text-[0.7rem] font-bold uppercase tracking-[0.18em] text-ink-mute">
        {title}
      </h3>
      {children}
    </nav>
  );
}

function FooterBtn({
  children,
  onClick,
  active,
  dir,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  active?: boolean;
  dir?: "ltr" | "rtl";
}) {
  return (
    <button
      type="button"
      dir={dir}
      onClick={onClick}
      className={
        "w-fit rounded-md py-1 text-start text-[0.85rem] transition-colors " +
        (active
          ? "font-semibold text-ink"
          : "text-ink-soft hover:text-ink")
      }
    >
      {children}
    </button>
  );
}
