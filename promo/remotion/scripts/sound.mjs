// Sound for the PinPin Notes promo — everything synthesised here (no samples, no licences to track).
// Sound effects are placed from the film's own events (src/timeline.json), music is a soft lo-fi bed that
// drops out during the film's quiet hold, darkens at night and ducks under every hit. One shared room (reverb).
//
//   node scripts/sound.mjs   →   public/audio/score.wav  (44.1 kHz, 16-bit stereo)
//   node scripts/sound.mjs <timeline.json> <out.wav>  — the feature clips use this (scripts/clip-sounds.mjs)

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const TL = JSON.parse(fs.readFileSync(process.argv[2] ?? path.join(here, '../src/timeline.json'), 'utf8'));
/** seconds the music takes to open up (low-pass sweep) */
const INTRO = TL.music.intro ?? 2.2;
const SR = 44100;
const N = Math.ceil(TL.duration * SR) + SR; // a second of tail
const TAU = Math.PI * 2;

// ── deterministic noise ──
let seed = 1234567;
const rnd = () => {
  seed = (seed * 1664525 + 1013904223) >>> 0;
  return seed / 4294967296;
};
const noise = () => rnd() * 2 - 1;

// ── biquad filter (RBJ) ──
class Biquad {
  constructor(type, freq, q = 0.707) {
    this.type = type;
    this.x1 = this.x2 = this.y1 = this.y2 = 0;
    this.set(freq, q);
  }
  set(freq, q = this.q) {
    this.q = q;
    const w = (TAU * Math.min(freq, SR * 0.45)) / SR;
    const a = Math.sin(w) / (2 * q);
    const c = Math.cos(w);
    let b0, b1, b2;
    if (this.type === 'lp') [b0, b1, b2] = [(1 - c) / 2, 1 - c, (1 - c) / 2];
    else if (this.type === 'hp') [b0, b1, b2] = [(1 + c) / 2, -(1 + c), (1 + c) / 2];
    else [b0, b1, b2] = [a, 0, -a]; // band-pass (0 dB peak)
    const a0 = 1 + a;
    this.b0 = b0 / a0;
    this.b1 = b1 / a0;
    this.b2 = b2 / a0;
    this.a1 = (-2 * c) / a0;
    this.a2 = (1 - a) / a0;
  }
  run(x) {
    const y = this.b0 * x + this.b1 * this.x1 + this.b2 * this.x2 - this.a1 * this.y1 - this.a2 * this.y2;
    this.x2 = this.x1;
    this.x1 = x;
    this.y2 = this.y1;
    this.y1 = y;
    return y;
  }
}

const env = (t, attack, decay) => (t < 0 ? 0 : (1 - Math.exp(-t / attack)) * Math.exp(-t / decay));

