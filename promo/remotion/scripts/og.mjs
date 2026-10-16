// Social share images for the website, one per language: the promo film's finished board with the end card
// (composition og-<lang>, Film.tsx → EndStill), cropped from 16:9 to 1.91:1 and scaled to 1200 × 630.
//
//   node scripts/og.mjs   →   ../../site/public/og/<lang>.png  (+ ../../site/public/og.png = English)
//
// The website picks the right one per language in site/functions/_middleware.ts.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');
const out = path.join(root, '../../site/public/og');
const LANGS = ['en', 'zh-TW', 'zh-CN', 'ja', 'ko', 'ar'];
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'og-'));
fs.mkdirSync(out, {recursive: true});
const npx = process.platform === 'win32' ? 'npx.cmd' : 'npx';
for (const lang of LANGS) {
  const raw = path.join(tmp, `${lang}.png`);
  execFileSync(npx, ['remotion', 'still', `og-${lang}`, raw, '--log=error'], {cwd: root, stdio: 'inherit', shell: process.platform === 'win32'});
  const png = path.join(out, `${lang}.png`);
  // 1920 × 1080 → the middle 1920 × 1008 (1.905:1, same as 1200 × 630, the size Facebook / Threads / X expect)
  execFileSync('ffmpeg', ['-loglevel', 'error', '-y', '-i', raw, '-vf', 'crop=1920:1008:0:36,scale=1200:630:flags=lanczos', png]);
  console.log(path.relative(root, png));
}
fs.copyFileSync(path.join(out, 'en.png'), path.join(out, '..', 'og.png'));
fs.rmSync(tmp, {recursive: true, force: true});
