/**
 * /api/stats — usage numbers for the private stats page (/stats.html).
 *
 * Protected by a key: send `Authorization: Bearer <STATS_KEY>`.
 * Reads the last 30 days from Workers Analytics Engine through the SQL API and counts unique
 * install ids here (one row per id per day), so it doesn't depend on SQL DISTINCT support.
 *
 * Pages environment variables (Cloudflare → Workers & Pages → pinpin-notes → Settings → Variables and Secrets):
 *   STATS_KEY       any long random string; you type it into the stats page
 *   CF_ACCOUNT_ID   your Cloudflare account id
 *   CF_STATS_TOKEN  API token with permission "Account Analytics: Read"
 */

interface Env {
  STATS_KEY?: string;
  CF_ACCOUNT_ID?: string;
  CF_STATS_TOKEN?: string;
}

interface Row {
  id: string;
  day: string;
  v: string;
  l: string;
  os: string;
  country: string;
}

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json; charset=utf-8", "cache-control": "no-store" },
  });

/** Constant-time string compare (the key is short, but no reason to leak timing). */
function same(a: string, b: string) {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

function windowsName(build: string) {
  const n = Number(build);
  if (!n) return "unknown";
  return n >= 22000 ? `Windows 11 (${build})` : `Windows 10 (${build})`;
}

function tally(values: string[]) {
  const counts = new Map<string, number>();
  for (const v of values) counts.set(v || "unknown", (counts.get(v || "unknown") ?? 0) + 1);
  return [...counts.entries()].sort((a, b) => b[1] - a[1]).map(([name, count]) => ({ name, count }));
}

export const onRequestGet = async ({ request, env }: { request: Request; env: Env }) => {
  const auth = request.headers.get("authorization") ?? "";
  const key = auth.startsWith("Bearer ") ? auth.slice(7) : "";
  if (!env.STATS_KEY || !key || !same(key, env.STATS_KEY)) return json({ error: "unauthorized" }, 401);
  if (!env.CF_ACCOUNT_ID || !env.CF_STATS_TOKEN) return json({ error: "CF_ACCOUNT_ID / CF_STATS_TOKEN not set" }, 503);

  const sql = `
    SELECT index1 AS id, formatDateTime(toStartOfDay(timestamp), '%Y-%m-%d') AS day,
           blob1 AS v, blob2 AS l, blob3 AS os, blob4 AS country
    FROM pinpin_usage
    WHERE timestamp > NOW() - INTERVAL '30' DAY
    LIMIT 200000
    FORMAT JSON`;
  const res = await fetch(`https://api.cloudflare.com/client/v4/accounts/${env.CF_ACCOUNT_ID}/analytics_engine/sql`, {
    method: "POST",
    headers: { authorization: `Bearer ${env.CF_STATS_TOKEN}` },
    body: sql,
  });
  if (!res.ok) return json({ error: `analytics query failed (${res.status})`, detail: (await res.text()).slice(0, 500) }, 502);
  const rows = ((await res.json()) as { data?: Row[] }).data ?? [];

  const now = Date.now();
  const dayOf = (offset: number) => new Date(now - offset * 86_400_000).toISOString().slice(0, 10);
  const days = Array.from({ length: 30 }, (_, i) => dayOf(29 - i));
  const perDay = new Map<string, Set<string>>(days.map((d) => [d, new Set()]));
  const latest = new Map<string, Row>();
  for (const r of rows) {
    perDay.get(r.day)?.add(r.id);
    const prev = latest.get(r.id);
    if (!prev || r.day >= prev.day) latest.set(r.id, r);
  }
  const uniqueSince = (n: number) => {
    const from = dayOf(n - 1);
    return new Set(rows.filter((r) => r.day >= from).map((r) => r.id)).size;
  };
  const users = [...latest.values()];

  return json({
    generated: new Date(now).toISOString(),
    today: perDay.get(dayOf(0))?.size ?? 0,
    last7: uniqueSince(7),
    last30: latest.size,
    daily: days.map((day) => ({ day, users: perDay.get(day)?.size ?? 0 })),
    versions: tally(users.map((u) => u.v)),
    languages: tally(users.map((u) => u.l)),
    windows: tally(users.map((u) => windowsName(u.os))),
    countries: tally(users.map((u) => u.country)),
  });
};
