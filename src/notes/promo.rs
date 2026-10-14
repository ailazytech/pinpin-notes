//! 宣傳影片和網站截圖（開發用）：用真正的便利貼繪圖程式在記憶體裡畫每一格，疊在桌布上、加字幕，交給 ffmpeg 編成 MP4。
//! 不動到桌面、不模擬滑鼠鍵盤。
//!
//! `--render-video out.mp4 淺色桌布.png 深色桌布.png`
//! `--render-stills 資料夾 淺色桌布.png 深色桌布.png`（每一段一張 PNG，給網站用）

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;

use super::assets;
use super::image::{self, Pixels};
use super::ink::{self, InkView};
use super::render::{self, AttachAnim, Canvas, Fonts, Fx, Rgb, Scene, Scramble};
use super::store::{self, Attach, InkPrefs, Kind, Line, Note, Paper, Stroke, Texture, Timer, TimerStyle, Tool};
use super::stickers;

const W: i32 = 1920;
const H: i32 = 1080;
const FPS: f32 = 30.0;
/// 換段時交叉淡入的秒數
const FADE: f32 = 0.5;
/// 便利貼的縮放（1080p 上看起來剛好）
const S: f32 = 1.75;

fn ease(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 一張便利貼這一格的樣子。
#[derive(Default)]
struct Look<'a> {
    hover: bool,
    /// 出現動畫的進度（None = 已經在了）
    appear: Option<f32>,
    attach: Option<(Option<usize>, f32)>,
    strike: Option<(usize, f32)>,
    scramble: Option<(&'a [Line], &'a [Line], f32)>,
    selected: Option<usize>,
    ink: Option<InkView<'a>>,
    fx: Fx,
    now_ms: u64,
    /// 額外的透明度（0 = 不透明）
    fade: f32,
}

struct Studio {
    assets: assets::Cache,
    fonts: HashMap<String, Fonts>,
    light: Pixels,
    dark: Pixels,
    logo: Pixels,
    mem: HDC,
    bits: *mut u8,
    title: HFONT,
    sub: HFONT,
    big: HFONT,
    /// 影片開始的「現在」（蕃茄鐘用）
    t0: u64,
    /// 自己做的貼紙（場景 4）
    shot: Pixels,
}

fn font(size: i32, weight: i32) -> HFONT {
    unsafe { CreateFontW(-size, 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0, w!("Segoe UI")) }
}

fn cover(p: &Pixels) -> Pixels {
    // 等比例放大到蓋滿 1920×1080，再裁中間
    let k = (W as f32 / p.w as f32).max(H as f32 / p.h as f32);
    let big = p.resized((p.w as f32 * k).ceil() as u32, (p.h as f32 * k).ceil() as u32);
    let (ox, oy) = ((big.w as i32 - W) / 2, (big.h as i32 - H) / 2);
    let mut out = Pixels { w: W as u32, h: H as u32, data: vec![0; (W * H * 4) as usize] };
    for y in 0..H {
        let s = (((y + oy) as u32 * big.w + ox as u32) * 4) as usize;
        out.data[(y * W * 4) as usize..((y + 1) * W * 4) as usize].copy_from_slice(&big.data[s..s + (W * 4) as usize]);
    }
    out
}

impl Studio {
    fn new(light: &Path, dark: &Path) -> anyhow::Result<Studio> {
        crate::i18n::set("en");
        let light = cover(&image::decode_file(light)?);
        let dark = cover(&image::decode_file(dark)?);
        let logo = image::decode_bytes(include_bytes!("../../assets/logo/pinpin-512.png"))?;
        // 「截圖」：柴犬貼在淡灰底上，再做成貼紙
        let src = super::assets::picture_2x("st-shiba")?;
        let mut shot = Pixels { w: src.w + 120, h: src.h + 90, data: vec![0; ((src.w + 120) * (src.h + 90) * 4) as usize] };
        for y in 0..shot.h {
            for x in 0..shot.w {
                let o = ((y * shot.w + x) * 4) as usize;
                let mut c = [232.0f32, 236.0, 238.0];
                if x >= 60 && y >= 45 && x < src.w + 60 && y < src.h + 45 {
                    let i = (((y - 45) * src.w + x - 60) * 4) as usize;
                    let a = src.data[i + 3] as f32 / 255.0;
                    for k in 0..3 {
                        c[k] = src.data[i + k] as f32 * a + c[k] * (1.0 - a);
                    }
                }
                shot.data[o..o + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        }
        let mut assets = assets::Cache::new(S);
        assets.insert("my-promo", stickers::make_sticker(&shot));
        unsafe {
            let screen = GetDC(None);
            let mem = CreateCompatibleDC(Some(screen));
            ReleaseDC(None, screen);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: W, biHeight: -H, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0, ..Default::default() },
                ..Default::default()
            };
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let dib = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
            SelectObject(mem, dib.into());
            SetBkMode(mem, TRANSPARENT);
            Ok(Studio {
                assets,
                fonts: HashMap::new(),
                light,
                dark,
                logo,
                mem,
                bits: bits as *mut u8,
                title: font(46, 600),
                sub: font(27, 400),
                big: font(110, 700),
                t0: store::now_ms(),
                shot,
            })
        }
    }

