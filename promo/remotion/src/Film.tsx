// PinPin Notes promo — one take across one desktop, the tool changes hands at every beat.
//
// The layout comes first: BOARD is the finished desktop (a full 16:9 board with a clear centre for the wordmark).
// The film walks through it in order — pen crosses off a to-do and its stroke becomes washi tape, scissors cut a
// picture into a sticker, a crayon draws a heart and circles the pomodoro, chalk sweeps the day into night and writes
// a slogan, a blue pen circles a secret note — then pulls out while the rest of the board drops into place, and the
// red pen signs the wordmark in the space that was left for it.
// Everything is a pure function of time. UI pictures are real PinPin renders (`pinpin.exe --export-sprites`).
//
// Language: render with --props='{"lang":"zh-TW"}' for the Traditional Chinese cut — captions, labels and the
// slogan come from COPY below, the PinPin pictures from public/sprites-zh-TW/ (exported with the zh-TW UI).

import React from 'react';
import {AbsoluteFill, Audio, Img, getInputProps, staticFile, useCurrentFrame, Easing} from 'remotion';
import {CameraMotionBlur} from '@remotion/motion-blur';
import {evolvePath, getLength, getPointAtLength, getTangentAtLength} from '@remotion/paths';
import manifestEn from '../public/sprites/sprites.json';
import manifestZhTW from '../public/sprites-zh-TW/sprites.json';
import TL from './timeline.json';

const T = TL.T as Record<string, any>;
const FPS = TL.fps;
const W = 1920;
const H = 1080;
/** world px per PinPin logical px */
const WS = 1.75;
const INK = '#2e2622';
const PAPER = '#fbf6ee';
const NIGHT = '#121a26';
const FONT = '"Segoe UI Variable Display", "Segoe UI", sans-serif';
const FONT_ZH_TW = `"Microsoft JhengHei UI", "Microsoft JhengHei", ${FONT}`;

const LANG = ((getInputProps() as {lang?: string}).lang ?? 'en') === 'zh-TW' ? 'zh-TW' : 'en';
const SPRITES = {en: {dir: 'sprites', manifest: manifestEn}, 'zh-TW': {dir: 'sprites-zh-TW', manifest: manifestZhTW}}[LANG];
const sprite = (file: string) => staticFile(`${SPRITES.dir}/${file}`);

type SpriteInfo = {file?: string; files?: string[]; w: number; h: number; paper?: number[]; rows?: number[][]; scale?: number};
const S = (SPRITES.manifest as any).sprites as Record<string, SpriteInfo>;
const SPRITE_SCALE = (SPRITES.manifest as any).scale as number;

// ───────────────────────── curves ─────────────────────────

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
const prog = (t: number, [a, b]: number[]) => clamp01((t - a) / (b - a));
const expoOut = Easing.out(Easing.exp);
const expoIn = Easing.in(Easing.exp);
const expoInOut = Easing.inOut(Easing.exp);
const sineInOut = Easing.inOut(Easing.sin);
const cubicOut = Easing.out(Easing.cubic);
const lerp = (a: number, b: number, p: number) => a + (b - a) * p;
/** damped spring 0→1 started at t0 */
const springAt = (t: number, t0: number, stiff = 16, damp = 7) => {
  const x = t - t0;
  if (x <= 0) return 0;
  return 1 - Math.exp(-damp * x) * Math.cos(stiff * x);
};

// ───────────────────────── the board (layout first) ─────────────────────────

const factor = (s: SpriteInfo) => WS / (s.scale ? s.scale : SPRITE_SCALE);
const paperSize = (name: string) => {
  const s = S[name];
  const [l, t, r, b] = s.paper!;
  return {w: (r - l) * factor(s), h: (b - t) * factor(s)};
};
const row = (name: string, at: {x: number; y: number}, i: number) => {
  const s = S[name];
  const f = factor(s);
  const [l, t] = s.paper!;
  const [x0, x1, y] = s.rows![i];
  return [at.x + (x0 - l) * f, at.x + (x1 - l) * f, at.y + (y - t) * f];
};

/** world is 4000 × 2250 (16:9). The centre (x 1270–2730, y 560–1500) stays clear for the wordmark. */
const BOARD = {w: 4000, h: 2250};
const P = {
  todo: {x: 690, y: 150},
  rose: {x: 1190, y: 110},
  sage: {x: 1690, y: 175},
  oat: {x: 2215, y: 95},
  picture: {cx: 2000, cy: 1000},
  timers: {
    tomato: {cx: 3000, cy: 330},
    hourglass: {cx: 3345, cy: 330},
    led: {cx: 3730, cy: 300},
    alarm: {cx: 3000, cy: 780},
    cat: {cx: 3345, cy: 780},
    dog: {cx: 3700, cy: 780},
  } as Record<string, {cx: number; cy: number}>,
  slogan: {x: 1440, y: 1790},
  secret: {x: 3330, y: 1130},
};

/** the rest of the board: drops in during the pull-out (order = from the centre outward) */
type Filler = {name: string; x?: number; y?: number; cx?: number; cy?: number; rot?: number};
const FILLERS: Filler[] = [
  {name: 'note-read', x: 760, y: 1150, rot: -2},
  {name: 'note-sale', x: 2860, y: 1230, rot: 3},
  {name: 'st-boba', cx: 1080, cy: 930, rot: 8},
  {name: 'st-star', cx: 2780, cy: 1700, rot: -6},
  {name: 'note-call', x: 150, y: 760, rot: 2},
  {name: 'starred', x: 110, y: 70, rot: -1.5},
  {name: 'polaroid', x: 170, y: 1260, rot: -5},
  {name: 'st-coffee', cx: 790, cy: 1880, rot: -8},
  {name: 'st-sparkles', cx: 1250, cy: 1690, rot: 0},
  {name: 'st-rainbow', cx: 3560, cy: 1840, rot: 6},
  {name: 'st-moon', cx: 3150, cy: 1880, rot: -10},
  {name: 'st-sakura', cx: 620, cy: 640, rot: 12},
];

// ───────────────────────── strokes (tool by tool) ─────────────────────────

