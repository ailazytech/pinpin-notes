// Render chosen frames as PNGs (one bundle, many stills):  node scripts/stills.mjs 1.0 3.0 7.2 ...
// Another language: PROMO_LANG=zh-TW node scripts/stills.mjs …  (same as rendering with --props='{"lang":"zh-TW"}')
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {bundle} from '@remotion/bundler';
import {renderStill, selectComposition} from '@remotion/renderer';

const here = path.dirname(fileURLToPath(import.meta.url));
const times = process.argv.slice(2).map(Number);
const inputProps = {lang: process.env.PROMO_LANG || 'en'};
const serveUrl = await bundle({entryPoint: path.join(here, '../src/index.ts')});
const composition = await selectComposition({serveUrl, id: 'PinPinPromo', inputProps});
for (const t of times) {
  const frame = Math.min(composition.durationInFrames - 1, Math.round(t * composition.fps));
  const output = path.join(here, `../out/still-${inputProps.lang === 'en' ? '' : inputProps.lang + '-'}${t.toFixed(2)}.png`);
  await renderStill({composition, serveUrl, output, frame, inputProps});
  console.log(output);
}
