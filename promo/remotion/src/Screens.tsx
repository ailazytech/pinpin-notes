// Website screenshots: one still per feature chapter, composed from real PinPin renders (public/sprites).
// Sizes match the website's media slots (width 1600, each slot's own aspect ratio).
//   node scripts/screens.mjs   →   ../../site/public/img/screens/<id>.webp

import React from 'react';
import {AbsoluteFill, Img, staticFile} from 'remotion';
import manifest from '../public/sprites/sprites.json';

type SpriteInfo = {file?: string; files?: string[]; w: number; h: number; paper?: number[]; rows?: number[][]; scale?: number};
const S = (manifest as any).sprites as Record<string, SpriteInfo>;
const SPRITE_SCALE = (manifest as any).scale as number;
const FONT = '"Segoe UI Variable Display", "Segoe UI", sans-serif';

/** slot id → aspect (height / width), same as the website's chapters.tsx */
export const SCREENS: Record<string, number> = {
  notes: 5.2 / 8,
  todo: 5.2 / 8,
  shop: 5.6 / 8,
  custom: 4.8 / 8,
  handwriting: 5.2 / 8,
  transparent: 4.6 / 8,
  text: 5.2 / 8,
  memories: 4.8 / 8,
  pomodoro: 5.6 / 8,
  snap: 5.2 / 8,
};

/** k = screen px per PinPin logical px */
const Sprite: React.FC<{name: string; k: number; x?: number; y?: number; cx?: number; cy?: number; rot?: number; index?: number; style?: React.CSSProperties}> = ({name, k, x, y, cx, cy, rot = 0, index = 0, style}) => {
  const s = S[name];
  const file = s.files ? s.files[Math.min(index, s.files.length - 1)] : s.file!;
  const f = k / (s.scale ?? SPRITE_SCALE);
  const [pl, pt, pr, pb] = s.paper ?? [0, 0, s.w, s.h];
  const left = cx !== undefined ? cx - ((pl + pr) / 2) * f : (x ?? 0) - pl * f;
  const top = cy !== undefined ? cy! - ((pt + pb) / 2) * f : (y ?? 0) - pt * f;
  return <Img src={staticFile('sprites/' + file)} style={{position: 'absolute', left, top, width: s.w * f, height: s.h * f, transform: `rotate(${rot}deg)`, transformOrigin: `${((pl + pr) / 2) * f}px ${((pt + pb) / 2) * f}px`, ...style}} />;
};
const paperW = (name: string, k: number) => {
  const s = S[name];
  const [l, , r] = s.paper!;
  return ((r - l) * k) / (s.scale ?? SPRITE_SCALE);
};
const rowAt = (name: string, k: number, x: number, y: number, i: number) => {
  const s = S[name];
  const f = k / SPRITE_SCALE;
  const [l, t] = s.paper!;
  const [x0, x1, yy] = s.rows![i];
  return [x + (x0 - l) * f, x + (x1 - l) * f, y + (yy - t) * f];
};
const wobbly = (x0: number, y0: number, x1: number, seed = 0) => {
  let d = `M ${x0} ${y0}`;
  for (let i = 1; i <= 14; i++) {
    const u = i / 14;
    d += ` L ${(x0 + (x1 - x0) * u).toFixed(1)} ${(y0 + Math.sin(u * 9 + seed) * 2 + Math.sin(u * 23 + seed * 2) * 0.8).toFixed(1)}`;
  }
  return d;
};

const Day: React.FC<{children: React.ReactNode}> = ({children}) => (
  <AbsoluteFill>
    <Img src={staticFile('wall-light.png')} style={{position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'cover'}} />
    {children}
  </AbsoluteFill>
);
const Night: React.FC<{children: React.ReactNode}> = ({children}) => (
  <AbsoluteFill style={{background: 'radial-gradient(ellipse 120% 90% at 50% 40%, #233049 0%, #18223a 45%, #0f1626 100%)'}}>{children}</AbsoluteFill>
);

const CHALK = (
  <defs>
    <filter id="chalk" x="-10%" y="-20%" width="120%" height="140%">
      <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="7" result="n" />
      <feColorMatrix in="n" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 3 -1.1" result="mask" />
      <feComposite in="SourceGraphic" in2="mask" operator="in" result="grain" />
      <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="1" seed="2" result="w" />
      <feDisplacementMap in="grain" in2="w" scale="5" />
    </filter>
  </defs>
);

