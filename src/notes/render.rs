//! 便利貼的繪圖：整張（陰影、紙、字、附件）畫在一張 32 位元點陣圖上，
//! 用 UpdateLayeredWindow 交給系統合成，所以圖釘、紙膠帶、貼紙可以超出紙張、陰影是真的半透明。
//!
//! 只在內容或狀態改變時畫一次，平常不佔 CPU；畫完點陣圖就釋放，系統只留合成好的結果。
//!
//! 順序：陰影 → 紙張和紙質（逐點）→ 文字和劃線（GDI）→ 把紙張範圍的 alpha 補回 255
//! （GDI 畫字會把 alpha 清成 0）→ 附件（逐點合成，可旋轉；紙膠帶沿任意方向拉長）→ 滑過時的按鈕。

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use super::assets::{self, Cat};
use super::image::Pixels;
use super::ink::{self, InkView};
use super::store::{Attach, Kind, Line, Note, Paper, Texture};

// ───────────────────────────── 版面常數（邏輯像素） ─────────────────────────────

/// 紙張外圍留給陰影的空間
const SHADOW_L: f32 = 18.0;
const SHADOW_R: f32 = 22.0;
const SHADOW_T: f32 = 12.0;
const SHADOW_B: f32 = 28.0;
/// 附件外圍多留一點（陰影、放大動畫）；上方多留給「落下」動畫
const ATTACH_PAD: f32 = 10.0;
const DROP_SPACE: f32 = 26.0;
const PAD_L: f32 = 14.0;
/// 左邊放圓圈（點了劃掉）的寬度
const GUTTER: f32 = 20.0;
const PAD_R: f32 = 14.0;
const PAD_T: f32 = 26.0;
const PAD_B: f32 = 14.0;
/// 一列字的高度（字級 1.0 時）
pub const ROW_H: f32 = 23.0;
const META_H: f32 = 17.0;
pub const MIN_W: i32 = 140;
pub const MIN_H: i32 = 100;
const POLAROID_SIDE: f32 = 10.0;
const POLAROID_BOTTOM: f32 = 38.0;
/// 紙膠帶兩端「撕開」的部分佔素材寬度的比例
const TAPE_CAP: f32 = 0.12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Btn {
    New,
    Menu,
    Delete,
    Lock,
    Smaller,
    Bigger,
    /// 手寫
    Ink,
    /// 蕃茄鐘重來
    Reset,
}

#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    fn mix(self, o: Rgb, t: f32) -> Rgb {
        let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Rgb(m(self.0, o.0), m(self.1, o.1), m(self.2, o.2))
    }
    fn shade(self, k: f32) -> Rgb {
        let m = |a: u8| (a as f32 * k).round().clamp(0.0, 255.0) as u8;
        Rgb(m(self.0), m(self.1), m(self.2))
    }
    pub fn colorref(self) -> COLORREF {
        COLORREF(self.0 as u32 | (self.1 as u32) << 8 | (self.2 as u32) << 16)
    }
}

pub fn paper_color(p: Paper) -> Rgb {
    match p {
        Paper::Yellow => Rgb(255, 238, 145),
        Paper::Pink => Rgb(255, 208, 214),
        Paper::Blue => Rgb(200, 229, 255),
        Paper::Green => Rgb(204, 240, 212),
        Paper::Orange => Rgb(255, 214, 165),
        Paper::Lavender => Rgb(228, 217, 252),
        Paper::White => Rgb(253, 253, 250),
        Paper::Kraft => Rgb(214, 186, 146),
        Paper::MorandiRose => Rgb(222, 196, 192),
        Paper::MorandiBlue => Rgb(190, 204, 216),
        Paper::Sage => Rgb(200, 210, 190),
        Paper::Oat => Rgb(236, 226, 208),
        Paper::MorandiLilac => Rgb(206, 196, 214),
        Paper::MilkTea => Rgb(214, 192, 170),
        Paper::MorandiYellow => Rgb(230, 212, 160),
        Paper::MorandiGrey => Rgb(212, 206, 198),
        // 透明紙：編輯框、按鈕用白色
        Paper::Clear => Rgb(253, 253, 252),
    }
}

fn ink_color(p: Paper) -> Rgb {
    match p {
        Paper::Kraft | Paper::MilkTea => Rgb(66, 44, 26),
        Paper::Blue | Paper::MorandiBlue => Rgb(30, 45, 70),
        _ => Rgb(48, 42, 34),
    }
}

fn effective_paper(note: &Note) -> Paper {
    match note.kind {
        Kind::Image => Paper::White,
        _ => note.paper,
    }
}

pub fn paper_colorref(note: &Note) -> COLORREF {
    paper_color(effective_paper(note)).colorref()
}

pub fn ink_colorref(note: &Note) -> COLORREF {
    ink_color(effective_paper(note)).colorref()
}

/// 劃線一律用紅筆。
const STRIKE: Rgb = Rgb(226, 30, 36);

// ───────────────────────────── 字型 ─────────────────────────────

pub struct Fonts {
    pub body: HFONT,
    pub bold: HFONT,
    pub small: HFONT,
    pub icons: HFONT,
    pub caption: HFONT,
    /// 這組字型的字級倍率
    pub zoom: f32,
}

impl Fonts {
    /// `face` = 字型名稱（"" = 預設的微軟正黑體），`heavy` = 內文用粗體。
    pub fn new(scale: f32, zoom: f32, face: &str, heavy: bool) -> Fonts {
        let body_face = crate::win::wide(if face.is_empty() { super::store::DEFAULT_FACE } else { face });
        let body_face = PCWSTR(body_face.as_ptr());
        let make = |size: f32, weight: i32, italic: bool, face: PCWSTR| unsafe {
            CreateFontW(
                -((size * scale).round() as i32),
                0,
                0,
                0,
                weight,
                italic as u32,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                0,
                face,
            )
        };
        Fonts {
            body: make(15.0 * zoom, if heavy { 700 } else { 400 }, false, body_face),
            bold: make(15.0 * zoom, 700, false, body_face),
            small: make(11.0 * zoom.clamp(0.9, 1.6), 400, false, w!("Microsoft JhengHei UI")),
            icons: make(12.0, 400, false, w!("Segoe MDL2 Assets")),
            // 圖說：預設用手寫風的 Segoe Print，有選字型就用選的
            caption: if face.is_empty() { make(14.0 * zoom.min(2.0), 400, true, w!("Segoe Print")) } else { make(14.0 * zoom.min(2.0), if heavy { 700 } else { 400 }, false, body_face) },
            zoom,
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            for f in [self.body, self.bold, self.small, self.icons, self.caption] {
                let _ = DeleteObject(f.into());
            }
        }
    }
}

// ───────────────────────────── 版面 ─────────────────────────────

/// 視窗大小和紙張在視窗裡的位置（實際像素）。附件會撐大視窗。
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Layout {
    pub w: i32,
    pub h: i32,
    pub paper: RECT,
}

/// 附件的邏輯外框（相對紙張左上角）。
fn attach_bounds(a: &Attach, cache: &mut assets::Cache) -> (f32, f32, f32, f32) {
    let (w, h) = cache.logical_size(&a.asset);
    if assets::cat_of(&a.asset) == Cat::Tape {
        let half = h / 2.0;
        (a.x.min(a.x2) - half, a.y.min(a.y2) - half, a.x.max(a.x2) + half, a.y.max(a.y2) + half)
    } else {
        let r = (w * w + h * h).sqrt() / 2.0 * a.scale;
        (a.x - r, a.y - r, a.x + r, a.y + r)
    }
}

pub fn layout(note: &Note, scale: f32, cache: &mut assets::Cache) -> Layout {
    layout_ex(note, scale, cache, false)
}

/// `bar` = 手寫中，紙張下面多一條工具列。
pub fn layout_ex(note: &Note, scale: f32, cache: &mut assets::Cache, bar: bool) -> Layout {
    let (pw, ph) = if note.kind == Kind::Sticker { (0.0, 0.0) } else { (note.w as f32, note.h as f32) };
    let (mut l, mut t, mut r, mut b) = if note.kind == Kind::Sticker {
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN)
    } else {
        (-SHADOW_L, -SHADOW_T, pw + SHADOW_R, ph + SHADOW_B)
    };
    for a in &note.items {
        let (x0, y0, x1, y1) = attach_bounds(a, cache);
        l = l.min(x0 - ATTACH_PAD);
        t = t.min(y0 - ATTACH_PAD - DROP_SPACE);
        r = r.max(x1 + ATTACH_PAD);
        b = b.max(y1 + ATTACH_PAD);
    }
    if l > r {
        (l, t, r, b) = (-20.0, -20.0, 20.0, 20.0);
    }
    if bar {
        let (bl, bt, br, bb) = ink::bar_bounds(pw, ph);
        l = l.min(bl - 12.0);
        t = t.min(bt);
        r = r.max(br + 12.0);
        b = b.max(bb + 16.0);
    }
    let px = |v: f32| (v * scale).floor() as i32;
    let (left, top) = (px(l), px(t));
    let (right, bottom) = ((r * scale).ceil() as i32, (b * scale).ceil() as i32);
    let paper = RECT { left: -left, top: -top, right: -left + (pw * scale).round() as i32, bottom: -top + (ph * scale).round() as i32 };
    Layout { w: right - left, h: bottom - top, paper }
}

