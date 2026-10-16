// PinPin Notes — feature clips (no captions), paced like the promo film on the site. The camera never chases
// the cursor: it pushes in on the UI while the hand picks something, opens up to the whole desk as it is placed,
// and leans in on the thing being resized or turned. A little shake when something lands.
//
//   clip-polaroid  drag pet photos out of a photo window → polaroids; spin them with the rotate knob
//   clip-stickers  stickers from the shop onto a note and the desktop; resize, rotate, then a quick burst
//   clip-paste     drop any picture (an anime character, a pop idol) into the shop → die-cut stickers → desktop
//   clip-lucky     spring couplets: 旺, 日進斗金, 天官賜福, 金榜題名 (upright) — then 日進斗金 grows and turns
//   clip-timer     the six pomodoro timers pop in, quick cuts through each, the tomato counts down and rings
//
// Timing and sound events live in src/clips.json. Recorded effects ("samples": click, whoosh, page turn,
// shutter — CC0) are played here; scripts/clip-sounds.mjs synthesises the rest plus the marimba music.
// Every UI picture is a real PinPin render (scripts/clips-assets.sh → public/clips). The photos are
// AI-generated: made-up pets, an original anime character, a fictional idol. Nothing touches a real desktop.

import React, {useEffect, useState} from 'react';
import {AbsoluteFill, Audio, Img, Sequence, staticFile, useCurrentFrame, continueRender, delayRender, Easing} from 'remotion';
import CL from './clips.json';

type ClipDef = {duration: number; T: Record<string, any>; events: {t: number; kind: string; dur?: number}[]};
const DEFS = CL.clips as unknown as Record<string, ClipDef>;
/** recorded effects (CC0) played here; everything else is in the clip's wav */
const SAMPLES = CL.samples as Record<string, {file: string; volume: number; rate?: number}>;
export const CLIP_FPS = CL.fps;
export const CLIPS: Record<string, number> = Object.fromEntries(Object.entries(DEFS).map(([id, c]) => [id, c.duration]));

type SpriteInfo = {file: string; w: number; h: number; paper?: number[]; off?: number[]; scale?: number};
type Manifest = {scale: number; sprites: Record<string, SpriteInfo>};
type P = [number, number];

const W = 1920;
const H = 1080;
/** The recording pretends to be a 1080p screen at 125 %. */
const DPI = 1.25;
/** shop renders are 2× (PINPIN_DPI=2) */
const SHOP_F = DPI / 2;
const SHOP = {w: 1120 * SHOP_F, h: 1240 * SHOP_F};
const SHOP_AT: P = [1140, 130];

// ───────────────────────── curves ─────────────────────────

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
const prog = (t: number, a: number, b: number) => clamp01((t - a) / (b - a));
const span = (t: number, ab: number[]) => prog(t, ab[0], ab[1]);
const sineInOut = Easing.inOut(Easing.sin);
const cubicInOut = Easing.inOut(Easing.cubic);
const quintInOut = Easing.inOut(Easing.poly(5));
const cubicOut = Easing.out(Easing.cubic);
const lerp = (a: number, b: number, p: number) => a + (b - a) * p;
const springAt = (t: number, t0: number, stiff = 30, damp = 11) => {
  const x = t - t0;
  if (x <= 0) return 0;
  return 1 - Math.exp(-damp * x) * Math.cos(stiff * x);
};
/** rotate p around c by deg (screen coordinates, positive = clockwise, same as CSS and PinPin) */
const rot = (p: P, c: P, deg: number): P => {
  const a = (deg * Math.PI) / 180;
  const dx = p[0] - c[0];
  const dy = p[1] - c[1];
  return [c[0] + dx * Math.cos(a) - dy * Math.sin(a), c[1] + dx * Math.sin(a) + dy * Math.cos(a)];
};
/** dropped things land a touch big and snap down */
const settle = (t: number, t0: number) => (t < t0 ? 0 : 1.16 - 0.16 * springAt(t, t0));

// ───────────────────────── cursor & camera ─────────────────────────

/** Move to a point (eased, slight arc like a real hand) or follow a function of u (0→1). */
type Seg = {at: number[]; to: P | ((u: number) => P); e?: (x: number) => number};

const track = (t: number, start: P, segs: Seg[]): P => {
  let p = start;
  for (const s of segs) {
    if (t < s.at[0]) return p;
    const u = span(t, s.at);
    const end = typeof s.to === 'function' ? s.to(1) : s.to;
    if (t < s.at[1]) {
      if (typeof s.to === 'function') return s.to(u);
      const e = (s.e ?? quintInOut)(u);
      const dx = end[0] - p[0];
      const dy = end[1] - p[1];
      const bow = Math.sin(Math.PI * e) * 0.07;
      return [lerp(p[0], end[0], e) - dy * bow, lerp(p[1], end[1], e) + dx * bow];
    }
    p = end;
  }
  return p;
};
const isDown = (t: number, spans: number[][]) => spans.some(([a, b]) => t >= a && t < b);