    fn buf(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.bits, (W * H * 4) as usize) }
    }

    fn canvas(&mut self) -> Canvas<'_> {
        Canvas { buf: self.buf(), w: W, h: H }
    }

    fn background(&mut self, dark: bool) {
        let src = if dark { self.dark.data.clone() } else { self.light.data.clone() };
        let buf = self.buf();
        for (d, s) in buf.chunks_mut(4).zip(src.chunks(4)) {
            d.copy_from_slice(&[s[0], s[1], s[2], 255]);
        }
    }

    /// 畫一張便利貼，紙張左上角放在 (x, y)。
    fn note(&mut self, n: &Note, x: i32, y: i32, look: Look) {
        let st_face = n.face.clone().unwrap_or_default();
        let bold = n.bold.unwrap_or(false);
        let key = format!("{}|{}|{}", (n.font * 20.0).round(), st_face, bold);
        self.fonts.entry(key.clone()).or_insert_with(|| Fonts::new(S, n.font, &st_face, bold));
        // 出現動畫（和真的程式一樣）
        let (mut alpha, mut dy, mut lift, mut attach) = (1.0f32, 0.0f32, 0.0f32, look.attach.map(|(w, t)| AttachAnim { which: w, t }));
        if let Some(t) = look.appear {
            if t <= 0.0 {
                return;
            }
            alpha = (t / 0.3).min(1.0);
            dy = -14.0 * (1.0 - ease(t / 0.45));
            lift = 1.0 - ease(t / 0.5);
            attach = Some(AttachAnim { which: None, t: ((t - 0.3) / 0.7).clamp(0.0, 1.0) });
        }
        alpha *= 1.0 - look.fade;
        if alpha <= 0.0 {
            return;
        }
        let ink_bar = look.ink.is_some();
        let lay = render::layout_ex(n, S, &mut self.assets, ink_bar);
        let mut out = Vec::new();
        {
            let scene = Scene {
                note: n,
                lines: &n.lines,
                scale: S,
                hover: look.hover,
                hot: None,
                lift,
                attach_anim: attach,
                dragging_item: None,
                selected: look.selected,
                strike: look.strike,
                scramble: look.scramble.map(|(from, to, t)| Scramble { from, to, t }),
                unlocked: false,
                scroll: 0,
                editing: false,
                alpha: 255,
                image: None,
                assets: &mut self.assets,
                fonts: &self.fonts[&key],
                now_ms: look.now_ms,
                fx: look.fx,
                ink: look.ink,
                text_color: n.color.map(|c| Rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)),
            };
            render::present(render::Target::Buffer(&mut out), scene);
        }
        let (ox, oy) = (x - lay.paper.left, y - lay.paper.top + (dy * S) as i32);
        self.composite(&out, lay.w, lay.h, ox, oy, alpha);
    }

    /// 預乘的 BGRA 疊到畫面上。
    fn composite(&mut self, src: &[u8], w: i32, h: i32, ox: i32, oy: i32, alpha: f32) {
        let buf = self.buf();
        for y in 0..h {
            let dy = oy + y;
            if !(0..H).contains(&dy) {
                continue;
            }
            for x in 0..w {
                let dx = ox + x;
                if !(0..W).contains(&dx) {
                    continue;
                }
                let s = ((y * w + x) * 4) as usize;
                let a = src[s + 3] as f32 / 255.0 * alpha;
                if a <= 0.004 {
                    continue;
                }
                let d = ((dy * W + dx) * 4) as usize;
                for k in 0..3 {
                    buf[d + k] = (src[s + k] as f32 * alpha + buf[d + k] as f32 * (1.0 - a)).round().min(255.0) as u8;
                }
            }
        }
    }

    /// 沒預乘的圖，以 (cx, cy) 為中心、縮放 k、旋轉、透明度。
    fn picture(&mut self, p: &Pixels, cx: f32, cy: f32, k: f32, angle: f32, alpha: f32, shadow: bool) {
        let mut c = self.canvas();
        if shadow {
            c.blit_xf(p, cx + 4.0, cy + 8.0, angle, k, 0.22 * alpha, true);
        }
        c.blit_xf(p, cx, cy, angle, k, alpha, false);
        // 畫面本身不透明，alpha 一律補滿
        for px in c.buf.chunks_mut(4) {
            px[3] = 255;
        }
    }

    unsafe fn text(&mut self, text: &str, rc: RECT, f: HFONT, color: Rgb, align: DRAW_TEXT_FORMAT) {
        SelectObject(self.mem, f.into());
        SetTextColor(self.mem, color.colorref());
        let mut t: Vec<u16> = text.encode_utf16().collect();
        let mut r = rc;
        DrawTextW(self.mem, &mut t, &mut r, align | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
    }

    /// 左下角的字幕卡。`t` = 這一段進行了幾秒（淡入、往上滑）。
    fn caption(&mut self, title: &str, sub: &str, t: f32) {
        let a = smoothstep((t - 0.25) / 0.45);
        if a <= 0.0 {
            return;
        }
        let rise = (1.0 - a) * 18.0;
        let (l, top) = (96.0, H as f32 - 214.0 + rise);
        let measure = |f: HFONT, s: &str| unsafe {
            SelectObject(self.mem, f.into());
            let w: Vec<u16> = s.encode_utf16().collect();
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(self.mem, &w, &mut size);
            size.cx as f32
        };
        let width = 56.0 + 40.0 + measure(self.title, title).max(measure(self.sub, sub));
        let rc = (l, top, l + width.min(1720.0), top + 128.0);
        {
            let mut c = self.canvas();
            c.shadow(RECT { left: rc.0 as i32, top: rc.1 as i32, right: rc.2 as i32, bottom: rc.3 as i32 }, 0.0, 8.0, 22.0, 0.25 * a);
            ink::round_rect(&mut c, rc, 26.0, Rgb(255, 253, 248), 0.93 * a);
            ink::round_rect(&mut c, (rc.0 + 28.0, rc.1 + 26.0, rc.0 + 36.0, rc.3 - 26.0), 4.0, Rgb(226, 66, 52), a);
            for px in c.buf.chunks_mut(4) {
                px[3] = 255;
            }
        }
        let fg = |v: u8| (255.0 + (v as f32 - 255.0) * a) as u8;
        unsafe {
            self.text(title, RECT { left: rc.0 as i32 + 56, top: rc.1 as i32 + 18, right: rc.2 as i32 - 20, bottom: rc.1 as i32 + 74 }, self.title, Rgb(fg(44), fg(38), fg(34)), DT_LEFT);
            self.text(sub, RECT { left: rc.0 as i32 + 56, top: rc.1 as i32 + 72, right: rc.2 as i32 - 20, bottom: rc.1 as i32 + 112 }, self.sub, Rgb(fg(110), fg(100), fg(92)), DT_LEFT);
        }
    }

    unsafe fn finish(&mut self) {
        let _ = GdiFlush();
    }
}