/// 實際像素換算邏輯像素。
pub fn logical(v: i32, scale: f32) -> i32 {
    (v as f32 / scale).round() as i32
}

// ───────────────────────────── 場景 ─────────────────────────────

/// 圖片便利貼：原圖和目前大小的縮圖（避免每次重畫都重新縮放）。
pub struct ImageCache {
    pub original: Pixels,
    pub scaled: Option<Pixels>,
}

/// 附件的動畫：哪一個（None = 全部）、進度 0～1。
#[derive(Clone, Copy)]
pub struct AttachAnim {
    pub which: Option<usize>,
    pub t: f32,
}

/// 文字亂碼動畫（解鎖／上鎖）：從 from 亂掉，再一個個字變成 to。
pub struct Scramble<'a> {
    pub from: &'a [Line],
    pub to: &'a [Line],
    pub t: f32,
}

pub struct Scene<'a> {
    pub note: &'a Note,
    /// 要顯示的文字（解鎖時是真正的內容，否則是 note.lines）
    pub lines: &'a [Line],
    pub scale: f32,
    pub hover: bool,
    pub hot: Option<Btn>,
    /// 0 = 貼在紙上，1 = 拿起來（拖曳中）
    pub lift: f32,
    pub attach_anim: Option<AttachAnim>,
    /// 正在拖的附件（畫成拿起來的樣子）
    pub dragging_item: Option<usize>,
    /// 選取中的附件（顯示旋轉、縮放的控制點）
    pub selected: Option<usize>,
    pub strike: Option<(usize, f32)>,
    pub scramble: Option<Scramble<'a>>,
    pub unlocked: bool,
    pub scroll: i32,
    pub editing: bool,
    /// 整張的不透明度（出現動畫）
    pub alpha: u8,
    pub image: Option<&'a mut ImageCache>,
    pub assets: &'a mut assets::Cache,
    pub fonts: &'a Fonts,
    /// 現在時間（毫秒，蕃茄鐘用）
    pub now_ms: u64,
    /// 蕃茄鐘的動畫
    pub fx: Fx,
    /// 手寫中：工具列和正在寫的那一筆
    pub ink: Option<InkView<'a>>,
    /// 字的顏色（None = 依紙色自動）
    pub text_color: Option<Rgb>,
}

/// 蕃茄鐘的短動畫（進度 0～1）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Fx {
    #[default]
    None,
    /// 時間到：原地閃、一圈圈擴散（不搖）
    Ring(f32),
}

/// 附件在視窗上的形狀（滑鼠判斷用）。
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// 中心、半寬、半高、角度（弧度）
    Box { cx: f32, cy: f32, hw: f32, hh: f32, angle: f32 },
    /// 起點、終點、半寬
    Tape { x1: f32, y1: f32, x2: f32, y2: f32, half: f32 },
}

impl Shape {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        match *self {
            Shape::Box { cx, cy, hw, hh, angle } => {
                let (s, c) = angle.sin_cos();
                let (dx, dy) = (x - cx, y - cy);
                let u = dx * c + dy * s;
                let v = -dx * s + dy * c;
                u.abs() <= hw * 0.8 && v.abs() <= hh * 0.8
            }
            Shape::Tape { .. } => self.tape_pos(x, y).is_some(),
        }
    }
    /// 紙膠帶：點在帶子上的位置（0 = 起點、1 = 終點）。
    pub fn tape_pos(&self, x: f32, y: f32) -> Option<f32> {
        let Shape::Tape { x1, y1, x2, y2, half } = *self else { return None };
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let (px, py) = (x - x1, y - y1);
        let u = (px * dx + py * dy) / len;
        let v = (-px * dy + py * dx) / len;
        (u >= -4.0 && u <= len + 4.0 && v.abs() <= half + 3.0).then_some(u / len)
    }
}

/// 選取附件時的控制點。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    /// 右上角：拖曳旋轉
    Rotate,
    /// 右下角：拖曳縮放
    Scale,
    /// 紙膠帶的兩端
    TapeStart,
    TapeEnd,
}

const ACCENT: Rgb = Rgb(99, 102, 241);

/// 畫完之後給滑鼠判斷用的位置（視窗座標）。
#[derive(Default, Clone)]
pub struct Hits {
    pub layout: Layout,
    /// (行號, 圓圈範圍)
    pub bullets: Vec<(usize, RECT)>,
    pub buttons: Vec<(Btn, RECT)>,
    pub items: Vec<(usize, Shape)>,
    /// 選取中附件的控制點：(附件, 種類, x, y)
    pub handles: Vec<(usize, Handle, f32, f32)>,
    pub grip: RECT,
    /// 文字區（編輯框放這裡）
    pub text_area: RECT,
    pub content_h: i32,
    pub view_h: i32,
    /// 手寫工具列的按鈕和整條的範圍
    pub ink_buttons: Vec<(ink::InkBtn, RECT)>,
    pub bar: RECT,
}

/// 畫好的結果要交給誰。
pub enum Target<'a> {
    /// 更新到分層視窗上，有位置的話順便移動
    Window(HWND, Option<POINT>),
    /// 開發用：複製一份預乘的 BGRA 像素出來（離屏截圖）
    Buffer(&'a mut Vec<u8>),
}

/// 一行文字排版後的一列。
struct Row {
    line: usize,
    text: Vec<u16>,
    width: i32,
    y: i32,
    first: bool,
    meta: bool,
    /// 由右到左的行（阿拉伯文、希伯來文…）
    rtl: bool,
}

