//! 自己做的貼紙：貼上或拖進一張圖，自動去背、加一圈白邊（像割好的貼紙），縮成小張存起來，
//! 放進貼紙小舖裡自己分的群組。
//!
//! 去背不靠 AI，用「邊緣的顏色」判斷背景：從圖的四邊往內灌水，和背景色差不多的就是背景。
//! 截圖、白底或單色底的圖效果最好；背景很花（例如照片）就不去背，整張做成圓角的照片貼紙。
//! 本來就有透明背景的 PNG 直接用它的透明度。
//!
//! 保留原圖解析度（最長邊 4096），存成 JPEG XR；原圖另存一份 `.src.jxr`，右鍵「不去背」時從它重做。

use std::collections::VecDeque;

use super::image::{self, Pixels};
use super::store::{self, StickerGroup};
use super::with;
use crate::i18n::{t, tf};

/// 處理時先縮到這麼大（夠準又快）
const WORK_MAX: u32 = 480;
/// 去背遮罩、白邊形狀在這個大小以內算，再放大套回原圖（形狀很平滑，夠準，大照片快好幾倍）
const SHAPE_MAX: u32 = 1600;
/// 存檔的最長邊：保留原圖的解析度（超大圖才縮），貼上去顯示多大另外算，不會糊掉
const OUT_MAX: u32 = 4096;
/// 存檔的品質（JPEG XR，0～1）：不縮尺寸，改壓品質省空間
const QUALITY: f32 = 0.8;
/// 白邊寬度（以 200px 的貼紙為準，大圖等比例加寬）
const BORDER: f32 = 8.0;

// ───────────────────────────── 去背 ─────────────────────────────

fn rgb_at(p: &Pixels, i: usize) -> [f32; 3] {
    [p.data[i * 4 + 2] as f32, p.data[i * 4 + 1] as f32, p.data[i * 4] as f32]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// 每一點是主體的程度（0～1）。回傳 None = 背景太花，不去背。
fn cut_mask(p: &Pixels, holes: bool, shadows: bool, photo: bool) -> Option<Vec<f32>> {
    let (w, h) = (p.w as usize, p.h as usize);
    let n = w * h;
    // 已經有透明背景
    let clear = (0..n).filter(|&i| p.data[i * 4 + 3] < 24).count();
    if clear > n / 100 {
        return Some((0..n).map(|i| p.data[i * 4 + 3] as f32 / 255.0).collect());
    }
    // 背景色 = 四邊顏色的中位數
    let border: Vec<usize> = (0..w).flat_map(|x| [x, (h - 1) * w + x]).chain((0..h).flat_map(|y| [y * w, y * w + w - 1])).collect();
    let channel = |c: usize| {
        let mut v: Vec<f32> = border.iter().map(|&i| rgb_at(p, i)[c]).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };
    let bg = [channel(0), channel(1), channel(2)];
    let mut spread: Vec<f32> = border.iter().map(|&i| dist(rgb_at(p, i), bg)).collect();
    spread.sort_by(|a, b| a.total_cmp(b));
    let tol = (spread[spread.len() / 2] * 3.0 + 20.0).clamp(22.0, 64.0);
    // 四邊大部分都不像背景色：背景很花
    if border.iter().filter(|&&i| dist(rgb_at(p, i), bg) < tol).count() < border.len() * 55 / 100 {
        return None;
    }
    // 從四邊灌水：和背景色接近，或和旁邊已經是背景的點幾乎一樣（漸層、陰影）
    let mut is_bg = vec![false; n];
    let mut queue: VecDeque<usize> = VecDeque::new();
    for &i in &border {
        if !is_bg[i] && dist(rgb_at(p, i), bg) < tol {
            is_bg[i] = true;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % w, i / w);
        let here = rgb_at(p, i);
        let mut visit = |j: usize| {
            if !is_bg[j] {
                let c = rgb_at(p, j);
                let d = dist(c, bg);
                if d < tol || (dist(c, here) < 5.0 && d < tol * 1.7) {
                    is_bg[j] = true;
                    queue.push_back(j);
                }
            }
        };
        if x > 0 {
            visit(i - 1);
        }
        if x + 1 < w {
            visit(i + 1);
        }
        if y > 0 {
            visit(i - w);
        }
        if y + 1 < h {
            visit(i + w);
        }
    }
    // 包在主體裡面、和背景同色的大塊區域（提把裡面、腳中間）：除了最大的那塊（錶面）都挖掉
    if holes {
        remove_holes(p, &mut is_bg, bg, tol);
    }
    if shadows {
        eat_shadows(p, &mut is_bg);
    }
    // 只留大塊的主體（拿掉雜點）；包在主體裡面的同色區不會被灌到，保持不透明
    let mut label = vec![0u32; n];
    let mut sizes = vec![0usize];
    for start in 0..n {
        if is_bg[start] || label[start] != 0 {
            continue;
        }
        let id = sizes.len() as u32;
        let mut count = 0;
        let mut stack = vec![start];
        label[start] = id;
        while let Some(i) = stack.pop() {
            count += 1;
            let (x, y) = (i % w, i / w);
            for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                if ok && !is_bg[j] && label[j] == 0 {
                    label[j] = id;
                    stack.push(j);
                }
            }
        }
        sizes.push(count);
    }
    let largest = sizes.iter().copied().max().unwrap_or(0);
    let total_fg: usize = sizes.iter().sum();
    if largest == 0 || total_fg < n / 50 || total_fg > n * 97 / 100 {
        return None;
    }
    let keep = (largest / 12).max(n / 500);
    if photo && looks_like_photo(p, &is_bg) {
        let solid: Vec<bool> = (0..n).map(|i| !is_bg[i] && sizes[label[i] as usize] >= keep).collect();
        return Some(smooth_mask(p, &solid, bg, tol));
    }
    let mut mask: Vec<f32> = (0..n).map(|i| if !is_bg[i] && sizes[label[i] as usize] >= keep { 1.0 } else { 0.0 }).collect();
    // 邊緣柔一點（3×3 平均，只動邊界）
    let src = mask.clone();
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let i = y * w + x;
            let mut sum = 0.0;
            for dy in 0..3 {
                for dx in 0..3 {
                    sum += src[(y + dy - 1) * w + x + dx - 1];
                }
            }
            if sum > 0.0 && sum < 9.0 {
                mask[i] = (src[i] * 0.5 + sum / 18.0).min(1.0);
            }
        }
    }
    Some(mask)
}