// ── sound effects (mono buffers) ──
const SFX = {
  pen(dur) {
    const n = Math.floor((dur + 0.08) * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 3400, 0.9);
    const body = new Biquad('lp', 900);
    let grain = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      if (i % Math.floor(SR / 38) === 0) grain = 0.55 + 0.45 * rnd();
      const shape = Math.min(1, t / 0.012) * (t > dur ? Math.exp(-(t - dur) / 0.03) : 1);
      const x = noise();
      out[i] = (bp.run(x) * 0.9 + body.run(x) * 0.15) * grain * shape * 0.55;
    }
    return out;
  },
  crayon(dur) {
    const n = Math.floor((dur + 0.1) * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 1700, 1.1);
    let grain = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      if (i % Math.floor(SR / 52) === 0) grain = 0.3 + 0.7 * rnd();
      const shape = Math.min(1, t / 0.03) * (t > dur ? Math.exp(-(t - dur) / 0.05) : 1);
      out[i] = bp.run(noise()) * grain * shape * 0.5;
    }
    return out;
  },
  whoosh(dur) {
    const n = Math.floor(dur * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 500, 1.4);
    for (let i = 0; i < n; i++) {
      const u = i / n;
      if (i % 64 === 0) bp.set(400 + 2200 * Math.sin(Math.PI * u) ** 1.4, 1.4);
      out[i] = bp.run(noise()) * Math.sin(Math.PI * u) ** 1.5 * 0.42;
    }
    return out;
  },
  paper() {
    const n = Math.floor(0.22 * SR);
    const out = new Float32Array(n);
    const lp = new Biquad('lp', 1300);
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      out[i] = lp.run(noise()) * env(t, 0.006, 0.05) * 0.5 + Math.sin(TAU * 120 * t) * env(t, 0.003, 0.05) * 0.35;
    }
    return out;
  },
  tape() {
    const n = Math.floor(0.35 * SR);
    const out = new Float32Array(n);
    const hp = new Biquad('hp', 2200);
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      const thump = Math.sin(TAU * (95 + 40 * Math.exp(-t / 0.02)) * t) * env(t, 0.002, 0.11);
      const slap = hp.run(noise()) * env(t, 0.001, 0.045);
      const crinkle = rnd() < 0.004 * Math.exp(-t / 0.08) * 40 ? noise() * 0.6 : 0;
      out[i] = thump * 0.8 + slap * 0.55 + crinkle * 0.3;
    }
    return out;
  },
  drop() {
    const n = Math.floor(0.25 * SR);
    const out = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      out[i] = (Math.sin(TAU * 540 * t) * env(t, 0.001, 0.045) + Math.sin(TAU * 1130 * t) * env(t, 0.001, 0.025) * 0.5 + noise() * env(t, 0.0005, 0.004) * 0.6) * 0.6;
    }
    return out;
  },
  cut(dur) {
    const n = Math.floor((dur + 0.05) * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 1500, 2);
    for (let i = 0; i < n; i++) {
      const u = i / n;
      if (i % 64 === 0) bp.set(1500 + 3800 * u, 2);
      const snip = 0.5 + 0.5 * Math.sin(TAU * 55 * (i / SR));
      out[i] = bp.run(noise()) * snip * Math.min(1, u * 12) * (1 - u) ** 0.4 * 0.5;
    }
    return out;
  },
  snip(dur) {
    // scissors: a metallic snip every ~0.11 s
    const n = Math.floor((dur + 0.1) * SR);
    const out = new Float32Array(n);
    for (let s = 0; s < dur; s += 0.11 + rnd() * 0.02) {
      const s0 = Math.floor(s * SR);
      const bp = new Biquad('bp', 4200, 3);
      for (let i = 0; i < 0.03 * SR && s0 + i < n; i++) {
        const t = i / SR;
        out[s0 + i] += (bp.run(noise()) * env(t, 0.0005, 0.006) * 1.2 + Math.sin(TAU * 3100 * t) * env(t, 0.0005, 0.012) * 0.25) * 0.6;
      }
    }
    return out;
  },
  chalk(dur) {
    const n = Math.floor((dur + 0.08) * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 950, 0.7);
    const hp = new Biquad('hp', 3000);
    let grain = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      if (i % Math.floor(SR / 90) === 0) grain = rnd() < 0.8 ? 0.4 + 0.6 * rnd() : 0.05;
      const shape = Math.min(1, t / 0.02) * (t > dur ? Math.exp(-(t - dur) / 0.04) : 1);
      const x = noise();
      out[i] = (bp.run(x) * 0.7 + hp.run(x) * 0.25) * grain * shape * 0.55;
    }
    return out;
  },
  cascade(dur) {
    // the rest of the board landing: a soft paper/sticker thump every ~0.09 s, a little random
    const n = Math.floor((dur + 0.3) * SR);
    const out = new Float32Array(n);
    const lp = new Biquad('lp', 1400);
    for (let s = 0; s < dur; s += 0.075 + rnd() * 0.03) {
      const s0 = Math.floor(s * SR);
      const f = 420 + rnd() * 300;
      for (let i = 0; i < 0.12 * SR && s0 + i < n; i++) {
        const t = i / SR;
        out[s0 + i] += (Math.sin(TAU * f * t) * env(t, 0.001, 0.035) * 0.35 + lp.run(noise()) * env(t, 0.002, 0.03) * 0.3) * (0.6 + 0.4 * rnd());
      }
    }
    return out;
  },
  swell(dur) {
    // dusk opening from the tomato: a soft low swell with a warm tone under it
    const n = Math.floor((dur + 0.4) * SR);
    const out = new Float32Array(n);
    const lp = new Biquad('lp', 600, 0.8);
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      const u = Math.min(1, t / dur);
      if (i % 64 === 0) lp.set(300 + 1200 * u, 0.8);
      const shape = Math.sin(Math.PI * Math.min(1, t / (dur + 0.4))) ** 1.2;
      out[i] = (lp.run(noise()) * 0.6 + Math.sin(TAU * 220 * t) * 0.12 + Math.sin(TAU * 329.6 * t) * 0.08) * shape * 0.5;
    }
    return out;
  },
  stamp(dur) {
    // a rubber stamp per letter
    const n = Math.floor((dur + 0.2) * SR);
    const out = new Float32Array(n);
    const lp = new Biquad('lp', 900);
    for (let k = 0; k * 0.045 < dur - 0.05; k++) {
      if (k === 4) continue; // the space
      const s0 = Math.floor((k * 0.045 + 0.09) * SR);
      for (let i = 0; i < 0.09 * SR && s0 + i < n; i++) {
        const t = i / SR;
        out[s0 + i] += (Math.sin(TAU * (120 + 60 * Math.exp(-t / 0.01)) * t) * env(t, 0.001, 0.035) * 0.7 + lp.run(noise()) * env(t, 0.0008, 0.012) * 0.5) * 0.7;
      }
    }
    return out;
  },
  click() {
    const n = Math.floor(0.12 * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 3200, 4);
    for (const d of [0, 0.045]) {
      const s0 = Math.floor(d * SR);
      for (let i = 0; i < 0.03 * SR; i++) out[s0 + i] += bp.run(noise()) * env(i / SR, 0.0003, 0.005) * 1.6;
    }
    return out;
  },
  // feature clips: a bubbly "bloop" when something lands (pitch springs up), with a tiny glint on top
  plop() {
    const n = Math.floor(0.3 * SR);
    const out = new Float32Array(n);
    let ph = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      ph += (TAU * (240 + 620 * (1 - Math.exp(-t / 0.028)))) / SR;
      out[i] = (Math.sin(ph) + 0.25 * Math.sin(2 * ph)) * env(t, 0.002, 0.075) * 0.75 + Math.sin(TAU * 2637 * t) * env(t - 0.03, 0.001, 0.05) * (t > 0.03 ? 0.12 : 0);
    }
    return out;
  },
  // three quick music-box notes going up (C7 E7 G7)
  sparkle() {
    const n = Math.floor(0.6 * SR);
    const out = new Float32Array(n);
    [2093, 2637, 3136].forEach((f, k) => {
      const s0 = Math.floor(k * 0.055 * SR);
      for (let i = 0; s0 + i < n; i++) {
        const t = i / SR;
        out[s0 + i] += (Math.sin(TAU * f * t) + 0.2 * Math.sin(TAU * 3 * f * t)) * env(t, 0.001, 0.14) * 0.22;
      }
    });
    return out;
  },
  pop() {
    const n = Math.floor(0.2 * SR);
    const out = new Float32Array(n);
    let ph = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      ph += (TAU * (360 * Math.pow(3.2, Math.min(1, t / 0.07)))) / SR;
      out[i] = Math.sin(ph) * env(t, 0.002, 0.06) * 0.7;
    }
    return out;
  },
  ding() {
    const n = Math.floor(1.6 * SR);
    const out = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      out[i] = (Math.sin(TAU * 1318.5 * t) + 0.3 * Math.sin(TAU * 2637 * t) * Math.exp(-t / 0.2)) * env(t, 0.002, 0.5) * 0.4;
    }
    return out;
  },
  tick() {
    const n = Math.floor(0.05 * SR);
    const out = new Float32Array(n);
    const bp = new Biquad('bp', 2300, 3);
    for (let i = 0; i < n; i++) out[i] = bp.run(noise()) * env(i / SR, 0.0005, 0.008) * 1.4;
    return out;
  },
  hit() {
    const n = Math.floor(0.3 * SR);
    const out = new Float32Array(n);
    let ph = 0;
    for (let i = 0; i < n; i++) {
      const t = i / SR;
      ph += (TAU * (52 + 110 * Math.exp(-t / 0.03))) / SR;
      out[i] = Math.sin(ph) * env(t, 0.001, 0.12) * 0.8 + noise() * env(t, 0.0005, 0.006) * 0.35;
    }
    return out;
  },
  bell() {
    const n = Math.floor(1.4 * SR);
    const out = new Float32Array(n);
    for (let k = 0; k < 7; k++) {
      const s0 = Math.floor(k * 0.085 * SR);
      for (let i = 0; i < n - s0; i++) {
        const t = i / SR;
        out[s0 + i] += (Math.sin(TAU * 1760 * t) + 0.4 * Math.sin(TAU * 2637 * t)) * env(t, 0.001, 0.12) * 0.22;
      }
    }
    return out;
  },
  glitch(dur) {
    const n = Math.floor((dur + 0.05) * SR);
    const out = new Float32Array(n);
    for (let k = 0; k < 11; k++) {
      const s0 = Math.floor(rnd() * (n - 0.04 * SR));
      const f = 600 + rnd() * 1600;
      for (let i = 0; i < 0.026 * SR; i++) out[s0 + i] += Math.sign(Math.sin((TAU * f * i) / SR)) * 0.12 * (1 - i / (0.026 * SR));
    }
    return out;
  },
  chime() {
    const n = Math.floor(2.8 * SR);
    const out = new Float32Array(n);
    for (const [f, d, g] of [[1046.5, 0, 0.35], [1568, 0.12, 0.3], [2093, 0.24, 0.16]]) {
      const s0 = Math.floor(d * SR);
      for (let i = 0; i < n - s0; i++) {
        const t = i / SR;
        out[s0 + i] += (Math.sin(TAU * f * t) + 0.25 * Math.sin(TAU * f * 2.01 * t) * Math.exp(-t / 0.3)) * env(t, 0.003, 0.9) * g;
      }
    }
    return out;
  },
};

