/**
 * Share previews per language.
 *
 * The site is one page with hash routes, and link-preview crawlers (Facebook, Threads, LINE, X, Discord…)
 * don't run JavaScript — they only read the HTML. So for the page itself this rewrites <head> on the way
 * out: title, description, Open Graph and Twitter tags, canonical and hreflang links, in the visitor's
 * language, with the matching share image (public/og/<lang>.png, made by promo/remotion/scripts/og.mjs).
 *
 * Language: ?lang= → the language picked on the site last time (cookie) → Accept-Language → English.
 * Share a language on purpose with e.g. https://pinpin.fast01.net/?lang=ja
 *
 * URLs are built from the request's own origin, so previews work on pinpin-notes.pages.dev and on the
 * custom domain alike. Everything else (assets, /feed.json, /api/…) passes straight through.
 */

import { LOCALES, LOCALE_META, STORAGE_KEY, isLocale, matchBrowserLocale, type Locale } from "../src/i18n/config";
import { en } from "../src/i18n/locales/en";
import { zhTW } from "../src/i18n/locales/zh-TW";
import { zhCN } from "../src/i18n/locales/zh-CN";
import { ja } from "../src/i18n/locales/ja";
import { ko } from "../src/i18n/locales/ko";
import { ar } from "../src/i18n/locales/ar";

/** Cloudflare's streaming HTML rewriter (Workers runtime global). */
interface Element {
  remove(): void;
  setInnerContent(content: string): void;
  setAttribute(name: string, value: string): void;
  append(content: string, options: { html: boolean }): void;
}
declare const HTMLRewriter: {
  new (): {
    on(selector: string, handler: { element(el: Element): void }): InstanceType<typeof HTMLRewriter>;
    transform(response: Response): Response;
  };
};

const META: Record<Locale, { title: string; description: string }> = {
  en: en.meta,
  "zh-TW": zhTW.meta,
  "zh-CN": zhCN.meta,
  ja: ja.meta,
  ko: ko.meta,
  ar: ar.meta,
};

/** Open Graph locale codes */
const OG_LOCALE: Record<Locale, string> = {
  en: "en_US",
  "zh-TW": "zh_TW",
  "zh-CN": "zh_CN",
  ja: "ja_JP",
  ko: "ko_KR",
  ar: "ar_AR",
};

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function pickLocale(request: Request, url: URL): Locale {
  const q = url.searchParams.get("lang");
  if (isLocale(q)) return q;
  const cookie = request.headers.get("cookie") ?? "";
  const saved = cookie.match(new RegExp(`(?:^|;\\s*)${STORAGE_KEY}=([^;]+)`))?.[1];
  if (isLocale(saved)) return saved;
  for (const part of (request.headers.get("accept-language") ?? "").split(",")) {
    const matched = matchBrowserLocale(part.split(";")[0].trim());
    if (matched) return matched;
  }
  return "en";
}

/** the page's address in one language (English is the plain root) */
const pageUrl = (origin: string, lang: Locale) => (lang === "en" ? `${origin}/` : `${origin}/?lang=${lang}`);

function headTags(origin: string, lang: Locale): string {
  const { title, description } = META[lang];
  const image = `${origin}/og/${lang}.png`;
  const url = pageUrl(origin, lang);
  const tags = [
    `<meta name="description" content="${esc(description)}" />`,
    `<link rel="canonical" href="${esc(url)}" />`,
    ...LOCALES.map((l) => `<link rel="alternate" hreflang="${l}" href="${esc(pageUrl(origin, l))}" />`),
    `<link rel="alternate" hreflang="x-default" href="${esc(pageUrl(origin, "en"))}" />`,
    `<meta property="og:site_name" content="PinPin Notes" />`,
    `<meta property="og:type" content="website" />`,
    `<meta property="og:title" content="${esc(title)}" />`,
    `<meta property="og:description" content="${esc(description)}" />`,
    `<meta property="og:url" content="${esc(url)}" />`,
    `<meta property="og:locale" content="${OG_LOCALE[lang]}" />`,
    ...LOCALES.filter((l) => l !== lang).map((l) => `<meta property="og:locale:alternate" content="${OG_LOCALE[l]}" />`),
    `<meta property="og:image" content="${esc(image)}" />`,
    `<meta property="og:image:secure_url" content="${esc(image)}" />`,
    `<meta property="og:image:type" content="image/png" />`,
    `<meta property="og:image:width" content="1200" />`,
    `<meta property="og:image:height" content="630" />`,
    `<meta property="og:image:alt" content="${esc(title)}" />`,
    `<meta name="twitter:card" content="summary_large_image" />`,
    `<meta name="twitter:title" content="${esc(title)}" />`,
    `<meta name="twitter:description" content="${esc(description)}" />`,
    `<meta name="twitter:image" content="${esc(image)}" />`,
    `<meta name="twitter:image:alt" content="${esc(title)}" />`,
  ];
  return tags.join("\n    ");
}

interface Context {
  request: Request;
  next(): Promise<Response>;
}

export const onRequest = async ({ request, next }: Context) => {
  const url = new URL(request.url);
  const response = await next();
  const isPage = request.method === "GET" && (url.pathname === "/" || url.pathname === "/index.html");
  if (!isPage || !(response.headers.get("content-type") ?? "").includes("text/html")) return response;

  const lang = pickLocale(request, url);
  const rewritten = new HTMLRewriter()
    .on("html", {
      element: (el) => {
        el.setAttribute("lang", lang);
        el.setAttribute("dir", LOCALE_META[lang].dir);
      },
    })
    .on("title", { element: (el) => el.setInnerContent(META[lang].title) })
    .on('meta[name="description"]', { element: (el) => el.remove() })
    .on('meta[property^="og:"]', { element: (el) => el.remove() })
    .on('meta[name^="twitter:"]', { element: (el) => el.remove() })
    .on('link[rel="canonical"]', { element: (el) => el.remove() })
    .on('link[rel="alternate"]', { element: (el) => el.remove() })
    .on("head", { element: (el) => el.append(`    ${headTags(url.origin, lang)}\n  `, { html: true }) })
    .transform(response);

  const headers = new Headers(rewritten.headers);
  // the same URL now differs by cookie / Accept-Language; keep shared caches from mixing them up
  headers.set("vary", [headers.get("vary"), "Cookie", "Accept-Language"].filter(Boolean).join(", "));
  return new Response(rewritten.body, { status: rewritten.status, statusText: rewritten.statusText, headers });
};
