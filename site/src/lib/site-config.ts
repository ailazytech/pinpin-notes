/**
 * Central place for links & assets that are still pending.
 * When the real URLs exist, fill them in here — the whole site wakes up.
 *
 * Screenshots: drop WebP files into public/img/screens/ using the ids below
 *              (e.g. notes.webp, todo.webp, shop.webp …). Until a file
 *              exists, its slot shows the hand-built demo instead.
 * Video:       drop the film at public/promo/pinpin-promo.mp4.
 */
export const siteConfig = {
  name: "PinPin Notes",

  /** Canonical site URL (no trailing slash). Also set in index.html and src/base.rs (SITE_URL). */
  url: "https://pinpin.fast01.net",

  /** Custom work enquiries */
  contactEmail: "ailazytech@gmail.com",

  /**
   * Download link for the latest pinpin.exe. Filled in at build time by scripts/fetch-release.mjs
   * (GitHub's releases/latest/download/… link of the latest published release). Empty = the button
   * shows “coming soon”.
   */
  downloadUrl: (import.meta.env.VITE_DOWNLOAD_URL as string | undefined) ?? "",

  /** Source repository */
  githubUrl: "https://github.com/ailazytech/pinpin-notes",

  /** Donation link (Ko-fi). Empty string = “coming soon”. */
  donate: {
    kofi: "https://ko-fi.com/ailazytech",
  },

  /**
   * Couplet orders (#/order). Point endpoint at a form backend that accepts a JSON POST
   * (Formspree: https://formspree.io/f/xxxx, or a Google Apps Script web app). Empty = open the mail app.
   */
  order: {
    endpoint: "",
  },

  /** Media slots — files are optional until you drop them in. */
  media: {
    videoSrc: "promo/pinpin-promo.mp4",
    screenshotBase: "img/screens",
    screenshotExtension: ".webp",
  },
} as const;

export function screenshotPath(id: string): string {
  return asset(`${siteConfig.media.screenshotBase}/${id}${siteConfig.media.screenshotExtension}`);
}

/** A file in public/, resolved against the site's base (works under a GitHub Pages sub-path too). */
export function asset(path: string): string {
  return import.meta.env.BASE_URL + path.replace(/^\//, "");
}