// ── music ──
const music = new Float32Array(N);
const beat = 60 / TL.music.bpm;
const bar = beat * 4;
const midi = (m) => 440 * Math.pow(2, (m - 69) / 12);
// Fmaj7 · Em7 · Dm7 · Cmaj7
const CHORDS = [
  [53, 57, 60, 64],
  [52, 55, 59, 62],
  [50, 53, 57, 60],
  [48, 52, 55, 59],
];
const BASS = [41, 40, 38, 36];
const inQuiet = (t) => TL.music.quiet.some(([a, b]) => t > a && t < b);
const end = TL.music.end ?? TL.T.endUnderline[1];

const addTone = (buf, t0, f, dur, gain, bright = 1.0) => {
  const s0 = Math.floor(t0 * SR);
  const n = Math.floor(dur * SR);
  for (let i = 0; i < n && s0 + i < buf.length; i++) {
    const t = i / SR;
    // electric piano: a little FM bell on top of a sine
    const fm = bright * 1.1 * Math.exp(-t / 0.25) * Math.sin(TAU * f * t);
    buf[s0 + i] += Math.sin(TAU * f * t + fm) * env(t, 0.004, dur * 0.55) * gain;
  }
};

for (let b = 0; b * bar < TL.duration; b++) {
  const t0 = b * bar;
  const ch = CHORDS[b % 4];
  const finale = t0 >= end - 0.5;
  const chord = finale ? CHORDS[0] : ch;
  // chord stabs: 1 · and-of-2 · 4  (soft, a bit late like a human)
  for (const [off, notes, g] of [[0, chord, 0.075], [beat * 1.5 + 0.02, chord.slice(2), 0.055], [beat * 3, chord, 0.05]]) {
    const t = t0 + off;
    if (inQuiet(t) && off > 0) continue;
    notes.forEach((m, k) => addTone(music, t + k * 0.012, midi(m), finale ? 3.2 : 1.6, g * (inQuiet(t) ? 0.7 : 1)));
    if (finale) break;
  }
  // bass
  if (!inQuiet(t0)) {
    addTone(music, t0, midi(BASS[b % 4]), beat * 1.8, 0.16, 0.2);
    if (!finale) addTone(music, t0 + beat * 2, midi(BASS[b % 4] + 7), beat * 1.6, 0.11, 0.2);
  }
  // a few music-box notes on top
  const melody = [72, 76, 79, 81, 79, 76, 74, 72];
  for (let k = 0; k < 4; k++) {
    const t = t0 + k * beat + beat * 0.5;
    if (rnd() < 0.45 && !finale && t > 2.0) addTone(music, t, midi(melody[(b * 3 + k) % melody.length] + 12), 0.9, 0.03, 0.6);
  }
}
// drums (from bar 1, off in the quiet hold and in the finale)
const drums = new Float32Array(N);
const hatHp = new Biquad('hp', 6500);
for (let t = bar * 0.75; t < end - 0.3; t += beat / 2) {
  const step = Math.round(t / (beat / 2)) % 8;
  if (inQuiet(t)) continue;
  const night = t > TL.music.night[0] && t < TL.music.night[1];
  const s0 = Math.floor(t * SR);
  const swing = step % 2 === 1 ? Math.floor(0.045 * SR) : 0;
  if (step === 0 || step === 4 || (step === 3 && rnd() < 0.5)) {
    let ph = 0;
    for (let i = 0; i < 0.35 * SR; i++) {
      const x = i / SR;
      ph += (TAU * (46 + 90 * Math.exp(-x / 0.025))) / SR;
      drums[s0 + i] += Math.sin(ph) * env(x, 0.001, 0.13) * 0.32;
    }
  }
  if (step === 2 || step === 6) {
    const bp = new Biquad('bp', 1900, 1.2);
    for (let i = 0; i < 0.08 * SR; i++) drums[s0 + i] += bp.run(noise()) * env(i / SR, 0.0008, 0.025) * 0.28;
  }
  if (!night) {
    for (let i = 0; i < 0.03 * SR; i++) drums[s0 + swing + i] += hatHp.run(noise()) * env(i / SR, 0.0005, 0.008) * (step % 2 ? 0.05 : 0.08);
  }
}