export const Screen: React.FC<{id: string}> = ({id}) => {
  const W = 1600;
  const H = Math.round(1600 * SCREENS[id]);
  switch (id) {
    case 'notes': {
      const k = 2.0;
      return (
        <Day>
          <Sprite name="note-todo" k={k} cx={330} cy={H / 2 + 20} rot={-2} />
          <Sprite name="note-rose" k={k} cx={800} cy={H / 2 - 10} rot={1.5} />
          <Sprite name="tape" k={k} x={800 - 170} y={H / 2 - 10 - 200 + 8} rot={1.5} />
          <Sprite name="st-cat" k={k} cx={800 + 150} cy={H / 2 + 120} rot={-8} />
          <Sprite name="note-sage" k={k} cx={1270} cy={H / 2 + 25} rot={-1} />
          <Sprite name="pin-gold" k={k} cx={1270 - 160} cy={H / 2 + 25 - 192} />
        </Day>
      );
    }
    case 'todo': {
      const k = 3.0;
      const x = W / 2 - paperW('note-todo', k) / 2;
      const y = 150;
      const rows = [0, 1].map((i) => rowAt('note-todo', k, x, y, i));
      return (
        <Day>
          <Sprite name="note-todo" k={k} x={x} y={y} />
          <svg width={W} height={H} style={{position: 'absolute', inset: 0}}>
            {rows.map((r, i) => (
              <path key={i} d={wobbly(r[0], r[2], r[1], i + 1)} stroke="#e2241e" strokeWidth={6} strokeLinecap="round" fill="none" />
            ))}
          </svg>
          <Sprite name="st-sunflower" k={k * 0.8} cx={x + paperW('note-todo', k) - 40} cy={y + 560} rot={10} />
        </Day>
      );
    }
    case 'shop': {
      const k = 2.6;
      const cx = W / 2 - 40;
      const cy = H / 2 + 30;
      return (
        <Day>
          <Sprite name="note-rose" k={k} cx={cx} cy={cy} />
          <Sprite name="tape" k={k} x={cx - 220} y={cy - 260 + 8} />
          <Sprite name="pin-gold" k={k} cx={cx + 250} cy={cy - 248} />
          <Sprite name="st-sunflower" k={k} cx={cx - 200} cy={cy + 170} rot={-10} />
          <Sprite name="st-cat" k={k} cx={cx + 170} cy={cy + 140} rot={14} />
          {/* selected: frame + rotate / scale handles */}
          <svg width={W} height={H} style={{position: 'absolute', inset: 0}}>
            <g transform={`rotate(14 ${cx + 170} ${cy + 140})`}>
              <rect x={cx + 170 - 98} y={cy + 140 - 87} width={196} height={174} fill="none" stroke="#6366f1" strokeWidth={2.5} />
              <circle cx={cx + 170 + 98} cy={cy + 140 - 87} r={11} fill="#6366f1" stroke="#fff" strokeWidth={3} />
              <circle cx={cx + 170 + 98} cy={cy + 140 + 87} r={11} fill="#fff" stroke="#6366f1" strokeWidth={4} />
            </g>
          </svg>
          {/* stickers straight on the desktop */}
          <Sprite name="st-boba" k={k} cx={170} cy={210} rot={-8} />
          <Sprite name="st-star" k={k} cx={1430} cy={230} rot={10} />
          <Sprite name="st-sakura" k={k} cx={1400} cy={H - 190} rot={-6} />
          <Sprite name="st-coffee" k={k} cx={190} cy={H - 210} rot={6} />
        </Day>
      );
    }
    case 'custom': {
      const pic = S['picture'];
      const ph = 640;
      const pw = (pic.w / pic.h) * ph;
      return (
        <Day>
          <div style={{position: 'absolute', left: 230 - pw / 2 + 150, top: H / 2 - ph / 2 - 20, width: pw + 36, height: ph + 36, borderRadius: 22, background: '#fff', boxShadow: '0 30px 60px -24px rgba(60,40,20,.45)'}}>
            <Img src={staticFile('sprites/picture.png')} style={{position: 'absolute', left: 18, top: 18, width: pw, height: ph, borderRadius: 10}} />
          </div>
          <svg width={W} height={H} style={{position: 'absolute', inset: 0}}>
            <path d={`M 720 ${H / 2} C 790 ${H / 2 - 50}, 860 ${H / 2 - 50}, 930 ${H / 2}`} stroke="#5b5250" strokeWidth={6} strokeDasharray="14 10" fill="none" strokeLinecap="round" />
            <path d={`M 912 ${H / 2 - 22} L 934 ${H / 2 + 2} L 906 ${H / 2 + 14}`} stroke="#5b5250" strokeWidth={6} fill="none" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          <Sprite name="picture-sticker" k={1} cx={1230} cy={H / 2} style={{height: 660, width: 'auto', left: 1230 - 240, top: H / 2 - 330, filter: 'drop-shadow(0 18px 22px rgba(60,40,20,.35))'}} />
        </Day>
      );
    }
    case 'handwriting': {
      const k = 2.15;
      const x = W / 2 - paperW('note-oat', k) / 2;
      const y = 70;
      return (
        <Day>
          <Sprite name="note-oat-ink" k={k} x={x} y={y} />
          <Sprite name="heart" k={k} x={x} y={y} index={24} />
          <Img src={staticFile('sprites/picture-sticker.png')} style={{position: 'absolute', left: x + 95 * k - 95, top: y + 140 * k - 110, height: 220, transform: 'rotate(-7deg)', filter: 'drop-shadow(0 6px 8px rgba(60,40,20,.35))'}} />
        </Day>
      );
    }
    case 'transparent': {
      const y = H / 2 - 40;
      return (
        <Night>
          <div style={{position: 'absolute', left: 0, right: 0, top: y - 110, textAlign: 'center', fontFamily: 'Impact, sans-serif', fontSize: 168, color: '#f7cf3b', letterSpacing: 2}}>STAY FOCUSED</div>
          <svg width={W} height={H} style={{position: 'absolute', inset: 0}}>
            {CHALK}
            <path d={`M 330 ${y + 105} C 600 ${y + 130}, 1000 ${y + 85}, 1270 ${y + 110}`} stroke="#f4f1ea" strokeWidth={16} strokeLinecap="round" fill="none" filter="url(#chalk)" />
          </svg>
          <Sprite name="st-sparkles" k={2.6} cx={250} cy={170} />
          <Sprite name="st-moon" k={2.6} cx={1400} cy={H - 150} rot={-10} />
          {/* the paper's edge shows only on hover */}
          <div style={{position: 'absolute', left: 260, top: y - 190, width: 1080, height: 380, border: '3px dashed rgba(255,255,255,.55)', borderRadius: 4}} />
        </Night>
      );
    }
    case 'text': {
      const k = 2.0;
      return (
        <Day>
          <Sprite name="note-call" k={k} cx={330} cy={H / 2 + 10} rot={-2} />
          <Sprite name="note-sale" k={k} cx={800} cy={H / 2 - 20} rot={2} />
          <Sprite name="note-read" k={k} cx={1270} cy={H / 2 + 20} rot={-1} />
        </Day>
      );
    }
    case 'memories': {
      return (
        <Day>
          <Sprite name="polaroid" k={2.0} cx={480} cy={H / 2 + 20} rot={-4} />
          <Sprite name="secret" k={2.2} cx={1110} cy={H / 2} index={20} rot={2} />
          <svg width={140} height={160} viewBox="-60 -80 120 140" style={{position: 'absolute', left: 1110 + 270, top: H / 2 - 240, filter: 'drop-shadow(0 8px 10px rgba(0,0,0,.35))'}}>
            <path d="M -20 -2 L -20 -26 A 20 20 0 0 1 20 -26 L 20 -2" fill="none" stroke="#c9ccd2" strokeWidth={9} strokeLinecap="round" transform="translate(0 -14) rotate(-24 -20 -2)" />
            <rect x={-32} y={-6} width={64} height={52} rx={10} fill="#e8b84a" />
            <rect x={-32} y={-6} width={64} height={16} rx={8} fill="#f3cf72" />
            <circle cx={0} cy={18} r={6} fill="#7a5a1a" />
          </svg>
        </Day>
      );
    }
    case 'pomodoro': {
      const k = 2.0;
      const cols = [W / 2 - 470, W / 2, W / 2 + 470];
      const rows = [H / 2 - 220, H / 2 + 230];
      const names = ['tomato', 'timer-hourglass', 'timer-led', 'timer-alarm', 'timer-cat', 'timer-dog'];
      return (
        <Day>
          {names.map((n, i) => (
            <Sprite key={n} name={n} k={k} cx={cols[i % 3]} cy={rows[Math.floor(i / 3)]} index={1} />
          ))}
        </Day>
      );
    }
    case 'snap': {
      const k = 2.0;
      const w = paperW('note-todo', k);
      const gap = 10 * k;
      const x0 = W / 2 - (3 * w + 2 * gap) / 2;
      const y = 150;
      return (
        <Day>
          <Sprite name="note-todo" k={k} x={x0} y={y} />
          <Sprite name="note-rose" k={k} x={x0 + w + gap} y={y} />
          <Sprite name="note-sage" k={k} x={x0 + 2 * (w + gap)} y={y} />
          <svg width={W} height={H} style={{position: 'absolute', inset: 0}}>
            <line x1={x0 - 60} y1={y} x2={x0 + 3 * w + 2 * gap + 60} y2={y} stroke="#6366f1" strokeWidth={2.5} strokeDasharray="10 8" />
            {[0, 1].map((i) => {
              const gx = x0 + w + i * (w + gap);
              return (
                <g key={i}>
                  <line x1={gx} y1={y + 430} x2={gx + gap} y2={y + 430} stroke="#6366f1" strokeWidth={3} />
                  <text x={gx + gap / 2} y={y + 470} textAnchor="middle" fontFamily={FONT} fontSize={30} fontWeight={700} fill="#6366f1">10 px</text>
                </g>
              );
            })}
          </svg>
        </Day>
      );
    }
    default:
      return <AbsoluteFill style={{background: '#fff'}} />;
  }
};