/** camera key: zoom and the point to look at */
type Cam = {t: number; z: number; at: P};
const IMPACT: Record<string, number> = {plop: 3};
/** the whole screen */
const WIDE: P = [960, 540];
/** close on the sticker shop (the camera stops at the screen's edge, so the shop sits at the right) */
const AT_SHOP: P = [1500, 600];
const ZIN = 1.8;
/** close on the thing being worked on */
const ZWORK = 1.45;
/**
 * Pick from the UI close up, place on the desk wide: zoomed in by the time the hand reaches the item (`reach`,
 * the push starts at the previous key), stay close as the drag starts, open up to the whole screen as it lands.
 */
const pickShots = (ui: P, reach: number[], drag: number[], z = ZIN): Cam[] => [
  {t: reach[1], z, at: ui},
  {t: drag[0] + 0.12, z, at: ui},
  {t: drag[1], z: 1, at: WIDE},
];
/** ends every clip: settle back to the whole desk and breathe */
const ending = (D: ClipDef, pull: number[]): Cam[] => [
  {t: pull[1], z: 1, at: WIDE},
  {t: D.duration, z: 1.02, at: WIDE},
];

const camera = (t: number, keys: Cam[], events: ClipDef['events']) => {
  let i = keys.findIndex((k) => k.t > t);
  if (i === -1) i = keys.length - 1;
  if (i === 0) i = 1;
  const a = keys[i - 1];
  const b = keys[i];
  const p = cubicInOut(prog(t, a.t, b.t));
  const z = Math.exp(lerp(Math.log(a.z), Math.log(b.z), p));
  let x = lerp(a.at[0], b.at[0], p);
  let y = lerp(a.at[1], b.at[1], p);
  // a hand-held breath, as in the film
  x += (2.4 * Math.sin(t * 0.9) + 1.2 * Math.sin(t * 2.3 + 1.0)) / z;
  y += (1.8 * Math.sin(t * 0.7 + 2.0) + 1.0 * Math.sin(t * 1.9)) / z;
  // never show past the screen's edge
  x = Math.min(W - W / (2 * z), Math.max(W / (2 * z), x));
  y = Math.min(H - H / (2 * z), Math.max(H / (2 * z), y));
  let r = 0;
  for (const e of events) {
    const amp = IMPACT[e.kind];
    const d = t - e.t;
    if (amp && d > 0 && d < 0.4) {
      const k = Math.exp(-d * 13);
      x += (amp * k * Math.sin(d * 2 * Math.PI * 13)) / z;
      y += (amp * 0.8 * k * Math.cos(d * 2 * Math.PI * 11)) / z;
      r += 0.01 * amp * k * Math.sin(d * 2 * Math.PI * 9);
    }
  }
  return {x, y, z, r};
};

// ───────────────────────── pieces ─────────────────────────

const useManifest = () => {
  const [m, setM] = useState<Manifest | null>(null);
  const [handle] = useState(() => delayRender('clips.json'));
  useEffect(() => {
    fetch(staticFile('clips/clips.json'))
      .then((r) => r.json())
      .then((j) => {
        setM(j);
        continueRender(handle);
      });
  }, [handle]);
  return m;
};

const Wall: React.FC = () => (
  <Img src={staticFile('wall-light.png')} style={{position: 'absolute', left: 0, top: 0, width: W, height: H, objectFit: 'cover'}} />
);

/** Windows 11 arrow cursor */
const Cursor: React.FC<{p: P; down: boolean}> = ({p, down}) => (
  <svg
    width={34}
    height={44}
    viewBox="0 0 17 22"
    style={{position: 'absolute', left: p[0] - 1.6, top: p[1] - 1.2, transform: `scale(${down ? 0.88 : 1})`, transformOrigin: '2px 2px', filter: 'drop-shadow(0 1.5px 2px rgba(0,0,0,.35))'}}
  >
    <path d="M1 1 L1 17 L5 13.2 L8 20 L10.6 18.9 L7.7 12.3 L12.8 12.3 Z" fill="#fff" stroke="#111" strokeWidth={1.1} strokeLinejoin="round" />
  </svg>
);

/** Plain app window (title bar with the three caption buttons, no title text). */
const Win: React.FC<{x: number; y: number; w: number; h: number; dark?: boolean; opacity?: number; scale?: number; children?: React.ReactNode}> = ({x, y, w, h, dark, opacity = 1, scale = 1, children}) => {
  const fg = dark ? '#e8e8e8' : '#333';
  return (
    <div
      style={{
        position: 'absolute', left: x, top: y, width: w, height: h, borderRadius: 9, overflow: 'hidden', opacity,
        transform: `scale(${scale})`, background: dark ? '#202020' : '#f6f6f6',
        boxShadow: '0 22px 60px rgba(0,0,0,.30), 0 0 0 1px rgba(0,0,0,.10)',
      }}
    >
      <div style={{height: 40, display: 'flex', justifyContent: 'flex-end', alignItems: 'center', background: dark ? '#2b2b2b' : '#efefef'}}>
        <svg width={138} height={40} viewBox="0 0 138 40">
          <line x1={18} y1={20} x2={28} y2={20} stroke={fg} strokeWidth={1} />
          <rect x={64} y={15} width={10} height={10} fill="none" stroke={fg} strokeWidth={1} />
          <line x1={110} y1={15} x2={120} y2={25} stroke={fg} strokeWidth={1} />
          <line x1={120} y1={15} x2={110} y2={25} stroke={fg} strokeWidth={1} />
        </svg>
      </div>
      <div style={{position: 'absolute', left: 0, top: 40, right: 0, bottom: 0}}>{children}</div>
    </div>
  );
};