// feature clips (music.style "cute"): a bouncy marimba tune instead — I · V · vi · IV in C, arpeggio melody,
// plucked bass, kick on 1 and 3, finger snaps on 2 and 4, a light shaker
if (TL.music.style === 'cute') {
  music.fill(0);
  drums.fill(0);
  const mallet = (buf, t0, f, decay, gain) => {
    const s0 = Math.floor(t0 * SR);
    for (let i = 0; i < decay * 6 * SR && s0 + i < buf.length; i++) {
      const t = i / SR;
      buf[s0 + i] += (Math.sin(TAU * f * t) * env(t, 0.002, decay) + 0.35 * Math.sin(TAU * 3.93 * f * t) * env(t, 0.001, 0.03)) * gain;
    }
  };
  const ROOT = [48, 43, 45, 41];
  const TUNE = [
    [72, 76, 79, 76, 84, null, 79, 76],
    [71, 74, 79, 74, 83, null, 79, 74],
    [72, 76, 81, 76, 84, null, 81, 79],
    [72, 77, 81, 77, 84, 81, 79, 77],
  ];
  const e8 = beat / 2;
  for (let b = 0; b * bar < TL.duration; b++) {
    const t0 = b * bar;
    if (t0 >= end - 0.05) {
      // last chord rings out
      [60, 64, 67, 72].forEach((m, k) => mallet(music, t0 + k * 0.03, midi(m), 0.9, 0.09));
      mallet(music, t0, midi(36), 1.0, 0.18);
      break;
    }
    TUNE[b % 4].forEach((m, k) => m && mallet(music, t0 + k * e8, midi(m), 0.32, k % 2 ? 0.07 : 0.1));
    mallet(music, t0, midi(ROOT[b % 4]), 0.45, 0.2);
    mallet(music, t0 + beat * 2, midi(ROOT[b % 4] + 7), 0.4, 0.15);
    for (let k = 0; k < 8; k++) {
      const t = t0 + k * e8;
      if (t >= end - 0.3) break;
      const s0 = Math.floor(t * SR);
      if (k === 0 || k === 4) {
        let ph = 0;
        for (let i = 0; i < 0.25 * SR; i++) {
          const x = i / SR;
          ph += (TAU * (55 + 80 * Math.exp(-x / 0.02))) / SR;
          drums[s0 + i] += Math.sin(ph) * env(x, 0.001, 0.09) * 0.28;
        }
      }
      if (k === 2 || k === 6) {
        const bp = new Biquad('bp', 2600, 1.6);
        for (let i = 0; i < 0.06 * SR; i++) drums[s0 + i] += bp.run(noise()) * env(i / SR, 0.0006, 0.018) * 0.35;
      }
      for (let i = 0; i < 0.04 * SR; i++) drums[s0 + i] += hatHp.run(noise()) * env(i / SR, 0.004, 0.012) * (k % 2 ? 0.07 : 0.04);
    }
  }
}