type Tool = 'pen' | 'blue' | 'scissors' | 'crayon' | 'chalk' | 'lock';
const TOOL_COLOR: Record<Tool, string> = {pen: '#e2241e', blue: '#2f6fdb', scissors: '#5b5250', crayon: '#e2241e', chalk: '#f4f1ea', lock: '#e8b84a'};

const wobbly = (x0: number, y0: number, x1: number, y1: number, seed = 0) => {
  let d = `M ${x0} ${y0}`;
  for (let i = 1; i <= 14; i++) {
    const u = i / 14;
    const w = Math.sin(u * 9 + seed) * 1.6 + Math.sin(u * 23 + seed * 2) * 0.6;
    d += ` L ${lerp(x0, x1, u).toFixed(1)} ${(lerp(y0, y1, u) + w).toFixed(1)}`;
  }
  return d;
};
const heartPath = (cx: number, cy: number, k: number) => {
  let d = '';
  for (let i = 0; i <= 80; i++) {
    const t = (i / 80) * Math.PI * 2;
    d += `${i ? ' L' : 'M'} ${(cx + 16 * Math.pow(Math.sin(t), 3) * k).toFixed(1)} ${(cy - (13 * Math.cos(t) - 5 * Math.cos(2 * t) - 2 * Math.cos(3 * t) - Math.cos(4 * t)) * k).toFixed(1)}`;
  }
  return d;
};
const loop = (cx: number, cy: number, rx: number, ry: number, from = -Math.PI / 2) => {
  let d = '';
  for (let i = 0; i <= 90; i++) {
    const a = from + (i / 90) * Math.PI * 2;
    const wob = 1 + 0.05 * Math.sin(a * 3 + 0.6) + 0.03 * Math.sin(a * 7);
    d += `${i ? ' L' : 'M'} ${(cx + Math.cos(a) * rx * wob).toFixed(1)} ${(cy + Math.sin(a) * ry * wob).toFixed(1)}`;
  }
  return d;
};

type Seg = {d: string; t: number[]; tool: Tool; ease?: (x: number) => number; width?: number; fade?: number[]; hidden?: boolean; dash?: string; tape?: boolean};

const todoRows = [0, 1].map((i) => row('note-todo', P.todo, i));
const rose = paperSize('note-rose');
const tapeA = {x: P.rose.x + rose.w / 2 - 150, y: P.rose.y + 6};
const tapeB = {x: P.rose.x + rose.w / 2 + 150, y: P.rose.y + 2};
const pic = S['picture'];
const picH = 560;
const picW = (pic.w / pic.h) * picH;
const oat = paperSize('note-oat');
const heartC = {x: P.oat.x + 210 * WS, y: P.oat.y + 150 * WS, k: 2.4 * WS};
const heartD = heartPath(heartC.x, heartC.y, heartC.k);
const heartBottom = {x: heartC.x, y: heartC.y + 17 * heartC.k};
const tomato = P.timers.tomato;
const ringR = 190;
const sloganRow = row('slogan', P.slogan, 0);
const secret = paperSize('secret');
const secretC = {x: P.secret.x + secret.w / 2, y: P.secret.y + secret.h / 2};
const sloganC = {x: (sloganRow[0] + sloganRow[1]) / 2, y: sloganRow[2] + 30};

const SEGS: Seg[] = [
  // the pen arrives from the right (no ink yet)
  {d: `M ${todoRows[0][1] + 520} ${todoRows[0][2] - 260} C ${todoRows[0][1] + 300} ${todoRows[0][2] - 220}, ${todoRows[0][0] - 60} ${todoRows[0][2] - 90}, ${todoRows[0][0]} ${todoRows[0][2]}`, t: T.penIn, tool: 'pen', ease: expoOut, hidden: true},
  {d: wobbly(todoRows[0][0], todoRows[0][2], todoRows[0][1], todoRows[0][2], 1), t: T.strike1, tool: 'pen', ease: expoOut},
  {d: `M ${todoRows[0][1]} ${todoRows[0][2]} Q ${todoRows[0][1] - 40} ${todoRows[0][2] + 50}, ${todoRows[1][0]} ${todoRows[1][2]}`, t: [T.strike1[1] + 0.05, T.strike2[0]], tool: 'pen', ease: sineInOut, hidden: true},
  {d: wobbly(todoRows[1][0], todoRows[1][2], todoRows[1][1], todoRows[1][2], 2), t: T.strike2, tool: 'pen', ease: expoOut},
  // ...the pen keeps going, off the paper, up to the rose note — and its stroke lays down as tape
  {d: `M ${todoRows[1][1]} ${todoRows[1][2]} C ${todoRows[1][1] + 220} ${todoRows[1][2] + 40}, ${tapeA.x - 200} ${tapeA.y - 150}, ${tapeA.x} ${tapeA.y}`, t: T.exit, tool: 'pen', ease: sineInOut, fade: [2.68, 2.9]},
  {d: `M ${tapeA.x} ${tapeA.y} L ${tapeB.x} ${tapeB.y}`, t: T.tape, tool: 'pen', ease: expoOut, width: 7, fade: [2.68, 2.8], tape: true},
  // scissors fly in from the tape's end to the picture
  {d: `M ${tapeB.x + 30} ${tapeB.y} C ${tapeB.x + 360} ${tapeB.y + 160}, ${P.picture.cx + 520} ${P.picture.cy - 520}, ${P.picture.cx} ${P.picture.cy - picH * 0.48}`, t: T.scissorsIn, tool: 'scissors', ease: expoInOut, hidden: true},
  // cut along a dashed line around the dog
  {d: loop(P.picture.cx, P.picture.cy + 8, picW * 0.44, picH * 0.48), t: T.cut, tool: 'scissors', ease: sineInOut, dash: '14 10', width: 4, fade: [5.35, 5.6]},
  // the crayon draws the heart (the crayon frames follow it)
  {d: heartD, t: T.heart, tool: 'crayon', ease: (x) => x, hidden: true},
  // ...and runs over to circle the pomodoro
  {d: `M ${heartBottom.x} ${heartBottom.y} C ${heartBottom.x + 120} ${heartBottom.y + 140}, ${tomato.cx - 380} ${tomato.cy + 300}, ${tomato.cx} ${tomato.cy + ringR} ${loop(tomato.cx, tomato.cy, ringR, ringR, Math.PI / 2).replace('M', 'L')}`, t: T.toTomato, tool: 'crayon', ease: sineInOut, width: 9, fade: [8.95, 9.3]},
  // chalk underline under the slogan
  {d: `M ${sloganRow[0] + 10} ${sloganRow[2] + 92} C ${sloganRow[0] + 300} ${sloganRow[2] + 118}, ${sloganRow[1] - 300} ${sloganRow[2] + 72}, ${sloganRow[1] + 30} ${sloganRow[2] + 98}`, t: T.underline, tool: 'chalk', ease: expoOut, width: 13},
];
const SEG_LEN = SEGS.map((s) => getLength(s.d));