// ───────────────────────────── 素材 ─────────────────────────────

fn text_note(paper: Paper, texture: Texture, lines: &[&str], items: Vec<Attach>) -> Note {
    Note { paper, texture, w: 240, h: 200, lines: lines.iter().map(|l| Line::new(*l)).collect(), items, v: store::VERSION, ..Default::default() }
}

fn notes_abc() -> [Note; 3] {
    [
        text_note(Paper::Yellow, Texture::Sticky, &["Buy milk", "Reply to Amy", "Finish the report"], vec![Attach::at("pin-red", 120.0, 2.0)]),
        text_note(Paper::MorandiRose, Texture::Washi, &["Weekend", "Farmers market", "Call grandma"], vec![Attach::tape("tape-floral", 60.0, 2.0, 180.0, -4.0), Attach { asset: "st-cat".into(), x: 205.0, y: 168.0, angle: -8.0, ..Default::default() }]),
        text_note(Paper::Sage, Texture::Grid, &["Ideas", "Plant herbs", "Learn watercolor"], vec![Attach { asset: "clip-heart".into(), x: 200.0, y: 4.0, angle: 6.0, ..Default::default() }]),
    ]
}

const POS_ABC: [(i32, i32); 3] = [(230, 250), (750, 200), (1270, 270)];