// ── mix ──
const L = new Float32Array(N);
const R = new Float32Array(N);
const sendL = new Float32Array(N);
const sendR = new Float32Array(N);

// music bus: fade in, darker at night (low-pass), ducked under hits
const lp = new Biquad('lp', 9000, 0.6);
const hits = TL.events.filter((e) => ['tape', 'drop', 'hit', 'bell', 'ding', 'pop', 'plop'].includes(e.kind)).map((e) => e.t);
for (let i = 0; i < N; i++) {
  const t = i / SR;
  if (i % 128 === 0) {
    const night = t > TL.music.night[0] && t < TL.music.night[1];
    lp.set(night ? 1400 : t < INTRO ? 700 + 3000 * (t / INTRO) : 9000, 0.6);
  }
  let duck = 1;
  for (const h of hits) if (t >= h && t < h + 0.5) duck = Math.min(duck, 1 - 0.45 * Math.exp(-(t - h) / 0.14));
  const fadeIn = Math.min(1, t / 0.6);
  const fadeOut = t > TL.duration - 0.6 ? Math.max(0, (TL.duration + 0.4 - t) / 1.0) : 1;
  const m = lp.run(music[i] + drums[i]) * duck * fadeIn * fadeOut;
  L[i] += m * 1.9;
  R[i] += m * 1.9;
  sendL[i] += m * 0.3;
  sendR[i] += m * 0.3;
}