/// 畫好並交給視窗（或複製出來）。
pub fn present(target: Target, mut scene: Scene) -> Hits {
    let note = scene.note;
    let s = scene.scale;
    let lay = layout_ex(note, s, scene.assets, scene.ink.is_some());
    let (w, h) = (lay.w, lay.h);
    let paper = lay.paper;
    let paper_rgb = paper_color(effective_paper(note));
    let ink = ink_color(effective_paper(note));
    let text_ink = scene.text_color.unwrap_or(ink);
    let px = |v: f32| (v * s).round() as i32;
    let mut hits = Hits {
        layout: lay,
        grip: RECT { left: paper.right - px(16.0), top: paper.bottom - px(16.0), right: paper.right, bottom: paper.bottom },
        ..Default::default()
    };
    let has_paper = !matches!(note.kind, Kind::Sticker | Kind::Timer);
    let clear = has_paper && effective_paper(note) == Paper::Clear;

    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // 由上往下
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(dib) = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = DeleteDC(mem);
            ReleaseDC(None, screen);
            return hits;
        };
        let old_bmp = SelectObject(mem, dib.into());
        let buf = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
        let mut canvas = Canvas { buf, w, h };
        SetBkMode(mem, TRANSPARENT);

        if clear {
            // ── 透明紙：沒有紙也沒有陰影，字和手寫直接浮在桌面上；滑過時才淡淡看得出範圍 ──
            let area = text_area(note.kind, paper, s, scene.fonts.zoom);
            if scene.editing {
                hits.text_area = area;
            } else {
                clear_text(mem, &mut canvas, &scene, paper, text_ink, &mut hits);
            }
            let framed = scene.hover || scene.ink.is_some() || scene.lift > 0.0;
            let tint = if scene.editing { 0.85 } else if framed { 0.14 } else { 0.0 };
            canvas.under(paper, Rgb(255, 255, 255), tint);
            if framed && !scene.editing {
                canvas.dashed_frame(paper, s);
            }
            // 透明的地方也要收得到滑鼠（才拖得動、滑得出按鈕）
            canvas.catch_mouse(paper);
        } else if has_paper {
            // ── 陰影：貼著時小而淡，拿起來時大而散 ──
            let lift = scene.lift.clamp(0.0, 1.0);
            let lerp = |a: f32, b: f32| a + (b - a) * lift;
            canvas.shadow(paper, 0.0, s * lerp(3.0, 12.0), s * lerp(7.0, 18.0), lerp(0.26, 0.30));
            if lift < 0.99 && note.texture == Texture::Sticky {
                // 底部兩角翹起的陰影（便利貼只有上緣有黏膠）
                let bottom = RECT { left: paper.left + px(6.0), top: paper.bottom - px(14.0), right: paper.right - px(6.0), bottom: paper.bottom };
                canvas.shadow(bottom, 0.0, s * 5.0, s * 5.0, 0.22 * (1.0 - lift));
            }

            // ── 紙張 ──
            let row_h = (ROW_H * scene.fonts.zoom * s).round() as i32;
            let area = text_area(note.kind, paper, s, scene.fonts.zoom);
            let texture = match note.kind {
                Kind::Image => Texture::Plain,
                Kind::Starred if note.texture == Texture::Sticky => Texture::Lined,
                _ => note.texture,
            };
            let lines_from = area.top - scene.scroll % row_h.max(1);
            canvas.paper(paper, paper_rgb, texture, scene.editing, s, row_h, lines_from);

            // ── 文字 ──
            match note.kind {
                Kind::Image => draw_polaroid(mem, &mut canvas, &mut scene, paper, text_ink, &mut hits),
                _ if scene.editing => hits.text_area = area,
                _ => draw_text(mem, &mut canvas, &scene, paper, paper_rgb, text_ink, &mut hits, true),
            }

            // GDI 畫過的地方 alpha 會變 0；紙張是不透明的，整塊補回 255
            let _ = GdiFlush();
            canvas.opaque(paper);
        }

        // ── 手寫（在字上面、圖釘下面）──
        if has_paper {
            ink::draw_strokes(&mut canvas, &note.strokes, scene.ink.as_ref().and_then(|i| i.cur), paper, s);
        }

        // ── 蕃茄鐘 ──
        if note.kind == Kind::Timer {
            super::timer::draw(mem, &mut canvas, &mut scene, paper, &mut hits);
        }

        // ── 附件 ──
        for (i, a) in note.items.iter().enumerate() {
            let anim = scene.attach_anim.filter(|an| an.which.is_none() || an.which == Some(i)).map(|an| an.t);
            let lifted = scene.dragging_item == Some(i);
            if let Some(shape) = draw_attach(&mut canvas, scene.assets, a, paper, s, anim, lifted) {
                hits.items.push((i, shape));
            }
        }

        // ── 選取中的附件：外框和控制點 ──
        if let Some(sel) = scene.selected {
            if let Some((_, shape)) = hits.items.iter().find(|(i, _)| *i == sel).copied() {
                draw_handles(&mut canvas, sel, shape, s, &mut hits);
            }
        }

        // ── 滑過時的按鈕和調整大小的角（畫在附件上面，才不會被擋住）──
        if has_paper && scene.hover && !scene.editing && scene.dragging_item.is_none() && scene.selected.is_none() && scene.ink.is_none() {
            draw_buttons(mem, &mut canvas, &scene, paper, paper_rgb, ink, &mut hits);
        }

        // ── 手寫工具列 ──
        if let Some(iv) = &scene.ink {
            ink::draw_bar(mem, &mut canvas, iv, paper, s, scene.fonts, &mut hits);
        }
        let _ = GdiFlush();

        // ── 交給系統 ──
        let (hwnd, pos) = match target {
            Target::Window(hwnd, pos) => (hwnd, pos),
            Target::Buffer(out) => {
                out.clear();
                out.extend_from_slice(canvas.buf);
                SelectObject(mem, old_bmp);
                let _ = DeleteObject(dib.into());
                let _ = DeleteDC(mem);
                ReleaseDC(None, screen);
                return hits;
            }
        };
        let size = SIZE { cx: w, cy: h };
        let src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: scene.alpha, AlphaFormat: AC_SRC_ALPHA as u8 };
        let _ = UpdateLayeredWindow(
            hwnd,
            Some(screen),
            pos.as_ref().map(|p| p as *const POINT),
            Some(&size),
            Some(mem),
            Some(&src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
        SelectObject(mem, old_bmp);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
    }
    hits
}

/// 編輯框要放的位置（視窗座標）。
pub fn edit_area(note: &Note, paper: RECT, s: f32) -> RECT {
    text_area(note.kind, paper, s, note.font)
}

fn text_area(kind: Kind, paper: RECT, s: f32, zoom: f32) -> RECT {
    let px = |v: f32| (v * s).round() as i32;
    let top = if kind == Kind::Starred { PAD_T + ROW_H * zoom + 6.0 } else { PAD_T };
    RECT {
        left: paper.left + px(PAD_L + GUTTER),
        top: paper.top + px(top),
        right: paper.right - px(PAD_R),
        bottom: paper.bottom - px(PAD_B),
    }
}

// ───────────────────────────── 附件 ─────────────────────────────

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// 畫一個附件，回傳它的形狀。`anim` = 貼上去的動畫進度。
fn draw_attach(canvas: &mut Canvas, cache: &mut assets::Cache, a: &Attach, paper: RECT, s: f32, anim: Option<f32>, lifted: bool) -> Option<Shape> {
    let cat = assets::cat_of(&a.asset);
    let img = cache.get(&a.asset)?;
    let ox = paper.left as f32;
    let oy = paper.top as f32;
    if cat == Cat::Tape {
        let (x1, y1, x2, y2) = (ox + a.x * s, oy + a.y * s, ox + a.x2 * s, oy + a.y2 * s);
        // 動畫：從起點「撫平」到終點
        let reveal = anim.map_or(1.0, ease_out);
        let alpha = if lifted { 0.75 } else { 0.9 };
        canvas.tape(img, x1, y1, x2, y2, alpha, reveal);
        return Some(Shape::Tape { x1, y1, x2, y2, half: img.h as f32 / 2.0 });
    }
    let (mut dy, mut k, mut alpha, mut shadow) = (0.0f32, a.scale, 1.0f32, 1.0f32);
    if lifted {
        dy = -4.0;
        k *= 1.08;
        shadow = 2.4;
    } else if let Some(t) = anim {
        if t < 0.55 {
            // 從上面落下、由大變小
            let u = t / 0.55;
            let e = u * u;
            let (drop, grow) = match cat {
                Cat::Pin => (24.0, 0.2),
                Cat::Clip => (16.0, 0.08),
                _ => (6.0, 0.28),
            };
            dy = -drop * (1.0 - e);
            k *= 1.0 + grow * (1.0 - e);
            alpha = (u * 2.5).min(1.0);
            shadow = 1.0 + 1.5 * (1.0 - e);
        } else {
            // 碰到紙：壓一下再彈回來
            let v = (t - 0.55) / 0.45;
            let wave = (v * std::f32::consts::PI).sin() * (1.0 - v);
            dy = -3.0 * wave;
            k *= 1.0 - 0.05 * wave;
        }
    }
    let cx = ox + a.x * s;
    let cy = oy + a.y * s + dy * s;
    let angle = a.angle.to_radians();
    let (hw, hh) = (img.w as f32 * a.scale / 2.0, img.h as f32 * a.scale / 2.0);
    // 放大時改用從原圖縮的大圖，不會糊
    let (img, step) = cache.get_at(&a.asset, k)?;
    let k = k / step;
    // 貼紙、圖釘、夾子都有一點點落影
    canvas.blit_xf(img, cx + 1.2 * s * shadow, cy + 2.2 * s * shadow, angle, k, 0.22 * alpha, true);
    canvas.blit_xf(img, cx, cy, angle, k, alpha, false);
    Some(Shape::Box { cx, cy: oy + a.y * s, hw, hh, angle })
}

// ───────────────────────────── 逐點繪圖 ─────────────────────────────

pub(super) struct Canvas<'a> {
    pub buf: &'a mut [u8],
    pub w: i32,
    pub h: i32,
}

pub(super) fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 便宜的整數雜湊雜訊（紙張纖維、亂碼動畫用），回傳 0～1。
pub(super) fn noise(x: i32, y: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n ^ (n >> 16)) & 0xffff) as f32 / 65535.0
}