/// 一筆（點是邏輯像素），只畫到 p（0～1）的地方。
fn stroke(tool: Tool, color: u32, size: f32, pts: &[(f32, f32)], p: f32) -> Option<Stroke> {
    let n = ((pts.len() as f32 * p.clamp(0.0, 1.0)).ceil() as usize).min(pts.len());
    (n >= 1).then(|| Stroke { tool, color, size, pts: pts[..n].iter().flat_map(|&(x, y)| [(x * 4.0) as i32, (y * 4.0) as i32]).collect() })
}

fn path(n: usize, f: impl Fn(f32) -> (f32, f32)) -> Vec<(f32, f32)> {
    (0..=n).map(|i| f(i as f32 / n as f32)).collect()
}

fn heart(cx: f32, cy: f32, k: f32) -> Vec<(f32, f32)> {
    path(70, |u| {
        let t = u * std::f32::consts::TAU;
        (cx + 16.0 * t.sin().powi(3) * k, cy - (13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos()) * k)
    })
}

fn star(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    // 外圈、內圈交錯的五角星
    let pts: Vec<(f32, f32)> = (0..=10)
        .map(|i| {
            let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            let rr = if i % 2 == 0 { r } else { r * 0.45 };
            (cx + a.cos() * rr, cy + a.sin() * rr)
        })
        .collect();
    // 每段之間補點，畫起來比較順
    pts.windows(2).flat_map(|w| (0..8).map(move |k| { let u = k as f32 / 8.0; (w[0].0 + (w[1].0 - w[0].0) * u, w[0].1 + (w[1].1 - w[0].1) * u) })).collect()
}

/// 草寫的「一串圈圈」，像在簽名。
fn cursive(x0: f32, y0: f32, len: f32) -> Vec<(f32, f32)> {
    path(160, |u| (x0 + len * u + 15.0 * (u * std::f32::consts::TAU * 6.0).sin(), y0 - 12.0 * (u * std::f32::consts::TAU * 6.0).cos() + 4.0 * (u * 9.0).sin()))
}

fn timer_note(style: TimerStyle, t: Timer) -> Note {
    let (w, h) = super::timer::size_of(style);
    Note { kind: Kind::Timer, w, h, timer: Some(Timer { style, ..t }), ..Default::default() }
}

// ───────────────────────────── 分段 ─────────────────────────────

/// (秒數, 字幕標題, 字幕說明)
const SCENES: [(f32, &str, &str); 11] = [
    (4.0, "", ""),
    (5.5, "Sticky notes that live on your desktop", "Pins, washi tape, Morandi colors & real paper textures"),
    (4.5, "Click to cross things off", "A quick red stroke — so satisfying"),
    (5.0, "Pins, clips, washi tape & stickers", "Drag them anywhere · rotate and resize with handles"),
    (7.0, "Turn any picture into a sticker", "Background removed and a rounded white border added — automatically"),
    (8.0, "Handwrite with pen, marker, chalk or crayon", "8 colors · 3 sizes · eraser · undo"),
    (5.0, "Transparent paper for desktop slogans", "Write big, write bold — it floats right on your wallpaper"),
    (4.5, "Your colors, your fonts, any size", "Set a default style, or style a single note"),
    (9.5, "Pomodoro timers in six styles", "Tomato · hourglass · LED · alarm clock · cat · shiba — with sound"),
    (5.0, "Lock notes with a password", "Locked notes show a harmless decoy list"),
    (5.0, "", ""),
];

impl Studio {
    unsafe fn scene(&mut self, idx: usize, t: f32) {
        self.scene_inner(idx, t, true);
    }

