// Runs before `pnpm run build` on Cloudflare Pages (root directory: site).
//
// The repo is private, so GitHub release assets aren't public. This copies the latest *published*
// release's .exe into public/download/ and points feed.json at it, so the website serves the download
// and the in-app online update.
//
//   public/download/PinPin-Notes-x64.exe            stable link for the download button
//   public/download/PinPin-Notes-<version>-x64.exe  versioned link used by feed.json
//
// Environment (Cloudflare Pages → Settings → Variables and Secrets):
//   GITHUB_TOKEN  fine-grained token, repository ailazytech/pinpin-notes only, "Contents: Read-only"
//   GITHUB_REPO   optional, default ailazytech/pinpin-notes
//   SITE_URL      optional, default https://pinpin.fast01.net
//
// No token or no published release → nothing happens and the download button says "coming soon".
// Any failure here is printed but never fails the build: the site must always deploy.

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";

const token = process.env.GITHUB_TOKEN;
const repo = process.env.GITHUB_REPO || "ailazytech/pinpin-notes";
const site = (process.env.SITE_URL || "https://pinpin.fast01.net").replace(/\/$/, "");

async function main() {
  if (!token) {
    console.log("fetch-release: GITHUB_TOKEN not set, skipping (download button stays 'coming soon')");
    return;
  }
  const api = (path, accept = "application/vnd.github+json") =>
    fetch(`https://api.github.com/repos/${repo}${path}`, {
      headers: { authorization: `Bearer ${token}`, accept, "x-github-api-version": "2022-11-28", "user-agent": "pinpin-site-build" },
    });

  const res = await api("/releases/latest");
  if (res.status === 404) {
    console.log("fetch-release: no published release yet");
    return;
  }
  if (!res.ok) throw new Error(`GitHub API ${res.status}: ${await res.text()}`);
  const release = await res.json();
  const asset = (release.assets || []).find((a) => a.name.toLowerCase().endsWith(".exe"));
  if (!asset) throw new Error(`release ${release.tag_name} has no .exe`);

  // Asset download: GitHub redirects to storage; fetch drops the Authorization header on the cross-origin hop.
  const bin = await api(`/releases/assets/${asset.id}`, "application/octet-stream");
  if (!bin.ok) throw new Error(`download ${asset.name}: ${bin.status}`);
  const bytes = Buffer.from(await bin.arrayBuffer());
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  const version = String(release.tag_name).replace(/^v/, "");
  const versioned = `PinPin-Notes-${version}-x64.exe`;

  await mkdir("public/download", { recursive: true });
  await writeFile(`public/download/${versioned}`, bytes);
  await writeFile("public/download/PinPin-Notes-x64.exe", bytes);

  // Online updates: version, url and sha256 follow the release; notes / critical / ads / packs stay as committed.
  const feed = JSON.parse(await readFile("public/feed.json", "utf8"));
  feed.app = { ...(feed.app || {}), version, url: `${site}/download/${versioned}`, sha256 };
  await writeFile("public/feed.json", JSON.stringify(feed, null, 2) + "\n");

  // Vite reads this at build time (site-config.ts → downloadUrl). *.local is git-ignored.
  await writeFile(".env.production.local", `VITE_DOWNLOAD_URL=${site}/download/PinPin-Notes-x64.exe\n`);

  console.log(`fetch-release: ${release.tag_name} → /download/${versioned} (${(bytes.length / 1048576).toFixed(1)} MB, sha256 ${sha256})`);
}

main().catch((e) => console.log(`fetch-release: skipped — ${e.message}`));