/// 圖片上某一點的預乘顏色（雙線性）。
fn sample(img: &Pixels, u: f32, v: f32) -> [f32; 4] {
    let (w, h) = (img.w as i32, img.h as i32);
    let fx = u - 0.5;
    let fy = v - 0.5;
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    let mut acc = [0f32; 4];
    for (dx, dy, wt) in [(0, 0, (1.0 - tx) * (1.0 - ty)), (1, 0, tx * (1.0 - ty)), (0, 1, (1.0 - tx) * ty), (1, 1, tx * ty)] {
        let (x, y) = (x0 + dx, y0 + dy);
        if x < 0 || y < 0 || x >= w || y >= h || wt <= 0.0 {
            continue;
        }
        let i = ((y * w + x) * 4) as usize;
        let a = img.data[i + 3] as f32 / 255.0;
        acc[0] += img.data[i] as f32 * a * wt;
        acc[1] += img.data[i + 1] as f32 * a * wt;
        acc[2] += img.data[i + 2] as f32 * a * wt;
        acc[3] += a * wt;
    }
    acc
}

impl Canvas<'_> {
    /// 預乘顏色疊到 (x, y)。
    pub(super) fn over(&mut self, x: i32, y: i32, c: [f32; 4], alpha: f32) {
        let a = c[3] * alpha;
        if a <= 0.003 {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        for k in 0..3 {
            self.buf[i + k] = (c[k] * alpha + self.buf[i + k] as f32 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        }
        let da = self.buf[i + 3] as f32 / 255.0;
        self.buf[i + 3] = ((a + da * (1.0 - a)) * 255.0).round() as u8;
    }

    /// 模糊矩形陰影：兩個方向各自做平滑的邊緣再相乘，便宜又好看。
    pub(super) fn shadow(&mut self, rc: RECT, ox: f32, oy: f32, blur: f32, strength: f32) {
        let b = blur.max(1.0);
        let (l, t, r, btm) = (rc.left as f32 + ox, rc.top as f32 + oy, rc.right as f32 + ox, rc.bottom as f32 + oy);
        let x0 = ((l - b) as i32).max(0);
        let x1 = ((r + b) as i32 + 1).min(self.w);
        let y0 = ((t - b) as i32).max(0);
        let y1 = ((btm + b) as i32 + 1).min(self.h);
        for y in y0..y1 {
            let fy = y as f32 + 0.5;
            let cy = smooth((fy - (t - b)) / (2.0 * b)) * smooth(((btm + b) - fy) / (2.0 * b));
            if cy <= 0.0 {
                continue;
            }
            for x in x0..x1 {
                let fx = x as f32 + 0.5;
                let cx = smooth((fx - (l - b)) / (2.0 * b)) * smooth(((r + b) - fx) / (2.0 * b));
                self.over(x, y, [0.0, 0.0, 0.0, 1.0], strength * cx * cy);
            }
        }
    }

    /// 紙張和紙質。`row_h`、`lines_from`：橫線紙的線要對齊文字列。
    #[allow(clippy::too_many_arguments)]
    fn paper(&mut self, rc: RECT, base: Rgb, texture: Texture, flat: bool, s: f32, row_h: i32, lines_from: i32) {
        let ph = (rc.bottom - rc.top).max(1) as f32;
        let pw = (rc.right - rc.left).max(1) as f32;
        let glue = 16.0 * s;
        let curl = 46.0 * s;
        let cell = (18.0 * s).round().max(4.0) as i32;
        let line_c = base.shade(0.86);
        let dot_r2 = (1.1 * s) * (1.1 * s);
        for y in rc.top.max(0)..rc.bottom.min(self.h) {
            let ty = (y - rc.top) as f32;
            for x in rc.left.max(0)..rc.right.min(self.w) {
                let tx = (x - rc.left) as f32;
                let mut k = 1.0;
                let mut c = base;
                if !flat {
                    k *= 1.02 - 0.04 * (ty / ph);
                    match texture {
                        Texture::Sticky => {
                            if ty < glue {
                                k *= 0.975;
                            }
                            let d = (pw - tx) + (ph - ty);
                            if d < curl {
                                k *= 0.94 + 0.06 * (d / curl);
                            }
                        }
                        Texture::Grid => {
                            let (gx, gy) = ((x - rc.left) % cell, (y - rc.top) % cell);
                            if gx == 0 || gy == 0 {
                                c = line_c.mix(base, 0.35);
                            }
                        }
                        Texture::Lined => {
                            if row_h > 0 && y > rc.top + (6.0 * s) as i32 && (y - lines_from).rem_euclid(row_h) == row_h - 1 {
                                c = Rgb(170, 196, 228).mix(base, 0.35);
                            }
                        }
                        Texture::Dots => {
                            let (gx, gy) = ((x - rc.left) % cell, (y - rc.top) % cell);
                            let (dx, dy) = (gx as f32 - cell as f32 / 2.0, gy as f32 - cell as f32 / 2.0);
                            if dx * dx + dy * dy <= dot_r2 {
                                c = line_c.shade(0.92);
                            }
                        }
                        Texture::Washi => {
                            // 細細的纖維：橫向拉長的雜訊
                            let n = noise(x / 5, y) * 0.6 + noise(x, y) * 0.4;
                            k *= 0.985 + 0.03 * n;
                        }
                        Texture::Fiber => {
                            let n = noise(x / 3, y / 2) * 0.5 + noise(x, y) * 0.5;
                            k *= 0.96 + 0.07 * n;
                            if noise(x + 7, y + 13) > 0.995 {
                                k *= 0.85;
                            }
                        }
                        Texture::Plain => {}
                    }
                }
                let c = c.shade(k);
                let i = ((y * self.w + x) * 4) as usize;
                self.buf[i] = c.2;
                self.buf[i + 1] = c.1;
                self.buf[i + 2] = c.0;
                self.buf[i + 3] = 255;
            }
        }
    }

    /// GDI 畫完後把範圍內的 alpha 補成不透明。
    pub(super) fn opaque(&mut self, rc: RECT) {
        for y in rc.top.max(0)..rc.bottom.min(self.h) {
            for x in rc.left.max(0)..rc.right.min(self.w) {
                self.buf[((y * self.w + x) * 4 + 3) as usize] = 255;
            }
        }
    }

    /// 圖片（沒預乘的 BGRA）以 (cx, cy) 為中心、旋轉 angle（弧度）、縮放 k 疊上去。`shadow` = 畫成黑色剪影。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn blit_xf(&mut self, img: &Pixels, cx: f32, cy: f32, angle: f32, k: f32, alpha: f32, shadow: bool) {
        let k = k.max(0.05);
        let (sn, cs) = angle.sin_cos();
        let (hw, hh) = (img.w as f32 * k / 2.0, img.h as f32 * k / 2.0);
        let r = (hw * hw + hh * hh).sqrt() + 1.0;
        let x0 = ((cx - r).floor() as i32).max(0);
        let x1 = ((cx + r).ceil() as i32).min(self.w);
        let y0 = ((cy - r).floor() as i32).max(0);
        let y1 = ((cy + r).ceil() as i32).min(self.h);
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let u = (dx * cs + dy * sn) / k + img.w as f32 / 2.0;
                let v = (-dx * sn + dy * cs) / k + img.h as f32 / 2.0;
                if u < -1.0 || v < -1.0 || u > img.w as f32 + 1.0 || v > img.h as f32 + 1.0 {
                    continue;
                }
                let mut c = sample(img, u, v);
                if shadow {
                    c = [0.0, 0.0, 0.0, c[3]];
                }
                self.over(x, y, c, alpha);
            }
        }
    }

    /// 紙膠帶：從 (x1, y1) 拉到 (x2, y2)。兩端用素材撕開的邊，中間的花紋鏡像接續（看不出接縫）。
    #[allow(clippy::too_many_arguments)]
    fn tape(&mut self, tex: &Pixels, x1: f32, y1: f32, x2: f32, y2: f32, alpha: f32, reveal: f32) {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 2.0 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len);
        let width = tex.h as f32;
        let tw = tex.w as f32;
        let cap = (tw * TAPE_CAP).min(len / 2.0);
        let mid = (tw - 2.0 * cap).max(1.0);
        let pad = width / 2.0 + 1.0;
        let x0 = ((x1.min(x2) - pad).floor() as i32).max(0);
        let xe = ((x1.max(x2) + pad).ceil() as i32).min(self.w);
        let y0 = ((y1.min(y2) - pad).floor() as i32).max(0);
        let ye = ((y1.max(y2) + pad).ceil() as i32).min(self.h);
        let limit = len * reveal;
        for y in y0..ye {
            for x in x0..xe {
                let (px, py) = (x as f32 + 0.5 - x1, y as f32 + 0.5 - y1);
                let u = px * ux + py * uy;
                let v = -px * uy + py * ux + width / 2.0;
                if u < 0.0 || u > limit || v < 0.0 || v >= width {
                    continue;
                }
                let tx = if u < cap {
                    u
                } else if u > len - cap {
                    tw - (len - u)
                } else {
                    let m = (u - cap) % (2.0 * mid);
                    cap + if m < mid { m } else { 2.0 * mid - m }
                };
                let c = sample(tex, tx, v);
                self.over(x, y, c, alpha);
            }
        }
    }

    /// 沒預乘的 BGRA 圖片直接疊上去（不縮放）。
    fn blit(&mut self, img: &Pixels, x: i32, y: i32) {
        for sy in 0..img.h as i32 {
            let dy = y + sy;
            if dy < 0 || dy >= self.h {
                continue;
            }
            for sx in 0..img.w as i32 {
                let dx = x + sx;
                if dx < 0 || dx >= self.w {
                    continue;
                }
                let si = ((sy * img.w as i32 + sx) * 4) as usize;
                let a = img.data[si + 3] as f32 / 255.0;
                let c = [img.data[si] as f32 * a, img.data[si + 1] as f32 * a, img.data[si + 2] as f32 * a, a];
                self.over(dx, dy, c, 1.0);
            }
        }
    }

    /// 文字區上下緣淡出到紙色（內容超出、可以捲動時）。
    fn fade(&mut self, area: RECT, color: Rgb, top: bool, bottom: bool, band: i32) {
        for y in area.top.max(0)..area.bottom.min(self.h) {
            let t = if top && y < area.top + band {
                1.0 - (y - area.top) as f32 / band as f32
            } else if bottom && y >= area.bottom - band {
                (y - (area.bottom - band)) as f32 / band as f32
            } else {
                continue;
            };
            for x in area.left.max(0)..area.right.min(self.w) {
                let i = ((y * self.w + x) * 4) as usize;
                let c = Rgb(self.buf[i + 2], self.buf[i + 1], self.buf[i]).mix(color, t);
                self.buf[i] = c.2;
                self.buf[i + 1] = c.1;
                self.buf[i + 2] = c.0;
            }
        }
    }

    /// 實心圓（按鈕底），預乘疊上去。
    pub(super) fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, alpha: f32) {
        let (x0, x1) = (((cx - r - 1.0) as i32).max(0), ((cx + r + 1.0) as i32).min(self.w));
        let (y0, y1) = (((cy - r - 1.0) as i32).max(0), ((cy + r + 1.0) as i32).min(self.h));
        for y in y0..y1 {
            for x in x0..x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (r - d + 0.5).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.over(x, y, [c.2 as f32, c.1 as f32, c.0 as f32, 1.0], alpha * cov);
                }
            }
        }
    }
}