    unsafe fn scene_inner(&mut self, idx: usize, t: f32, captions: bool) {
        let (_, title, sub) = SCENES[idx];
        match idx {
            0 | 10 => {
                self.background(false);
                let a = smoothstep(t / 0.8);
                let k = 0.55 + 0.1 * ease(t / 1.2);
                let logo = self.logo.clone();
                self.picture(&logo, W as f32 / 2.0, 380.0, k, 0.0, a, true);
                let ta = smoothstep((t - 0.5) / 0.6);
                let c = |v: u8| (255.0 + (v as f32 - 255.0) * ta) as u8;
                if ta > 0.0 {
                    self.text("PinPin Notes", RECT { left: 0, top: 560, right: W, bottom: 690 }, self.big, Rgb(c(52), c(40), c(36)), DT_CENTER);
                }
                let sa = smoothstep((t - 1.0) / 0.6);
                let c2 = |v: u8| (255.0 + (v as f32 - 255.0) * sa) as u8;
                let line = if idx == 0 { "Cozy sticky notes for your Windows desktop" } else { "Free & open source · 6 languages · almost zero CPU when idle" };
                if sa > 0.0 {
                    self.text(line, RECT { left: 0, top: 690, right: W, bottom: 760 }, self.title, Rgb(c2(110), c2(96), c2(88)), DT_CENTER);
                }
            }
            1 | 2 | 3 => {
                self.background(false);
                let mut notes = notes_abc();
                if idx >= 2 {
                    // 劃掉第一張的兩行
                    let done = |at: f32| t > at + 0.5 || idx > 2;
                    notes[0].lines[0].done = done(0.6);
                    notes[0].lines[1].done = done(1.8);
                }
                if idx == 3 {
                    notes[0].lines[0].done = true;
                    notes[0].lines[1].done = true;
                    notes[1].items.push(Attach { asset: "st-sunflower".into(), x: 40.0, y: 170.0, angle: -10.0, ..Default::default() });
                    notes[2].items.push(Attach::at("pin-gold", 40.0, 6.0));
                    // 選取貓貓貼紙，轉一轉、放大一點
                    let r = smoothstep((t - 2.2) / 1.8);
                    notes[1].items[1].angle = -8.0 + 26.0 * r;
                    notes[1].items[1].scale = 1.0 + 0.3 * r;
                }
                for (i, n) in notes.iter().enumerate() {
                    let (x, y) = POS_ABC[i];
                    let mut look = Look::default();
                    if idx == 1 {
                        look.appear = Some((t - 0.3 - i as f32 * 0.7) / 0.64);
                    }
                    if idx == 2 && i == 0 {
                        look.hover = true;
                        for (line, at) in [(0usize, 0.6f32), (1, 1.8)] {
                            if t >= at && t < at + 0.5 {
                                look.strike = Some((line, ease((t - at) / 0.5)));
                            }
                        }
                    }
                    if idx == 3 {
                        if i == 1 && (0.5..1.2).contains(&t) {
                            look.attach = Some((Some(2), (t - 0.5) / 0.7));
                        }
                        if i == 2 && (1.3..2.0).contains(&t) {
                            look.attach = Some((Some(1), (t - 1.3) / 0.7));
                        }
                        if i == 1 && t > 2.0 {
                            look.selected = Some(1);
                        }
                    }
                    self.note(n, x, y, look);
                }
            }
            4 => {
                self.background(false);
                // 左邊：一張「截圖」；中間：變成貼紙；最後飛到右邊的便利貼上
                let shot = self.shot.clone();
                let (cx, cy) = (620.0, 470.0);
                let shot_a = 1.0 - smoothstep((t - 2.0) / 0.8);
                if shot_a > 0.0 {
                    let rc = (cx - shot.w as f32 * 0.9 - 26.0, cy - shot.h as f32 * 0.9 - 70.0, cx + shot.w as f32 * 0.9 + 26.0, cy + shot.h as f32 * 0.9 + 26.0);
                    {
                        let mut c = self.canvas();
                        c.shadow(RECT { left: rc.0 as i32, top: rc.1 as i32, right: rc.2 as i32, bottom: rc.3 as i32 }, 0.0, 10.0, 26.0, 0.25 * shot_a);
                        ink::round_rect(&mut c, rc, 18.0, Rgb(250, 250, 252), shot_a);
                        for (k, col) in [Rgb(255, 95, 87), Rgb(254, 188, 46), Rgb(40, 200, 64)].into_iter().enumerate() {
                            c.disc(rc.0 + 30.0 + k as f32 * 24.0, rc.1 + 26.0, 7.0, col, shot_a);
                        }
                        for px in c.buf.chunks_mut(4) {
                            px[3] = 255;
                        }
                    }
                    self.picture(&shot, cx, cy, 1.8, 0.0, shot_a, false);
                    // 掃描線：在做去背
                    if (1.0..2.0).contains(&t) {
                        let y = rc.1 + 50.0 + (rc.3 - rc.1 - 60.0) * (t - 1.0);
                        let mut c = self.canvas();
                        c.line(rc.0 + 10.0, y, rc.2 - 10.0, y, 5.0, Rgb(99, 102, 241), 0.7);
                    }
                }
                let target = (1250, 280);
                let mut note = text_note(Paper::Oat, Texture::Dots, &["My stickers", "made from screenshots"], vec![Attach::at("pin-teal", 120.0, 2.0)]);
                note.w = 260;
                note.h = 230;
                let land = smoothstep((t - 4.0) / 1.0);
                if t > 5.0 {
                    note.items.push(Attach { asset: "my-promo".into(), x: 150.0, y: 150.0, angle: 8.0, ..Default::default() });
                }
                let n_items = note.items.len();
                let attach = (5.0..5.7).contains(&t).then(|| (Some(n_items - 1), (t - 5.0) / 0.7));
                self.note(&note, target.0, target.1, Look { appear: Some((t - 0.2) / 0.64), attach, ..Default::default() });
                if (2.0..5.0).contains(&t) {
                    if let Some(img) = self.assets.get("my-promo").cloned() {
                        let pop = smoothstep((t - 2.0) / 0.6);
                        let bounce = 1.0 + 0.12 * ((t - 2.0) * 9.0).sin() * (1.0 - smoothstep((t - 2.0) / 1.2));
                        let (sx, sy) = (cx + (target.0 as f32 + 150.0 * S - cx) * land, cy + (target.1 as f32 + 150.0 * S - cy) * land);
                        let k = (2.2 - 1.2 * land) * bounce;
                        self.picture(&img, sx, sy, k, (8.0f32).to_radians() * land, pop, true);
                    }
                }
            }
            5 => {
                self.background(false);
                let mut note = text_note(Paper::Oat, Texture::Dots, &[], vec![Attach::tape("tape-kraft", 160.0, 0.0, 300.0, -4.0)]);
                note.w = 460;
                note.h = 300;
                let plan: [(Tool, u32, f32, Vec<(f32, f32)>, f32, f32); 5] = [
                    (Tool::Pen, 0x26221E, 2.6, cursive(40.0, 70.0, 300.0), 0.4, 2.2),
                    (Tool::Marker, 0xF7CF3B, 13.0, path(30, |u| (36.0 + 310.0 * u, 92.0 + (u * 3.0).sin())), 2.4, 3.1),
                    (Tool::Crayon, 0xE21E24, 7.0, heart(380.0, 110.0, 2.2), 3.3, 4.6),
                    (Tool::Chalk, 0x2F6FDB, 7.0, star(110.0, 210.0, 46.0), 4.8, 6.0),
                    (Tool::Pen, 0x3DAA5C, 4.2, path(40, |u| { let x = 220.0 + 180.0 * u; (x, 230.0 - if u < 0.3 { -u * 60.0 } else { (u - 0.3) * 150.0 - 18.0 }) }), 6.2, 6.9),
                ];
                let mut cur_prefs = InkPrefs { tool: Tool::Pen, color: 0x26221E, size: 1 };
                for (tool, color, size, pts, a, b) in &plan {
                    if t >= *a {
                        if let Some(st) = stroke(*tool, *color, *size, pts, (t - a) / (b - a)) {
                            note.strokes.push(st);
                        }
                        cur_prefs = InkPrefs { tool: *tool, color: *color, size: if *size > 8.0 { 2 } else { 1 } };
                    }
                }
                let ink = Some(InkView { prefs: cur_prefs, eraser: false, cur: None });
                self.note(&note, (W - (460.0 * S) as i32) / 2, 110, Look { ink, ..Default::default() });
            }
            6 => {
                self.background(true);
                let mut note = Note { paper: Paper::Clear, w: 640, h: 200, font: 3.8, face: Some("Impact".into()), color: Some(0xF7CF3B), lines: vec![Line::new("STAY FOCUSED")], ..Default::default() };
                let under = path(50, |u| (36.0 + 400.0 * u, 128.0 + 4.0 * (u * 7.0).sin()));
                if let Some(st) = stroke(Tool::Chalk, 0xFAFAF7, 9.0, &under, (t - 1.0) / 1.2) {
                    note.strokes.push(st);
                }
                if let Some(st) = stroke(Tool::Chalk, 0xEE78A8, 6.0, &heart(520.0, 80.0, 1.9), (t - 2.4) / 1.2) {
                    note.strokes.push(st);
                }
                self.note(&note, (W - (640.0 * S) as i32) / 2 + 60, 260, Look { hover: t < 0.9, appear: Some(t / 0.64), ..Default::default() });
            }
            7 => {
                self.background(false);
                let mut a = text_note(Paper::MorandiBlue, Texture::Lined, &["Call mom", "Gym 7pm"], vec![Attach::at("pin-pearl", 120.0, 2.0)]);
                a.face = Some("Segoe Print".into());
                a.color = Some(0x2F6FDB);
                a.font = 1.3;
                let mut b = text_note(Paper::Yellow, Texture::Sticky, &["SALE!"], vec![Attach::tape("tape-gingham", 70.0, 0.0, 170.0, -3.0)]);
                b.face = Some("Impact".into());
                b.color = Some(0xE21E24);
                b.font = 3.2;
                let mut c = text_note(Paper::MilkTea, Texture::Fiber, &["Read 20 pages", "Water the plants"], vec![Attach { asset: "clip-binder".into(), x: 46.0, y: -12.0, ..Default::default() }]);
                c.face = Some("Georgia".into());
                c.color = Some(0x8A5CD6);
                c.bold = Some(true);
                c.font = 1.15;
                for (i, n) in [a, b, c].iter().enumerate() {
                    let (x, y) = [(230, 260), (750, 220), (1270, 270)][i];
                    self.note(n, x, y, Look { appear: Some((t - 0.2 - i as f32 * 0.5) / 0.64), ..Default::default() });
                }
            }
            8 => {
                self.background(false);
                let now = self.t0 + (t * 1000.0) as u64;
                let base = Timer { done: 2, ..Default::default() };
                let running = |left_s: u64| Timer { end_ms: self.t0 + left_s * 1000, ..base.clone() };
                let row = [
                    (TimerStyle::Tomato, running(24 * 60 + 37)),
                    (TimerStyle::Hourglass, running(9 * 60 + 12)),
                    (TimerStyle::Led, running(18 * 60 + 5)),
                    (TimerStyle::Alarm, running(12 * 60 + 48)),
                    (TimerStyle::Cat, running(21 * 60 + 30)),
                    (TimerStyle::Dog, Timer { phase: store::Phase::Break, left_ms: 5 * 60_000, ..base.clone() }),
                ];
                // 依實際寬度平均排開
                let widths: Vec<f32> = row.iter().map(|(st, _)| super::timer::size_of(*st).0 as f32 * S).collect();
                let gap = (W as f32 - 160.0 - widths.iter().sum::<f32>()) / 5.0;
                let mut xs = Vec::new();
                let mut x = 80.0;
                for w in &widths {
                    xs.push(x as i32);
                    x += w + gap;
                }
                for (i, (style, tm)) in row.into_iter().enumerate() {
                    let n = timer_note(style, tm);
                    let (_, h) = super::timer::size_of(style);
                    let y = 460 - (h as f32 * S / 2.0) as i32;
                    let fx = if i == 0 && t > 6.5 && t < 8.1 { Fx::Ring((t - 6.5) / 1.6) } else { Fx::None };
                    self.note(&n, xs[i], y, Look { appear: Some((t - 0.15 - i as f32 * 0.25) / 0.64), now_ms: now, fx, ..Default::default() });
                }
            }
            9 => {
                self.background(false);
                let decoy = store::decoy(3);
                let real: Vec<Line> = ["Gift ideas for Sam", "Vinyl player", "Concert tickets"].iter().map(|l| Line::new(*l)).collect();
                let mut n = text_note(Paper::MorandiLilac, Texture::Plain, &[], vec![Attach::at("pin-heart", 120.0, 2.0)]);
                n.w = 300;
                n.h = 220;
                let p = (t - 1.2) / 1.6;
                n.lines = if p >= 1.0 { real.clone() } else { decoy.clone() };
                let scramble = (0.0..1.0).contains(&p).then_some((decoy.as_slice(), real.as_slice(), p));
                self.note(&n, (W - (300.0 * S) as i32) / 2, 220, Look { scramble, ..Default::default() });
            }
            _ => self.background(false),
        }
        if captions && !title.is_empty() {
            self.caption(title, sub, t);
        }
        self.finish();
    }

