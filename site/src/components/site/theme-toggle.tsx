"use client";

import { useTheme } from "next-themes";
import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/i18n/locale-context";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const { dict } = useI18n();

  return (
    <Button
      variant="ghost"
      size="icon"
      className="relative size-9 rounded-full text-ink-soft hover:bg-paper-deep hover:text-ink"
      aria-label={dict.theme.toggle}
      title={dict.theme.toggle}
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
    >
      {/* both icons live in the DOM; CSS decides which one shows —
          this keeps SSR markup identical to the client's first paint */}
      <Sun className="size-[18px] rotate-0 scale-100 transition-all duration-500 dark:-rotate-90 dark:scale-0" />
      <Moon className="absolute size-[18px] rotate-90 scale-0 transition-all duration-500 dark:rotate-0 dark:scale-100" />
    </Button>
  );
}