/// 主體看起來是照片嗎：截圖、插圖大部分相鄰的點顏色完全一樣，照片有雜訊、幾乎都不一樣。
/// 只有照片才做邊緣修整（截圖的細線、字和字中間的空隙要原樣留著）。
fn looks_like_photo(p: &Pixels, is_bg: &[bool]) -> bool {
    let w = p.w as usize;
    let (mut same, mut pairs) = (0usize, 0usize);
    for i in 0..is_bg.len().saturating_sub(1) {
        if (i + 1) % w != 0 && !is_bg[i] && !is_bg[i + 1] {
            pairs += 1;
            if p.data[i * 4..i * 4 + 3] == p.data[i * 4 + 4..i * 4 + 7] {
                same += 1;
            }
        }
    }
    pairs > 0 && same * 2 < pairs
}

/// 照片的主體邊緣修整（只動邊緣，插畫的深色外框、細線不受影響）：
/// 1. 補缺口（先長大再縮回）：主體邊上和背景顏色接近、被挖掉的小塊（深色的肉球）補回來
/// 2. 去灰邊：最外圈一小段裡「比較像背景、不像主體」的點（混到背景色的毛邊、毛縫透出來的背景）拿掉，
///    插畫的深色外框和白底差很多，不受影響；挖完留下的碎點也拿掉
/// 3. 去毛刺（縮 1px 再長回）、羽化：邊緣用到邊界的距離做出柔和的透明度
fn smooth_mask(p: &Pixels, solid: &[bool], bg: [f32; 3], tol: f32) -> Vec<f32> {
    let (w, h) = (p.w as usize, p.h as usize);
    let s = w.max(h) as f32;
    let (close, band, open, feather) = ((s * 0.025).max(2.0), (s * 0.006).max(1.5), (s * 0.001).max(0.75), (s * 0.001).max(0.6));
    let not = |m: &[bool]| m.iter().map(|&b| !b).collect::<Vec<_>>();
    let grown: Vec<bool> = distance(solid, w, h).iter().map(|&d| d > close).collect();
    let closed: Vec<bool> = distance(&grown, w, h).iter().map(|&d| d > close).collect();
    // 補回來的點不管顏色（肉球本來就像背景）；原本就在的點，邊緣上像背景的才拿掉
    let depth = distance(&not(&closed), w, h);
    // 主體的代表色：離邊緣夠遠的點的平均
    let (mut sum, mut cnt) = ([0f32; 3], 0f32);
    for i in (0..w * h).filter(|&i| depth[i] > band * 3.0) {
        let c = rgb_at(p, i);
        (sum[0], sum[1], sum[2], cnt) = (sum[0] + c[0], sum[1] + c[1], sum[2] + c[2], cnt + 1.0);
    }
    let body = if cnt > 0.0 { [sum[0] / cnt, sum[1] / cnt, sum[2] / cnt] } else { [f32::MAX; 3] };
    let like_bg = |i: usize| {
        let c = rgb_at(p, i);
        dist(c, bg) < tol * 1.6 || dist(c, bg) < dist(c, body)
    };
    let mut kept: Vec<bool> = (0..w * h).map(|i| closed[i] && !(depth[i] <= band && like_bg(i))).collect();
    // 挖完邊緣留下的碎點拿掉（只留大塊，和去背時同樣的標準）
    let mut seen = vec![false; w * h];
    let mut blobs: Vec<Vec<usize>> = Vec::new();
    for start in 0..w * h {
        if !kept[start] || seen[start] {
            continue;
        }
        let (mut blob, mut stack) = (Vec::new(), vec![start]);
        seen[start] = true;
        while let Some(i) = stack.pop() {
            blob.push(i);
            let (x, y) = (i % w, i / w);
            for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                if ok && kept[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        blobs.push(blob);
    }
    let largest = blobs.iter().map(|b| b.len()).max().unwrap_or(0);
    for b in blobs.iter().filter(|b| b.len() < (largest / 12).max(w * h / 500)) {
        for &i in b {
            kept[i] = false;
        }
    }
    let shrunk: Vec<bool> = distance(&not(&kept), w, h).iter().map(|&d| d > open).collect();
    let opened: Vec<bool> = distance(&shrunk, w, h).iter().map(|&d| d <= open).collect();
    // 有號距離：裡面為正、外面為負
    let inside = distance(&not(&opened), w, h);
    let outside = distance(&opened, w, h);
    (0..w * h)
        .map(|i| {
            let d = if opened[i] { inside[i] - 0.5 } else { 0.5 - outside[i] };
            (d / (feather * 2.0) + 0.5).clamp(0.0, 1.0)
        })
        .collect()
}

/// 包在主體裡、和背景同色的區塊：最大的留著（錶面），其他夠大的挖掉（太小的是眼睛的反光，留著）。
fn remove_holes(p: &Pixels, is_bg: &mut [bool], bg: [f32; 3], tol: f32) {
    let (w, h) = (p.w as usize, p.h as usize);
    let n = w * h;
    let like = |i: usize| dist(rgb_at(p, i), bg) < tol * 0.6;
    let mut seen = vec![false; n];
    let mut blobs: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || is_bg[start] || !like(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut blob = Vec::new();
        seen[start] = true;
        while let Some(i) = stack.pop() {
            blob.push(i);
            let (x, y) = (i % w, i / w);
            for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                if ok && !seen[j] && !is_bg[j] && like(j) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        blobs.push(blob);
    }
    let Some(largest) = blobs.iter().map(|b| b.len()).max() else { return };
    for b in blobs {
        if b.len() < largest && b.len() >= n / 800 {
            for i in b {
                is_bg[i] = true;
            }
        }
    }
}

/// 從背景往內吃掉「沒什麼顏色、又不太暗」的點：物體底下的灰色影子。有顏色的東西、深色的線條都會擋住它。
fn eat_shadows(p: &Pixels, is_bg: &mut [bool]) {
    let (w, h) = (p.w as usize, p.h as usize);
    let neutral = |i: usize| {
        let [r, g, b] = rgb_at(p, i);
        let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
        mx - mn < 16.0 && mx > 120.0
    };
    let mut stack: Vec<usize> = (0..w * h).filter(|&i| is_bg[i]).collect();
    while let Some(i) = stack.pop() {
        let (x, y) = (i % w, i / w);
        for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
            if ok && !is_bg[j] && neutral(j) {
                is_bg[j] = true;
                stack.push(j);
            }
        }
    }
}

/// 貼紙的做法。
#[derive(Clone, Copy)]
pub struct Recipe {
    /// 存檔的最長邊（2 倍大小）
    pub out_max: u32,
    /// 白邊寬度（存檔的像素）
    pub border: f32,
    /// 凹進去的地方用多大的圓弧填平（耳朵中間、手臂和身體之間），越大越圓潤
    pub round: f32,
    /// 包在主體裡、和背景同色的大塊區域也挖掉（最大的那塊留著，例如時鐘的錶面）
    pub holes: bool,
    /// 小圖最多放大幾倍（1 = 不放大，保持清楚）
    pub upscale: f32,
    /// 連著背景的灰色陰影也去掉（生圖常在物體下面畫一片柔和的影子）
    pub shadows: bool,
    /// 照片的邊緣修整：補回邊上被挖掉的小缺口、去掉毛刺和灰邊、羽化（見 `smooth_mask`）
    pub smooth: bool,
}

/// 使用者自己做的貼紙。
pub const USER: Recipe = Recipe { out_max: OUT_MAX, border: BORDER, round: 16.0, holes: false, upscale: 1.0, shadows: false, smooth: true };

/// 每一點到最近的 `inside` 點的距離（精確的歐幾里得距離，Felzenszwalb 演算法）。
fn distance(inside: &[bool], w: usize, h: usize) -> Vec<f32> {
    const INF: f32 = 1e12;
    fn pass(f: &[f32], out: &mut [f32], v: &mut [usize], z: &mut [f32]) {
        let n = f.len();
        let mut k = 0;
        v[0] = 0;
        z[0] = -INF;
        z[1] = INF;
        for q in 1..n {
            loop {
                let p = v[k];
                let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * q as f32 - 2.0 * p as f32);
                if s <= z[k] && k > 0 {
                    k -= 1;
                    continue;
                }
                if s <= z[k] {
                    // k == 0：取代
                    v[0] = q;
                    z[0] = -INF;
                    z[1] = INF;
                    break;
                }
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = INF;
                break;
            }
        }
        k = 0;
        for (q, o) in out.iter_mut().enumerate() {
            while z[k + 1] < q as f32 {
                k += 1;
            }
            let p = v[k];
            *o = (q as f32 - p as f32).powi(2) + f[p];
        }
    }
    let n = w.max(h);
    let (mut v, mut z) = (vec![0usize; n], vec![0f32; n + 1]);
    let mut grid: Vec<f32> = inside.iter().map(|&b| if b { 0.0 } else { INF }).collect();
    let (mut col, mut out) = (vec![0f32; h], vec![0f32; h]);
    for x in 0..w {
        for y in 0..h {
            col[y] = grid[y * w + x];
        }
        pass(&col, &mut out, &mut v, &mut z);
        for y in 0..h {
            grid[y * w + x] = out[y];
        }
    }
    let mut row_out = vec![0f32; w];
    for y in 0..h {
        pass(&grid[y * w..(y + 1) * w].to_vec(), &mut row_out, &mut v, &mut z);
        grid[y * w..(y + 1) * w].copy_from_slice(&row_out);
    }
    grid.into_iter().map(|d| d.sqrt()).collect()
}

/// 白邊的形狀（每一點白底的不透明度）。
/// 先往外長 border + round，再往內縮 round —— 凹進去的角被填成圓弧，凸出去的角也是圓的。
/// 大圖在縮小的格子上算，存「到邊界的距離」再放大，邊緣一樣銳利。
fn white_shape(sub: &Pixels, pad: usize, ow: usize, oh: usize, border: f32, round: f32) -> Vec<f32> {
    let k = (SHAPE_MAX as f32 / ow.max(oh) as f32).min(1.0);
    let (sw, sh) = (((ow as f32 * k).ceil() as usize).max(1), ((oh as f32 * k).ceil() as usize).max(1));
    let (kx, ky) = (ow as f32 / sw as f32, oh as f32 / sh as f32);
    let mut solid = vec![false; sw * sh];
    for y in 0..sh {
        for x in 0..sw {
            let (fx, fy) = (((x as f32 + 0.5) * kx) as i64 - pad as i64, ((y as f32 + 0.5) * ky) as i64 - pad as i64);
            if fx >= 0 && fy >= 0 && fx < sub.w as i64 && fy < sub.h as i64 {
                solid[y * sw + x] = sub.data[((fy as u32 * sub.w + fx as u32) * 4 + 3) as usize] >= 128;
            }
        }
    }
    let (border, round) = (border / kx, round / kx);
    let grow = distance(&solid, sw, sh);
    let outside: Vec<bool> = grow.iter().map(|&d| d > border + round).collect();
    // 到白邊邊界的距離（裡面為正），換回原本的像素
    let edge: Vec<f32> = distance(&outside, sw, sh).iter().map(|&d| (d - round) * kx).collect();
    let full = if sw == ow && sh == oh { edge } else { upscale(&edge, sw, sh, ow, oh) };
    full.into_iter().map(|d| (d + 0.5).clamp(0.0, 1.0)).collect()
}

/// 做成貼紙：去背、裁切、縮小、加白邊。回傳沒預乘的 BGRA。
pub fn make_sticker(src: &Pixels) -> Pixels {
    make(src, &USER)
}

/// 不去背：整張照片做成圓角貼紙（右鍵「不去背」）。
pub fn make_whole(src: &Pixels) -> Pixels {
    build(src, &USER, true)
}

pub fn make(src: &Pixels, r: &Recipe) -> Pixels {
    build(src, r, false)
}

/// 已經有透明背景（直接用它的透明度，不用再去背）
fn has_alpha(p: &Pixels) -> bool {
    let n = (p.w * p.h) as usize;
    (0..n).filter(|&i| p.data[i * 4 + 3] < 24).count() > n / 100
}

/// 放大遮罩（雙線性內插，和 `Pixels::resized` 同樣的取樣位置）
fn upscale(m: &[f32], sw: usize, sh: usize, w: usize, h: usize) -> Vec<f32> {
    let (kx, ky) = (sw as f32 / w as f32, sh as f32 / h as f32);
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        let fy = ((y as f32 + 0.5) * ky - 0.5).clamp(0.0, (sh - 1) as f32);
        let (y0, ty) = (fy as usize, fy.fract());
        let y1 = (y0 + 1).min(sh - 1);
        for x in 0..w {
            let fx = ((x as f32 + 0.5) * kx - 0.5).clamp(0.0, (sw - 1) as f32);
            let (x0, tx) = (fx as usize, fx.fract());
            let x1 = (x0 + 1).min(sw - 1);
            let top = m[y0 * sw + x0] + (m[y0 * sw + x1] - m[y0 * sw + x0]) * tx;
            let bot = m[y1 * sw + x0] + (m[y1 * sw + x1] - m[y1 * sw + x0]) * tx;
            out[y * w + x] = top + (bot - top) * ty;
        }
    }
    out
}