/** The sticker shop (it draws its own title bar). `flash` = a new tile lighting up (shop pixels). */
const Shop: React.FC<{frames: {file: string; a: number}[]; flash?: {x: number; y: number; a: number}}> = ({frames, flash}) => (
  <div style={{position: 'absolute', left: SHOP_AT[0], top: SHOP_AT[1], width: SHOP.w, height: SHOP.h, borderRadius: 9, overflow: 'hidden', background: '#fafafa', boxShadow: '0 22px 60px rgba(0,0,0,.30), 0 0 0 1px rgba(0,0,0,.10)'}}>
    {frames.filter((f) => f.a > 0).map((f) => (
      <Img key={f.file} src={staticFile(`clips/${f.file}`)} style={{position: 'absolute', left: 0, top: 0, width: SHOP.w, height: SHOP.h, opacity: f.a}} />
    ))}
    {flash && flash.a > 0 && (
      <div style={{position: 'absolute', left: flash.x * SHOP_F - 75, top: flash.y * SHOP_F - 78, width: 150, height: 156, borderRadius: 10, background: 'rgba(255,255,255,.9)', opacity: flash.a, boxShadow: '0 0 30px 10px rgba(130,120,255,.5)'}} />
    )}
  </div>
);
/** a point inside the shop (shop pixels, 2×) */
const shopPt = (sx: number, sy: number): P => [SHOP_AT[0] + sx * SHOP_F, SHOP_AT[1] + sy * SHOP_F];
/** centre of a sticker card's picture: 4 columns, rows 287.5 px apart, `scroll` in shop pixels */
const tile = (col: number, row: number, scroll = 0): P => shopPt(170 + 260 * col, 615 + 287.5 * row - scroll);

// ───────────────────────── sprites ─────────────────────────

/** selection box in sprite pixels: [x0, y0, x1, y1] (rotate knob = top-right, resize = bottom-right) */
const BOX: Record<string, number[]> = {
  'dk-st-shiba': [73, 137, 318, 394],
  'dk-my-idol': [128, 112, 321, 477],
  'dk-cn-rijin': [96, 159, 446, 524],
};

const kit = (m: Manifest) => {
  const F = DPI / m.scale;
  const S = m.sprites;
  /** a note (or polaroid) with its paper's top-left at (x, y); rotated renders carry their own offset */
  const note = (name: string, x: number, y: number) => {
    const s = S[name];
    const [l, t] = s.paper!;
    const [ox, oy] = s.off ?? [0, 0];
    return <Img key={name} src={staticFile(`clips/${s.file}`)} style={{position: 'absolute', left: x + (ox - l) * F, top: y + (oy - t) * F, width: s.w * F, height: s.h * F}} />;
  };
  /** a desktop sticker centred on (cx, cy), resized and rotated around its centre like PinPin does */
  const sticker = (name: string, c: P, o: {scale?: number; rot?: number; opacity?: number; key?: string} = {}) => {
    const s = S[name];
    const [ax, ay] = s.paper!;
    return (
      <Img
        key={o.key ?? name}
        src={staticFile(`clips/${s.file}`)}
        style={{
          position: 'absolute', left: c[0] - ax * F, top: c[1] - ay * F, width: s.w * F, height: s.h * F, opacity: o.opacity ?? 1,
          transformOrigin: `${ax * F}px ${ay * F}px`, transform: `rotate(${o.rot ?? 0}deg) scale(${o.scale ?? 1})`,
        }}
      />
    );
  };
  /** where a selection handle is on screen */
  const handle = (name: string, which: 'rotate' | 'resize', c: P, scale = 1, deg = 0): P => {
    const [ax, ay] = S[name].paper!;
    const box = BOX[name];
    const h: P = which === 'rotate' ? [box[2], box[1]] : [box[2], box[3]];
    return rot([c[0] + (h[0] - ax) * F * scale, c[1] + (h[1] - ay) * F * scale], c, deg);
  };
  return {F, S, note, sticker, handle};
};
type Kit = ReturnType<typeof kit>;

/** one clip = a cursor track, camera keys and the world drawn at time t */
type Body = (k: Kit, T: Record<string, any>, D: ClipDef) => {cur: (t: number) => P; down: number[][]; cam: Cam[]; draw: (t: number, cur: P) => React.ReactNode};

// ───────────────────────── 1. pet photo → polaroid ─────────────────────────

