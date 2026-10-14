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
  url: "https://pinpin.aiverything.app",

  /** Custom work enquiries */
  contactEmail: "ailazytech@gmail.com",

  /** TODO: GitHub Releases download URL (e.g. https://github.com/…/releases/latest) */
  downloadUrl: "",

  /** TODO: source repository URL */
  githubUrl: "",

  /** Donation links — TODO. Empty string renders as “coming soon”. */
  donate: {
    paypal: "",
    kofi: "",
    githubSponsors: "",
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
