// Render chosen frames as PNGs (one bundle, many stills):  node scripts/stills.mjs 1.0 3.0 7.2 ...
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {bundle} from '@remotion/bundler';
import {renderStill, selectComposition} from '@remotion/renderer';

const here = path.dirname(fileURLToPath(import.meta.url));
const times = process.argv.slice(2).map(Number);
const serveUrl = await bundle({entryPoint: path.join(here, '../src/index.ts')});
const composition = await selectComposition({serveUrl, id: 'PinPinPromo'});
for (const t of times) {
  const frame = Math.min(composition.durationInFrames - 1, Math.round(t * composition.fps));
  const output = path.join(here, `../out/still-${t.toFixed(2)}.png`);
  await renderStill({composition, serveUrl, output, frame});
  console.log(output);
}
