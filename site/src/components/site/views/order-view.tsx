import { useState } from "react";
import { Globe, Loader2, Send, Truck } from "lucide-react";
import { toast } from "sonner";
import { useI18n } from "@/i18n/locale-context";
import { Reveal, SectionHeading, NotePaper, PinHead, type NoteColor } from "../ui";
import { asset, siteConfig } from "@/lib/site-config";
import { cn } from "@/lib/utils";

const STYLE_COLORS: NoteColor[] = ["butter", "peach", "mint", "lavender"];

type Form = Record<string, string>;

/**
 * Couplet commissions: a gold-calligraphy spring couplet written for you and shipped anywhere.
 * The form posts to `siteConfig.order.endpoint` (Formspree / Google Apps Script / any JSON form backend);
 * without an endpoint it opens the visitor's mail app with everything filled in.
 */
export function OrderView() {
  const { dict } = useI18n();
  const o = dict.order;
  const [form, setForm] = useState<Form>({ style: o.styles[0].name, quantity: "1" });
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const set = (k: string) => (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) =>
    setForm((f) => ({ ...f, [k]: e.target.value }));

  const fields = o.fields;
  const summary = () =>
    (Object.keys(fields) as (keyof typeof fields)[])
      .filter((k) => form[k])
      .map((k) => `${fields[k]}: ${form[k]}`)
      .join("\n");

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const endpoint = siteConfig.order.endpoint;
    if (!endpoint) {
      window.location.href = `mailto:${siteConfig.contactEmail}?subject=${encodeURIComponent(o.mailSubject)}&body=${encodeURIComponent(summary())}`;
      return;
    }
    setSending(true);
    try {
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json", Accept: "application/json" },
        body: JSON.stringify({ ...form, _subject: o.mailSubject, _summary: summary() }),
      });
      if (!res.ok) throw new Error(String(res.status));
      setSent(true);
      toast(o.success);
    } catch {
      toast(o.error, { description: siteConfig.contactEmail });
    } finally {
      setSending(false);
    }
  };

  const input = "h-11 w-full rounded-xl border border-line bg-card px-3.5 text-[0.9rem] text-ink outline-none transition-colors focus:border-pin";

  return (
    <div className="mx-auto w-full max-w-6xl px-4 py-12 sm:px-6 sm:py-16">
      {/* hero */}
      <div className="grid items-center gap-10 lg:grid-cols-[1.1fr_0.9fr]">
        <Reveal>
          <div className="flex flex-col items-start gap-5">
            <span className="inline-flex items-center gap-2 rounded-full border border-line bg-paper px-3.5 py-1.5 text-[0.72rem] font-bold uppercase tracking-[0.14em] text-ink-soft shadow-note">
              <Globe className="size-3.5 text-pin" />
              {o.eyebrow}
            </span>
            <h1 className="text-balance font-display text-4xl font-semibold tracking-tight text-ink sm:text-5xl">{o.title}</h1>
            <p className="text-pretty max-w-xl text-[0.98rem] leading-relaxed text-ink-soft">{o.sub}</p>
            <p className="inline-flex items-center gap-2 text-[0.85rem] font-semibold text-ink">
              <Truck className="size-4 text-pin" />
              {o.shipping}
            </p>
          </div>
        </Reveal>
        <Reveal delay={0.15}>
          <div className="relative mx-auto flex w-fit items-end gap-5">
            <img src={asset("img/couplets/strip.webp")} alt="" className="h-72 w-auto -rotate-3 drop-shadow-[0_10px_18px_rgb(80_10_10/0.3)]" />
            <img src={asset("img/couplets/diamond.webp")} alt="" className="h-52 w-auto rotate-3 drop-shadow-[0_10px_18px_rgb(80_10_10/0.3)]" />
          </div>
        </Reveal>
      </div>

      {/* styles */}
      <div className="mt-20">
        <Reveal>
          <SectionHeading title={o.stylesTitle} align="start" className="mb-8" />
        </Reveal>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {o.styles.map((st, i) => (
            <Reveal key={st.name} delay={i * 0.06} y={20}>
              <NotePaper color={STYLE_COLORS[i]} className={cn("relative h-full px-5 pb-5 pt-7", i % 2 ? "rotate-1" : "-rotate-1")}>
                <PinHead className="left-1/2 top-[-9px]" />
                <p className="text-[1rem] font-bold text-ink">{st.name}</p>
                <p className="mt-1.5 text-[0.82rem] leading-relaxed text-ink-soft">{st.desc}</p>
              </NotePaper>
            </Reveal>
          ))}
        </div>
      </div>

      {/* form + notes */}
      <div className="mt-20 grid gap-10 lg:grid-cols-[1.25fr_0.75fr]">
        <Reveal>
          <form onSubmit={submit} className="paper-grain flex flex-col gap-4 rounded-2xl border border-line bg-paper p-6 shadow-note sm:p-8">
            <p className="font-display text-2xl font-semibold text-ink">{o.formTitle}</p>
            {sent ? (
              <p className="rounded-xl bg-note-mint/60 p-5 text-[0.95rem] leading-relaxed text-ink">{o.success}</p>
            ) : (
              <>
                <div className="grid gap-4 sm:grid-cols-2">
                  <Field label={fields.name} required>
                    <input className={input} required value={form.name ?? ""} onChange={set("name")} autoComplete="name" />
                  </Field>
                  <Field label={fields.email} required>
                    <input className={input} required type="email" value={form.email ?? ""} onChange={set("email")} autoComplete="email" />
                  </Field>
                  <Field label={fields.style} required>
                    <select className={input} value={form.style} onChange={set("style")}>
                      {o.styles.map((st) => (
                        <option key={st.name}>{st.name}</option>
                      ))}
                    </select>
                  </Field>
                  <Field label={fields.quantity} required>
                    <input className={input} required type="number" min={1} value={form.quantity} onChange={set("quantity")} />
                  </Field>
                </div>
                <Field label={fields.text} required hint={o.textHint}>
                  <input className={input} required value={form.text ?? ""} onChange={set("text")} />
                </Field>
                <div className="grid gap-4 sm:grid-cols-2">
                  <Field label={fields.country} required>
                    <input className={input} required value={form.country ?? ""} onChange={set("country")} autoComplete="country-name" />
                  </Field>
                  <Field label={fields.deadline}>
                    <input className={input} type="date" value={form.deadline ?? ""} onChange={set("deadline")} />
                  </Field>
                </div>
                <Field label={fields.address} required>
                  <textarea className={cn(input, "h-20 py-2.5")} required value={form.address ?? ""} onChange={set("address")} autoComplete="street-address" />
                </Field>
                <Field label={fields.notes}>
                  <textarea className={cn(input, "h-24 py-2.5")} value={form.notes ?? ""} onChange={set("notes")} />
                </Field>
                <p className="text-[0.75rem] leading-relaxed text-ink-mute">{o.privacy}</p>
                <button
                  type="submit"
                  disabled={sending}
                  className="inline-flex h-12 items-center justify-center gap-2 self-start rounded-full bg-ink px-7 text-[0.92rem] font-bold text-paper shadow-note transition-all hover:-translate-y-0.5 hover:shadow-lift disabled:opacity-60"
                >
                  {sending ? <Loader2 className="size-4 animate-spin" /> : <Send className="size-4" />}
                  {sending ? o.sending : o.submit}
                </button>
              </>
            )}
          </form>
        </Reveal>
        <Reveal delay={0.1}>
          <NotePaper color="butter" className="relative px-6 py-7 -rotate-1">
            <PinHead className="left-1/2 top-[-9px]" />
            <p className="mb-3 font-display text-xl font-semibold text-ink">{o.notesTitle}</p>
            <ul className="flex flex-col gap-2.5">
              {o.notes.map((n) => (
                <li key={n} className="flex items-start gap-2.5 text-[0.86rem] leading-relaxed text-ink-soft">
                  <span className="mt-2 size-1.5 shrink-0 rounded-full bg-pin" />
                  {n}
                </li>
              ))}
            </ul>
          </NotePaper>
        </Reveal>
      </div>
    </div>
  );
}

function Field({ label, required, hint, children }: { label: string; required?: boolean; hint?: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-[0.8rem] font-semibold text-ink">
        {label}
        {required && <span className="ms-0.5 text-pin">*</span>}
      </span>
      {children}
      {hint && <span className="text-[0.72rem] text-ink-mute">{hint}</span>}
    </label>
  );
}