type Hand = {tool: Tool; x: number; y: number; dx: number; dy: number};
/** the tool in hand at time t (world px) and the way its stroke is heading */
const toolAt = (t: number): Hand | null => {
  let best: Hand | null = null;
  SEGS.forEach((s, i) => {
    if (t >= s.t[0] && t <= s.t[1] + 0.25) {
      const p = (s.ease ?? ((x: number) => x))(prog(t, s.t));
      const len = SEG_LEN[i] * p;
      const pt = getPointAtLength(s.d, len);
      const tg = getTangentAtLength(s.d, Math.max(0.1, len));
      if (pt && tg) best = {tool: s.tool, x: pt.x, y: pt.y, dx: tg.x, dy: tg.y};
    }
  });
  return best;
};

// ───────────────────────── camera ─────────────────────────

type Key = {t: number; x: number; y: number; z: number; e?: (x: number) => number};
const todoC = {x: P.todo.x + 210, y: P.todo.y + 175};
/** rule of thirds: put the subject right of centre, the words get the left third */
const R = (x: number, z: number) => x - 300 / z;
const KEYS_RAW: (Key & {right?: boolean})[] = [
  {t: 0, x: todoC.x + 30, y: todoC.y + 10, z: 2.1},
  {t: 0.75, x: todoC.x, y: todoC.y, z: 1.9, e: sineInOut},
  {t: 2.0, x: todoC.x + 30, y: todoC.y - 5, z: 1.82, e: sineInOut},
  {t: 2.6, x: P.rose.x + rose.w / 2, y: P.rose.y + 170, z: 1.55, e: expoInOut},
  {t: 3.1, x: P.rose.x + rose.w / 2 + 20, y: P.rose.y + 180, z: 1.5, e: sineInOut},
  {t: 3.75, x: 1500, y: 330, z: 1.05, e: sineInOut},
  {t: 4.45, x: P.picture.cx + 20, y: P.picture.cy - 10, z: 1.28, e: expoInOut},
  {t: 5.6, x: P.picture.cx + 60, y: P.picture.cy - 40, z: 1.18, e: sineInOut},
  {t: 6.3, x: P.oat.x + oat.w / 2, y: P.oat.y + oat.h / 2 + 50, z: 1.4, e: expoInOut},
  {t: 6.7, x: P.oat.x + oat.w / 2 + 50, y: P.oat.y + oat.h / 2 + 40, z: 1.75, e: sineInOut},
  {t: 7.9, x: heartC.x - 20, y: heartC.y + 30, z: 1.82, e: sineInOut},
  {t: 8.95, x: tomato.cx, y: tomato.cy, z: 1.95, e: sineInOut},
  {t: 11.3, x: tomato.cx, y: tomato.cy, z: 2.02, e: sineInOut},
  {t: 12.3, x: tomato.cx, y: tomato.cy, z: 1.95, e: sineInOut},
  {t: 12.95, x: tomato.cx, y: tomato.cy + 10, z: 1.9, e: sineInOut},
  {t: 13.5, x: tomato.cx, y: tomato.cy + 10, z: 1.62, e: sineInOut},
  {t: 14.0, x: sloganC.x, y: sloganC.y + 40, z: 1.1, e: expoInOut},
  {t: 15.8, x: sloganC.x, y: sloganC.y + 50, z: 1.14, e: sineInOut},
  {t: 16.35, x: secretC.x, y: secretC.y, z: 1.6, e: expoInOut},
  {t: 17.7, x: secretC.x, y: secretC.y, z: 1.66, e: sineInOut},
  {t: 18.95, x: BOARD.w / 2, y: BOARD.h / 2, z: W / BOARD.w, e: expoInOut},
  {t: 24, x: BOARD.w / 2, y: BOARD.h / 2 + 10, z: (W / BOARD.w) * 1.03, e: sineInOut},
];
// every beat that has words on screen frames its subject on the right
const FRAMED_RIGHT = [[0, 3.8], [4.4, 11.3], [12.3, 13.0], [16.3, 17.75]];
const KEYS: Key[] = KEYS_RAW.map((k) => (FRAMED_RIGHT.some(([a, b]) => k.t >= a && k.t <= b) ? {...k, x: R(k.x, k.z)} : k));
const BURST = ['hourglass', 'led', 'alarm', 'cat', 'dog'];
const HOLDS = [[1.75, 2.0], [7.8, 8.1], [9.05, 11.25], [15.15, 15.8], [18.95, 24]];

// ───────────────────────── words (per language) ─────────────────────────