const polaroid: Body = (k, T, D) => {
  const gal = {x: 1300, y: 690, w: 540, h: 330};
  const thumb = (i: number) => ({x: 14 + i * 263, y: 14, w: 249, h: 262});
  const thumbC = (i: number): P => [gal.x + thumb(i).x + thumb(i).w / 2, gal.y + 40 + thumb(i).y + thumb(i).h / 2];
  const GAL: P = [gal.x + gal.w / 2, gal.y + gal.h / 2];
  const pets = [
    {name: 'cat', at: [600, 130] as P, drop: T.catDrop, rotA: T.catRot, to: -8},
    {name: 'corgi', at: [1050, 105] as P, drop: T.corgiDrop, rotA: T.corgiRot, to: 7},
  ];
  type Pet = (typeof pets)[0];
  const paper = (p: Pet) => {
    const [l, tp, r, b] = k.S[`polaroid-${p.name}`].paper!;
    return {w: (r - l) * k.F, h: (b - tp) * k.F};
  };
  const centre = (p: Pet): P => [p.at[0] + paper(p).w / 2, p.at[1] + paper(p).h / 2];
  // rotate knob, measured from the hover render: sprite (102.5, 225)
  const knob = (p: Pet, deg: number): P => {
    const [l, tp] = k.S[`polaroid-${p.name}`].paper!;
    return rot([p.at[0] + (102.5 - l) * k.F, p.at[1] + (225 - tp) * k.F], centre(p), deg);
  };
  const angleOf = (p: Pet, t: number) => p.to * cubicInOut(span(t, p.rotA));
  const [cat, corgi] = pets;
  const cur = (t: number) =>
    track(t, thumbC(0), [
      {at: T.catDrag, to: centre(cat)},
      {at: T.catKnob, to: knob(cat, 0)},
      {at: T.catRot, to: (u) => knob(cat, cat.to * cubicInOut(u))},
      {at: T.corgiThumb, to: thumbC(1)},
      {at: T.corgiDrag, to: centre(corgi)},
      {at: T.corgiKnob, to: knob(corgi, 0)},
      {at: T.corgiRot, to: (u) => knob(corgi, corgi.to * cubicInOut(u))},
      {at: T.away, to: [1200, 960]},
    ]);
  const drags = [[T.catDrag[0], T.catDrop, 0], [T.corgiDrag[0], T.corgiDrop, 1]];
  return {
    cur,
    down: [[T.catDrag[0], T.catDrop + 0.03], [T.catRot[0] - 0.02, T.catRot[1] + 0.04], [T.corgiDrag[0], T.corgiDrop + 0.03], [T.corgiRot[0] - 0.02, T.corgiRot[1] + 0.04]],
    cam: [
      {t: 0, z: 1.9, at: GAL},
      {t: T.catDrag[0] + 0.12, z: 1.9, at: GAL},
      {t: T.catDrop, z: 1, at: WIDE},
      {t: T.catKnob[0], z: 1, at: WIDE},
      {t: T.catKnob[1], z: ZWORK, at: centre(cat)},
      {t: T.catRot[1] + 0.15, z: ZWORK, at: centre(cat)},
      ...pickShots(GAL, T.corgiThumb, T.corgiDrag, 1.9),
      {t: T.corgiKnob[0], z: 1, at: WIDE},
      {t: T.corgiKnob[1], z: ZWORK, at: centre(corgi)},
      {t: T.corgiRot[1] + 0.15, z: ZWORK, at: centre(corgi)},
      ...ending(D, T.pullOut),
    ],
    draw: (t, c) => {
      const inside = (p: Pet) => {
        const ctr = centre(p);
        const q = rot(c, ctr, -angleOf(p, t));
        const s = paper(p);
        return Math.abs(q[0] - ctr[0]) < s.w / 2 + 6 && Math.abs(q[1] - ctr[1]) < s.h / 2 + 6;
      };
      const dragging = drags.find(([a, b]) => t >= a && t < b);
      return (
        <>
          <Wall />
          {k.note('note-plan', 140, 170)}
          {pets.map((p) => {
            if (t < p.drop) return null;
            const hover = inside(p) || (t >= p.rotA[0] - 0.1 && t < p.rotA[1] + 0.05);
            const s = settle(t, p.drop);
            const ctr = centre(p);
            return (
              <div key={p.name} style={{position: 'absolute', left: 0, top: 0, width: W, height: H, transformOrigin: `${ctr[0]}px ${ctr[1]}px`, transform: `scale(${s})`, opacity: prog(t, p.drop, p.drop + 0.08)}}>
                {k.note(`polaroid-${p.name}-${hover ? 'h' : 'a'}${Math.round(angleOf(p, t))}`, p.at[0], p.at[1])}
              </div>
            );
          })}
          <Win x={gal.x} y={gal.y} w={gal.w} h={gal.h}>
            {['cat', 'corgi'].map((n, i) => (
              <Img key={n} src={staticFile(`clips/photo-${n}.png`)} style={{position: 'absolute', left: thumb(i).x, top: thumb(i).y, width: thumb(i).w, height: thumb(i).h, objectFit: 'cover', borderRadius: 6, outline: dragging && dragging[2] === i ? '3px solid #4f8ef7' : 'none'}} />
            ))}
          </Win>
          {dragging && (
            <Img src={staticFile(`clips/photo-${['cat', 'corgi'][dragging[2]]}.png`)} style={{position: 'absolute', left: c[0] - 70, top: c[1] - 60, width: 140, height: 175, objectFit: 'cover', opacity: 0.72, borderRadius: 4}} />
          )}
        </>
      );
    },
  };
};

