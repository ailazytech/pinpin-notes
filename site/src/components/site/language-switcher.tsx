"use client";

import { Check, ChevronDown, Globe } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useI18n } from "@/i18n/locale-context";
import { LOCALES, LOCALE_META } from "@/i18n/config";
import { cn } from "@/lib/utils";

export function LanguageSwitcher({ compact = false }: { compact?: boolean }) {
  const { locale, setLocale, dict } = useI18n();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className={cn(
          "inline-flex h-9 items-center gap-1.5 rounded-full px-3 text-sm font-semibold text-ink-soft outline-none transition-colors hover:bg-paper-deep hover:text-ink data-[state=open]:bg-paper-deep data-[state=open]:text-ink",
          compact && "px-2.5"
        )}
        aria-label={dict.lang.choose}
      >
        <Globe className="size-[17px]" />
        <span className={cn("hidden sm:inline", compact && "sm:hidden")}>
          {LOCALE_META[locale].label}
        </span>
        <ChevronDown className="size-3.5 opacity-60" />
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        sideOffset={8}
        className="w-52 rounded-xl border-line bg-popover p-1.5"
      >
        <DropdownMenuLabel className="px-2.5 py-1.5 text-[0.68rem] font-bold uppercase tracking-[0.16em] text-ink-mute">
          {dict.lang.label}
        </DropdownMenuLabel>
        {LOCALES.map((code) => {
          const meta = LOCALE_META[code];
          const active = code === locale;
          return (
            <DropdownMenuItem
              key={code}
              onSelect={() => setLocale(code)}
              dir={meta.dir}
              className={cn(
                "flex cursor-pointer items-center justify-between gap-3 rounded-lg px-2.5 py-2 text-[0.9rem] outline-none",
                meta.dir === "rtl" && "flex-row-reverse text-right"
              )}
            >
              <span className="flex flex-col">
                <span className={cn("font-semibold", active ? "text-ink" : "text-ink-soft")}>
                  {meta.label}
                </span>
                <span className="text-[0.68rem] text-ink-mute">{meta.english}</span>
              </span>
              {active && <Check className="size-4 text-pin" />}
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