type Word = {big: string; small: string; t: number[]; tool: Tool};
type Copy = {font: string; tracking: string; words: Word[]; burst: Record<string, string>; slogan: string; sloganFont: string};
/** caption times and tools are shared; only the words change */
const BEATS: {t: number[]; tool: Tool}[] = [
  {t: [0.8, 1.95], tool: 'pen'},
  {t: [2.68, 3.28], tool: 'pen'},
  {t: [3.32, 3.86], tool: 'pen'},
  {t: [4.5, 5.55], tool: 'scissors'},
  {t: [6.7, 8.05], tool: 'crayon'},
  {t: [9.05, 11.22], tool: 'crayon'},
  {t: [14.15, 15.8], tool: 'chalk'},
  {t: [16.4, 17.62], tool: 'lock'},
];
const words = (lines: [string, string][]): Word[] => lines.map(([big, small], i) => ({big, small, ...BEATS[i]}));
const COPY: Record<'en' | 'zh-TW', Copy> = {
  en: {
    font: FONT,
    tracking: '-0.04em',
    words: words([
      ['Cross it off.', 'Click a line — it gets a red stroke'],
      ['Tape it.', 'Washi tape you can stretch'],
      ['Stick it.', 'Pins, clips & stickers'],
      ['Cut it out.', 'Any picture becomes a sticker'],
      ['Draw.', 'Pen · marker · chalk · crayon'],
      ['Focus.', 'Pomodoro timers in six styles'],
      ['Say it big.', 'Transparent paper for desktop slogans'],
      ['Keep it secret.', 'Locked notes show a harmless decoy'],
    ]),
    burst: {hourglass: 'Hourglass', led: 'LED clock', alarm: 'Alarm clock', cat: 'Cat', dog: 'Shiba'},
    slogan: 'STAY FOCUSED',
    sloganFont: 'Impact, sans-serif',
  },
  'zh-TW': {
    font: FONT_ZH_TW,
    tracking: '0.02em',
    words: words([
      ['劃掉', '點一下那一行，紅筆就劃過去'],
      ['貼紙膠帶', '紙膠帶可以拉長'],
      ['貼上去', '圖釘、夾子、貼紙'],
      ['剪下來', '任何圖片都能變成貼紙'],
      ['隨手畫', '筆 · 螢光筆 · 粉筆 · 蠟筆'],
      ['專心', '六種造型的番茄鐘'],
      ['大聲說', '透明紙，把標語掛在桌面'],
      ['藏起來', '上鎖的便利貼只顯示偽裝內容'],
    ]),
    burst: {hourglass: '沙漏', led: 'LED 數字錶', alarm: '鬧鐘', cat: '抱時鐘的貓', dog: '抱時鐘的柴犬'},
    slogan: '保持專注',
    sloganFont: FONT_ZH_TW,
  },
};
const C = COPY[LANG];
const SLOGAN = C.slogan;
const STAMP_AT = (i: number) => T.stamp[0] + i * 0.045;
const IMPACTS: [number, number][] = [[2.7, 9], [3.43, 4], [3.68, 4], [6.28, 6], [8.9, 5], [12.3, 6], [16.35, 3], ...SLOGAN.split('').map((c, i): [number, number] => [STAMP_AT(i) + 0.09, c === ' ' ? 0 : 2.2])];

const cameraAt = (t: number) => {
  // burst: hard cuts to the other five timers on the board
  if (t >= T.burst[0] && t < T.burst[1]) {
    const c = P.timers[BURST[Math.min(4, Math.floor((t - T.burst[0]) / 0.2))]];
    return {x: c.cx, y: c.cy + 30, z: 2.35, rot: 0};
  }
  let i = KEYS.findIndex((k) => k.t > t);
  if (i === -1) i = KEYS.length - 1;
  if (i === 0) i = 1;
  const a = KEYS[i - 1];
  const b = KEYS[i];
  const p = (b.e ?? sineInOut)(prog(t, [a.t, b.t]));
  let x = lerp(a.x, b.x, p);
  let y = lerp(a.y, b.y, p);
  const z = Math.exp(lerp(Math.log(a.z), Math.log(b.z), p));
  const still = HOLDS.some(([h0, h1]) => t > h0 + 0.12 && t < h1 - 0.08) ? 0 : 1;
  x += (still * (2.4 * Math.sin(t * 0.9) + 1.2 * Math.sin(t * 2.3 + 1.0))) / z;
  y += (still * (1.8 * Math.sin(t * 0.7 + 2.0) + 1.0 * Math.sin(t * 1.9))) / z;
  let rot = 0;
  for (const [ti, amp] of IMPACTS) {
    const d = t - ti;
    if (d > 0 && d < 0.5) {
      const e = Math.exp(-d * 11);
      x += (amp * e * Math.sin(d * 2 * Math.PI * 13)) / z;
      y += (amp * 0.8 * e * Math.cos(d * 2 * Math.PI * 11)) / z;
      rot += 0.012 * amp * e * Math.sin(d * 2 * Math.PI * 9);
    }
  }
  return {x, y, z, rot};
};
const toScreen = (t: number, p: {x: number; y: number}) => {
  const c = cameraAt(t);
  return {x: (p.x - c.x) * c.z + W / 2, y: (p.y - c.y) * c.z + H / 2};
};

// ───────────────────────── pieces ─────────────────────────

type SpriteProps = {name: string; x?: number; y?: number; cx?: number; cy?: number; height?: number; index?: number; opacity?: number; scale?: number; rotate?: number; dy?: number; style?: React.CSSProperties};
const Sprite: React.FC<SpriteProps> = ({name, x, y, cx, cy, height, index = 0, opacity = 1, scale = 1, rotate = 0, dy = 0, style}) => {
  const s = S[name];
  if (!s || opacity <= 0) return null;
  const file = s.files ? s.files[Math.max(0, Math.min(s.files.length - 1, index))] : s.file!;
  const f = height ? height / s.h : factor(s);
  const [pl, pt, pr, pb] = s.paper ?? [0, 0, s.w, s.h];
  const left = cx !== undefined ? cx - ((pl + pr) / 2) * f : (x ?? 0) - pl * f;
  const top = cy !== undefined ? cy - ((pt + pb) / 2) * f : (y ?? 0) - pt * f;
  return (
    <Img
      src={sprite(file)}
      style={{position: 'absolute', left, top: top + dy, width: s.w * f, height: s.h * f, opacity, transform: `rotate(${rotate}deg) scale(${scale})`, transformOrigin: `${((pl + pr) / 2) * f}px ${((pt + pb) / 2) * f}px`, ...style}}
    />
  );
};
const arrive = (t: number, t0: number) => {
  if (t < t0) return {opacity: 0, dy: -40, scale: 0.96};
  const s = springAt(t, t0, 15, 8);
  return {opacity: clamp01((t - t0) / 0.12), dy: -40 * (1 - s), scale: 0.96 + 0.04 * s};
};
const drop = (t: number, t0: number, landAt: number) => {
  if (t < t0) return {opacity: 0, dy: -90, scale: 1.25};
  if (t < landAt) {
    const u = prog(t, [t0, landAt]);
    return {opacity: clamp01(u * 3), dy: -90 * (1 - u * u), scale: 1.25 - 0.25 * u * u};
  }
  const d = t - landAt;
  return {opacity: 1, dy: 0, scale: 1 - 0.06 * Math.exp(-d * 14) * Math.cos(d * 40)};
};

