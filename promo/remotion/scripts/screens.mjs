// Website screenshots → ../../site/public/img/screens/<id>.webp   (node scripts/screens.mjs [id ...])
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {bundle} from '@remotion/bundler';
import {getCompositions, renderStill} from '@remotion/renderer';

const here = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(here, '../../../site/public/img/screens');
const only = process.argv.slice(2);
const serveUrl = await bundle({entryPoint: path.join(here, '../src/index.ts')});
for (const c of await getCompositions(serveUrl)) {
  if (!c.id.startsWith('screen-')) continue;
  const id = c.id.slice('screen-'.length);
  if (only.length && !only.includes(id)) continue;
  const png = path.join(here, `../out/screen-${id}.png`);
  await renderStill({composition: c, serveUrl, output: png});
  execFileSync('ffmpeg', ['-loglevel', 'error', '-y', '-i', png, '-quality', '84', path.join(outDir, `${id}.webp`)]);
  console.log(id);
}