fn build(src: &Pixels, r: &Recipe, whole: bool) -> Pixels {
    let work = if src.w.max(src.h) > WORK_MAX.max(r.out_max * 2) {
        let k = WORK_MAX.max(r.out_max * 2) as f32 / src.w.max(src.h) as f32;
        src.resized(((src.w as f32 * k) as u32).max(1), ((src.h as f32 * k) as u32).max(1))
    } else {
        src.clone()
    };
    let (w, h) = (work.w as usize, work.h as usize);
    // 去背在縮小的圖上算（形狀很平滑，縮小不影響），再放大套回原圖：顏色、細節都還是原圖的
    let shape = if whole {
        None
    } else if work.w.max(work.h) > SHAPE_MAX && !has_alpha(&work) {
        let k = SHAPE_MAX as f32 / work.w.max(work.h) as f32;
        let small = work.resized(((work.w as f32 * k).round() as u32).max(1), ((work.h as f32 * k).round() as u32).max(1));
        cut_mask(&small, r.holes, r.shadows, r.smooth).map(|m| upscale(&m, small.w as usize, small.h as usize, w, h))
    } else {
        cut_mask(&work, r.holes, r.shadows, r.smooth)
    };
    let mask = shape.unwrap_or_else(|| {
        // 不去背：整張做成圓角的照片貼紙
        let rr = (w.min(h) as f32 * 0.08).max(2.0);
        (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
                let qx = (x - x.clamp(rr, w as f32 - rr)).abs();
                let qy = (y - y.clamp(rr, h as f32 - rr)).abs();
                (rr + 0.5 - (qx * qx + qy * qy).sqrt()).clamp(0.0, 1.0)
            })
            .collect()
    });
    // 套上遮罩、裁到主體的範圍
    let mut cut = work.clone();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for i in 0..w * h {
        let a = (cut.data[i * 4 + 3] as f32 * mask[i]).round() as u8;
        cut.data[i * 4 + 3] = a;
        if a > 12 {
            let (x, y) = (i % w, i / w);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    if x1 < x0 {
        (x0, y0, x1, y1) = (0, 0, w - 1, h - 1);
    }
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut crop = Pixels { w: cw as u32, h: ch as u32, data: vec![0; cw * ch * 4] };
    for y in 0..ch {
        let s = ((y + y0) * w + x0) * 4;
        crop.data[y * cw * 4..(y + 1) * cw * 4].copy_from_slice(&cut.data[s..s + cw * 4]);
    }
    // 縮到存檔大小（太大才縮，留白邊的空間）
    let room = r.out_max as f32 * 0.9;
    let k = (room / cw.max(ch) as f32).min(r.upscale);
    let sub = if (k - 1.0).abs() < 0.001 { crop } else { crop.resized(((cw as f32 * k).round() as u32).max(1), ((ch as f32 * k).round() as u32).max(1)) };
    // 白邊和圓弧跟著圖的大小走（以 200px 的貼紙為準）
    let unit = (sub.w.max(sub.h) as f32 / 186.0).max(0.4);
    let r = &Recipe { border: r.border * unit, round: r.round * unit, ..*r };
    // 白邊：先往外長 border + round，再往內縮 round —— 凹進去的角被填成圓弧，凸出去的角也是圓的
    let pad = (r.border + r.round + 3.0).ceil() as u32;
    let (ow, oh) = ((sub.w + pad * 2) as usize, (sub.h + pad * 2) as usize);
    let white_layer = white_shape(&sub, pad as usize, ow, oh, r.border, r.round);
    let mut out = Pixels { w: ow as u32, h: oh as u32, data: vec![0; ow * oh * 4] };
    let (mut bx0, mut by0, mut bx1, mut by1) = (ow, oh, 0, 0);
    for y in 0..oh {
        for x in 0..ow {
            let i = y * ow + x;
            let white = white_layer[i];
            // 主體疊在白底上
            let (sx, sy) = (x as i64 - pad as i64, y as i64 - pad as i64);
            let (mut c, mut a) = ([255.0f32, 255.0, 255.0], white);
            if sx >= 0 && sy >= 0 && sx < sub.w as i64 && sy < sub.h as i64 {
                let j = ((sy as u32 * sub.w + sx as u32) * 4) as usize;
                let fa = sub.data[j + 3] as f32 / 255.0;
                for k in 0..3 {
                    c[k] = sub.data[j + k] as f32 * fa + c[k] * white * (1.0 - fa);
                }
                a = fa + white * (1.0 - fa);
                if a > 0.0 {
                    for v in c.iter_mut() {
                        *v /= a;
                    }
                }
            }
            let o = i * 4;
            out.data[o] = c[0].round().min(255.0) as u8;
            out.data[o + 1] = c[1].round().min(255.0) as u8;
            out.data[o + 2] = c[2].round().min(255.0) as u8;
            out.data[o + 3] = (a * 255.0).round() as u8;
            if out.data[o + 3] > 0 {
                (bx0, by0, bx1, by1) = (bx0.min(x), by0.min(y), bx1.max(x), by1.max(y));
            }
        }
    }
    // 裁掉多留的空白（留 2px 給反鋸齒的邊）
    if bx1 < bx0 {
        return out;
    }
    let (bx0, by0) = (bx0.saturating_sub(2), by0.saturating_sub(2));
    let (bx1, by1) = ((bx1 + 2).min(ow - 1), (by1 + 2).min(oh - 1));
    let (fw, fh) = (bx1 - bx0 + 1, by1 - by0 + 1);
    let mut fin = Pixels { w: fw as u32, h: fh as u32, data: vec![0; fw * fh * 4] };
    for y in 0..fh {
        let s = ((y + by0) * ow + bx0) * 4;
        fin.data[y * fw * 4..(y + 1) * fw * 4].copy_from_slice(&out.data[s..s + fw * 4]);
    }
    fin
}

/// 圖裡最大一塊「被主體包住的白色區域」（例如時鐘的錶面）：回傳 (中心 x, 中心 y, 半徑)，像素座標。
/// 開發用：做蕃茄鐘的素材時量錶面的位置。
pub fn find_face(p: &Pixels) -> Option<(f32, f32, f32)> {
    let (w, h) = (p.w as usize, p.h as usize);
    let white = |i: usize| p.data[i * 4 + 3] > 200 && p.data[i * 4] > 235 && p.data[i * 4 + 1] > 235 && p.data[i * 4 + 2] > 235;
    let mut seen = vec![false; w * h];
    let mut best: Option<Vec<usize>> = None;
    for start in 0..w * h {
        if seen[start] || !white(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut blob = Vec::new();
        seen[start] = true;
        while let Some(i) = stack.pop() {
            blob.push(i);
            let (x, y) = (i % w, i / w);
            for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                if ok && !seen[j] && white(j) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        // 碰到圖邊的是白邊，不算
        let touches = blob.iter().any(|&i| i % w == 0 || i / w == 0 || i % w == w - 1 || i / w == h - 1);
        if !touches && best.as_ref().is_none_or(|b| blob.len() > b.len()) {
            best = Some(blob);
        }
    }
    let blob = best?;
    let n = blob.len() as f32;
    let cx = blob.iter().map(|&i| (i % w) as f32 + 0.5).sum::<f32>() / n;
    let cy = blob.iter().map(|&i| (i / w) as f32 + 0.5).sum::<f32>() / n;
    Some((cx, cy, (n / std::f32::consts::PI).sqrt()))
}

// ───────────────────────────── 群組 ─────────────────────────────

fn new_group_id() -> String {
    format!("g{}", store::now_ms())
}

/// 第一個群組（沒有就建一個「我的貼紙」）。
fn ensure_group() -> Option<String> {
    with(|m| {
        if m.board.sticker_groups.is_empty() {
            m.board.sticker_groups.push(StickerGroup { id: new_group_id(), name: t("shop.default_group"), items: vec![] });
        }
        m.board.sticker_groups[0].id.clone()
    })
}

pub fn groups() -> Vec<StickerGroup> {
    with(|m| m.board.sticker_groups.clone()).unwrap_or_default()
}

/// 新增一個群組，回傳代碼。
pub fn add_group() -> Option<String> {
    with(|m| {
        let n = m.board.sticker_groups.len() + 1;
        let id = new_group_id();
        m.board.sticker_groups.push(StickerGroup { id: id.clone(), name: tf("shop.group_name", &[("n", &n.to_string())]), items: vec![] });
        store::save(&m.board);
        id
    })
}

pub fn rename_group(id: &str, name: &str) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    with(|m| {
        if let Some(g) = m.board.sticker_groups.iter_mut().find(|g| g.id == id) {
            g.name = name.chars().take(40).collect();
        }
        store::save(&m.board);
    });
}

/// 刪掉群組；裡面自己做的貼紙沒有其他地方用到的話一起刪。
pub fn delete_group(id: &str) {
    let items = with(|m| {
        let i = m.board.sticker_groups.iter().position(|g| g.id == id)?;
        let g = m.board.sticker_groups.remove(i);
        store::save(&m.board);
        Some(g.items)
    })
    .flatten()
    .unwrap_or_default();
    for a in items {
        collect_garbage(&a);
    }
}

/// 群組往左、往右移。
pub fn move_group(id: &str, delta: i32) {
    with(|m| {
        let list = &mut m.board.sticker_groups;
        if let Some(i) = list.iter().position(|g| g.id == id) {
            let j = (i as i32 + delta).clamp(0, list.len() as i32 - 1) as usize;
            let g = list.remove(i);
            list.insert(j, g);
        }
        store::save(&m.board);
    });
}

/// 收進群組（已經在裡面就不重複）。
pub fn add_to_group(group: &str, asset: &str) {
    with(|m| {
        if let Some(g) = m.board.sticker_groups.iter_mut().find(|g| g.id == group) {
            if !g.items.iter().any(|a| a == asset) {
                g.items.push(asset.to_string());
            }
        }
        store::save(&m.board);
    });
}

pub fn remove_from_group(group: &str, asset: &str) {
    with(|m| {
        if let Some(g) = m.board.sticker_groups.iter_mut().find(|g| g.id == group) {
            g.items.retain(|a| a != asset);
        }
        store::save(&m.board);
    });
    collect_garbage(asset);
}

/// 從所有群組拿掉並刪檔（便利貼上還貼著的話，檔案留著）。
pub fn delete_sticker(asset: &str) {
    with(|m| {
        for g in &mut m.board.sticker_groups {
            g.items.retain(|a| a != asset);
        }
        store::save(&m.board);
    });
    collect_garbage(asset);
}

/// 自己做的貼紙：沒有任何群組、便利貼（含回收的）用到，就把檔案刪掉。
fn collect_garbage(asset: &str) {
    if !super::assets::is_custom(asset) {
        return;
    }
    let used = with(|m| {
        m.board.sticker_groups.iter().any(|g| g.items.iter().any(|a| a == asset))
            || m.board.notes.iter().chain(&m.board.trash).any(|n| n.items.iter().any(|a| a.asset == asset))
    })
    .unwrap_or(true);
    if !used {
        let _ = std::fs::remove_file(file_of(asset));
        let _ = std::fs::remove_file(source_of(asset));
        with(|m| {
            m.board.whole_stickers.retain(|a| a != asset);
            m.assets.forget(asset);
        });
    }
}

/// 把幾張圖做成貼紙，收進群組（None = 第一個群組）。回傳收進去的群組和張數。
pub fn add_images(group: Option<String>, images: Vec<Pixels>) -> Option<(String, usize)> {
    let group = match group {
        Some(g) if groups().iter().any(|x| x.id == g) => g,
        _ => ensure_group()?,
    };
    let mut added = 0;
    for (i, img) in images.into_iter().enumerate() {
        let sticker = make_sticker(&img);
        let id = format!("my-{}{}", store::now_ms(), i % 100);
        // 原圖留一份，之後可以改成「不去背」或改回來（太大的縮到 4096）
        let big = img.w.max(img.h);
        let src = if big > OUT_MAX { img.resized(img.w * OUT_MAX / big, img.h * OUT_MAX / big) } else { img };
        if let Err(e) = image::save_jxr(&src, &source_of(&id), QUALITY) {
            crate::base::log_error(format!("儲存貼紙原圖失敗：{e:#}"));
        }
        let path = store::sticker_dir().join(format!("{id}.jxr"));
        match image::save_jxr(&sticker, &path, QUALITY) {
            Ok(()) => {
                add_to_group(&group, &id);
                added += 1;
            }
            Err(e) => crate::base::log_error(format!("儲存貼紙失敗：{e:#}")),
        }
    }
    with(|m| store::save(&m.board));
    (added > 0).then_some((group, added))
}

/// 做貼紙用的原圖（舊版做的沒有）。
fn source_of(id: &str) -> std::path::PathBuf {
    store::sticker_dir().join(format!("{id}.src.jxr"))
}

/// 有原圖，可以切換去不去背。
pub fn has_source(id: &str) -> bool {
    super::assets::is_custom(id) && source_of(id).exists()
}

/// 選了「不去背（整張照片）」。
pub fn is_whole(id: &str) -> bool {
    with(|m| m.board.whole_stickers.iter().any(|a| a == id)).unwrap_or(false)
}

/// 從原圖重做：whole = 整張照片（圓角），否則自動去背。便利貼上已經貼著的也跟著換。
pub fn set_whole(id: &str, whole: bool) -> bool {
    let made = image::decode_file(&source_of(id)).and_then(|src| {
        let s = if whole { make_whole(&src) } else { make_sticker(&src) };
        image::save_jxr(&s, &store::sticker_dir().join(format!("{id}.jxr")), QUALITY)
    });
    if let Err(e) = made {
        crate::base::log_error(format!("重做貼紙失敗：{e:#}"));
        return false;
    }
    let _ = std::fs::remove_file(store::sticker_dir().join(format!("{id}.png")));
    with(|m| {
        m.board.whole_stickers.retain(|a| a != id);
        if whole {
            m.board.whole_stickers.push(id.to_string());
        }
        store::save(&m.board);
        m.assets.forget(id);
    });
    true
}

/// 自己做的貼紙存在哪個檔（新的是 .jxr，舊版做的是 .png）。
pub fn file_of(id: &str) -> std::path::PathBuf {
    let jxr = store::sticker_dir().join(format!("{id}.jxr"));
    if jxr.exists() {
        jxr
    } else {
        store::sticker_dir().join(format!("{id}.png"))
    }
}

/// 匯入時依檔案內容決定副檔名（JPEG XR 開頭是 "II\xBC"）。
pub fn file_for_bytes(id: &str, bytes: &[u8]) -> std::path::PathBuf {
    let ext = if bytes.starts_with(b"II\xBC") { "jxr" } else { "png" };
    store::sticker_dir().join(format!("{id}.{ext}"))
}

/// 自己做的貼紙檔名是不是安全（匯入用：只能是 my- 開頭的單純代碼）。
pub fn valid_id(id: &str) -> bool {
    super::assets::is_custom(id) && id.len() < 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, bg: [u8; 3]) -> Pixels {
        let mut p = Pixels { w, h, data: vec![255; (w * h * 4) as usize] };
        for i in 0..(w * h) as usize {
            p.data[i * 4..i * 4 + 3].copy_from_slice(&[bg[2], bg[1], bg[0]]);
        }
        p
    }

    #[test]
    fn white_background_is_removed_and_bordered() {
        // 白底中間一個紅色方塊
        let mut p = solid(100, 80, [255, 255, 255]);
        for y in 20..60 {
            for x in 30..70 {
                let i = ((y * 100 + x) * 4) as usize;
                p.data[i..i + 3].copy_from_slice(&[40, 40, 220]);
            }
        }
        let s = make_sticker(&p);
        assert!(s.w.max(s.h) <= OUT_MAX);
        // 角落透明、中間是紅色、紅色外圍一圈白
        assert_eq!(s.data[3], 0);
        let center = (((s.h / 2) * s.w + s.w / 2) * 4) as usize;
        assert!(s.data[center + 2] > 180 && s.data[center + 1] < 80 && s.data[center + 3] == 255);
        // 中間那一列：從左邊進來先碰到白邊，再碰到紅色
        let row = (s.h / 2 * s.w * 4) as usize;
        let px = |x: u32| &s.data[row + x as usize * 4..row + x as usize * 4 + 4];
        let first = (0..s.w).find(|&x| px(x)[3] > 200).expect("有東西");
        assert!(px(first)[0] > 240 && px(first)[1] > 240, "白邊");
        // 解析度保留：紅色方塊還是 40px 寬（沒有被縮小）
        let red = (0..s.w).filter(|&x| px(x)[2] > 180 && px(x)[1] < 80).count();
        assert!((38..=41).contains(&red), "紅色寬 {red}");
    }

    #[test]
    fn flat_graphics_skip_photo_smoothing() {
        // 截圖、插圖：細線和字中間的空隙要原樣留著，和不修整的結果一模一樣
        let mut p = solid(300, 200, [40, 110, 210]);
        let mut paint = |x0: u32, x1: u32, y0: u32, y1: u32, c: [u8; 3]| {
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * 300 + x) * 4) as usize;
                    p.data[i..i + 3].copy_from_slice(&c);
                }
            }
        };
        paint(60, 120, 50, 150, [60, 210, 250]);
        paint(128, 188, 50, 150, [60, 210, 250]);
        paint(188, 260, 99, 101, [20, 20, 20]);
        let raw = make(&p, &Recipe { smooth: false, ..USER });
        let s = make_sticker(&p);
        assert_eq!((s.w, s.h), (raw.w, raw.h));
        assert!(s.data == raw.data);
    }

    #[test]
    fn whole_keeps_the_entire_picture() {
        // 白底中間一個方塊：自動去背只剩方塊，不去背整張都在（圓角、外面一圈白邊）
        let mut p = solid(200, 100, [255, 255, 255]);
        for y in 30..70 {
            for x in 80..120 {
                let i = ((y * 200 + x) * 4) as usize;
                p.data[i..i + 3].copy_from_slice(&[40, 40, 220]);
            }
        }
        let cut = make_sticker(&p);
        let whole = make_whole(&p);
        assert!(cut.w < 120, "去背 {}", cut.w);
        assert!(whole.w > 200 && whole.h > 100 && whole.w - 200 < 40, "不去背 {}x{}", whole.w, whole.h);
        assert_eq!(whole.data[3], 0, "角落是圓的");
        let center = (((whole.h / 2) * whole.w + whole.w / 2) * 4) as usize;
        assert_eq!(whole.data[center + 3], 255);
    }

    #[test]
    fn big_pictures_match_full_size_shape() {
        // 大圖的去背、白邊在縮小的格子上算：和原尺寸算的形狀差不多（邊緣差不到幾個像素）
        let (w, h) = (3200u32, 2000u32);
        let mut p = solid(w, h, [250, 250, 250]);
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = (x as f32 - 1600.0, y as f32 - 1000.0);
                if dx * dx / (900.0 * 900.0) + dy * dy / (600.0 * 600.0) < 1.0 {
                    let i = ((y * w + x) * 4) as usize;
                    p.data[i..i + 3].copy_from_slice(&[30, 140, 60]);
                }
            }
        }
        let s = make_sticker(&p);
        // 橢圓 1800×1200，加上兩邊白邊
        assert!(s.w > 1800 && s.h > 1200, "大小 {}x{}", s.w, s.h);
        let row = (s.h / 2 * s.w * 4) as usize;
        let px = |x: u32| &s.data[row + x as usize * 4..row + x as usize * 4 + 4];
        let green = (0..s.w).filter(|&x| px(x)[1] > 120 && px(x)[0] < 90).count() as i32;
        assert!((green - 1800).abs() <= 6, "綠色寬 {green}");
        let white = (0..s.w).take_while(|&x| px(x)[3] < 250 || px(x)[1] > 240).count() as i32;
        let border = (s.w as i32 - green) / 2;
        assert!((white - border).abs() <= 3, "白邊 {white} vs {border}");
    }

    #[test]
    fn noisy_images_look_like_photos() {
        let mut p = solid(64, 64, [200, 200, 200]);
        let flat = vec![false; 64 * 64];
        assert!(!looks_like_photo(&p, &flat));
        for (i, v) in p.data.chunks_mut(4).enumerate() {
            let n = ((i * 7919) % 23) as u8;
            v[0] = 190 + n;
            v[1] = 195 + n / 2;
        }
        assert!(looks_like_photo(&p, &flat));
    }

    #[test]
    fn jxr_keeps_size_and_alpha() {
        let mut p = solid(300, 200, [255, 255, 255]);
        for y in 50..150 {
            for x in 80..220 {
                let i = ((y * 300 + x) * 4) as usize;
                p.data[i..i + 3].copy_from_slice(&[40, 160, 220]);
            }
        }
        let s = make_sticker(&p);
        let dir = std::env::temp_dir().join(format!("pinpin-jxr-{}", std::process::id()));
        let (jxr, png) = (dir.join("a.jxr"), dir.join("a.png"));
        image::save_jxr(&s, &jxr, QUALITY).expect("存 jxr");
        image::save_png(&s, &png).expect("存 png");
        let bytes = std::fs::read(&jxr).unwrap();
        assert!(bytes.starts_with(b"II\xBC"), "JPEG XR 檔頭");
        let back = image::decode_file(&jxr).expect("讀 jxr");
        assert_eq!((back.w, back.h), (s.w, s.h));
        assert_eq!(back.data[3], 0, "角落還是透明");
        let center = (((s.h / 2) * s.w + s.w / 2) * 4) as usize;
        assert!(back.data[center + 3] > 250 && (back.data[center + 2] as i32 - 220).abs() < 16, "中間顏色差不多");
        eprintln!("jxr {} bytes, png {} bytes", bytes.len(), std::fs::metadata(&png).unwrap().len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn busy_background_becomes_photo_sticker() {
        // 雜訊背景：不去背，整張保留
        let mut p = solid(60, 60, [0, 0, 0]);
        for i in 0..3600usize {
            let v = (i * 7919 % 251) as u8;
            p.data[i * 4..i * 4 + 3].copy_from_slice(&[v, v.wrapping_mul(3), v.wrapping_mul(7)]);
        }
        assert!(cut_mask(&p, false, false, false).is_none());
        let s = make_sticker(&p);
        let center = (((s.h / 2) * s.w + s.w / 2) * 4) as usize;
        assert_eq!(s.data[center + 3], 255);
    }
}