/** the tool in hand (screen space); (0,0) is where it touches the paper */
const ToolIcon: React.FC<{hand: Hand; t: number}> = ({hand, t}) => {
  const {tool, x, y, dx, dy} = hand;
  const shadow = 'drop-shadow(0 10px 10px rgba(30,20,10,.35))';
  if (tool === 'scissors') {
    const ang = (Math.atan2(dy, dx) * 180) / Math.PI;
    const open = 9 + 9 * Math.sin(t * 26);
    return (
      <svg width={180} height={180} viewBox="-90 -90 180 180" style={{position: 'absolute', left: x - 90, top: y - 90, overflow: 'visible', filter: shadow}}>
        <g transform={`rotate(${ang + 180})`}>
          {[-1, 1].map((s) => (
            <g key={s} transform={`rotate(${s * open})`}>
              <path d="M 0 0 L 58 -5 L 58 5 Z" fill="#cfd4da" stroke="#7b8691" strokeWidth={1.5} />
              <ellipse cx={78} cy={s * 13} rx={15} ry={10} fill="none" stroke="#e2241e" strokeWidth={7} />
            </g>
          ))}
          <circle cx={56} cy={0} r={4} fill="#5b6672" />
        </g>
      </svg>
    );
  }
  const body = {
    pen: {len: 120, w: 15, fill: '#24324a', tip: '#d9b45a', band: '#d9b45a'},
    blue: {len: 120, w: 15, fill: '#2f6fdb', tip: '#d9b45a', band: '#d9b45a'},
    crayon: {len: 96, w: 18, fill: '#e2241e', tip: '#c81c17', band: '#f6e6c8'},
    chalk: {len: 70, w: 20, fill: '#f4f1ea', tip: '#e8e3d8', band: ''},
  }[tool];
  if (!body) return null;
  // held like a pen, leaning up-right
  return (
    <svg width={240} height={240} viewBox="-20 -220 240 240" style={{position: 'absolute', left: x - 20, top: y - 220, overflow: 'visible', filter: shadow}}>
      <g transform="rotate(-48)">
        <path d={`M 0 0 L 24 ${-body.w / 2} L 24 ${body.w / 2} Z`} fill={body.tip} />
        <rect x={24} y={-body.w / 2} width={body.len} height={body.w} rx={tool === 'chalk' ? 6 : 3} fill={body.fill} />
        {body.band && <rect x={tool === 'crayon' ? 46 : 110} y={-body.w / 2 - (tool === 'crayon' ? 0 : 1)} width={tool === 'crayon' ? 38 : 8} height={body.w + (tool === 'crayon' ? 0 : 2)} fill={body.band} />}
        {tool === 'chalk' && <rect x={24} y={-body.w / 2} width={body.len} height={body.w / 3} rx={4} fill="#ffffff" opacity={0.5} />}
      </g>
    </svg>
  );
};

/** a little padlock that pops onto the secret note and clicks open */
const Padlock: React.FC<{x: number; y: number; t: number}> = ({x, y, t}) => {
  const pop = springAt(t, T.lock, 18, 7);
  const open = expoOut(prog(t, [T.unlock, T.unlock + 0.2]));
  const gone = prog(t, [T.scramble[1] + 0.15, T.scramble[1] + 0.45]);
  return (
    <svg width={120} height={140} viewBox="-60 -80 120 140" style={{position: 'absolute', left: x - 60, top: y - 80, overflow: 'visible', opacity: clamp01(pop * 2) * (1 - gone), transform: `scale(${0.4 + 0.6 * pop})`, filter: 'drop-shadow(0 8px 10px rgba(0,0,0,.35))'}}>
      <path d="M -20 -2 L -20 -26 A 20 20 0 0 1 20 -26 L 20 -2" fill="none" stroke="#c9ccd2" strokeWidth={9} strokeLinecap="round" transform={`translate(0 ${-14 * open}) rotate(${-24 * open} -20 -2)`} />
      <rect x={-32} y={-6} width={64} height={52} rx={10} fill="#e8b84a" />
      <rect x={-32} y={-6} width={64} height={16} rx={8} fill="#f3cf72" />
      <circle cx={0} cy={18} r={6} fill="#7a5a1a" />
      <rect x={-2.5} y={18} width={5} height={14} rx={2} fill="#7a5a1a" />
    </svg>
  );
};

// kinetic type: a big verb (letters spring in), a stroke in the tool's colour, a small line under it
const Words: React.FC<{t: number; night: boolean}> = ({t, night}) => (
  <>
    {C.words.map((w, i) => {
      if (t < w.t[0] - 0.02 || t > w.t[1] + 0.02) return null;
      const out = expoIn(prog(t, [w.t[1] - 0.16, w.t[1]]));
      const sub = cubicOut(prog(t, [w.t[0] + 0.16, w.t[0] + 0.42]));
      const bar = expoOut(prog(t, [w.t[0] + 0.08, w.t[0] + 0.45]));
      return (
        <div key={i} lang={LANG} style={{position: 'absolute', left: 120, top: 790, fontFamily: C.font, color: night ? PAPER : INK, opacity: 1 - out, transform: `translateY(${-out * 26}px)`, filter: out > 0 ? `blur(${out * 6}px)` : undefined}}>
          <div style={{fontSize: 112, fontWeight: 800, letterSpacing: C.tracking, lineHeight: 1, whiteSpace: 'pre'}}>
            {w.big.split('').map((ch, k) => {
              const s = springAt(t, w.t[0] + k * 0.022, 20, 10);
              return (
                <span key={k} style={{display: 'inline-block', opacity: clamp01(s * 1.6), transform: `translateY(${(1 - s) * 70}px) rotate(${(1 - s) * 8}deg)`}}>
                  {ch}
                </span>
              );
            })}
          </div>
          <div style={{marginTop: 18, height: 7, width: 120 * bar, borderRadius: 4, background: TOOL_COLOR[w.tool]}} />
          <div style={{marginTop: 16, fontSize: 34, fontWeight: 500, opacity: sub * 0.85, transform: `translateX(${(1 - sub) * -16}px)`}}>{w.small}</div>
        </div>
      );
    })}
  </>
);

