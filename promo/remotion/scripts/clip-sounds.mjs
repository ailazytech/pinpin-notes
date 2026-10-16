// Sound for the feature clips: one wav per clip from src/clips.json — the cute marimba bed plus the synthesised
// effects (plop, sparkle, tick, bell…). Event kinds listed under "samples" are recorded sounds that Clips.tsx
// plays itself, so they are left out here.
//   node scripts/clip-sounds.mjs   →   public/audio/clip-<id>.wav

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const all = JSON.parse(fs.readFileSync(path.join(here, '../src/clips.json'), 'utf8'));
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'clip-sounds-'));
for (const [id, c] of Object.entries(all.clips)) {
  const tl = path.join(tmp, `${id}.json`);
  fs.writeFileSync(tl, JSON.stringify({fps: all.fps, duration: c.duration, music: c.music, events: c.events.filter((e) => !all.samples[e.kind]), T: c.T}));
  execFileSync(process.execPath, [path.join(here, 'sound.mjs'), tl, path.join(here, `../public/audio/clip-${id}.wav`)], {stdio: 'inherit'});
}
fs.rmSync(tmp, {recursive: true, force: true});