// sound effects from the film's events, panned a little by type
const PAN = {swell: 0, stamp: 0, click: 0.15, snip: -0.05, chalk: -0.1, cascade: 0, pen: -0.1, crayon: -0.15, whoosh: 0.2, paper: 0, tape: 0.1, drop: 0.15, cut: -0.1, pop: 0.05, ding: 0, tick: 0.25, hit: 0, bell: 0, glitch: 0.1, chime: 0};
const GAIN = {swell: 0.8, stamp: 0.9, click: 0.8, snip: 0.9, chalk: 0.85, cascade: 0.8, pen: 0.9, crayon: 0.85, whoosh: 0.55, paper: 0.7, tape: 1.0, drop: 0.85, cut: 0.7, pop: 0.9, ding: 0.8, tick: 0.6, hit: 0.9, bell: 0.75, glitch: 0.6, chime: 0.9};
for (const e of TL.events) {
  const buf = SFX[e.kind](e.dur ?? 0.3);
  const s0 = Math.floor(e.t * SR);
  const p = PAN[e.kind] ?? 0;
  const g = GAIN[e.kind] ?? 0.8;
  for (let i = 0; i < buf.length && s0 + i < N; i++) {
    const x = buf[i] * g;
    L[s0 + i] += x * (1 - p) * 0.8;
    R[s0 + i] += x * (1 + p) * 0.8;
    sendL[s0 + i] += x * 0.22;
    sendR[s0 + i] += x * 0.22;
  }
}