    /// 第 t 秒的畫面（換段時交叉淡入）。
    unsafe fn at(&mut self, t: f32) {
        let mut start = 0.0;
        for i in 0..SCENES.len() {
            let dur = SCENES[i].0;
            if t < start + dur || i == SCENES.len() - 1 {
                let local = t - start;
                if i > 0 && local < FADE {
                    self.scene(i - 1, SCENES[i - 1].0 + local);
                    let prev = self.buf().to_vec();
                    self.scene(i, local);
                    let k = smoothstep(local / FADE);
                    for (d, p) in self.buf().iter_mut().zip(prev) {
                        *d = (p as f32 + (*d as f32 - p as f32) * k).round() as u8;
                    }
                } else {
                    self.scene(i, local);
                }
                // 結尾淡出成白色
                let total: f32 = SCENES.iter().map(|s| s.0).sum();
                let out = smoothstep((t - (total - 0.8)) / 0.8);
                if out > 0.0 {
                    for d in self.buf().iter_mut() {
                        *d = (*d as f32 + (255.0 - *d as f32) * out) as u8;
                    }
                }
                return;
            }
            start += dur;
        }
    }
}

fn ffmpeg(args: &[&str]) -> std::io::Result<std::process::Child> {
    Command::new("ffmpeg").args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
}

