/**
 * /feed.json — the update feed the app fetches at most once a day.
 *
 * If the request carries anonymous usage parameters (the "Send anonymous usage stats" switch in the
 * app's tray menu), record one data point in Workers Analytics Engine, then serve the static
 * public/feed.json exactly as before. Recording can never break the feed: any error is ignored.
 *
 * Query: v = app version, l = UI language, os = Windows build number, id = random install id (32 hex).
 * Nothing else is stored: no IP address, no user agent. The country comes from Cloudflare's edge.
 * See docs/PRIVACY.md.
 */

interface AnalyticsEngine {
  writeDataPoint(point: { indexes?: string[]; blobs?: string[]; doubles?: number[] }): void;
}

interface Env {
  ASSETS: { fetch(request: Request): Promise<Response> };
  /** Bound in site/wrangler.toml; missing locally or before Cloudflare is set up. */
  USAGE?: AnalyticsEngine;
}

const clean = (s: string | null) => (s ?? "").replace(/[^0-9A-Za-z._-]/g, "").slice(0, 32);

export const onRequestGet = async ({ request, env }: { request: Request; env: Env }) => {
  const url = new URL(request.url);
  const q = url.searchParams;
  const id = q.get("id") ?? "";
  if (env.USAGE && /^[0-9a-f]{32}$/.test(id)) {
    try {
      const country = clean(String((request as unknown as { cf?: { country?: string } }).cf?.country ?? ""));
      env.USAGE.writeDataPoint({
        indexes: [id],
        blobs: [clean(q.get("v")), clean(q.get("l")), clean(q.get("os")), country],
        doubles: [1],
      });
    } catch {
      // statistics are best-effort; the feed must always be served
    }
  }
  url.search = "";
  return env.ASSETS.fetch(new Request(url.toString(), request));
};