// ───────────────────────────── 文字 ─────────────────────────────

/// 把一行字依寬度折成多列。中文逐字斷，英文盡量在空白斷。
unsafe fn wrap(hdc: HDC, units: &[u16], max_w: i32) -> Vec<(Vec<u16>, i32)> {
    if units.is_empty() {
        return vec![(Vec::new(), 0)];
    }
    let mut rows = Vec::new();
    let mut start = 0;
    while start < units.len() {
        let rest = &units[start..];
        let mut fit = 0i32;
        let mut size = SIZE::default();
        let _ = GetTextExtentExPointW(hdc, PCWSTR(rest.as_ptr()), rest.len() as i32, max_w.max(1), Some(&mut fit), None, &mut size);
        let mut n = (fit.max(1) as usize).min(rest.len());
        // 不要把代理對（emoji）切成兩半
        if n < rest.len() && (0xDC00..=0xDFFF).contains(&rest[n]) && n > 1 {
            n -= 1;
        }
        if n < rest.len() {
            let is_word = |c: u16| c < 128 && (c as u8).is_ascii_alphanumeric();
            if is_word(rest[n]) && is_word(rest[n - 1]) {
                if let Some(sp) = rest[..n].iter().rposition(|&c| c == b' ' as u16) {
                    if sp > 0 {
                        n = sp + 1;
                    }
                }
            }
        }
        let piece = rest[..n].to_vec();
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &piece, &mut sz);
        rows.push((piece, sz.cx));
        start += n;
    }
    rows
}

/// 亂碼動畫用的字。
const GLYPHS: &str = "的一是了我不人在有這個上們來到時大地為子中你說生國年著就那和要出也得裡後自以會家可下而過天去能對小多然於心學之都好看起發當沒成只如事把還用第樣道想作種開美總從無情己面最但現前些所同日手又行意動方期它頭經長兒回位分愛老因很給名法間知世什兩次使身者被高已親其進此話常與活正感見明問力理點文幾定本公特做外孩相西果走將月十實向聲車全信重三機工物氣每並別真打太新比才便夫再書部水像眼等體卻加電主界門利海受聽表德少克代員許先口由安寫性馬光白或住難望教命花結樂色更拉東神記處讓母父應直字場平報友關放至張認接告入笑內英候民往何度山覺路帶萬男邊風解叫任金快原吃媽變通師立象數四失滿遠格士音輕目條始達深完今提求清王化空業思切怎非找片錢吧ABCDEFGHJKLMNPQRSTUVWXYZabcdefghkmnpqrstuvwxyz0123456789#%&?";

/// 英數字亂碼用的字（英文的內容亂成英文，看起來才自然）。
const LATIN: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghkmnpqrstuvwxyz0123456789#%&?";

/// 亂碼的一個字：原本是英數字就亂成英數字，其他（中日韓…）亂成漢字。
fn scramble_char(orig: char, i: usize, frame: u32) -> char {
    let set = if orig.is_ascii() { LATIN } else { GLYPHS };
    let count = set.chars().count();
    let h = noise(i as i32 * 31 + 7, frame as i32 * 17 + 3);
    set.chars().nth(((h * (count - 1) as f32) as usize).min(count - 1)).unwrap_or('#')
}