// ───────────────────────── 2. stickers from the shop ─────────────────────────

const stickers: Body = (k, T, D) => {
  const catAt: P = [545, 448];
  const shibaAt: P = [720, 760];
  const SC = 1.55;
  const DEG = 20;
  const burst = [
    {name: 'dk-st-heart', from: tile(1, 0), at: [430, 800] as P, rot: -8},
    {name: 'dk-st-star', from: tile(2, 0), at: [1000, 880] as P, rot: 10},
    {name: 'dk-st-thumbsup', from: tile(3, 0), at: [1010, 560] as P, rot: -6},
    {name: 'dk-st-fire', from: tile(2, 1), at: [470, 600] as P, rot: 7},
  ].map((b, i) => ({...b, t0: T.burst[0] + i * T.burst[1], t1: T.burst[0] + i * T.burst[1] + 0.45}));
  const segs: Seg[] = [
    {at: T.catDrag, to: catAt},
    {at: T.shibaTile, to: tile(3, 2)},
    {at: T.shibaDrag, to: shibaAt},
    {at: [T.select - 0.12, T.select], to: [shibaAt[0] + 15, shibaAt[1] + 10]},
    {at: T.toResize, to: k.handle('dk-st-shiba', 'resize', shibaAt)},
    {at: T.resize, to: (u) => k.handle('dk-st-shiba', 'resize', shibaAt, lerp(1, SC, cubicInOut(u)))},
    {at: T.toRotate, to: k.handle('dk-st-shiba', 'rotate', shibaAt, SC)},
    {at: T.rotate, to: (u) => k.handle('dk-st-shiba', 'rotate', shibaAt, SC, DEG * cubicInOut(u))},
    ...burst.flatMap((b) => [
      {at: [b.t0 - 0.15, b.t0 - 0.01], to: b.from},
      {at: [b.t0, b.t1], to: b.at},
    ]),
    {at: [T.pullOut[0], T.pullOut[1]], to: [1240, 980]},
  ];
  const cur = (t: number) => track(t, tile(2, 2), segs);
  const sc = (t: number) => lerp(1, SC, cubicInOut(span(t, T.resize)));
  const deg = (t: number) => DEG * cubicInOut(span(t, T.rotate));
  return {
    cur,
    down: [[T.catDrag[0], T.catDrop + 0.03], [T.shibaDrag[0], T.shibaDrop + 0.03], [T.select, T.select + 0.1], [T.resize[0] - 0.02, T.resize[1] + 0.04], [T.rotate[0] - 0.02, T.rotate[1] + 0.04], ...burst.map((b) => [b.t0, b.t1 + 0.02])],
    cam: [
      {t: 0, z: ZIN, at: AT_SHOP},
      {t: T.catDrag[0] + 0.12, z: ZIN, at: AT_SHOP},
      {t: T.catDrop, z: 1, at: WIDE},
      {t: T.shibaTile[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.shibaTile, T.shibaDrag),
      {t: T.select, z: ZWORK, at: [shibaAt[0] + 40, shibaAt[1] - 60]},
      {t: T.rotate[1] + 0.1, z: ZWORK, at: [shibaAt[0] + 40, shibaAt[1] - 60]},
      {t: T.burst[0], z: 1, at: WIDE},
      {t: T.pullOut[0], z: 1, at: WIDE},
      ...ending(D, T.pullOut),
    ],
    draw: (t, c) => {
      const selected = t >= T.select && t < T.burst[0];
      return (
        <>
          <Wall />
          {k.note('note-plan', 230, 200)}
          {t >= T.catDrop && k.sticker('dk-st-cat', catAt, {scale: settle(t, T.catDrop), rot: -6})}
          {t >= T.shibaDrop && k.sticker(selected ? 'dk-st-shiba-sel' : 'dk-st-shiba', shibaAt, {scale: sc(t) * settle(t, T.shibaDrop), rot: deg(t), key: 'shiba'})}
          {burst.map((b) => (t >= b.t1 ? k.sticker(b.name, b.at, {scale: settle(t, b.t1), rot: b.rot}) : null))}
          <Shop frames={[{file: 'shop-stickers.png', a: 1}]} />
          {t >= T.catDrag[0] && t < T.catDrop && k.sticker('dk-st-cat', c, {opacity: 0.85, scale: 0.9, key: 'ghost-cat'})}
          {t >= T.shibaDrag[0] && t < T.shibaDrop && k.sticker('dk-st-shiba', c, {opacity: 0.85, scale: 0.9, key: 'ghost-shiba'})}
          {burst.map((b) => (t >= b.t0 && t < b.t1 ? k.sticker(b.name, c, {opacity: 0.85, scale: 0.9, key: `ghost-${b.name}`}) : null))}
        </>
      );
    },
  };
};

// ───────────────────────── 3. any picture → sticker ─────────────────────────

const paste: Body = (k, T, D) => {
  const view = {x: 110, y: 130, w: 640, h: 780};
  const viewC: P = [view.x + view.w / 2, view.y + 40 + (view.h - 40) / 2];
  const closeBtn: P = [view.x + view.w - 23, view.y + 20];
  const drop = shopPt(560, 800);
  const tile3 = shopPt(690, 572);
  const tile4 = shopPt(950, 572);
  const animeAt: P = [380, 560];
  const idolAt: P = [760, 600];
  const SC = 1.6;
  const DEG = -10;
  const cur = (t: number) =>
    track(t, viewC, [
      {at: T.animeDrag, to: drop},
      {at: T.idolBack, to: [viewC[0] + 30, viewC[1] - 20]},
      {at: T.idolDrag, to: drop},
      {at: T.toClose, to: closeBtn},
      {at: T.tile3, to: tile3},
      {at: T.animeOut, to: animeAt},
      {at: T.tile4, to: tile4},
      {at: T.idolOut, to: idolAt},
      {at: [T.select - 0.12, T.select], to: [idolAt[0] + 10, idolAt[1] + 30]},
      {at: T.toResize, to: k.handle('dk-my-idol', 'resize', idolAt)},
      {at: T.resize, to: (u) => k.handle('dk-my-idol', 'resize', idolAt, lerp(1, SC, cubicInOut(u)))},
      {at: T.toRotate, to: k.handle('dk-my-idol', 'rotate', idolAt, SC)},
      {at: T.rotate, to: (u) => k.handle('dk-my-idol', 'rotate', idolAt, SC, DEG * cubicInOut(u))},
      {at: T.away, to: [1000, 980]},
    ]);
  const sc = (t: number) => lerp(1, SC, cubicInOut(span(t, T.resize)));
  const deg = (t: number) => DEG * cubicInOut(span(t, T.rotate));
  const work: P = [idolAt[0] + 40, idolAt[1] - 40];
  return {
    cur,
    down: [[T.animeDrag[0], T.animeMade + 0.03], [T.idolDrag[0], T.idolMade + 0.03], [T.close, T.close + 0.1], [T.animeOut[0], T.animeDrop + 0.03], [T.idolOut[0], T.idolDrop + 0.03], [T.select, T.select + 0.1], [T.resize[0] - 0.02, T.resize[1] + 0.04], [T.rotate[0] - 0.02, T.rotate[1] + 0.04], [T.deselect, T.deselect + 0.1]],
    cam: [
      {t: 0, z: 1, at: WIDE},
      {t: T.closing[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.tile3, T.animeOut),
      {t: T.tile4[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.tile4, T.idolOut),
      {t: T.select, z: ZWORK, at: work},
      {t: T.rotate[1] + 0.1, z: ZWORK, at: work},
      ...ending(D, T.pullOut),
    ],
    draw: (t, c) => {
      const f1 = prog(t, T.animeMade, T.animeMade + 0.08);
      const f2 = prog(t, T.idolMade, T.idolMade + 0.08);
      const frames = [
        {file: 'shop-stickers.png', a: 1 - f1},
        {file: 'shop-mine-1.png', a: f1 * (1 - f2)},
        {file: 'shop-mine-2.png', a: f2},
      ];
      const flash = t < T.idolMade ? {x: 690, y: 572, a: t > T.animeMade ? 0.85 * (1 - prog(t, T.animeMade, T.animeMade + 0.45)) : 0} : {x: 950, y: 572, a: 0.85 * (1 - prog(t, T.idolMade, T.idolMade + 0.45))};
      const closing = span(t, T.closing);
      const photo = t < T.swap ? 'anime' : 'idol';
      const selected = t >= T.select && t < T.deselect;
      return (
        <>
          <Wall />
          {t >= T.animeDrop && k.sticker('dk-my-anime', animeAt, {scale: settle(t, T.animeDrop), rot: -5})}
          {t >= T.idolDrop && k.sticker(selected ? 'dk-my-idol-sel' : 'dk-my-idol', idolAt, {scale: sc(t) * settle(t, T.idolDrop), rot: deg(t), key: 'idol'})}
          {closing < 1 && (
            <Win x={view.x} y={view.y} w={view.w} h={view.h} dark opacity={1 - closing} scale={1 - 0.05 * closing}>
              <Img src={staticFile(`clips/photo-${photo}.png`)} style={{position: 'absolute', left: 18, top: 18, width: view.w - 36, height: view.h - 76, objectFit: 'contain'}} />
            </Win>
          )}
          <Shop frames={frames} flash={flash} />
          {[[T.animeDrag[0], T.animeMade, 'anime'], [T.idolDrag[0], T.idolMade, 'idol']].map(([a, b, n]) =>
            t >= (a as number) && t < (b as number) ? (
              <Img key={n as string} src={staticFile(`clips/photo-${n}.png`)} style={{position: 'absolute', left: c[0] - 64, top: c[1] - 56, width: 128, height: 160, objectFit: 'cover', opacity: 0.72, borderRadius: 4}} />
            ) : null,
          )}
          {t >= T.animeOut[0] && t < T.animeDrop && k.sticker('dk-my-anime', c, {opacity: 0.85, scale: 0.85, key: 'ghost-anime'})}
          {t >= T.idolOut[0] && t < T.idolDrop && k.sticker('dk-my-idol', c, {opacity: 0.85, scale: 0.85, key: 'ghost-idol'})}
        </>
      );
    },
  };
};

// ───────────────────────── 4. lucky couplets ─────────────────────────

const lucky: Body = (k, T, D) => {
  const list = shopPt(700, 900);
  // 旺 (row 1) · 日進斗金 (row 2, scrolled 400) · 天官賜福 (row 5, 1200) · 金榜題名 upright (row 8, 1800)
  const wangAt: P = [260, 300];
  const rijinAt: P = [560, 640];
  const tgAt: P = [1010, 720];
  const jbAt: P = [830, 330];
  const SC = 1.7;
  const DEG = 12;
  const scroll = (t: number) => 400 * cubicInOut(span(t, T.scroll1)) + 800 * cubicInOut(span(t, T.scroll2)) + 600 * cubicInOut(span(t, T.scroll3));
  const cur = (t: number) =>
    track(t, tile(1, 1), [
      {at: T.wangDrag, to: wangAt},
      {at: T.toList1, to: list},
      {at: T.rijinTile, to: tile(3, 2, 400)},
      {at: T.rijinDrag, to: rijinAt},
      {at: T.toList2, to: list},
      {at: T.tianguanTile, to: tile(1, 5, 1200)},
      {at: T.tianguanDrag, to: tgAt},
      {at: T.toList3, to: list},
      {at: T.jinbangTile, to: tile(1, 8, 1800)},
      {at: T.jinbangDrag, to: jbAt},
      {at: [T.select - 0.3, T.select], to: [rijinAt[0] + 10, rijinAt[1] + 20]},
      {at: T.toResize, to: k.handle('dk-cn-rijin', 'resize', rijinAt)},
      {at: T.resize, to: (u) => k.handle('dk-cn-rijin', 'resize', rijinAt, lerp(1, SC, cubicInOut(u)))},
      {at: T.toRotate, to: k.handle('dk-cn-rijin', 'rotate', rijinAt, SC)},
      {at: T.rotate, to: (u) => k.handle('dk-cn-rijin', 'rotate', rijinAt, SC, DEG * cubicInOut(u))},
      {at: T.away, to: [1060, 990]},
    ]);
  const drags: [number, number, string][] = [
    [T.wangDrag[0], T.wangDrop, 'dk-cn-wang'],
    [T.rijinDrag[0], T.rijinDrop, 'dk-cn-rijin'],
    [T.tianguanDrag[0], T.tianguanDrop, 'dk-cl-tianguan'],
    [T.jinbangDrag[0], T.jinbangDrop, 'dk-cl-jinbang'],
  ];
  const sc = (t: number) => lerp(1, SC, cubicInOut(span(t, T.resize)));
  const deg = (t: number) => DEG * cubicInOut(span(t, T.rotate));
  const work: P = [rijinAt[0] + 40, rijinAt[1] - 40];
  return {
    cur,
    down: [...drags.map(([a, b]) => [a, b + 0.03]), [T.select, T.select + 0.1], [T.resize[0] - 0.02, T.resize[1] + 0.04], [T.rotate[0] - 0.02, T.rotate[1] + 0.04], [T.deselect, T.deselect + 0.1]],
    cam: [
      {t: 0, z: ZIN, at: AT_SHOP},
      {t: T.wangDrag[0] + 0.12, z: ZIN, at: AT_SHOP},
      {t: T.wangDrop, z: 1, at: WIDE},
      {t: T.toList1[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.toList1, T.rijinDrag),
      {t: T.toList2[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.toList2, T.tianguanDrag),
      {t: T.toList3[0], z: 1, at: WIDE},
      ...pickShots(AT_SHOP, T.toList3, T.jinbangDrag),
      {t: T.focus[0], z: 1, at: WIDE},
      {t: T.focus[1], z: ZWORK, at: work},
      {t: T.rotate[1] + 0.1, z: ZWORK, at: work},
      ...ending(D, T.pullOut),
    ],
    draw: (t, c) => {
      const selected = t >= T.select && t < T.deselect;
      const idx = Math.min(45, Math.round(scroll(t) / 40));
      return (
        <>
          <Wall />
          {t >= T.wangDrop && k.sticker('dk-cn-wang', wangAt, {scale: settle(t, T.wangDrop), rot: -4})}
          {t >= T.tianguanDrop && k.sticker('dk-cl-tianguan', tgAt, {scale: settle(t, T.tianguanDrop), rot: -2})}
          {t >= T.jinbangDrop && k.sticker('dk-cl-jinbang', jbAt, {scale: settle(t, T.jinbangDrop), rot: 3})}
          {t >= T.rijinDrop && k.sticker(selected ? 'dk-cn-rijin-sel' : 'dk-cn-rijin', rijinAt, {scale: sc(t) * settle(t, T.rijinDrop), rot: deg(t), key: 'rijin'})}
          <Shop frames={[{file: `shop-couplets-${String(idx).padStart(2, '0')}.png`, a: 1}]} />
          {drags.map(([a, b, n]) => (t >= a && t < b ? k.sticker(n, c, {opacity: 0.85, scale: 0.9, key: `ghost-${n}`}) : null))}
        </>
      );
    },
  };
};

// ───────────────────────── 5. pomodoro timers ─────────────────────────

const timers: Body = (k, T, D) => {
  const STYLES = ['tomato', 'hourglass', 'led', 'alarm', 'cat', 'dog'];
  const size = (n: string) => {
    const [l, tp, r, b] = k.S[`tm-${n}-0`].paper!;
    return {w: (r - l) * k.F, h: (b - tp) * k.F};
  };
  // one row across the screen, evenly spaced
  const gap = (W - 240 - STYLES.reduce((a, n) => a + size(n).w, 0)) / 5;
  const centres: Record<string, P> = {};
  let x = 120;
  for (const n of STYLES) {
    centres[n] = [x + size(n).w / 2, 540];
    x += size(n).w + gap;
  }
  // close-ups: hard cuts through the other five, a whip back to the tomato for the last seconds
  const cuts: Cam[] = STYLES.slice(1).flatMap((n, i) => {
    const t0 = T.burst[0] + i * T.burst[1];
    return [{t: t0, z: 2.4, at: centres[n]}, {t: t0 + T.burst[1] - 0.001, z: 2.5, at: centres[n]}];
  });
  const last = cuts[cuts.length - 1];
  return {
    cur: () => [-200, -200],
    down: [],
    cam: [
      {t: 0, z: 1.12, at: [960, 540]},
      {t: T.burst[0] - 0.001, z: 1.18, at: [960, 540]},
      ...cuts.map((c, i) => (i === 0 ? {...c, t: T.burst[0]} : c)),
      {t: T.whip[0], z: last.z, at: last.at},
      {t: T.whip[1], z: 2.3, at: centres.tomato},
      {t: T.ring[1], z: 2.15, at: centres.tomato},
      {t: T.pullOut[0], z: 2.1, at: centres.tomato},
      {t: T.pullOut[1], z: 1.05, at: [960, 540]},
      {t: D.duration, z: 1.07, at: [960, 540]},
    ],
    draw: (t) => {
      const sec = Math.min(7, Math.floor(t));
      const cd = Math.floor((t - T.countdown[0]) / T.countdown[1]);
      const ring = Math.min(15, Math.floor(span(t, T.ring) * 15));
      return (
        <>
          <Wall />
          {STYLES.map((n, i) => {
            const t0 = T.appear[0] + i * T.appear[1];
            if (t < t0) return null;
            let name = `tm-${n}-${sec}`;
            if (n === 'tomato' && t >= T.countdown[0]) name = t < T.ring[0] ? `tm-end-${Math.max(0, 3 - cd)}` : `tm-ring-${ring}`;
            const c = centres[n];
            const s = size(n);
            return (
              <div key={n} style={{position: 'absolute', left: 0, top: 0, width: W, height: H, transformOrigin: `${c[0]}px ${c[1]}px`, transform: `scale(${settle(t, t0)})`}}>
                {k.note(name, c[0] - s.w / 2, c[1] - s.h / 2)}
              </div>
            );
          })}
        </>
      );
    },
  };
};

const BODIES: Record<string, Body> = {polaroid, stickers, paste, lucky, timer: timers};

const Scene: React.FC<{id: string; m: Manifest}> = ({id, m}) => {
  const t = useCurrentFrame() / CLIP_FPS;
  const D = DEFS[id];
  const b = BODIES[id](kit(m), D.T, D);
  const c = b.cur(t);
  const cam = camera(t, b.cam, D.events);
  return (
    <AbsoluteFill style={{background: '#000', overflow: 'hidden'}}>
      <div style={{position: 'absolute', left: 0, top: 0, width: W, height: H, transformOrigin: '0 0', transform: `translate(${W / 2}px, ${H / 2}px) rotate(${cam.r}deg) scale(${cam.z}) translate(${-cam.x}px, ${-cam.y}px)`}}>
        {b.draw(t, c)}
        <Cursor p={c} down={isDown(t, b.down)} />
      </div>
    </AbsoluteFill>
  );
};

export const Clip: React.FC<{id: string}> = ({id}) => {
  const m = useManifest();
  return (
    <AbsoluteFill style={{background: '#000'}}>
      {m && <Scene id={id} m={m} />}
      <Audio src={staticFile(`audio/clip-${id}.wav`)} />
      {DEFS[id].events.map((e, i) => {
        const s = SAMPLES[e.kind];
        return s ? (
          <Sequence key={i} from={Math.round(e.t * CLIP_FPS)} layout="none">
            <Audio src={staticFile(s.file)} volume={s.volume} playbackRate={s.rate ?? 1} />
          </Sequence>
        ) : null;
      })}
    </AbsoluteFill>
  );
};
