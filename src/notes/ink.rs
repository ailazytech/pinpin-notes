//! 手寫：鋼筆、螢光筆、粉筆、蠟筆，寫在便利貼上（透明紙就是浮在桌面上的標語）。
//!
//! 每一筆先算出覆蓋範圍（遮罩，同一筆重疊的地方不會越疊越深），再依筆的種類加上質感：
//! 粉筆是顆粒和毛邊，蠟筆是蠟的條紋和紙紋露白。質感跟著紙張座標走，搬動便利貼不會「閃」。
//! 只在畫的時候重畫，平常不佔 CPU。

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::*;

use super::render::{noise, smooth, Canvas, Fonts, Hits, Rgb};
use super::store::{InkPrefs, Stroke, Tool};

/// 工具列（邏輯像素）：紙張下方，水平置中
const BAR_W: f32 = 290.0;
const BAR_H: f32 = 70.0;
const BAR_GAP: f32 = 10.0;

pub const TOOLS: [Tool; 4] = [Tool::Pen, Tool::Marker, Tool::Chalk, Tool::Crayon];

/// 調色盤（0xRRGGBB）：墨黑、白（粉筆、深色桌布）、紅、橘、黃、綠、藍、粉紅
pub const COLORS: [u32; 8] = [0x26221E, 0xFAFAF7, 0xE21E24, 0xF28C28, 0xF7CF3B, 0x3DAA5C, 0x2F6FDB, 0xEE78A8];

const ACCENT: Rgb = Rgb(99, 102, 241);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InkBtn {
    Tool(Tool),
    Eraser,
    Color(usize),
    Size(u8),
    Undo,
    Done,
}

/// 畫面要的手寫狀態。
pub struct InkView<'a> {
    pub prefs: InkPrefs,
    pub eraser: bool,
    /// 正在寫的那一筆
    pub cur: Option<&'a Stroke>,
}

/// 筆寬（邏輯像素）。
pub fn width_of(tool: Tool, size: u8) -> f32 {
    let table = match tool {
        Tool::Pen => [1.6, 2.6, 4.2],
        Tool::Marker => [8.0, 13.0, 20.0],
        Tool::Chalk => [4.0, 7.0, 12.0],
        Tool::Crayon => [4.0, 7.0, 11.0],
    };
    table[(size as usize).min(2)]
}

/// 工具列相對紙張左上角的範圍（邏輯像素）：(左, 上, 右, 下)。
pub fn bar_bounds(pw: f32, ph: f32) -> (f32, f32, f32, f32) {
    let left = pw / 2.0 - BAR_W / 2.0;
    (left, ph + BAR_GAP, left + BAR_W, ph + BAR_GAP + BAR_H)
}

