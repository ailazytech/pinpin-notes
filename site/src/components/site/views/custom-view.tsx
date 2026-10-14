import { motion } from "framer-motion";
import { ArrowRight, Check, Copy, Mail, Scissors } from "lucide-react";
import { toast } from "sonner";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, StickerImg, NotePaper, PinHead, WashiTape, type NoteColor } from "../ui";
import { asset, siteConfig } from "@/lib/site-config";
import { cn } from "@/lib/utils";

const PACKAGE_COLORS: NoteColor[] = ["butter", "peach", "mint", "lavender", "sky"];
const STEP_COLORS: NoteColor[] = ["butter", "rose", "sage", "mist"];

/** Custom work: stickers, washi tape and timer characters made from the visitor's own photos. */
export function CustomView() {
  const { dict } = useI18n();
  const c = dict.commission;
  const email = siteConfig.contactEmail;
  const mailto = (pkg?: string) => {
    const body = pkg ? c.mailBody.replace(/(\n[^\n:：]+[:：])\s*\n/, `$1 ${pkg}\n`) : c.mailBody;
    return `mailto:${email}?subject=${encodeURIComponent(c.mailSubject + (pkg ? ` · ${pkg}` : ""))}&body=${encodeURIComponent(body)}`;
  };
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(email);
      toast(c.copied, { description: email });
    } catch {
      window.location.href = mailto();
    }
  };

  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-12 sm:px-6 sm:py-16">
      {/* hero: a photo becomes a die-cut sticker */}
      <div className="grid items-center gap-10 lg:grid-cols-[1.1fr_0.9fr]">
        <Reveal>
          <div className="flex flex-col items-start gap-5">
            <span className="inline-flex items-center gap-2 rounded-full border border-line bg-paper px-3.5 py-1.5 text-[0.72rem] font-bold uppercase tracking-[0.14em] text-ink-soft shadow-note">
              <Scissors className="size-3.5 text-pin" />
              {c.eyebrow}
            </span>
            <h1 className="text-balance font-display text-4xl font-semibold tracking-tight text-ink sm:text-5xl">{c.title}</h1>
            <p className="text-pretty max-w-xl text-[0.98rem] leading-relaxed text-ink-soft">{c.sub}</p>
            <a
              href={mailto()}
              className="inline-flex h-11 items-center gap-2 rounded-full bg-ink px-6 text-[0.88rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
            >
              <Mail className="size-4" />
              {c.ctaButton}
            </a>
          </div>
        </Reveal>
        <Reveal delay={0.15}>
          <div className="relative mx-auto flex w-fit items-center gap-2 sm:gap-6">
            <div className="relative shrink-0 -rotate-3 rounded-xl bg-white p-2.5 shadow-lift">
              <img src={asset("img/photos/corgi-photo.webp")} alt="" className="h-40 w-auto rounded-md sm:h-56" />
            </div>
            <ArrowRight className="size-7 shrink-0 text-ink-mute rtl:rotate-180" />
            <motion.div animate={{ y: [0, -8, 0], rotate: [3, 0, 3] }} transition={{ duration: 6, repeat: Infinity, ease: "easeInOut" }}>
              <StickerImg name="corgi" size={160} alt="a die-cut corgi sticker" />
            </motion.div>
          </div>
        </Reveal>
      </div>

      {/* packages */}
      <div className="mt-20">
        <Reveal>
          <SectionHeading title={c.packagesTitle} align="start" className="mb-8" />
        </Reveal>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-5">
          {c.packages.map((p, i) => (
            <Reveal key={p.name} delay={i * 0.06} y={22}>
              <motion.a
                href={mailto(p.name)}
                whileHover={{ y: -6, rotate: i % 2 === 0 ? -1 : 1 }}
                className="block h-full"
              >
                <NotePaper color={PACKAGE_COLORS[i]} lift className={cn("relative flex h-full flex-col gap-3 px-5 pb-5 pt-7", i === 2 && "ring-2 ring-pin/50")}>
                  <PinHead className="left-1/2 top-[-9px]" />
                  {i === 2 && (
                    <span className="absolute -top-3 end-3 rotate-3 rounded-full bg-pin px-2.5 py-0.5 text-[0.6rem] font-bold uppercase tracking-wider text-white shadow-note">
                      {c.popular}
                    </span>
                  )}
                  <span className="font-display text-3xl font-semibold tracking-tight text-ink">{p.price}</span>
                  <span className="text-[0.95rem] font-bold text-ink">{p.name}</span>
                  <span className="text-[0.8rem] leading-relaxed text-ink-soft">{p.desc}</span>
                  <ul className="mt-auto flex flex-col gap-1.5 pt-2">
                    {p.items.map((it) => (
                      <li key={it} className="flex items-start gap-2 text-[0.78rem] text-ink-soft">
                        <Check className="mt-0.5 size-3.5 shrink-0 text-pin" />
                        {it}
                      </li>
                    ))}
                  </ul>
                </NotePaper>
              </motion.a>
            </Reveal>
          ))}
        </div>
      </div>

      {/* how it works */}
      <div className="mt-20">
        <Reveal>
          <SectionHeading title={c.stepsTitle} align="start" className="mb-8" />
        </Reveal>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {c.steps.map((s, i) => (
            <Reveal key={s.title} delay={i * 0.07} y={18}>
              <NotePaper color={STEP_COLORS[i]} className={cn("relative h-full px-5 py-6", i % 2 ? "rotate-1" : "-rotate-1")}>
                <WashiTape color="var(--note-rose)" pattern={i % 3} className="left-1/2 top-[-12px] -translate-x-1/2" rotate={-3} width={70} />
                <span className="font-hand text-4xl leading-none text-pin">{i + 1}</span>
                <p className="mt-2 text-[0.95rem] font-bold text-ink">{s.title}</p>
                <p className="mt-1 text-[0.8rem] leading-relaxed text-ink-soft">{s.desc}</p>
              </NotePaper>
            </Reveal>
          ))}
        </div>
      </div>

      {/* fine print + contact */}
      <div className="mt-20 grid gap-8 lg:grid-cols-2">
        <Reveal>
          <div className="paper-grain h-full rounded-xl border border-line bg-card p-6 shadow-note">
            <p className="mb-3 font-display text-xl font-semibold text-ink">{c.notesTitle}</p>
            <ul className="flex flex-col gap-2.5">
              {c.notes.map((n) => (
                <li key={n} className="flex items-start gap-2.5 text-[0.86rem] leading-relaxed text-ink-soft">
                  <span className="mt-2 size-1.5 shrink-0 rounded-full bg-pin" />
                  {n}
                </li>
              ))}
            </ul>
          </div>
        </Reveal>
        <Reveal delay={0.1}>
          <NotePaper color="butter" className="relative flex h-full flex-col items-start gap-4 px-7 py-7 -rotate-1">
            <PinHead className="left-1/2 top-[-9px]" />
            <p className="font-display text-2xl font-semibold text-ink">{c.ctaTitle}</p>
            <p className="text-[0.9rem] leading-relaxed text-ink-soft">{c.ctaSub}</p>
            <div className="mt-auto flex flex-wrap items-center gap-3">
              <a
                href={mailto()}
                className="inline-flex h-11 items-center gap-2 rounded-full bg-ink px-6 text-[0.88rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift"
              >
                <Mail className="size-4" />
                {c.ctaButton}
              </a>
              <button
                type="button"
                onClick={copy}
                className="inline-flex h-11 items-center gap-2 rounded-full border border-line bg-card px-5 text-[0.84rem] font-semibold text-ink shadow-note transition-all hover:-translate-y-0.5"
              >
                <Copy className="size-4" />
                <span className="font-mono text-[0.8rem]">{email}</span>
              </button>
            </div>
          </NotePaper>
        </Reveal>
      </div>
    </div>
  );
}
