// Runs before `pnpm run build` on Cloudflare Pages (root directory: site).
//
// Points the website at the latest *published* GitHub release — the repo is public, so the files are
// downloaded straight from GitHub (which also counts the downloads for the README badge):
//
//   download button   https://github.com/<repo>/releases/latest/download/PinPin-Notes-x64.exe
//   feed.json app.url https://github.com/<repo>/releases/download/<tag>/PinPin-Notes-<version>-x64.exe
//                     with the sha256 from the release's .sha256 file (the app verifies it before installing)
//
// Environment (Cloudflare Pages → Settings → Variables and Secrets), all optional:
//   GITHUB_TOKEN  raises the API rate limit; read-only access to public repos is enough
//   GITHUB_REPO   default ailazytech/pinpin-notes
//
// No published release → nothing happens and the download button says "coming soon".
// Any failure here is printed but never fails the build: the site must always deploy.

import { readFile, writeFile } from "node:fs/promises";

const token = process.env.GITHUB_TOKEN;
const repo = process.env.GITHUB_REPO || "ailazytech/pinpin-notes";

async function main() {
  const headers = { accept: "application/vnd.github+json", "x-github-api-version": "2022-11-28", "user-agent": "pinpin-site-build" };
  if (token) headers.authorization = `Bearer ${token}`;

  const res = await fetch(`https://api.github.com/repos/${repo}/releases/latest`, { headers });
  if (res.status === 404) {
    console.log("fetch-release: no published release yet");
    return;
  }
  if (!res.ok) throw new Error(`GitHub API ${res.status}: ${await res.text()}`);
  const release = await res.json();
  const version = String(release.tag_name).replace(/^v/, "");
  const versioned = `PinPin-Notes-${version}-x64.exe`;
  const assets = release.assets || [];
  const exe = assets.find((a) => a.name === versioned);
  const sum = assets.find((a) => a.name === `${versioned}.sha256`);
  if (!exe || !sum) throw new Error(`release ${release.tag_name} is missing ${versioned} or its .sha256`);

  const sumRes = await fetch(sum.browser_download_url, { headers: { "user-agent": "pinpin-site-build" } });
  if (!sumRes.ok) throw new Error(`download ${sum.name}: ${sumRes.status}`);
  const sha256 = (await sumRes.text()).trim().split(/\s+/)[0].toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(sha256)) throw new Error(`bad sha256 in ${sum.name}`);

  // Online updates: version, url and sha256 follow the release; notes / critical / ads / packs stay as committed.
  const feed = JSON.parse(await readFile("public/feed.json", "utf8"));
  feed.app = { ...(feed.app || {}), version, url: exe.browser_download_url, sha256 };
  await writeFile("public/feed.json", JSON.stringify(feed, null, 2) + "\n");

  // Vite reads this at build time (site-config.ts → downloadUrl). *.local is git-ignored.
  const latest = assets.some((a) => a.name === "PinPin-Notes-x64.exe")
    ? `https://github.com/${repo}/releases/latest/download/PinPin-Notes-x64.exe`
    : exe.browser_download_url;
  await writeFile(".env.production.local", `VITE_DOWNLOAD_URL=${latest}\n`);

  console.log(`fetch-release: ${release.tag_name} → ${latest} (sha256 ${sha256})`);
}

main().catch((e) => console.log(`fetch-release: skipped — ${e.message}`));
