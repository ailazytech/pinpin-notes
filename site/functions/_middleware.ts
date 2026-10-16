/**
 * Language paths and share previews.
 *
 * Every language has its own address: / (English), /zh-tw/, /zh-cn/, /ja/, /ko/, /ar/.
 * - /zh, /tw, /cn, /en/ … and old ?lang= links redirect (301) to the real path
 * - /ja/<anything else> is served from /<anything else> (the page uses relative asset paths)
 *
 * The site is one page with hash routes, and link-preview crawlers (Facebook, Threads, LINE, X, Discord…)
 * don't run JavaScript — they only read the HTML. So for the page itself this rewrites <head> on the way
 * out: title, description, Open Graph and Twitter tags, canonical and hreflang links, in the page's
 * language, with the matching share image (public/og/<lang>.png, made by promo/remotion/scripts/og.mjs).
 *
 * Language of the page: the path → (at /) the language picked on the site last time (cookie) →
 * Accept-Language → English. URLs are built from the request's own origin, so this works on
 * pinpin-notes.pages.dev and on the custom domain alike. /feed.json, /api/… pass straight through.
 */

import {
  LOCALES,
  LOCALE_META,
  STORAGE_KEY,
  isLocale,
  localeFromSlug,
  localePath,
  matchBrowserLocale,
  type Locale,
} from "../src/i18n/config";
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

/** at the root: the visitor's saved choice, then their browser's language */
function preferredLocale(request: Request): Locale {
  const cookie = request.headers.get("cookie") ?? "";
  const saved = cookie.match(new RegExp(`(?:^|;\\s*)${STORAGE_KEY}=([^;]+)`))?.[1];
  if (isLocale(saved)) return saved;
  for (const part of (request.headers.get("accept-language") ?? "").split(",")) {
    const matched = matchBrowserLocale(part.split(";")[0].trim());
    if (matched) return matched;
  }
  return "en";
}

function headTags(origin: string, lang: Locale): string {
  const { title, description } = META[lang];
  const image = `${origin}/og/${lang}.png`;
  const url = origin + localePath(lang);
  const tags = [
    `<meta name="description" content="${esc(description)}" />`,
    `<link rel="canonical" href="${esc(url)}" />`,
    ...LOCALES.map((l) => `<link rel="alternate" hreflang="${l}" href="${esc(origin + localePath(l))}" />`),
    `<link rel="alternate" hreflang="x-default" href="${esc(origin + "/")}" />`,
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

/** the HTML page with <head> (and <html lang dir>) in one language */
function localize(page: Response, origin: string, lang: Locale, varies: boolean): Response {
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
    .on("head", { element: (el) => el.append(`    ${headTags(origin, lang)}\n  `, { html: true }) })
    .transform(page);
  const headers = new Headers(rewritten.headers);
  // the root's language depends on cookie / Accept-Language; keep shared caches from mixing them up
  if (varies) headers.set("vary", [headers.get("vary"), "Cookie", "Accept-Language"].filter(Boolean).join(", "));
  return new Response(rewritten.body, { status: rewritten.status, statusText: rewritten.statusText, headers });
}

interface Context {
  request: Request;
  next(): Promise<Response>;
  env: { ASSETS: { fetch(request: Request | string): Promise<Response> } };
}

const isHtml = (r: Response) => (r.headers.get("content-type") ?? "").includes("text/html");
const redirect = (to: string) => new Response(null, { status: 301, headers: { location: to } });

export const onRequest = async ({ request, next, env }: Context) => {
  const url = new URL(request.url);
  const { origin, pathname, search } = url;
  const [, first = "", ...rest] = pathname.split("/");
  const pathLang = localeFromSlug(first);

  if (pathLang) {
    const tail = rest.length ? "/" + rest.join("/") : "";
    // aliases, wrong case, /en/ and a missing trailing slash go to the one real address
    const real = localePath(pathLang);
    if (first !== real.split("/")[1] || (pathLang === "en" && first === "en")) {
      return redirect(origin + (pathLang === "en" ? tail || "/" : real.slice(0, -1) + (tail || "/")) + search);
    }
    if (tail === "") return redirect(origin + real + search);
    if (request.method === "GET" && (tail === "/" || tail === "/index.html")) {
      const page = await env.ASSETS.fetch(new Request(origin + "/", request));
      return isHtml(page) ? localize(page, origin, pathLang, false) : page;
    }
    // relative asset paths from /ja/ (./assets/…, ./og/…, ./img/…) → the real files at the root
    return env.ASSETS.fetch(new Request(origin + tail + search, request));
  }

  const isRoot = request.method === "GET" && (pathname === "/" || pathname === "/index.html");
  if (isRoot) {
    // old ?lang= links → the language's own path
    const q = url.searchParams.get("lang");
    if (isLocale(q)) {
      url.searchParams.delete("lang");
      const left = url.searchParams.toString();
      return redirect(origin + localePath(q) + (left ? `?${left}` : ""));
    }
  }
  const response = await next();
  if (!isRoot || !isHtml(response)) return response;
  return localize(response, origin, preferredLocale(request), true);
};