/// 整支影片。
pub fn render_video(out: &str, light: &Path, dark: &Path) -> anyhow::Result<()> {
    let mut st = Studio::new(light, dark)?;
    let total: f32 = SCENES.iter().map(|s| s.0).sum();
    let frames = (total * FPS) as usize;
    let size = format!("{W}x{H}");
    let mut child = ffmpeg(&["-y", "-f", "rawvideo", "-pix_fmt", "bgra", "-s", &size, "-r", "30", "-i", "-", "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out])?;
    let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("ffmpeg stdin"))?;
    for f in 0..frames {
        unsafe { st.at(f as f32 / FPS) };
        stdin.write_all(st.buf())?;
        if f % 150 == 0 {
            crate::base::log_info(format!("render-video: {f}/{frames}"));
        }
    }
    drop(stdin);
    child.wait()?;
    Ok(())
}

/// 每一段挑一格存成 PNG（網站用）。
pub fn render_stills(dir: &str, light: &Path, dark: &Path) -> anyhow::Result<()> {
    let mut st = Studio::new(light, dark)?;
    std::fs::create_dir_all(dir)?;
    // (段, 秒數, 檔名)
    let picks: [(usize, f32, &str); 9] = [
        (1, 5.0, "notes"),
        (3, 4.9, "stickers"),
        (4, 2.9, "custom-sticker"),
        (5, 7.8, "handwriting"),
        (6, 4.8, "slogan"),
        (7, 4.4, "text-style"),
        (8, 4.5, "timers"),
        (9, 4.8, "password"),
        (0, 3.9, "hero"),
    ];
    let size = format!("{W}x{H}");
    for (idx, t, name) in picks {
        // 只要畫面、不要字幕
        unsafe { st.scene_plain(idx, t) };
        let path = format!("{dir}/{name}.png");
        let mut child = ffmpeg(&["-y", "-f", "rawvideo", "-pix_fmt", "bgra", "-s", &size, "-i", "-", "-frames:v", "1", &path])?;
        child.stdin.take().ok_or_else(|| anyhow::anyhow!("ffmpeg stdin"))?.write_all(st.buf())?;
        child.wait()?;
    }
    Ok(())
}

impl Studio {
    /// 沒有字幕的畫面。
    unsafe fn scene_plain(&mut self, idx: usize, t: f32) {
        self.scene_inner(idx, t, false);
    }
}