fn rgb_of(c: u32) -> Rgb {
    Rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

// ───────────────────────────── 筆畫 ─────────────────────────────

pub fn draw_strokes(canvas: &mut Canvas, strokes: &[Stroke], cur: Option<&Stroke>, paper: RECT, s: f32) {
    for st in strokes.iter().chain(cur) {
        draw_stroke(canvas, st, paper, paper, s);
    }
}

/// 畫一筆。`origin` = 座標原點（紙張左上角），`clip` = 只畫在這範圍裡。
fn draw_stroke(canvas: &mut Canvas, st: &Stroke, origin: RECT, clip: RECT, s: f32) {
    let pts: Vec<(f32, f32)> = st
        .pts
        .chunks_exact(2)
        .map(|p| (origin.left as f32 + p[0] as f32 * 0.25 * s, origin.top as f32 + p[1] as f32 * 0.25 * s))
        .collect();
    if pts.is_empty() {
        return;
    }
    let w = st.size.max(0.5) * s;
    let n = pts.len();
    // 鋼筆：頭尾稍微細一點，像真的下筆、收筆
    let widths: Vec<f32> = (0..n)
        .map(|i| if st.tool == Tool::Pen && n > 4 { w * (0.5 + 0.5 * (i.min(n - 1 - i) as f32 / 5.0).min(1.0)) } else { w })
        .collect();
    let pad = w / 2.0 + 2.0;
    let (mut l, mut t, mut r, mut b) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in &pts {
        (l, t, r, b) = (l.min(x), t.min(y), r.max(x), b.max(y));
    }
    let l = ((l - pad).floor() as i32).max(clip.left).max(0);
    let t = ((t - pad).floor() as i32).max(clip.top).max(0);
    let r = ((r + pad).ceil() as i32).min(clip.right).min(canvas.w);
    let b = ((b + pad).ceil() as i32).min(clip.bottom).min(canvas.h);
    if r <= l || b <= t {
        return;
    }
    let (bw, bh) = (r - l, b - t);
    let mut mask = vec![0f32; (bw * bh) as usize];
    let mut segment = |(ax, ay): (f32, f32), (bx, by): (f32, f32), wa: f32, wb: f32| {
        let reach = wa.max(wb) / 2.0 + 1.0;
        let x0 = ((ax.min(bx) - reach).floor() as i32).max(l);
        let x1 = ((ax.max(bx) + reach).ceil() as i32).min(r);
        let y0 = ((ay.min(by) - reach).floor() as i32).max(t);
        let y1 = ((ay.max(by) + reach).ceil() as i32).min(b);
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        for y in y0..y1 {
            for x in x0..x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let u = if len2 > 0.0001 { (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
                let d = ((px - ax - u * dx).powi(2) + (py - ay - u * dy).powi(2)).sqrt();
                let hw = (wa + (wb - wa) * u) / 2.0;
                let cov = (hw - d + 0.5).clamp(0.0, 1.0);
                let m = &mut mask[((y - t) * bw + (x - l)) as usize];
                if cov > *m {
                    *m = cov;
                }
            }
        }
    };
    if n == 1 {
        segment(pts[0], pts[0], widths[0], widths[0]);
    } else {
        for i in 0..n - 1 {
            segment(pts[i], pts[i + 1], widths[i], widths[i + 1]);
        }
    }
    let c = rgb_of(st.color);
    let color = [c.2 as f32, c.1 as f32, c.0 as f32, 1.0];
    for y in t..b {
        for x in l..r {
            let m = mask[((y - t) * bw + (x - l)) as usize];
            if m <= 0.0 {
                continue;
            }
            // 質感用紙張座標（邏輯像素），搬動、縮放都不會變
            let (lx, ly) = ((x - origin.left) as f32 / s, (y - origin.top) as f32 / s);
            let a = match st.tool {
                Tool::Pen => m,
                Tool::Marker => m * 0.42,
                Tool::Chalk => {
                    let grain = noise((lx * 1.7) as i32, (ly * 1.7) as i32);
                    let clump = noise((lx * 0.45) as i32 + 311, (ly * 0.45) as i32 + 97);
                    let edge = smooth((m - 0.4 * clump) / 0.5);
                    edge * if grain > 0.3 + 0.2 * clump { 0.93 } else { 0.2 }
                }
                Tool::Crayon => {
                    let streak = noise((lx * 0.3) as i32 + 17, (ly * 2.2) as i32);
                    let tooth = noise((lx * 2.2) as i32 + 3, (ly * 2.2) as i32 + 41);
                    let edge = smooth((m - 0.3 * tooth) / 0.55);
                    edge * (0.62 + 0.38 * streak) * if tooth < 0.2 { 0.3 } else { 0.97 }
                }
            };
            canvas.over(x, y, color, a);
        }
    }
}

/// 筆畫有沒有碰到 (x, y) 半徑 r 內（邏輯像素，相對紙張）。
pub fn stroke_hit(st: &Stroke, x: f32, y: f32, r: f32) -> bool {
    let pts: Vec<(f32, f32)> = st.pts.chunks_exact(2).map(|p| (p[0] as f32 * 0.25, p[1] as f32 * 0.25)).collect();
    let reach = r + st.size / 2.0;
    let near = |(ax, ay): (f32, f32), (bx, by): (f32, f32)| {
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        let u = if len2 > 0.0001 { (((x - ax) * dx + (y - ay) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
        (x - ax - u * dx).powi(2) + (y - ay - u * dy).powi(2) <= reach * reach
    };
    match pts.len() {
        0 => false,
        1 => near(pts[0], pts[0]),
        _ => pts.windows(2).any(|w| near(w[0], w[1])),
    }
}

/// 存檔前把太密的點拿掉（Ramer–Douglas–Peucker，誤差 0.3 像素以內），檔案小很多、看不出差別。
pub fn simplify(pts: &[i32]) -> Vec<i32> {
    let p: Vec<(f32, f32)> = pts.chunks_exact(2).map(|c| (c[0] as f32, c[1] as f32)).collect();
    if p.len() < 3 {
        return pts.to_vec();
    }
    let eps = 1.2; // 1/4 像素單位
    let mut keep = vec![false; p.len()];
    keep[0] = true;
    keep[p.len() - 1] = true;
    let mut stack = vec![(0usize, p.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (ax, ay) = p[a];
        let (bx, by) = p[b];
        let (dx, dy) = (bx - ax, by - ay);
        let len = (dx * dx + dy * dy).sqrt();
        let mut best = (0.0f32, 0usize);
        for (i, &(x, y)) in p.iter().enumerate().take(b).skip(a + 1) {
            let d = if len < 0.001 { ((x - ax).powi(2) + (y - ay).powi(2)).sqrt() } else { ((x - ax) * dy - (y - ay) * dx).abs() / len };
            if d > best.0 {
                best = (d, i);
            }
        }
        if best.0 > eps {
            keep[best.1] = true;
            stack.push((a, best.1));
            stack.push((best.1, b));
        }
    }
    p.iter().zip(keep).filter(|(_, k)| *k).flat_map(|(&(x, y), _)| [x as i32, y as i32]).collect()
}

// ───────────────────────────── 工具列 ─────────────────────────────

/// 圓角矩形（逐點、反鋸齒）。
pub fn round_rect(canvas: &mut Canvas, rc: (f32, f32, f32, f32), radius: f32, c: Rgb, alpha: f32) {
    let (l, t, r, b) = rc;
    let radius = radius.min((r - l) / 2.0).min((b - t) / 2.0).max(0.0);
    for y in (t.floor() as i32).max(0)..(b.ceil() as i32).min(canvas.h) {
        for x in (l.floor() as i32).max(0)..(r.ceil() as i32).min(canvas.w) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let qx = (px - px.clamp(l + radius, r - radius)).abs();
            let qy = (py - py.clamp(t + radius, b - radius)).abs();
            let d = (qx * qx + qy * qy).sqrt() - radius;
            let edge = |v: f32| (0.5 - v).clamp(0.0, 1.0);
            let cov = edge(d).min(edge(l - px)).min(edge(px - r)).min(edge(t - py)).min(edge(py - b));
            if cov > 0.0 {
                canvas.over(x, y, [c.2 as f32, c.1 as f32, c.0 as f32, 1.0], alpha * cov);
            }
        }
    }
}

/// 紙張下方的工具列：上排是筆（直接畫一小段當圖示）、橡皮擦、復原、完成；下排是顏色和粗細。
pub unsafe fn draw_bar(hdc: HDC, canvas: &mut Canvas, iv: &InkView, paper: RECT, s: f32, fonts: &Fonts, hits: &mut Hits) {
    let pw = (paper.right - paper.left) as f32 / s;
    let ph = (paper.bottom - paper.top) as f32 / s;
    let (bl, bt, br, bb) = bar_bounds(pw, ph);
    let (ox, oy) = (paper.left as f32, paper.top as f32);
    let at = |x: f32, y: f32| (ox + x * s, oy + y * s);
    let (l, t) = at(bl, bt);
    let (r, b) = at(br, bb);
    let bar = RECT { left: l as i32, top: t as i32, right: r.ceil() as i32, bottom: b.ceil() as i32 };
    hits.bar = bar;
    canvas.shadow(bar, 0.0, 3.0 * s, 9.0 * s, 0.24);
    round_rect(canvas, (l, t, r, b), 14.0 * s, Rgb(252, 252, 250), 1.0);
    let rect = |x: f32, y: f32, w: f32, h: f32| {
        let (x0, y0) = at(bl + x, bt + y);
        (x0, y0, x0 + w * s, y0 + h * s)
    };
    let to_rect = |(x0, y0, x1, y1): (f32, f32, f32, f32)| RECT { left: x0 as i32, top: y0 as i32, right: x1.ceil() as i32, bottom: y1.ceil() as i32 };
    let pen = rgb_of(iv.prefs.color);
    let light_ink = (pen.0 as u32 + pen.1 as u32 + pen.2 as u32) > 640;

    // ── 上排：四種筆 ──
    for (i, tool) in TOOLS.iter().enumerate() {
        let rc = rect(8.0 + i as f32 * 44.0, 8.0, 40.0, 28.0);
        let on = !iv.eraser && iv.prefs.tool == *tool;
        // 白色的筆在白底上看不到：墊深色
        let bg = if light_ink { Rgb(70, 72, 80) } else if on { Rgb(232, 233, 252) } else { Rgb(242, 241, 238) };
        round_rect(canvas, rc, 8.0 * s, bg, 1.0);
        if on {
            canvas.line(rc.0 + 6.0 * s, rc.3 + 2.5 * s, rc.2 - 6.0 * s, rc.3 + 2.5 * s, 2.2 * s, ACCENT, 1.0);
        }
        // 一小段波浪當圖示
        let (x0, y0, x1, y1) = rc;
        let cy = (y0 + y1) / 2.0;
        let pts: Vec<i32> = (0..=12)
            .flat_map(|k| {
                let u = k as f32 / 12.0;
                let x = x0 + 7.0 * s + u * (x1 - x0 - 14.0 * s);
                let y = cy + (u * std::f32::consts::PI * 2.0).sin() * 4.0 * s;
                [((x - ox) / s * 4.0) as i32, ((y - oy) / s * 4.0) as i32]
            })
            .collect();
        let sample = Stroke { tool: *tool, color: iv.prefs.color, size: width_of(*tool, 1).min(7.0), pts };
        draw_stroke(canvas, &sample, paper, to_rect(rc), s);
        hits.ink_buttons.push((InkBtn::Tool(*tool), to_rect(rc)));
    }
    // ── 上排：橡皮擦、復原、完成（字形圖示）──
    let glyphs = [(InkBtn::Eraser, "\u{E75C}", 190.0), (InkBtn::Undo, "\u{E7A7}", 222.0), (InkBtn::Done, "\u{E8FB}", 254.0)];
    for (btn, _, x) in glyphs {
        let rc = rect(x, 8.0, 28.0, 28.0);
        let on = btn == InkBtn::Eraser && iv.eraser;
        let bg = match btn {
            InkBtn::Done => ACCENT,
            _ if on => Rgb(232, 233, 252),
            _ => Rgb(242, 241, 238),
        };
        round_rect(canvas, rc, 8.0 * s, bg, 1.0);
        if on {
            canvas.line(rc.0 + 5.0 * s, rc.3 + 2.5 * s, rc.2 - 5.0 * s, rc.3 + 2.5 * s, 2.2 * s, ACCENT, 1.0);
        }
        hits.ink_buttons.push((btn, to_rect(rc)));
    }
    // ── 下排：顏色 ──
    for (i, &c) in COLORS.iter().enumerate() {
        let (x0, y0, x1, y1) = rect(8.0 + i as f32 * 22.0, 42.0, 22.0, 22.0);
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        if !iv.eraser && iv.prefs.color == c {
            canvas.ring(cx, cy, 9.6 * s, 2.0 * s, ACCENT);
        }
        canvas.disc(cx, cy, 7.0 * s, rgb_of(c), 1.0);
        if c == 0xFAFAF7 {
            canvas.ring(cx, cy, 7.0 * s, 1.0 * s, Rgb(196, 196, 196));
        }
        hits.ink_buttons.push((InkBtn::Color(i), to_rect((x0, y0, x1, y1))));
    }
    // ── 下排：粗細 ──
    for k in 0..3u8 {
        let (x0, y0, x1, y1) = rect(196.0 + k as f32 * 30.0, 42.0, 26.0, 22.0);
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        if iv.prefs.size == k {
            canvas.ring(cx, cy, 9.6 * s, 2.0 * s, ACCENT);
        }
        canvas.disc(cx, cy, [2.2, 3.6, 5.6][k as usize] * s, Rgb(70, 66, 62), 1.0);
        hits.ink_buttons.push((InkBtn::Size(k), to_rect((x0, y0, x1, y1))));
    }
    // 字形（GDI 會把 alpha 清掉，畫完補回；工具列底是不透明的）
    SetBkMode(hdc, TRANSPARENT);
    SelectObject(hdc, fonts.icons.into());
    for (btn, glyph, x) in glyphs {
        let rc = to_rect(rect(x, 8.0, 28.0, 28.0));
        SetTextColor(hdc, if btn == InkBtn::Done { Rgb(255, 255, 255) } else { Rgb(70, 66, 62) }.colorref());
        let mut text: Vec<u16> = glyph.encode_utf16().collect();
        let mut r = rc;
        DrawTextW(hdc, &mut text, &mut r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    }
    let _ = GdiFlush();
    for (btn, _, x) in glyphs {
        let _ = btn;
        let rc = to_rect(rect(x + 3.0, 11.0, 22.0, 22.0));
        canvas.opaque(rc);
    }
}