/// 亂碼動畫某一刻要顯示的文字。
fn scrambled(sc: &Scramble) -> Vec<Line> {
    let frame = (sc.t * 1000.0 / 45.0) as u32; // 大約每 45ms 換一次字
    let mut n = 0usize;
    let mut garble = |text: &str, keep: &dyn Fn(usize) -> bool| -> String {
        text.chars()
            .map(|c| {
                let i = n;
                n += 1;
                if c.is_whitespace() || keep(i) {
                    c
                } else {
                    scramble_char(c, i, frame)
                }
            })
            .collect()
    };
    if sc.t < 0.35 {
        // 原本的字一個個亂掉
        let p = sc.t / 0.35;
        sc.from
            .iter()
            .map(|l| Line { text: garble(&l.text, &|i| noise(i as i32, 99) > p), done: l.done, meta: l.meta.clone() })
            .collect()
    } else {
        // 新的字從左到右一個個「解出來」
        let total = sc.to.iter().map(|l| l.text.chars().count()).sum::<usize>().max(1);
        let u = (sc.t - 0.35) / 0.65;
        sc.to
            .iter()
            .map(|l| Line {
                text: garble(&l.text, &|i| (i as f32 / total as f32) + noise(i as i32, 5) * 0.25 < u * 1.25),
                done: l.done && u >= 1.0,
                meta: l.meta.clone(),
            })
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn draw_text(hdc: HDC, canvas: &mut Canvas, scene: &Scene, paper: RECT, paper_rgb: Rgb, ink: Rgb, hits: &mut Hits, fade: bool) {
    let note = scene.note;
    let s = scene.scale;
    let zoom = scene.fonts.zoom;
    let px = |v: f32| (v * s).round() as i32;
    let fonts = scene.fonts;
    let area = text_area(note.kind, paper, s, zoom);
    hits.text_area = area;
    let row_h = px(ROW_H * zoom);
    let meta_h = px(META_H * zoom.max(0.9));
    let scrambled_lines;
    let lines: &[Line] = match &scene.scramble {
        Some(sc) => {
            scrambled_lines = scrambled(sc);
            &scrambled_lines
        }
        None => scene.lines,
    };

    // 星號記事本：標題和左邊的紅色邊線
    if note.kind == Kind::Starred {
        SelectObject(hdc, fonts.bold.into());
        SetTextColor(hdc, ink.colorref());
        let title: Vec<u16> = crate::i18n::tf("starred.title", &[("count", &lines.len().to_string())]).encode_utf16().collect();
        if crate::i18n::is_rtl() {
            SetTextAlign(hdc, TA_RIGHT | TA_RTLREADING);
            let _ = TextOutW(hdc, paper.right - px(PAD_R), paper.top + px(PAD_T) - px(4.0), &title);
            SetTextAlign(hdc, TA_LEFT);
        } else {
            let _ = TextOutW(hdc, paper.left + px(PAD_L), paper.top + px(PAD_T) - px(4.0), &title);
        }
        let pen = CreatePen(PS_SOLID, px(1.0).max(1), Rgb(236, 120, 120).colorref());
        let old = SelectObject(hdc, pen.into());
        let mx = area.left - px(GUTTER) + px(4.0);
        let _ = MoveToEx(hdc, mx, paper.top + px(8.0), None);
        let _ = LineTo(hdc, mx, paper.bottom);
        SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }

    // 排版
    let mut rows = Vec::new();
    let mut y = 0;
    SelectObject(hdc, fonts.body.into());
    for (i, line) in lines.iter().enumerate() {
        let rtl = crate::i18n::text_is_rtl(&line.text);
        let units: Vec<u16> = line.text.encode_utf16().collect();
        for (k, (text, width)) in wrap(hdc, &units, area.right - area.left).into_iter().enumerate() {
            rows.push(Row { line: i, text, width, y, first: k == 0, meta: false, rtl });
            y += row_h;
        }
        if note.kind == Kind::Starred && !line.meta.is_empty() {
            rows.push(Row { line: i, text: line.meta.encode_utf16().collect(), width: 0, y, first: false, meta: true, rtl });
            y += meta_h;
        }
    }
    let content_h = y;
    let view_h = area.bottom - area.top;
    hits.content_h = content_h;
    hits.view_h = view_h;
    let scroll = scene.scroll.clamp(0, (content_h - view_h).max(0));

    let save = SaveDC(hdc);
    let _ = IntersectClipRect(hdc, area.left - px(GUTTER), area.top, area.right + px(4.0), area.bottom);
    let body_h = {
        let mut tm = TEXTMETRICW::default();
        let _ = GetTextMetricsW(hdc, &mut tm);
        tm.tmHeight
    };
    let mut strike_rows: Vec<(usize, i32, i32, i32, bool)> = Vec::new(); // (行號, x0, x1, y, 由右到左)
    // 由右到左的行：靠右對齊、從右邊開始排
    let text_out = |hdc: HDC, x_left: i32, y: i32, text: &[u16], rtl: bool| {
        if rtl {
            SetTextAlign(hdc, TA_RIGHT | TA_RTLREADING);
            let _ = TextOutW(hdc, area.right, y, text);
            SetTextAlign(hdc, TA_LEFT);
        } else {
            let _ = TextOutW(hdc, x_left, y, text);
        }
    };
    for row in &rows {
        let y = area.top + row.y - scroll;
        if y + row_h < area.top || y > area.bottom {
            continue;
        }
        let line = &lines[row.line];
        if row.meta {
            SelectObject(hdc, fonts.small.into());
            SetTextColor(hdc, ink.mix(paper_rgb, 0.5).colorref());
            text_out(hdc, area.left, y, &row.text, row.rtl);
            continue;
        }
        SelectObject(hdc, fonts.body.into());
        SetTextColor(hdc, if line.done { ink.mix(paper_rgb, 0.55) } else { ink }.colorref());
        let ty = y + (row_h - body_h) / 2;
        text_out(hdc, area.left, ty, &row.text, row.rtl);
        if row.first && !line.text.trim().is_empty() && scene.scramble.is_none() {
            let cx = area.left - px(GUTTER) / 2 - px(2.0);
            let cy = y + row_h / 2;
            let r = px(5.5);
            let bullet = RECT { left: cx - px(9.0), top: y, right: cx + px(9.0), bottom: y + row_h };
            if y >= area.top && y + row_h <= area.bottom + row_h / 2 {
                hits.bullets.push((row.line, bullet));
            }
            if line.done {
                SelectObject(hdc, fonts.icons.into());
                SetTextColor(hdc, STRIKE.colorref());
                let mut rc = bullet;
                let mut check: Vec<u16> = "\u{E8FB}".encode_utf16().collect();
                DrawTextW(hdc, &mut check, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            } else if scene.hover {
                let pen = CreatePen(PS_SOLID, px(1.2).max(1), ink.mix(paper_rgb, 0.55).colorref());
                let old_pen = SelectObject(hdc, pen.into());
                let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
                let _ = Ellipse(hdc, cx - r, cy - r, cx + r + 1, cy + r + 1);
                SelectObject(hdc, old_pen);
                SelectObject(hdc, old_brush);
                let _ = DeleteObject(pen.into());
            }
        }
        if !row.text.is_empty() {
            let (x0, x1) = if row.rtl { (area.right - row.width - px(2.0), area.right + px(2.0)) } else { (area.left - px(2.0), area.left + row.width + px(2.0)) };
            strike_rows.push((row.line, x0, x1, y + row_h / 2 + px(1.0), row.rtl));
        }
    }

    // 劃線：紅筆隨手一劃，稍微有點抖
    for (i, line) in lines.iter().enumerate() {
        let progress = match scene.strike {
            Some((li, p)) if li == i => p,
            _ if line.done => 1.0,
            _ => continue,
        };
        let segs: Vec<_> = strike_rows.iter().filter(|r| r.0 == i).collect();
        let total: i32 = segs.iter().map(|r| r.2 - r.1).sum();
        let mut budget = (total as f32 * progress) as i32;
        let pen = CreatePen(PS_SOLID, px(2.2).max(2), STRIKE.colorref());
        let old = SelectObject(hdc, pen.into());
        for (n, &&(_, x0, x1, y, rtl)) in segs.iter().enumerate() {
            if budget <= 0 {
                break;
            }
            // 由右到左的行從右邊劃過去
            let (from, to, dir) = if rtl { (x1, (x1 - budget).max(x0), -1) } else { (x0, (x0 + budget).min(x1), 1) };
            budget -= x1 - x0;
            let seed = (i * 7 + n * 3) as f32;
            let wob = |x: i32| y + ((x as f32 * 0.045 / s + seed).sin() * 1.1 * s) as i32;
            let _ = MoveToEx(hdc, from, wob(from), None);
            let mut x = from;
            while (x - to) * dir < 0 {
                x = if dir > 0 { (x + px(5.0)).min(to) } else { (x - px(5.0)).max(to) };
                let _ = LineTo(hdc, x, wob(x));
            }
        }
        SelectObject(hdc, old);
        let _ = DeleteObject(pen.into());
    }
    let _ = RestoreDC(hdc, save);

    if fade && content_h > view_h {
        canvas.fade(area, paper_rgb, scroll > 0, scroll < content_h - view_h, px(16.0));
    }
}

unsafe fn draw_polaroid(hdc: HDC, canvas: &mut Canvas, scene: &mut Scene, paper: RECT, ink: Rgb, hits: &mut Hits) {
    let s = scene.scale;
    let px = |v: f32| (v * s).round() as i32;
    let frame = RECT {
        left: paper.left + px(POLAROID_SIDE),
        top: paper.top + px(POLAROID_SIDE),
        right: paper.right - px(POLAROID_SIDE),
        bottom: paper.bottom - px(POLAROID_BOTTOM),
    };
    let caption = RECT { left: paper.left + px(10.0), top: frame.bottom, right: paper.right - px(10.0), bottom: paper.bottom };
    hits.text_area = caption;
    if let Some(cache) = scene.image.as_deref_mut() {
        let (fw, fh) = ((frame.right - frame.left).max(1), (frame.bottom - frame.top).max(1));
        let (ow, oh) = (cache.original.w as f32, cache.original.h as f32);
        let k = (fw as f32 / ow).min(fh as f32 / oh);
        let (tw, th) = (((ow * k).round() as u32).max(1), ((oh * k).round() as u32).max(1));
        if cache.scaled.as_ref().map(|p| (p.w, p.h)) != Some((tw, th)) {
            cache.scaled = Some(cache.original.resized(tw, th));
        }
        if let Some(img) = &cache.scaled {
            canvas.blit(img, frame.left + (fw - tw as i32) / 2, frame.top + (fh - th as i32) / 2);
        }
    }
    if !scene.editing {
        if let Some(line) = scene.lines.first().filter(|l| !l.text.is_empty()) {
            SelectObject(hdc, scene.fonts.caption.into());
            SetTextColor(hdc, ink.colorref());
            let mut text: Vec<u16> = line.text.encode_utf16().collect();
            let mut rc = caption;
            DrawTextW(hdc, &mut text, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX | if crate::i18n::text_is_rtl(&line.text) { DT_RTLREADING } else { DRAW_TEXT_FORMAT(0) });
        }
    }
}

/// 滑過時的按鈕：右上角「新增／上鎖／選單／刪除」，右下角「字小／字大」和調整大小的角。
/// 畫在附件之後，所以要自己處理 alpha（逐點畫底、GDI 畫字後補 alpha）。
unsafe fn draw_buttons(hdc: HDC, canvas: &mut Canvas, scene: &Scene, paper: RECT, paper_rgb: Rgb, ink: Rgb, hits: &mut Hits) {
    let s = scene.scale;
    let px = |v: f32| (v * s).round() as i32;
    let size = px(24.0);
    let gap = px(4.0);
    let top_y = paper.top + px(6.0);
    let bottom_y = paper.bottom - px(6.0) - size;
    let mut list: Vec<(Btn, &str, i32, i32)> = Vec::new();
    // 右上角，從右往左排
    let mut x = paper.right - px(6.0) - size;
    let mut top = vec![(Btn::Delete, "\u{E711}"), (Btn::Menu, "\u{E712}")];
    if scene.unlocked {
        top.push((Btn::Lock, "\u{E72E}"));
    }
    if scene.note.kind == Kind::Text {
        top.push((Btn::New, "\u{E710}"));
        if !scene.note.locked() {
            top.push((Btn::Ink, "\u{E70F}"));
        }
    }
    for (b, g) in top {
        list.push((b, g, x, top_y));
        x -= size + gap;
    }
    // 右下角：字大、字小（圖片便利貼沒有）
    if scene.note.kind != Kind::Image {
        let mut x = paper.right - px(20.0) - size;
        for (b, g) in [(Btn::Bigger, "A+"), (Btn::Smaller, "A−")] {
            list.push((b, g, x, bottom_y));
            x -= size + gap;
        }
    }
    for &(btn, _, x, y) in &list {
        let hot = scene.hot == Some(btn);
        let bg = if hot {
            if btn == Btn::Delete { Rgb(224, 82, 82) } else { ink.mix(paper_rgb, 0.25) }
        } else {
            paper_rgb.shade(0.9)
        };
        canvas.disc(x as f32 + size as f32 / 2.0, y as f32 + size as f32 / 2.0, size as f32 / 2.0, bg, if hot { 1.0 } else { 0.92 });
    }
    // 調整大小的角
    let grip = paper_rgb.mix(ink, 0.35);
    for (dx, dy) in [(4, 4), (8, 4), (4, 8), (12, 4), (8, 8), (4, 12)] {
        canvas.disc(
            (paper.right - px(dx as f32) - px(2.0)) as f32 + 1.0,
            (paper.bottom - px(dy as f32) - px(2.0)) as f32 + 1.0,
            px(1.1) as f32,
            grip,
            1.0,
        );
    }
    // 字（GDI 會把 alpha 清掉，畫完把字的範圍補回不透明；按鈕都在紙上，底下本來就不透明）
    for &(btn, glyph, x, y) in &list {
        let hot = scene.hot == Some(btn);
        let font = if glyph.starts_with('A') { scene.fonts.small } else { scene.fonts.icons };
        SelectObject(hdc, font.into());
        SetTextColor(hdc, if hot { Rgb(255, 255, 255) } else { ink.mix(paper_rgb, 0.2) }.colorref());
        let mut t: Vec<u16> = glyph.encode_utf16().collect();
        let mut rc = RECT { left: x, top: y, right: x + size, bottom: y + size };
        DrawTextW(hdc, &mut t, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
        canvas.opaque_disc(x as f32 + size as f32 / 2.0, y as f32 + size as f32 / 2.0, size as f32 / 2.0 - 1.0);
        hits.buttons.push((btn, RECT { left: x, top: y, right: x + size, bottom: y + size }));
    }
}

/// 點在不在範圍裡。
pub fn in_rect(rc: &RECT, x: i32, y: i32) -> bool {
    x >= rc.left && x < rc.right && y >= rc.top && y < rc.bottom
}

/// 給新便利貼估一個剛好的高度（邏輯像素）。
pub fn fit_height(lines: &[Line], w: i32, scale: f32, fonts: &Fonts) -> i32 {
    unsafe {
        let hdc = CreateCompatibleDC(None);
        SelectObject(hdc, fonts.body.into());
        let width = ((w as f32 - PAD_L - GUTTER - PAD_R) * scale) as i32;
        let rows: usize = lines.iter().map(|l| wrap(hdc, &l.text.encode_utf16().collect::<Vec<_>>(), width).len()).sum();
        let _ = DeleteDC(hdc);
        let h = PAD_T + rows.max(1) as f32 * ROW_H * fonts.zoom + PAD_B + 10.0;
        (h.round() as i32).clamp(MIN_H, 900)
    }
}

/// 工作區（扣掉工作列）。
pub fn work_area_at(pt: POINT) -> RECT {
    unsafe {
        let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(mon, &mut info).as_bool() {
            info.rcWork
        } else {
            RECT { left: 0, top: 0, right: 1280, bottom: 720 }
        }
    }
}

/// 畫一個素材的預覽（貼紙小舖的卡片、拖曳中的影子用）：回傳預乘 BGRA。
pub fn render_asset(cache: &mut assets::Cache, id: &str, box_w: i32, box_h: i32, alpha: f32) -> Option<Vec<u8>> {
    let img = cache.get(id)?.clone();
    let mut buf = vec![0u8; (box_w * box_h * 4) as usize];
    let mut canvas = Canvas { buf: &mut buf, w: box_w, h: box_h };
    let (cx, cy) = (box_w as f32 / 2.0, box_h as f32 / 2.0);
    if assets::cat_of(id) == Cat::Tape {
        let len = (box_w as f32 * 0.9).min(img.w as f32);
        canvas.tape(&img, cx - len / 2.0, cy + 4.0, cx + len / 2.0, cy - 4.0, alpha * 0.92, 1.0);
    } else {
        let k = (box_w as f32 * 0.86 / img.w as f32).min(box_h as f32 * 0.86 / img.h as f32).min(1.0);
        canvas.blit_xf(&img, cx + 1.5, cy + 2.5, 0.0, k, 0.22 * alpha, true);
        canvas.blit_xf(&img, cx, cy, 0.0, k, alpha, false);
    }
    Some(buf)
}

/// 選取中的附件：細外框＋控制點（旋轉＝實心圓，縮放＝白底圓框；紙膠帶兩端各一個）。
fn draw_handles(canvas: &mut Canvas, idx: usize, shape: Shape, s: f32, hits: &mut Hits) {
    let r = 6.0 * s;
    let dot = |canvas: &mut Canvas, x: f32, y: f32, filled: bool| {
        canvas.disc(x, y + 1.0 * s, r + 1.0, Rgb(0, 0, 0), 0.18);
        canvas.disc(x, y, r, if filled { ACCENT } else { Rgb(255, 255, 255) }, 1.0);
        if !filled {
            canvas.ring(x, y, r - 0.8 * s, 1.6 * s, ACCENT);
        }
    };
    match shape {
        Shape::Box { cx, cy, hw, hh, angle } => {
            let (sn, cs) = angle.sin_cos();
            let corner = |u: f32, v: f32| (cx + u * cs - v * sn, cy + u * sn + v * cs);
            let pts = [corner(-hw, -hh), corner(hw, -hh), corner(hw, hh), corner(-hw, hh)];
            for k in 0..4 {
                let (a, b) = (pts[k], pts[(k + 1) % 4]);
                canvas.line(a.0, a.1, b.0, b.1, 1.4 * s, ACCENT, 0.85);
            }
            // 外框到控制點之間常常是透明的：分層視窗在 alpha 0 的地方收不到滑鼠，游標一移過去
            // 就被當成離開視窗、控制點跟著消失。整塊（含控制點外圍）補上看不見的 alpha
            let reach = r + 8.0 * s;
            let (mut l, mut t, mut rr, mut b) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for p in pts {
                (l, t, rr, b) = (l.min(p.0), t.min(p.1), rr.max(p.0), b.max(p.1));
            }
            canvas.catch_mouse(RECT { left: (l - reach) as i32, top: (t - reach) as i32, right: (rr + reach).ceil() as i32, bottom: (b + reach).ceil() as i32 });
            dot(canvas, pts[1].0, pts[1].1, true);
            dot(canvas, pts[2].0, pts[2].1, false);
            hits.handles.push((idx, Handle::Rotate, pts[1].0, pts[1].1));
            hits.handles.push((idx, Handle::Scale, pts[2].0, pts[2].1));
        }
        Shape::Tape { x1, y1, x2, y2, half } => {
            let reach = half.max(r) + 8.0 * s;
            canvas.catch_mouse(RECT {
                left: (x1.min(x2) - reach) as i32,
                top: (y1.min(y2) - reach) as i32,
                right: (x1.max(x2) + reach).ceil() as i32,
                bottom: (y1.max(y2) + reach).ceil() as i32,
            });
            dot(canvas, x1, y1, false);
            dot(canvas, x2, y2, false);
            hits.handles.push((idx, Handle::TapeStart, x1, y1));
            hits.handles.push((idx, Handle::TapeEnd, x2, y2));
        }
    }
}

impl Canvas<'_> {
    /// 反鋸齒的線段。
    pub(super) fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, c: Rgb, alpha: f32) {
        let pad = width + 1.0;
        let (lx, hx) = (((x0.min(x1) - pad) as i32).max(0), ((x0.max(x1) + pad) as i32).min(self.w));
        let (ly, hy) = (((y0.min(y1) - pad) as i32).max(0), ((y0.max(y1) + pad) as i32).min(self.h));
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len2 = (dx * dx + dy * dy).max(0.001);
        for y in ly..hy {
            for x in lx..hx {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let t = (((px - x0) * dx + (py - y0) * dy) / len2).clamp(0.0, 1.0);
                let d = ((px - x0 - t * dx).powi(2) + (py - y0 - t * dy).powi(2)).sqrt();
                let cov = (width / 2.0 - d + 0.5).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.over(x, y, [c.2 as f32, c.1 as f32, c.0 as f32, 1.0], alpha * cov);
                }
            }
        }
    }

    /// 圓框。
    pub(super) fn ring(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: Rgb) {
        let (x0, x1) = (((cx - r - width) as i32).max(0), ((cx + r + width) as i32 + 1).min(self.w));
        let (y0, y1) = (((cy - r - width) as i32).max(0), ((cy + r + width) as i32 + 1).min(self.h));
        for y in y0..y1 {
            for x in x0..x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (width / 2.0 - (d - r).abs() + 0.5).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.over(x, y, [c.2 as f32, c.1 as f32, c.0 as f32, 1.0], cov);
                }
            }
        }
    }
}