// ───────────────────────── the film ─────────────────────────

export const Film: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{backgroundColor: NIGHT}}>
      <EndLang.Provider value={LANG}>
        <CameraMotionBlur shutterAngle={180} samples={6}>
          <Scene t={frame / FPS} />
        </CameraMotionBlur>
      </EndLang.Provider>
      <Audio src={staticFile('audio/score.wav')} />
    </AbsoluteFill>
  );
};

/** The end card's words in every website language; scripts/og.mjs renders one share image per language. */
export const END_TEXT: Record<string, {tagline: string; sub: string; font: string}> = {
  en: {tagline: 'Little notes that stick with you.', sub: 'Free & open source · 6 languages · almost zero idle CPU', font: FONT},
  'zh-TW': {tagline: '桌面上的小紙條，一直陪著你。', sub: '拼拼 · 免費開源 · 6 種語言 · 閒置幾乎不耗電', font: `"Microsoft JhengHei UI", "Microsoft JhengHei", ${FONT}`},
  'zh-CN': {tagline: '桌面上的小纸条，一直陪着你。', sub: '拼拼 · 免费开源 · 6 种语言 · 闲置几乎不耗电', font: `"Microsoft YaHei UI", "Microsoft YaHei", ${FONT}`},
  ja: {tagline: 'デスクトップの小さなメモが、いつもそばに。', sub: '無料・オープンソース · 6 言語 · アイドル時の CPU はほぼゼロ', font: `"Yu Gothic UI", "Meiryo UI", ${FONT}`},
  ko: {tagline: '바탕화면의 작은 메모가 늘 곁에 있어요.', sub: '무료 오픈소스 · 6개 언어 · 대기 중 CPU 거의 0%', font: `"Malgun Gothic", ${FONT}`},
  ar: {tagline: 'ملاحظات صغيرة تبقى معك.', sub: 'مجاني ومفتوح المصدر · 6 لغات · استهلاك شبه معدوم للمعالج', font: FONT},
};
const EndLang = React.createContext('en');
/** the moment the end card is complete (tagline and sub-line in), before the fade */
const STILL_AT = 22.5;

/** The finished board with the end card in one language — the source of the website's share images. */
export const EndStill: React.FC<{lang: string}> = ({lang}) => (
  <EndLang.Provider value={lang}>
    <AbsoluteFill style={{backgroundColor: NIGHT}}>
      <Scene t={STILL_AT} />
    </AbsoluteFill>
  </EndLang.Provider>
);

const STROKE_FILTERS = (
  <defs>
    <filter id="chalk" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="7" result="n" />
      <feColorMatrix in="n" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 3 -1.1" result="mask" />
      <feComposite in="SourceGraphic" in2="mask" operator="in" result="grain" />
      <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="1" seed="2" result="w" />
      <feDisplacementMap in="grain" in2="w" scale="5" />
    </filter>
    <filter id="crayon" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.55" numOctaves="2" seed="4" result="n" />
      <feColorMatrix in="n" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 2.4 -0.45" result="mask" />
      <feComposite in="SourceGraphic" in2="mask" operator="in" result="wax" />
      <feTurbulence type="fractalNoise" baseFrequency="0.08" numOctaves="1" seed="9" result="w" />
      <feDisplacementMap in="wax" in2="w" scale="3" />
    </filter>
  </defs>
);