// one room: Schroeder reverb (four combs, two all-passes), slightly different per side
const reverb = (input, offset) => {
  const out = new Float32Array(N);
  const combs = [29.7, 37.1, 41.1, 43.7].map((ms) => ({buf: new Float32Array(Math.floor(((ms + offset) * SR) / 1000)), i: 0, fb: 0.8, damp: 0.25, last: 0}));
  for (let n = 0; n < N; n++) {
    let acc = 0;
    for (const c of combs) {
      const y = c.buf[c.i];
      c.last = y * (1 - c.damp) + c.last * c.damp;
      c.buf[c.i] = input[n] + c.last * c.fb;
      c.i = (c.i + 1) % c.buf.length;
      acc += y;
    }
    out[n] = acc * 0.25;
  }
  for (const ms of [5.0, 1.7]) {
    const len = Math.floor(((ms + offset * 0.1) * SR) / 1000);
    const d = new Float32Array(len);
    let i = 0;
    for (let n = 0; n < N; n++) {
      const y = d[i];
      const x = out[n] + y * 0.7;
      d[i] = x;
      out[n] = y - x * 0.7;
      i = (i + 1) % len;
    }
  }
  return out;
};
const wetL = reverb(sendL, 0);
const wetR = reverb(sendR, 1.3);
for (let i = 0; i < N; i++) {
  L[i] += wetL[i];
  R[i] += wetR[i];
}

// master: gentle saturation, peak at -3 dBFS
let peak = 0;
for (let i = 0; i < N; i++) {
  L[i] = Math.tanh(L[i] * 1.1);
  R[i] = Math.tanh(R[i] * 1.1);
  peak = Math.max(peak, Math.abs(L[i]), Math.abs(R[i]));
}
const k = 0.708 / Math.max(peak, 1e-6);
const data = Buffer.alloc(N * 4);
for (let i = 0; i < N; i++) {
  data.writeInt16LE(Math.round(Math.max(-1, Math.min(1, L[i] * k)) * 32767), i * 4);
  data.writeInt16LE(Math.round(Math.max(-1, Math.min(1, R[i] * k)) * 32767), i * 4 + 2);
}
const header = Buffer.alloc(44);
header.write('RIFF', 0);
header.writeUInt32LE(36 + data.length, 4);
header.write('WAVEfmt ', 8);
header.writeUInt32LE(16, 16);
header.writeUInt16LE(1, 20);
header.writeUInt16LE(2, 22);
header.writeUInt32LE(SR, 24);
header.writeUInt32LE(SR * 4, 28);
header.writeUInt16LE(4, 32);
header.writeUInt16LE(16, 34);
header.write('data', 36);
header.writeUInt32LE(data.length, 40);
const outPath = process.argv[3] ?? path.join(here, '../public/audio/score.wav');
fs.mkdirSync(path.dirname(outPath), {recursive: true});
fs.writeFileSync(outPath, Buffer.concat([header, data]));
console.log(`${path.basename(outPath)}: ${(N / SR).toFixed(1)} s, peak scaled by ${k.toFixed(2)}`);