impl Canvas<'_> {
    /// 整塊填上不透明的顏色。
    pub(super) fn fill(&mut self, rc: RECT, c: Rgb) {
        for y in rc.top.max(0)..rc.bottom.min(self.h) {
            for x in rc.left.max(0)..rc.right.min(self.w) {
                let i = ((y * self.w + x) * 4) as usize;
                self.buf[i..i + 4].copy_from_slice(&[c.2, c.1, c.0, 255]);
            }
        }
    }

    /// 墊在已經畫好的東西「底下」的半透明底色。
    pub(super) fn under(&mut self, rc: RECT, c: Rgb, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        let tint = [c.2 as f32 * alpha, c.1 as f32 * alpha, c.0 as f32 * alpha, alpha * 255.0];
        for y in rc.top.max(0)..rc.bottom.min(self.h) {
            for x in rc.left.max(0)..rc.right.min(self.w) {
                let i = ((y * self.w + x) * 4) as usize;
                let rest = 1.0 - self.buf[i + 3] as f32 / 255.0;
                for k in 0..4 {
                    self.buf[i + k] = (self.buf[i + k] as f32 + tint[k] * rest).round().min(255.0) as u8;
                }
            }
        }
    }

    /// 分層視窗在 alpha 0 的地方滑鼠會穿透；補成 1（看不出來）讓這塊收得到滑鼠。
    pub(super) fn catch_mouse(&mut self, rc: RECT) {
        for y in rc.top.max(0)..rc.bottom.min(self.h) {
            for x in rc.left.max(0)..rc.right.min(self.w) {
                let i = ((y * self.w + x) * 4 + 3) as usize;
                if self.buf[i] == 0 {
                    self.buf[i] = 1;
                }
            }
        }
    }

    /// 圓形範圍內 alpha 補成不透明（GDI 在按鈕上畫字之後用）。
    pub(super) fn opaque_disc(&mut self, cx: f32, cy: f32, r: f32) {
        let (x0, x1) = (((cx - r) as i32).max(0), ((cx + r) as i32 + 1).min(self.w));
        let (y0, y1) = (((cy - r) as i32).max(0), ((cy + r) as i32 + 1).min(self.h));
        for y in y0..y1 {
            for x in x0..x1 {
                if (x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2) <= r * r {
                    self.buf[((y * self.w + x) * 4 + 3) as usize] = 255;
                }
            }
        }
    }

    /// 虛線外框（透明紙滑過時看得出範圍）：深色細邊＋白線，在深色、淺色桌布上都看得到。
    pub(super) fn dashed_frame(&mut self, rc: RECT, s: f32) {
        let (l, t, r, b) = (rc.left as f32 + 1.0, rc.top as f32 + 1.0, rc.right as f32 - 1.0, rc.bottom as f32 - 1.0);
        let (dash, gap) = (7.0 * s, 5.0 * s);
        for (x0, y0, x1, y1) in [(l, t, r, t), (r, t, r, b), (r, b, l, b), (l, b, l, t)] {
            let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
            let (ux, uy) = ((x1 - x0) / len.max(1.0), (y1 - y0) / len.max(1.0));
            let mut d = 0.0;
            while d < len {
                let e = (d + dash).min(len);
                let (ax, ay, bx, by) = (x0 + ux * d, y0 + uy * d, x0 + ux * e, y0 + uy * e);
                self.line(ax, ay, bx, by, 2.6 * s, Rgb(0, 0, 0), 0.22);
                self.line(ax, ay, bx, by, 1.2 * s, Rgb(255, 255, 255), 0.85);
                d += dash + gap;
            }
        }
    }
}