const Scene: React.FC<{t: number}> = ({t}) => {
  const cam = cameraAt(t);
  const inBurst = t >= T.burst[0] && t < T.burst[1];
  const show = (a: number, b = 99) => t >= a && t < b;

  // the tool in hand (screen space); the crayon follows the heart, the chalk sweeps along the slogan
  const w = toolAt(t);
  let hand: Hand | null = w ? {...w, ...toScreen(t, w)} : null;


  // night: the tomato's ripple opens an iris of dusk from the tomato
  const irisP = Easing.inOut(Easing.cubic)(prog(t, T.iris));
  const irisC = toScreen(t, {x: tomato.cx, y: tomato.cy});
  const irisR = irisP * 2600;
  const wall = (src: string) => (
    <Img src={staticFile(src)} style={{position: 'absolute', left: -80 - (cam.x - BOARD.w / 2) * 0.015, top: -50 - (cam.y - BOARD.h / 2) * 0.015, width: W + 160, height: H + 100, objectFit: 'cover'}} />
  );

  const world = (
    <div style={{position: 'absolute', left: 0, top: 0, transformOrigin: '0 0', transform: `translate(${W / 2 - cam.x * cam.z}px, ${H / 2 - cam.y * cam.z}px) scale(${cam.z}) rotate(${cam.rot}deg)`}}>
      {/* the rest of the desktop is there from the first frame: every move crosses content, the pull-out just reveals it */}
      {FILLERS.map((f) => <Sprite key={f.name} name={f.name} x={f.x} y={f.y} cx={f.cx} cy={f.cy} rotate={f.rot ?? 0} />)}

      {/* top row: to-do, rose + tape + cat, sage + pin */}
      <Sprite name="note-todo" x={P.todo.x} y={P.todo.y} />
      <Sprite name="note-rose" x={P.rose.x} y={P.rose.y} {...arrive(t, T.roseIn)} />
      {show(T.tape[1] - 0.04) && <Sprite name="tape" x={tapeA.x} y={tapeA.y} opacity={prog(t, [T.tape[1] - 0.04, T.tape[1] + 0.08])} scale={1 + 0.08 * Math.exp(-(t - T.tape[1]) * 12)} />}
      <Sprite name="st-cat" cx={P.rose.x + rose.w - 70} cy={P.rose.y + rose.h - 62} rotate={-8} {...drop(t, T.catDrop[0], T.catDrop[1])} />
      <Sprite name="note-sage" x={P.sage.x} y={P.sage.y} {...arrive(t, T.sageIn)} />
      <Sprite name="pin-gold" cx={P.sage.x + 70} cy={P.sage.y + 12} {...drop(t, T.pinDrop[0], T.pinDrop[1])} />

      {/* oat note: the sticker lands on it, then the heart */}
      {show(T.oatIn) && (() => {
        const a = arrive(t, T.oatIn);
        const bar = prog(t, [T.inkBar, T.inkBar + 0.25]) * (1 - prog(t, [T.heart[1] + 0.3, T.heart[1] + 0.55]));
        return (
          <>
            <Sprite name="note-oat" x={P.oat.x} y={P.oat.y} opacity={a.opacity * (1 - bar)} dy={a.dy} scale={a.scale} />
            {bar > 0 && <Sprite name="note-oat-ink" x={P.oat.x} y={P.oat.y} opacity={bar} />}
          </>
        );
      })()}
      {/* the picture card sits in the (still empty) centre of the board, then drops away */}
      {show(T.pictureIn, T.pop + 0.6) && (() => {
        const a = arrive(t, T.pictureIn);
        const away = prog(t, [T.pop - 0.08, T.pop + 0.45]);
        return (
          <div style={{position: 'absolute', left: P.picture.cx - picW / 2 - 18, top: P.picture.cy - picH / 2 - 18 + a.dy + 160 * Easing.in(Easing.cubic)(away), width: picW + 36, height: picH + 36, borderRadius: 22, background: '#fff', boxShadow: '0 30px 60px -24px rgba(60,40,20,.45)', opacity: a.opacity * (1 - away), transform: `scale(${a.scale * (1 - 0.15 * away)})`}}>
            <Img src={sprite('picture.png')} style={{position: 'absolute', left: 18, top: 18, width: picW, height: picH, borderRadius: 10}} />
          </div>
        );
      })()}
      {show(T.pop - 0.05) && (() => {
        const pop = springAt(t, T.pop, 14, 6);
        const fly = expoInOut(prog(t, T.fly));
        const cx = lerp(P.picture.cx, P.oat.x + 95 * WS, fly);
        const cy = lerp(P.picture.cy, P.oat.y + 140 * WS, fly) - Math.sin(fly * Math.PI) * 180;
        const h = lerp(picH * 0.98, 190, fly);
        const land = t > T.fly[1] ? 1 - 0.07 * Math.exp(-(t - T.fly[1]) * 14) * Math.cos((t - T.fly[1]) * 40) : 1;
        return <Sprite name="picture-sticker" cx={cx} cy={cy} height={h} scale={(0.92 + 0.08 * pop) * land} rotate={lerp(0, -7, fly)} opacity={clamp01((t - T.pop + 0.05) / 0.08)} style={{filter: `drop-shadow(0 ${lerp(26, 6, fly)}px ${lerp(28, 8, fly)}px rgba(60,40,20,.35))`}} />;
      })()}
      {show(T.heart[0]) && <Sprite name="heart" x={P.oat.x} y={P.oat.y} index={Math.floor(prog(t, T.heart) * 24)} />}

      {/* timers: the tomato first; the other five are already on the board when the burst cuts to them */}
      {show(T.tomatoPop - 0.02) && (() => {
        const pop = springAt(t, T.tomatoPop, 14, 6.5);
        return t >= T.ring[0] ? (
          <Sprite name="tomato-ring" cx={tomato.cx} cy={tomato.cy} index={Math.floor(prog(t, T.ring) * 15)} />
        ) : (
          <Sprite name="tomato" cx={tomato.cx} cy={tomato.cy} index={Math.floor(Math.max(0, t - T.tomatoPop - 1.0))} scale={0.6 + 0.4 * pop} opacity={clamp01((t - T.tomatoPop) / 0.06)} />
        );
      })()}
      {show(T.burst[0]) && BURST.map((n) => <Sprite key={n} name={`timer-${n}`} cx={P.timers[n].cx} cy={P.timers[n].cy} />)}

      {/* night: slogan revealed by the chalk, and the secret note */}
      {show(T.stamp[0]) && (
        <div lang={LANG} style={{position: 'absolute', left: sloganRow[0] + 3, top: sloganRow[2] - 64, fontFamily: C.sloganFont, fontWeight: LANG === 'en' ? undefined : 700, fontSize: 100, lineHeight: '128px', color: '#f7cf3b', whiteSpace: 'pre', display: 'flex'}}>
          {SLOGAN.split('').map((c, i) => {
            const u = prog(t, [STAMP_AT(i), STAMP_AT(i) + 0.1]);
            const k = expoOut(u);
            return (
              <span key={i} style={{display: 'inline-block', opacity: u > 0 ? 1 : 0, transform: `scale(${lerp(1.9, 1, k)}) rotate(${(1 - k) * (i % 2 ? 6 : -6)}deg)`, filter: u < 1 ? `blur(${(1 - k) * 4}px)` : undefined}}>
                {c}
              </span>
            );
          })}
        </div>
      )}
      <Sprite name="secret" x={P.secret.x} y={P.secret.y} index={Math.floor(prog(t, T.scramble) * 20)} />
      {show(T.lock - 0.02, T.scramble[1] + 0.45) && <Padlock x={P.secret.x + secret.w - 50} y={P.secret.y + 26} t={t} />}

      {/* strokes */}
      <svg style={{position: 'absolute', left: -2000, top: -2000, overflow: 'visible'}} width={8000} height={6500} viewBox="-2000 -2000 8000 6500">
        {STROKE_FILTERS}
        <g fill="none" strokeLinecap="round" strokeLinejoin="round">
          {SEGS.map((s, i) => {
            if (s.hidden || t < s.t[0]) return null;
            const p = (s.ease ?? ((x: number) => x))(prog(t, s.t));
            const fade = s.fade ? 1 - prog(t, s.fade) : 1;
            if (fade <= 0) return null;
            const ev = evolvePath(p, s.d);
            const width = s.tape ? lerp(5, 34, expoOut(prog(t, T.tape))) : s.width ?? 5;
            const filter = s.tool === 'chalk' ? 'url(#chalk)' : s.tool === 'crayon' ? 'url(#crayon)' : undefined;
            if (s.dash) {
              // a dashed cutting line, revealed through a mask so the dashes stay in place
              return (
                <g key={i} opacity={fade}>
                  <mask id={`m${i}`}>
                    <path d={s.d} stroke="#fff" strokeWidth={width + 12} strokeDasharray={ev.strokeDasharray} strokeDashoffset={ev.strokeDashoffset} />
                  </mask>
                  <path d={s.d} stroke="#fff" strokeWidth={width + 4} opacity={0.85} mask={`url(#m${i})`} />
                  <path d={s.d} stroke={TOOL_COLOR.scissors} strokeWidth={width} strokeDasharray={s.dash} mask={`url(#m${i})`} />
                </g>
              );
            }
            return <path key={i} d={s.d} stroke={TOOL_COLOR[s.tool]} strokeWidth={width} strokeDasharray={ev.strokeDasharray} strokeDashoffset={ev.strokeDashoffset} opacity={fade} filter={filter} />;
          })}
        </g>
      </svg>
    </div>
  );

  const night = t >= T.iris[0];
  const nightGround = <AbsoluteFill style={{background: 'radial-gradient(ellipse 120% 90% at 50% 40%, #233049 0%, #18223a 45%, #0f1626 100%)'}} />;
  return (
    <AbsoluteFill>
      {wall('wall-light.png')}
      {night && <div style={{position: 'absolute', inset: 0, clipPath: t < T.iris[1] ? `circle(${irisR}px at ${irisC.x}px ${irisC.y}px)` : undefined}}>{nightGround}</div>}
      {night && t < T.iris[1] + 0.1 && (
        <div style={{position: 'absolute', left: irisC.x - irisR, top: irisC.y - irisR, width: irisR * 2, height: irisR * 2, borderRadius: '50%', border: `${lerp(10, 2, irisP)}px solid rgba(255,236,200,${0.55 * (1 - irisP)})`, boxSizing: 'border-box'}} />
      )}
      {world}
      {hand && !inBurst && <ToolIcon hand={hand} t={t} />}
      {/* spotlight on the secret note */}
      {t > T.toSecret[0] && t < T.pullOut[0] + 0.4 && (() => {
        const c = toScreen(t, secretC);
        const a = prog(t, [T.toSecret[1] - 0.1, T.toSecret[1] + 0.3]) * (1 - prog(t, [T.pullOut[0], T.pullOut[0] + 0.4]));
        return <AbsoluteFill style={{background: `radial-gradient(circle 560px at ${c.x}px ${c.y}px, rgba(8,12,20,0) 55%, rgba(8,12,20,${0.55 * a}) 100%)`}} />;
      })()}
      <Words t={t} night={night && t > T.iris[1] - 0.2} />
      {inBurst && (
        <div lang={LANG} style={{position: 'absolute', left: 0, right: 0, top: H - 150, textAlign: 'center', fontFamily: C.font, fontSize: 44, fontWeight: 700, color: INK}}>
          {C.burst[BURST[Math.min(4, Math.floor((t - T.burst[0]) / 0.2))]]}
        </div>
      )}
      <EndCard t={t} />
    </AbsoluteFill>
  );
};

const EndCard: React.FC<{t: number}> = ({t}) => {
  const lang = React.useContext(EndLang);
  const words = END_TEXT[lang] ?? END_TEXT.en;
  if (t < T.logo - 0.4) return null;
  const veil = prog(t, [T.logo - 0.4, T.logo + 0.2]);
  const logo = springAt(t, T.logo, 13, 6);
  const write = sineInOut(prog(t, T.wordmark));
  const under = expoOut(prog(t, T.endUnderline));
  const tag = cubicOut(prog(t, [T.endUnderline[1] - 0.05, T.endUnderline[1] + 0.45]));
  const fade = prog(t, T.fadeOut);
  const ux0 = W / 2 - 395;
  const ux1 = W / 2 + 395;
  const uy = 640;
  const underline = wobbly(ux0, uy, ux1, uy - 4, 5);
  const ev = evolvePath(under, underline);
  const penX = t < T.wordmark[1] ? lerp(ux0, ux1, write) : lerp(ux0, ux1, under);
  return (
    <AbsoluteFill>
      {/* a soft spotlight in the clear centre of the board */}
      <AbsoluteFill style={{background: `radial-gradient(ellipse 640px 360px at 50% 50%, rgba(18,26,38,${0.72 * veil}), rgba(18,26,38,${0.25 * veil}) 70%, rgba(18,26,38,${0.1 * veil}))`}} />
      <div style={{position: 'absolute', left: W / 2 - 95, top: 250 - 60 * (1 - logo), width: 190, height: 190, opacity: clamp01(logo * 2), transform: `scale(${0.85 + 0.15 * logo})`}}>
        <Img src={sprite('logo.png')} style={{width: 190, height: 190}} />
      </div>
      <div style={{position: 'absolute', left: 0, right: 0, top: 440, textAlign: 'center', fontFamily: FONT, fontWeight: 800, fontSize: 136, letterSpacing: '-0.04em', color: PAPER, clipPath: `inset(-10% ${100 - write * 100}% -10% 0)`}}>
        PinPin Notes
      </div>
      <svg width={W} height={H} style={{position: 'absolute', left: 0, top: 0}}>
        <path d={underline} fill="none" stroke="#e2241e" strokeWidth={9} strokeLinecap="round" strokeDasharray={ev.strokeDasharray} strokeDashoffset={ev.strokeDashoffset} />
      </svg>
      {t >= T.wordmark[0] - 0.15 && t < T.endUnderline[1] + 0.15 && <ToolIcon hand={{tool: 'pen', x: penX, y: t < T.wordmark[1] ? uy - 60 : uy, dx: 1, dy: 0}} t={t} />}
      <div lang={lang} dir={lang === 'ar' ? 'rtl' : 'ltr'} style={{position: 'absolute', left: 0, right: 0, top: 690, textAlign: 'center', fontFamily: words.font, fontSize: 40, fontWeight: 600, color: PAPER, opacity: tag, transform: `translateY(${(1 - tag) * 20}px)`}}>
        {words.tagline}
      </div>
      <div lang={lang} dir={lang === 'ar' ? 'rtl' : 'ltr'} style={{position: 'absolute', left: 0, right: 0, top: 752, textAlign: 'center', fontFamily: words.font, fontSize: 27, color: '#c9bfb6', opacity: cubicOut(prog(t, [T.endUnderline[1] + 0.25, T.endUnderline[1] + 0.8]))}}>
        {words.sub}
      </div>
      <AbsoluteFill style={{background: NIGHT, opacity: fade}} />
    </AbsoluteFill>
  );
};