/// 透明紙上的字：GDI 不會寫 alpha，所以畫兩次（黑底、白底），從兩張的差算出每一點的不透明度和顏色。
unsafe fn clear_text(hdc: HDC, canvas: &mut Canvas, scene: &Scene, paper: RECT, ink: Rgb, hits: &mut Hits) {
    let (l, t, r, b) = (paper.left.max(0), paper.top.max(0), paper.right.min(canvas.w), paper.bottom.min(canvas.h));
    if r <= l || b <= t {
        return;
    }
    let soft = Rgb(225, 225, 225);
    canvas.fill(paper, Rgb(0, 0, 0));
    let mut scratch = Hits::default();
    draw_text(hdc, canvas, scene, paper, soft, ink, &mut scratch, false);
    let _ = GdiFlush();
    let row = ((r - l) * 4) as usize;
    let mut dark = Vec::with_capacity(row * (b - t) as usize);
    for y in t..b {
        let i = ((y * canvas.w + l) * 4) as usize;
        dark.extend_from_slice(&canvas.buf[i..i + row]);
    }
    canvas.fill(paper, Rgb(255, 255, 255));
    draw_text(hdc, canvas, scene, paper, soft, ink, hits, false);
    let _ = GdiFlush();
    let mut k = 0;
    for y in t..b {
        for x in l..r {
            let i = ((y * canvas.w + x) * 4) as usize;
            let d = &dark[k..k + 4];
            k += 4;
            let diff = (0..3).map(|c| canvas.buf[i + c] as f32 - d[c] as f32).sum::<f32>() / 3.0;
            let a = ((255.0 - diff).round()).clamp(0.0, 255.0) as u8;
            // 黑底上的結果就是預乘過的顏色
            canvas.buf[i] = d[0].min(a);
            canvas.buf[i + 1] = d[1].min(a);
            canvas.buf[i + 2] = d[2].min(a);
            canvas.buf[i + 3] = a;
        }
    }
}

/// 每一行字（第一列）在圖上的位置：(左, 右, 中線 y)，像素、相對整張圖（給宣傳影片的紅線對準字用）。
pub fn text_rows(note: &Note, lay: &Layout, s: f32, fonts: &Fonts) -> Vec<(i32, i32, i32)> {
    let area = text_area(note.kind, lay.paper, s, fonts.zoom);
    let row_h = (ROW_H * fonts.zoom * s).round() as i32;
    unsafe {
        let hdc = CreateCompatibleDC(None);
        SelectObject(hdc, fonts.body.into());
        let rows = note
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let w: Vec<u16> = l.text.encode_utf16().collect();
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(hdc, &w, &mut size);
                let y = area.top + i as i32 * row_h + row_h / 2 + (1.0 * s) as i32;
                (area.left - (2.0 * s) as i32, area.left + size.cx.min(area.right - area.left) + (2.0 * s) as i32, y)
            })
            .collect();
        let _ = DeleteDC(hdc);
        rows
    }
}
