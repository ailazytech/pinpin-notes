//! 貼紙小舖：像商店一樣一格格擺出圖釘、夾子、紙膠帶、貼紙，拖出來貼到便利貼或桌面上。
//!
//! 除了內建的分類，還可以自己開群組：把圖片拖進來、Ctrl+V 或點「＋」挑檔案，會自動去背加白邊做成貼紙；
//! 喜歡的內建貼紙也可以右鍵收進群組。群組可以改名（雙擊）、排順序、刪除（右鍵）。
//!
//! 一般視窗（跟著系統深淺色的自訂標題列），開著時才建立縮圖，關掉就全部釋放。
//! 拖曳時游標旁邊跟著一個半透明的「影子」（分層視窗、滑鼠穿透），放開時看底下是哪張便利貼。

use std::cell::RefCell;
use std::collections::HashMap;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Controls::EM_SETSEL;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Shell::{DragAcceptFiles, DragFinish, HDROP};
use windows::Win32::UI::WindowsAndMessaging::*;

use super::assets::{self, CATS};
use super::{image, render, stickers, with};
use crate::i18n::{is_rtl, t, tf};
use crate::win::{self, wide, MenuItem};
use crate::theme;

const CLASS: PCWSTR = w!("PinPin.StickerShop");
const GHOST_CLASS: PCWSTR = w!("PinPin.Ghost");
const WM_MOUSELEAVE: u32 = 0x02A3;
const CARD_W: f32 = 118.0;
const CARD_H: f32 = 132.0;
const THUMB_H: f32 = 92.0;
const GAP: f32 = 12.0;
const PAD: f32 = 18.0;
const BANNER_H: f32 = 100.0;
/// 廣告區輪播：計時器代號、幾毫秒換一則、一共幾則
const AD_TIMER: usize = 7;
const AD_MS: u32 = 6000;
/// 換廣告時的滑動動畫
const SLIDE_TIMER: usize = 8;
const SLIDE_MS: f32 = 360.0;
const TAB_H: f32 = 30.0;
/// 貼紙子分類的標籤高度
const SUB_H: f32 = 26.0;

#[derive(Clone, PartialEq, Eq, Debug)]
enum Tab {
    /// 內建分類（CATS 的第幾個）；貼紙（第 0 個）實際上一定是某個 Sub
    Cat(usize),
    /// 貼紙的子分類（assets::SUBS 的第幾個）
    Sub(usize),
    /// 自己的群組（代碼）
    Group(String),
    /// 「＋ 新群組」
    NewGroup,
}

#[derive(Clone)]
enum Card {
    /// 「＋ 加入圖片」
    Add,
    /// 「貼上剪貼簿的圖片」
    Paste,
    /// 「訂製」：開官網的訂製頁
    Custom,
    /// 「專屬春聯訂製・全球寄送」：開官網的接單頁（只在春聯分類）
    Order,
    Asset(String),
}

struct Drag {
    asset: String,
    start: POINT,
    ghost: Option<(HWND, i32, i32)>,
}

struct Shop {
    hwnd: HWND,
    tab: Tab,
    scroll: i32,
    hover: Option<usize>,
    close_hot: bool,
    thumbs: HashMap<String, HBITMAP>,
    drag: Option<Drag>,
    font: HFONT,
    bold: HFONT,
    small: HFONT,
    icon: HICON,
    /// 正在改名的群組：(文字框, 群組代碼)
    rename: Option<(HWND, String)>,
    /// 廣告區現在是第幾則
    ad: usize,
    /// 正在滑動：(從第幾則, 開始時間毫秒)
    slide: Option<(usize, u64)>,
}

thread_local! {
    static SHOP: RefCell<Option<Shop>> = const { RefCell::new(None) };
}

fn px(v: f32) -> i32 {
    theme::px(v)
}

fn get<R>(f: impl FnOnce(&Shop) -> R) -> Option<R> {
    SHOP.with(|s| s.try_borrow().ok().and_then(|s| s.as_ref().map(f)))
}

fn set(f: impl FnOnce(&mut Shop)) {
    SHOP.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            if let Some(s) = s.as_mut() {
                f(s);
            }
        }
    });
}

fn invalidate() {
    if let Some(h) = get(|s| s.hwnd) {
        unsafe {
            let _ = InvalidateRect(Some(h), None, false);
        }
    }
}

pub fn register() {
    unsafe {
        let wc = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(proc),
            hInstance: win::hinstance(),
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(ghost_proc),
            hInstance: win::hinstance(),
            lpszClassName: GHOST_CLASS,
            ..Default::default()
        };
        RegisterClassW(&wc);
    }
}

/// 打開貼紙小舖（已經開著就叫到前面）。
pub fn open() {
    if let Some(h) = get(|s| s.hwnd) {
        unsafe {
            let _ = ShowWindow(h, SW_RESTORE);
            let _ = SetForegroundWindow(h);
        }
        return;
    }
    if let Some(hwnd) = create() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            // 只有小舖開著才輪播，容許系統晚 1 秒順便叫醒
            SetCoalescableTimer(Some(hwnd), AD_TIMER, AD_MS, None, 1000);
        }
        // 打開小舖時順便看看有沒有新廣告、新貼紙（距離上次超過一天才查）
        let (auto, push) = crate::app::update_flags();
        if auto {
            crate::update::maybe_check(push);
        }
    }
}

/// 廣告區的範圍。
fn banner_rect(client_w: i32) -> RECT {
    RECT { left: px(PAD), top: px(theme::TITLE_H) + px(4.0), right: client_w - px(PAD), bottom: px(theme::TITLE_H + BANNER_H) }
}

/// 廣告區右下角的小圓點（點了直接跳到那一則）。
fn ad_dots(banner: &RECT) -> Vec<RECT> {
    let n = ad_count() as i32;
    let (r, gap) = (px(4.0), px(14.0));
    let cy = banner.bottom - px(12.0);
    let right = if is_rtl() { banner.left + px(14.0) + gap * (n - 1) } else { banner.right - px(14.0) };
    (0..n)
        .map(|i| {
            let cx = right - gap * (n - 1 - i);
            RECT { left: cx - r - px(4.0), top: cy - r - px(4.0), right: cx + r + px(4.0), bottom: cy + r + px(4.0) }
        })
        .collect()
}

/// 一則廣告：底圖＋文字（文字由程式畫，才能跟著語言換）。
#[derive(Clone)]
struct AdSlide {
    key: String,
    title: String,
    sub: String,
    image: AdImage,
    /// `site:#/order`、`tab:sub.spring`、`https://…`
    link: String,
}

#[derive(Clone)]
enum AdImage {
    Bundled(&'static [u8]),
    File(std::path::PathBuf),
}

thread_local! {
    /// 目前輪播的廣告（換語言、線上內容更新時清掉重算）
    static ADS: std::cell::RefCell<Option<Vec<AdSlide>>> = const { std::cell::RefCell::new(None) };
    /// 縮好的底圖：(代碼, 寬, 高) → 圖
    static AD_IMAGES: std::cell::RefCell<HashMap<(String, i32, i32), Option<super::image::Pixels>>> = std::cell::RefCell::new(HashMap::new());
}

/// 現在輪播的廣告：有線上廣告（而且使用者接收推送）就用線上的，否則用內建的三則。
fn ads() -> Vec<AdSlide> {
    if let Some(list) = ADS.with(|a| a.borrow().clone()) {
        return list;
    }
    let push = crate::app::update_flags().1;
    let remote: Vec<AdSlide> = crate::update::active_ads(push)
        .into_iter()
        .map(|(ad, path)| AdSlide {
            key: format!("remote-{}", ad.id),
            title: crate::i18n::pick(&ad.title),
            sub: crate::i18n::pick(&ad.sub),
            image: AdImage::File(path),
            link: ad.link,
        })
        .collect();
    let list = if remote.is_empty() {
        vec![
            AdSlide { key: "custom".into(), title: t("shop.ad_custom"), sub: t("shop.ad_custom_sub"), image: AdImage::Bundled(include_bytes!("../../assets/ads/custom.jxr")), link: "site:#/custom".into() },
            AdSlide { key: "new".into(), title: t("shop.ad_new"), sub: t("shop.ad_new_sub"), image: AdImage::Bundled(include_bytes!("../../assets/ads/new.jxr")), link: "tab:sub.taiwan".into() },
            AdSlide { key: "donate".into(), title: t("shop.ad_donate"), sub: t("shop.ad_donate_sub"), image: AdImage::Bundled(include_bytes!("../../assets/ads/donate.jxr")), link: "site:#/donate".into() },
        ]
    } else {
        remote
    };
    ADS.with(|a| *a.borrow_mut() = Some(list.clone()));
    list
}

fn ad_count() -> usize {
    ads().len().max(1)
}

/// 線上內容換了（廣告、貼紙包）或語言換了：重算廣告、縮圖，小舖重畫。
pub fn refresh_remote() {
    ADS.with(|a| *a.borrow_mut() = None);
    AD_IMAGES.with(|m| m.borrow_mut().clear());
    set(|s| {
        s.ad = 0;
        s.slide = None;
        for (_, b) in s.thumbs.drain() {
            unsafe {
                let _ = DeleteObject(b.into());
            }
        }
    });
    invalidate();
}

/// 廣告底圖縮到 w×h（cover：填滿、多的從中間裁掉）。
fn ad_image(slide: &AdSlide, w: i32, h: i32) -> Option<super::image::Pixels> {
    let key = (slide.key.clone(), w, h);
    if let Some(hit) = AD_IMAGES.with(|m| m.borrow().get(&key).cloned()) {
        return hit;
    }
    let decoded = match &slide.image {
        AdImage::Bundled(b) => super::image::decode_bytes(b),
        AdImage::File(p) => super::image::decode_file(p),
    };
    let out = decoded.ok().map(|src| {
        let k = (w as f32 / src.w as f32).max(h as f32 / src.h as f32);
        let (sw, sh) = (((src.w as f32 * k).ceil() as u32).max(w as u32), ((src.h as f32 * k).ceil() as u32).max(h as u32));
        let big = src.resized(sw, sh);
        let (ox, oy) = ((sw as i32 - w) / 2, (sh as i32 - h) / 2);
        let mut data = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            let s = (((y + oy) as u32 * sw + ox as u32) * 4) as usize;
            data[(y * w * 4) as usize..((y + 1) * w * 4) as usize].copy_from_slice(&big.data[s..s + (w * 4) as usize]);
        }
        super::image::Pixels { w: w as u32, h: h as u32, data }
    });
    AD_IMAGES.with(|m| m.borrow_mut().insert(key, out.clone()));
    out
}

/// 換到第 i 則廣告（往右滑進來）。
fn go_ad(hwnd: HWND, i: usize) {
    let n = ad_count();
    let from = get(|s| s.ad).unwrap_or(0);
    if from == i % n {
        return;
    }
    set(|s| {
        s.ad = i % n;
        s.slide = Some((from, store_now()));
    });
    unsafe {
        SetTimer(Some(hwnd), SLIDE_TIMER, 15, None);
    }
}

fn store_now() -> u64 {
    super::store::now_ms()
}

/// 點了廣告。
fn ad_click(i: usize) {
    let Some(slide) = ads().get(i).cloned() else { return };
    if let Some(key) = slide.link.strip_prefix("tab:") {
        if let Some(i) = (0..assets::sub_count()).position(|i| assets::sub_key(i) == key) {
            switch_to(Tab::Sub(i));
        }
    } else if let Some(path) = slide.link.strip_prefix("site:") {
        win::open_url(&crate::base::site_page(path));
    } else if slide.link.starts_with("https://") {
        win::open_url(&slide.link);
    }
}

/// 開發用：不顯示視窗，把某個分類（0 起算，超過就是第一個自己的群組）畫成 PNG。
pub fn render_png(path: &str, tab: usize) {
    // PINPIN_SHOP_ADD=圖檔：先做成貼紙收進第一個群組；PINPIN_SHOP_WHOLE=1：再切成「不去背」（測試用）
    if let Some(img) = std::env::var("PINPIN_SHOP_ADD").ok().and_then(|f| image::decode_file(std::path::Path::new(&f)).ok()) {
        if let Some((group, _)) = stickers::add_images(None, vec![img]) {
            let id = stickers::groups().into_iter().find(|g| g.id == group).and_then(|g| g.items.last().cloned());
            if let (Some(id), Ok(_)) = (id, std::env::var("PINPIN_SHOP_WHOLE")) {
                println!("whole {id}: {}", stickers::set_whole(&id, true));
            }
        }
    }
    let Some(hwnd) = create() else { return };
    let tab = if tab >= 100 {
        Tab::Sub(tab - 100)
    } else if tab == 0 {
        Tab::Sub(0)
    } else if tab < CATS.len() {
        Tab::Cat(tab)
    } else { stickers::groups().first().map_or(Tab::Cat(0), |g| Tab::Group(g.id.clone())) };
    // PINPIN_SHOP_HOVER=第幾張：預覽滑過卡片的樣子（浮出完整名稱）
    let hover = std::env::var("PINPIN_SHOP_HOVER").ok().and_then(|v| v.parse().ok());
    let ad = std::env::var("PINPIN_SHOP_AD").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    // PINPIN_SHOP_SCROLL=像素：預覽下面的卡片
    let scroll = std::env::var("PINPIN_SHOP_SCROLL").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    set(|s| {
        s.tab = tab;
        s.hover = hover;
        s.scroll = scroll;
        s.ad = ad;
        // PINPIN_SHOP_SLIDE=進度（0～1）：預覽滑到一半的樣子（從上一則滑過來）
        if let Some(u) = std::env::var("PINPIN_SHOP_SLIDE").ok().and_then(|v| v.parse::<f32>().ok()) {
            let u = u.clamp(0.0, 0.999);
            // 反推出開始時間，讓 1 - (1-t)^3 = u
            let t = 1.0 - (1.0 - u).cbrt();
            s.slide = Some(((ad + ad_count() - 1) % ad_count(), store_now().saturating_sub((t * SLIDE_MS) as u64)));
        }
    });
    unsafe {
        let rc = client(hwnd);
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, rc.right, rc.bottom);
        let old = SelectObject(mem, bmp.into());
        draw(hwnd, mem);
        let mut bmi = BITMAPINFO { bmiHeader: BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: rc.right, biHeight: -rc.bottom, biPlanes: 1, biBitCount: 32, ..Default::default() }, ..Default::default() };
        let mut data = vec![0u8; (rc.right * rc.bottom * 4) as usize];
        SelectObject(mem, old);
        GetDIBits(mem, bmp, 0, rc.bottom as u32, Some(data.as_mut_ptr().cast()), &mut bmi, DIB_RGB_COLORS);
        for px in data.chunks_mut(4) {
            px[3] = 255;
        }
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        let pixels = super::image::Pixels { w: rc.right as u32, h: rc.bottom as u32, data };
        if let Err(e) = super::image::save_png(&pixels, std::path::Path::new(path)) {
            eprintln!("{e:#}");
        }
        let _ = DestroyWindow(hwnd);
    }
}

fn create() -> Option<HWND> {
    unsafe {
        let wa = render::work_area_at(win::cursor_pos());
        let (w, h) = (px(PAD * 2.0 + CARD_W * 4.0 + GAP * 3.0 + 16.0), px(620.0));
        let x = wa.left + (wa.right - wa.left - w) / 2;
        let y = wa.top + (wa.bottom - wa.top - h) / 2;
        let Ok(hwnd) = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_ACCEPTFILES, CLASS, w!("PinPin Notes"), WS_POPUP | WS_THICKFRAME | WS_CLIPCHILDREN, x, y, w, h, None, None, Some(win::hinstance()), None) else {
            return None;
        };
        DragAcceptFiles(hwnd, true);
        theme::apply_window(hwnd);
        theme::setup_chrome(hwnd);
        let (font, bold) = theme::fonts(13.0);
        let (small, _) = theme::fonts(11.0);
        let icon = crate::app::small_icon();
        SHOP.with(|s| {
            *s.borrow_mut() = Some(Shop {
                hwnd,
                tab: Tab::Sub(0),
                scroll: 0,
                hover: None,
                close_hot: false,
                thumbs: HashMap::new(),
                drag: None,
                font,
                bold,
                small,
                icon,
                rename: None,
                ad: 0,
                slide: None,
            })
        });
        Some(hwnd)
    }
}

// ───────────────────────────── 版面 ─────────────────────────────

/// 所有分類標籤和上面的字。
fn tabs() -> Vec<(Tab, String)> {
    let mut list: Vec<(Tab, String)> = CATS
        .iter()
        .enumerate()
        .map(|(i, (cat, name))| (Tab::Cat(i), format!("{}  {}", t(name), assets::all().iter().filter(|a| a.cat == *cat).count())))
        .collect();
    for g in stickers::groups() {
        list.push((Tab::Group(g.id.clone()), format!("{}  {}", g.name, g.items.len())));
    }
    list.push((Tab::NewGroup, format!("＋ {}", t("shop.new_group"))));
    if matches!(get(|s| s.tab.clone()), Some(Tab::Sub(_))) {
        let all = assets::all();
        for i in 0..assets::sub_count() {
            let n = all.iter().filter(|a| a.cat == assets::Cat::Sticker && a.sub == i).count();
            list.push((Tab::Sub(i), format!("{}  {n}", assets::sub_name(i))));
        }
    }
    list
}

/// 這個分類的卡片。
fn cards(tab: &Tab) -> Vec<Card> {
    match tab {
        Tab::Cat(i) => {
            let cat = CATS[(*i).min(CATS.len() - 1)].0;
            assets::all().into_iter().filter(|a| a.cat == cat).map(|a| Card::Asset(a.id.to_string())).chain([Card::Custom]).collect()
        }
        Tab::Sub(i) => assets::all()
            .into_iter()
            .filter(|a| a.cat == assets::Cat::Sticker && a.sub == *i)
            .map(|a| Card::Asset(a.id.to_string()))
            .chain(if assets::sub_key(*i) == "sub.spring" { vec![Card::Order, Card::Custom] } else { vec![Card::Custom] })
            .collect(),
        Tab::Group(id) => {
            let items = stickers::groups().into_iter().find(|g| g.id == *id).map(|g| g.items).unwrap_or_default();
            [Card::Add, Card::Paste].into_iter().chain(items.into_iter().map(Card::Asset)).chain([Card::Custom]).collect()
        }
        Tab::NewGroup => vec![],
    }
}

/// 標籤的位置（放不下就換行）和卡片區從哪裡開始。
fn tab_layout(hwnd: HWND) -> (Vec<(Tab, RECT)>, i32) {
    let Some(font) = get(|s| s.font) else { return (vec![], 0) };
    unsafe {
        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let hdc = GetDC(Some(hwnd));
        SelectObject(hdc, font.into());
        let (mut x, mut y) = (px(PAD), px(theme::TITLE_H + BANNER_H) + px(8.0));
        let mut out = Vec::new();
        let mut row_h = px(TAB_H);
        let mut in_subs = false;
        for (tab, label) in tabs() {
            let sub = matches!(tab, Tab::Sub(_));
            if sub && !in_subs {
                // 子分類從新的一列開始
                in_subs = true;
                x = px(PAD);
                y += row_h + px(8.0);
                row_h = px(SUB_H);
            }
            let text: Vec<u16> = label.encode_utf16().collect();
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &text, &mut size);
            let w = size.cx + px(if sub { 20.0 } else { 28.0 });
            if x > px(PAD) && x + w > rc.right - px(PAD) {
                x = px(PAD);
                y += row_h + px(6.0);
            }
            let r = RECT { left: x, top: y, right: x + w, bottom: y + row_h };
            x = r.right + px(if sub { 6.0 } else { 8.0 });
            out.push((tab, mirror(r, rc.right)));
        }
        ReleaseDC(Some(hwnd), hdc);
        (out, y + row_h + px(14.0))
    }
}

/// 由右到左的語言：左右對調。
fn mirror(rc: RECT, client_w: i32) -> RECT {
    if is_rtl() {
        RECT { left: client_w - rc.right, top: rc.top, right: client_w - rc.left, bottom: rc.bottom }
    } else {
        rc
    }
}

/// 卡片的位置（已扣掉捲動）。
fn card_rects(client_w: i32, count: usize, scroll: i32, top: i32) -> Vec<RECT> {
    let (cw, ch, gap) = (px(CARD_W), px(CARD_H), px(GAP));
    let cols = ((client_w - px(PAD) * 2 + gap) / (cw + gap)).max(1);
    let used = cols * cw + (cols - 1) * gap;
    let left = (client_w - used) / 2;
    (0..count as i32)
        .map(|i| {
            let x = left + (i % cols) * (cw + gap);
            let y = top + (i / cols) * (ch + gap) - scroll;
            mirror(RECT { left: x, top: y, right: x + cw, bottom: y + ch }, client_w)
        })
        .collect()
}

fn client(hwnd: HWND) -> RECT {
    let mut rc = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rc);
    }
    rc
}

fn max_scroll(hwnd: HWND, tab: &Tab) -> i32 {
    let rc = client(hwnd);
    let (_, top) = tab_layout(hwnd);
    let cards = card_rects(rc.right, cards(tab).len(), 0, top);
    cards.last().map_or(0, |r| (r.bottom + px(PAD) - rc.bottom).max(0))
}

/// 縮圖（預乘 alpha 的 DIB，AlphaBlend 用），第一次畫到時才做。
fn thumb(id: &str, w: i32, h: i32) -> Option<HBITMAP> {
    if let Some(b) = get(|s| s.thumbs.get(id).copied()).flatten() {
        return Some(b);
    }
    let buf = with(|m| render::render_asset(&mut m.assets, id, w, h, 1.0)).flatten()?;
    let bmp = dib_from(&buf, w, h)?;
    set(|s| {
        s.thumbs.insert(id.to_string(), bmp);
    });
    Some(bmp)
}

fn dib_from(buf: &[u8], w: i32, h: i32) -> Option<HBITMAP> {
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let bmp = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        std::ptr::copy_nonoverlapping(buf.as_ptr(), bits as *mut u8, buf.len());
        Some(bmp)
    }
}

unsafe fn round_rect(hdc: HDC, rc: &RECT, fill: COLORREF, border: COLORREF, width: i32, radius: i32) {
    let brush = CreateSolidBrush(fill);
    let pen = CreatePen(PS_SOLID, width.max(1), border);
    let ob = SelectObject(hdc, brush.into());
    let op = SelectObject(hdc, pen.into());
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, ob);
    SelectObject(hdc, op);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

/// 虛線圓角框（「＋ 加入圖片」卡片）。
unsafe fn dashed_rect(hdc: HDC, rc: &RECT, fill: COLORREF, border: COLORREF, radius: i32) {
    let brush = CreateSolidBrush(fill);
    let lb = LOGBRUSH { lbStyle: BS_SOLID, lbColor: border, lbHatch: 0 };
    let pen = ExtCreatePen(PEN_STYLE(PS_GEOMETRIC.0 | PS_DASH.0), px(1.5).max(1) as u32, &lb, None);
    let ob = SelectObject(hdc, brush.into());
    let op = SelectObject(hdc, pen.into());
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, ob);
    SelectObject(hdc, op);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

fn mix(a: COLORREF, b: COLORREF, t: f32) -> COLORREF {
    let ch = |v: u32, s: u32| (((v >> s) & 0xff) as f32 * (1.0 - t) + ((b.0 >> s) & 0xff) as f32 * t) as u32;
    COLORREF(ch(a.0, 0) | ch(a.0, 8) << 8 | ch(a.0, 16) << 16)
}

// ───────────────────────────── 畫面 ─────────────────────────────

unsafe fn paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    draw(hwnd, hdc);
    let _ = EndPaint(hwnd, &ps);
}

/// 整個視窗畫到 hdc 上（雙緩衝）。
unsafe fn draw(hwnd: HWND, hdc: HDC) {
    let rc = client(hwnd);
    let Some((tab, scroll, hover, close_hot, font, bold, small, icon)) = get(|s| (s.tab.clone(), s.scroll, s.hover, s.close_hot, s.font, s.bold, s.small, s.icon)) else {
        return;
    };
    let (tab_rects, grid_top) = tab_layout(hwnd);
    let mem = CreateCompatibleDC(Some(hdc));
    let bmp = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    let old = SelectObject(mem, bmp.into());
    let p = theme::palette();
    theme::paint_background(mem, &rc);
    theme::paint_title(mem, hwnd, &t("shop.title"), icon, bold, close_hot);
    SetBkMode(mem, TRANSPARENT);

    // ── 橫幅：粉橘到薰衣草的漸層 ──
    // 自己的群組：教怎麼加圖片；其他分類：廣告輪播
    let banner = banner_rect(rc.right);
    let (bw, bh) = (banner.right - banner.left, banner.bottom - banner.top);
    let mine = matches!(tab, Tab::Group(_));
    let ad = get(|s| s.ad).unwrap_or(0);
    // 滑動進度：舊的一則往右滑出去，新的從左邊滑進來
    let (from, prog) = match get(|s| s.slide).flatten() {
        Some((from, start)) if !mine => {
            let u = ((store_now().saturating_sub(start)) as f32 / SLIDE_MS).clamp(0.0, 1.0);
            (from, 1.0 - (1.0 - u).powi(3))
        }
        _ => (ad, 1.0),
    };
    let shift = (prog * bw as f32).round() as i32;
    // (第幾則, 這一則的左邊在哪)
    let shown: Vec<(usize, i32)> = if prog >= 1.0 { vec![(ad, 0)] } else { vec![(ad, shift - bw), (from, shift)] };
    let list = ads();
    let slide_of = |i: usize| -> (String, String) {
        if mine {
            (t("shop.my_banner"), t("shop.my_hint"))
        } else {
            list.get(i).map(|a| (a.title.clone(), a.sub.clone())).unwrap_or_default()
        }
    };
    if bw > 0 && bh > 0 {
        let mut buf = vec![0u8; (bw * bh * 4) as usize];
        // 自己的群組：粉色漸層；廣告：底圖
        let rtl = is_rtl();
        let mine_stops: [(f32, [f32; 3]); 3] = [(0.0, [255.0, 214.0, 222.0]), (0.5, [255.0, 226.0, 200.0]), (1.0, [222.0, 214.0, 252.0])];
        let images: Vec<Option<super::image::Pixels>> = if mine { vec![] } else { list.iter().map(|a| ad_image(a, bw, bh)).collect() };
        let r = px(12.0) as f32;
        for y in 0..bh {
            for x in 0..bw {
                // 這一點屬於哪一則、在那一則裡的位置
                let (which, lx) = if prog < 1.0 && x < shift { (ad, x - (shift - bw)) } else if prog < 1.0 { (from, x - shift) } else { (ad, x) };
                let t = lx as f32 / bw as f32;
                let c: [f32; 3] = match images.get(which).and_then(|i| i.as_ref()) {
                    Some(img) => {
                        // 底圖（BGRA）；左半邊蓋淡白，字才看得清楚
                        // 由右到左的語言：圖左右翻過來，字那一邊才是空的
                        let (ix, tt) = if rtl { (bw - 1 - lx.clamp(0, bw - 1), 1.0 - t) } else { (lx.clamp(0, bw - 1), t) };
                        let j = ((y * bw + ix) * 4) as usize;
                        let veil = (0.55 * (1.0 - tt / 0.6)).clamp(0.0, 0.55);
                        let px = |k: usize| img.data[j + k] as f32 * (1.0 - veil) + 255.0 * veil;
                        [px(2), px(1), px(0)]
                    }
                    None => {
                        let stops = mine_stops;
                        let (a, b) = if t < 0.5 { (stops[0], stops[1]) } else { (stops[1], stops[2]) };
                        let u = (t - a.0) / (b.0 - a.0);
                        [0, 1, 2].map(|k| a.1[k] + (b.1[k] - a.1[k]) * u)
                    }
                };
                // 圓角
                let (cx, cy) = ((x as f32 + 0.5).clamp(r, bw as f32 - r), (y as f32 + 0.5).clamp(r, bh as f32 - r));
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (r - d + 0.5).clamp(0.0, 1.0);
                let i = ((y * bw + x) * 4) as usize;
                buf[i] = (c[2] * cov) as u8;
                buf[i + 1] = (c[1] * cov) as u8;
                buf[i + 2] = (c[0] * cov) as u8;
                buf[i + 3] = (255.0 * cov) as u8;
            }
        }
        if let Some(b) = dib_from(&buf, bw, bh) {
            blend(mem, b, banner.left, banner.top, bw, bh);
            let _ = DeleteObject(b.into());
        }
        // 文字跟著滑，超出廣告區的剪掉
        let save = SaveDC(mem);
        let _ = IntersectClipRect(mem, banner.left, banner.top, banner.right, banner.bottom);
        let (regular, big) = theme::fonts(17.0);
        let _ = DeleteObject(regular.into());
        let dots_w = if mine { 0 } else { px(14.0) * ad_count() as i32 + px(10.0) };
        for (i, dx) in &shown {
            let (title, sub) = slide_of(*i);
            // 只用文字那一側的 60%，不壓到另一側的插圖
            let text_w = if mine { bw } else { bw * 3 / 5 };
            let (l, r) = if is_rtl() { (banner.right + dx - text_w, banner.right + dx) } else { (banner.left + dx, banner.left + dx + text_w) };
            SelectObject(mem, big.into());
            SetTextColor(mem, COLORREF(0x00503A3A));
            let mut t1: Vec<u16> = title.encode_utf16().collect();
            let mut r1 = RECT { left: l + px(20.0), top: banner.top + px(18.0), right: r - px(16.0), bottom: banner.top + px(50.0) };
            DrawTextW(mem, &mut t1, &mut r1, DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX | rtl_align());
            SelectObject(mem, small.into());
            SetTextColor(mem, COLORREF(0x006A5858));
            let mut t2: Vec<u16> = sub.encode_utf16().collect();
            // 右下角留給小圓點
            let mut r2 = if is_rtl() {
                RECT { left: l + px(16.0), top: banner.top + px(52.0), right: r - px(20.0), bottom: banner.bottom - px(14.0) }
            } else {
                RECT { left: l + px(20.0), top: banner.top + px(52.0), right: r - px(16.0) - if mine { dots_w } else { 0 }, bottom: banner.bottom - px(14.0) }
            };
            DrawTextW(mem, &mut t2, &mut r2, DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX | rtl_align());
        }
        let _ = RestoreDC(mem, save);
        let _ = DeleteObject(big.into());
        if !mine {
            for (i, d) in ad_dots(&banner).iter().enumerate() {
                let on = i == ad % ad_count();
                let r = if on { px(4.0) } else { px(3.0) };
                let (cx, cy) = ((d.left + d.right) / 2, (d.top + d.bottom) / 2);
                let dot = RECT { left: cx - r, top: cy - r, right: cx + r, bottom: cy + r };
                let c = if on { COLORREF(0x00503A3A) } else { COLORREF(0x00A89090) };
                round_rect(mem, &dot, c, c, 1, r * 2);
            }
        }
    }

    // ── 分類標籤（自己的群組用另一種顏色）──
    let labels: HashMap<String, String> = tabs().into_iter().map(|(t, l)| (format!("{t:?}"), l)).collect();
    for (tb, r) in &tab_rects {
        let on = *tb == tab || (*tb == Tab::Cat(0) && matches!(tab, Tab::Sub(_)));
        let group = matches!(tb, Tab::Group(_));
        let sub = matches!(tb, Tab::Sub(_));
        let new = *tb == Tab::NewGroup;
        let tint = COLORREF(0x009A6BE0); // 自己的群組：粉紫
        let (fill, border, text) = if on && sub {
            (mix(p.card, p.accent, 0.16), p.accent, p.accent)
        } else if on {
            let c = if group { tint } else { p.accent };
            (c, c, COLORREF(0x00FFFFFF))
        } else if sub {
            (p.bg, mix(p.border, p.bg, 0.3), p.muted)
        } else if new {
            (p.bg, p.border, p.muted)
        } else if group {
            (mix(p.card, tint, 0.12), mix(p.border, tint, 0.4), p.text)
        } else {
            (p.card, p.border, p.text)
        };
        if new {
            dashed_rect(mem, r, fill, border, r.bottom - r.top);
        } else {
            round_rect(mem, r, fill, border, 1, r.bottom - r.top);
        }
        SetTextColor(mem, text);
        SelectObject(mem, if sub { small.into() } else { font.into() });
        let mut t: Vec<u16> = labels.get(&format!("{tb:?}")).cloned().unwrap_or_default().encode_utf16().collect();
        let mut rr = *r;
        DrawTextW(mem, &mut t, &mut rr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    }

    // ── 商品卡片 ──
    let items = cards(&tab);
    let save = SaveDC(mem);
    let _ = IntersectClipRect(mem, 0, grid_top - px(4.0), rc.right, rc.bottom);
    let (thumb_w, thumb_h) = (px(CARD_W) - px(12.0), px(THUMB_H));
    let mut tip: Option<(Vec<u16>, RECT, i32)> = None;
    for (i, (r, card)) in card_rects(rc.right, items.len(), scroll, grid_top).iter().zip(&items).enumerate() {
        if r.bottom < grid_top || r.top > rc.bottom {
            continue;
        }
        let hot = hover == Some(i);
        let lift = if hot { px(3.0) } else { 0 };
        let card_rc = RECT { left: r.left, top: r.top - lift, right: r.right, bottom: r.bottom - lift };
        let fill = if p.dark { p.card } else { COLORREF(0x00FFFFFF) };
        let id = match card {
            Card::Add | Card::Paste | Card::Custom | Card::Order => {
                // 「＋ 加入圖片」「貼上剪貼簿」
                dashed_rect(mem, &card_rc, mix(fill, p.accent, if hot { 0.08 } else { 0.03 }), if hot { p.accent } else { p.border }, px(16.0));
                let (plus_font, plus_bold) = theme::fonts(34.0);
                let _ = DeleteObject(plus_bold.into());
                let plus_font = if matches!(card, Card::Add) {
                    plus_font
                } else {
                    let _ = DeleteObject(plus_font.into());
                    CreateFontW(-px(28.0), 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0, w!("Segoe MDL2 Assets"))
                };
                SelectObject(mem, plus_font.into());
                SetTextColor(mem, if hot { p.accent } else { p.muted });
                let mut plus: Vec<u16> = match card { Card::Add => "＋", Card::Paste => "\u{E77F}", Card::Order => "\u{E774}", _ => "\u{E719}" }.encode_utf16().collect();
                let mut pr = RECT { left: card_rc.left, top: card_rc.top + px(12.0), right: card_rc.right, bottom: card_rc.top + px(84.0) };
                DrawTextW(mem, &mut plus, &mut pr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                let _ = DeleteObject(plus_font.into());
                SelectObject(mem, small.into());
                let mut label: Vec<u16> = t(match card { Card::Add => "shop.add_image", Card::Paste => "shop.paste_image", Card::Order => "shop.order", _ => "shop.custom" }).encode_utf16().collect();
                let mut lr = RECT { left: card_rc.left + px(6.0), top: card_rc.top + px(80.0), right: card_rc.right - px(6.0), bottom: card_rc.bottom - px(6.0) };
                DrawTextW(mem, &mut label, &mut lr, DT_CENTER | DT_WORDBREAK | DT_NOPREFIX);
                continue;
            }
            Card::Asset(id) => id,
        };
        if hot {
            // 浮起來的影子
            let sh = RECT { left: card_rc.left + px(2.0), top: card_rc.top + px(5.0), right: card_rc.right + px(2.0), bottom: card_rc.bottom + px(5.0) };
            round_rect(mem, &sh, mix(p.bg, COLORREF(0), 0.12), mix(p.bg, COLORREF(0), 0.12), 1, px(16.0));
        }
        round_rect(mem, &card_rc, fill, if hot { p.accent } else { p.border }, if hot { px(2.0) } else { 1 }, px(16.0));
        // 縮圖底下一塊淡淡的展示台
        let stage = RECT { left: card_rc.left + px(6.0), top: card_rc.top + px(6.0), right: card_rc.right - px(6.0), bottom: card_rc.top + px(6.0) + thumb_h };
        round_rect(mem, &stage, mix(fill, p.accent, 0.06), mix(fill, p.accent, 0.06), 1, px(12.0));
        if let Some(b) = thumb(id, thumb_w, thumb_h) {
            blend(mem, b, stage.left, stage.top, thumb_w, thumb_h);
        }
        // 小標籤：自己做的標「自製」
        if assets::is_custom(id) {
            SelectObject(mem, small.into());
            let mut tag_w16: Vec<u16> = t("shop.handmade").encode_utf16().collect();
            let mut fs = SIZE::default();
            let _ = GetTextExtentPoint32W(mem, &tag_w16, &mut fs);
            let tag_w = fs.cx + px(14.0);
            let mut tag = if is_rtl() {
                RECT { left: card_rc.left + px(10.0), top: card_rc.top + px(10.0), right: card_rc.left + px(10.0) + tag_w, bottom: card_rc.top + px(26.0) }
            } else {
                RECT { left: card_rc.right - px(10.0) - tag_w, top: card_rc.top + px(10.0), right: card_rc.right - px(10.0), bottom: card_rc.top + px(26.0) }
            };
            round_rect(mem, &tag, mix(fill, COLORREF(0x006E62F0), 0.18), mix(fill, COLORREF(0x006E62F0), 0.18), 1, px(16.0));
            SetTextColor(mem, COLORREF(0x00B0367F));
            DrawTextW(mem, &mut tag_w16, &mut tag, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
        }
        // 名稱
        SelectObject(mem, font.into());
        SetTextColor(mem, p.text);
        let name: Vec<u16> = assets::name_of(id).encode_utf16().collect();
        let mut nr = RECT { left: card_rc.left + px(6.0), top: stage.bottom + px(4.0), right: card_rc.right - px(6.0), bottom: card_rc.bottom - px(6.0) };
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(mem, &name, &mut size);
        if hot && size.cx > nr.right - nr.left {
            tip = Some((name.clone(), card_rc, stage.bottom));
        }
        DrawTextW(mem, &mut name.clone(), &mut nr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX);
    }
    // 名稱太長被截掉的卡片：滑過去時在縮圖上方浮出完整名稱
    if let Some((name, card_rc, stage_bottom)) = tip {
        SelectObject(mem, font.into());
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(mem, &name, &mut size);
        let (bw, bh) = (size.cx + px(20.0), size.cy + px(10.0));
        let cx = (card_rc.left + card_rc.right) / 2;
        let left = (cx - bw / 2).clamp(px(6.0), (rc.right - px(6.0) - bw).max(px(6.0)));
        let top = stage_bottom - bh - px(2.0);
        let bubble = RECT { left, top, right: left + bw, bottom: top + bh };
        round_rect(mem, &bubble, p.text, p.text, 1, px(10.0));
        SetTextColor(mem, if p.dark { COLORREF(0x00202020) } else { COLORREF(0x00FFFFFF) });
        let mut tr = bubble;
        DrawTextW(mem, &mut name.clone(), &mut tr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    }
    // 新群組還沒有東西：提示
    if tab == Tab::NewGroup || (items.len() <= 2 && matches!(tab, Tab::Group(_))) {
        SelectObject(mem, small.into());
        SetTextColor(mem, p.muted);
        let mut hint: Vec<u16> = t("shop.empty_group").encode_utf16().collect();
        let mut hr = RECT { left: px(PAD), top: rc.bottom - px(40.0), right: rc.right - px(PAD), bottom: rc.bottom - px(12.0) };
        DrawTextW(mem, &mut hint, &mut hr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    }
    let _ = RestoreDC(mem, save);

    let _ = BitBlt(hdc, 0, 0, rc.right, rc.bottom, Some(mem), 0, 0, SRCCOPY);
    SelectObject(mem, old);
    let _ = DeleteObject(bmp.into());
    let _ = DeleteDC(mem);
}

/// 由右到左的語言：文字靠右、由右往左排。
fn rtl_align() -> DRAW_TEXT_FORMAT {
    if is_rtl() { DT_RIGHT | DT_RTLREADING } else { DRAW_TEXT_FORMAT(0) }
}

/// 預乘 alpha 的 DIB 疊到 hdc。
unsafe fn blend(hdc: HDC, bmp: HBITMAP, x: i32, y: i32, w: i32, h: i32) {
    let src = CreateCompatibleDC(Some(hdc));
    let old = SelectObject(src, bmp.into());
    let bf = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
    let _ = AlphaBlend(hdc, x, y, w, h, src, 0, 0, w, h, bf);
    SelectObject(src, old);
    let _ = DeleteDC(src);
}

fn card_at(hwnd: HWND, pt: POINT) -> Option<(usize, Card)> {
    let (tab, scroll) = get(|s| (s.tab.clone(), s.scroll))?;
    let (_, top) = tab_layout(hwnd);
    if pt.y < top - px(4.0) {
        return None;
    }
    let list = cards(&tab);
    let i = card_rects(client(hwnd).right, list.len(), scroll, top).iter().position(|r| render::in_rect(r, pt.x, pt.y))?;
    Some((i, list[i].clone()))
}

fn tab_at(hwnd: HWND, pt: POINT) -> Option<(Tab, RECT)> {
    tab_layout(hwnd).0.into_iter().find(|(_, r)| render::in_rect(r, pt.x, pt.y))
}

fn switch_to(tab: Tab) {
    let tab = if tab == Tab::Cat(0) { Tab::Sub(0) } else { tab };
    set(|s| {
        s.tab = tab;
        s.scroll = 0;
        s.hover = None;
    });
    invalidate();
}

/// 拖曳中的影子：素材原本的大小，半透明，滑鼠穿透。
fn make_ghost(asset: &str) -> Option<(HWND, i32, i32)> {
    let (w, h) = with(|m| {
        let (lw, lh) = m.assets.logical_size(asset);
        let s = m.scale;
        if assets::cat_of(asset) == assets::Cat::Tape {
            ((130.0 * s) as i32, (lh * s + 14.0 * s) as i32)
        } else {
            ((lw * s * 1.15) as i32 + 8, (lh * s * 1.15) as i32 + 8)
        }
    })?;
    let buf = with(|m| render::render_asset(&mut m.assets, asset, w, h, 0.85)).flatten()?;
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            GHOST_CLASS,
            w!(""),
            WS_POPUP,
            0,
            0,
            w,
            h,
            None,
            None,
            Some(win::hinstance()),
            None,
        )
        .ok()?;
        let bmp = dib_from(&buf, w, h)?;
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let old = SelectObject(mem, bmp.into());
        let at = win::cursor_pos();
        let pos = POINT { x: at.x - w / 2, y: at.y - h / 2 };
        let size = SIZE { cx: w, cy: h };
        let src = POINT { x: 0, y: 0 };
        let bf = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let _ = UpdateLayeredWindow(hwnd, Some(screen), Some(&pos), Some(&size), Some(mem), Some(&src), COLORREF(0), Some(&bf), ULW_ALPHA);
        SelectObject(mem, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        Some((hwnd, w, h))
    }
}

// ───────────────────────────── 自己的貼紙 ─────────────────────────────

/// 目前在哪個自己的群組（在內建分類就是 None）。
fn current_group() -> Option<String> {
    match get(|s| s.tab.clone())? {
        Tab::Group(id) => Some(id),
        _ => None,
    }
}

/// 圖片做成貼紙收進來，切到那個群組。
fn add_pixels(images: Vec<image::Pixels>) {
    if images.is_empty() {
        return;
    }
    unsafe {
        SetCursor(LoadCursorW(None, IDC_WAIT).ok());
    }
    match stickers::add_images(current_group(), images) {
        Some((group, n)) => {
            switch_to(Tab::Group(group));
            crate::toast::show(tf("toast.stickers_made", &[("count", &n.to_string())]), false, 2);
        }
        None => crate::toast::show(t("toast.sticker_failed"), true, 2),
    }
}

fn add_files(files: Vec<std::path::PathBuf>) {
    let images: Vec<_> = files
        .into_iter()
        .filter(|f| image::is_image_file(f))
        .filter_map(|f| match image::decode_file(&f) {
            Ok(p) => Some(p),
            Err(e) => {
                crate::base::log_error(format!("{e:#}"));
                None
            }
        })
        .collect();
    add_pixels(images);
}

fn pick_files(hwnd: HWND) {
    let filter = format!("{}\0*.png;*.jpg;*.jpeg;*.gif;*.bmp;*.webp;*.tif;*.tiff\0", t("shop.image_filter"));
    let files = super::open_dialog(hwnd, &filter, true);
    add_files(files);
}

fn paste(hwnd: HWND) {
    match image::clipboard_image(hwnd) {
        Some(p) => add_pixels(vec![p]),
        None => crate::toast::show(t("toast.no_image"), false, 2),
    }
}

/// 在標籤上開一個文字框改群組名稱。
fn start_rename(hwnd: HWND, group: &str) {
    finish_rename(false);
    let Some((_, r)) = tab_layout(hwnd).0.into_iter().find(|(t, _)| *t == Tab::Group(group.to_string())) else { return };
    let name = stickers::groups().into_iter().find(|g| g.id == group).map(|g| g.name).unwrap_or_default();
    unsafe {
        let text = wide(&name);
        let width = (r.right - r.left).max(px(140.0));
        let Ok(edit) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR(text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            r.left,
            r.top,
            width,
            r.bottom - r.top,
            Some(hwnd),
            None,
            Some(win::hinstance()),
            None,
        ) else {
            return;
        };
        if let Some(f) = get(|s| s.font) {
            SendMessageW(edit, WM_SETFONT, Some(WPARAM(f.0 as usize)), Some(LPARAM(1)));
        }
        theme::style_edit(edit);
        let _ = SetFocus(Some(edit));
        SendMessageW(edit, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
        set(|s| s.rename = Some((edit, group.to_string())));
    }
}

/// 收起改名的文字框（save = 存下新名字）。
fn finish_rename(save: bool) {
    let Some((edit, group)) = SHOP.with(|s| s.try_borrow_mut().ok().and_then(|mut s| s.as_mut().and_then(|s| s.rename.take()))) else { return };
    if save {
        stickers::rename_group(&group, &theme::window_text(edit));
    }
    unsafe {
        let _ = DestroyWindow(edit);
    }
    invalidate();
}

/// 主迴圈先問這裡：改名框的 Enter、Esc。
pub fn pre_translate(msg: &MSG) -> bool {
    let Some(edit) = get(|s| s.rename.as_ref().map(|r| r.0)).flatten() else { return false };
    if msg.hwnd != edit || msg.message != WM_KEYDOWN {
        return false;
    }
    match VIRTUAL_KEY(msg.wParam.0 as u16) {
        VK_RETURN => {
            finish_rename(true);
            true
        }
        VK_ESCAPE => {
            finish_rename(false);
            true
        }
        _ => false,
    }
}

const C_REMOVE: u32 = 2;
const C_DELETE: u32 = 3;
const C_WHOLE: u32 = 4;
const C_NEW_GROUP: u32 = 99;
const C_GROUP: u32 = 100;

/// 卡片的右鍵選單：收進群組、從群組拿掉、刪除自己做的貼紙。
fn card_menu(hwnd: HWND, asset: &str) {
    let groups = stickers::groups();
    let tab = get(|s| s.tab.clone()).unwrap_or(Tab::Cat(0));
    let mut sub: Vec<MenuItem> = groups.iter().enumerate().map(|(i, g)| MenuItem::checked(C_GROUP + i as u32, g.name.clone(), g.items.iter().any(|a| a == asset))).collect();
    if !sub.is_empty() {
        sub.push(MenuItem::Separator);
    }
    sub.push(MenuItem::item(C_NEW_GROUP, t("shop.new_group")));
    let mut items = vec![MenuItem::disabled(assets::name_of(asset)), MenuItem::Submenu { text: t("shop.collect"), items: sub }];
    if matches!(tab, Tab::Group(_)) {
        items.push(MenuItem::item(C_REMOVE, t("shop.remove_from_group")));
    }
    if assets::is_custom(asset) {
        items.push(MenuItem::Separator);
        if stickers::has_source(asset) {
            items.push(MenuItem::checked(C_WHOLE, t("shop.whole_photo"), stickers::is_whole(asset)));
        }
        items.push(MenuItem::item(C_DELETE, t("shop.delete_sticker")));
    }
    match win::popup_menu(hwnd, &items, win::cursor_pos()) {
        C_REMOVE => {
            if let Tab::Group(g) = &tab {
                stickers::remove_from_group(g, asset);
            }
        }
        C_DELETE => stickers::delete_sticker(asset),
        C_WHOLE => {
            unsafe {
                SetCursor(LoadCursorW(None, IDC_WAIT).ok());
            }
            if stickers::set_whole(asset, !stickers::is_whole(asset)) {
                set(|s| {
                    if let Some(b) = s.thumbs.remove(asset) {
                        unsafe {
                            let _ = DeleteObject(b.into());
                        }
                    }
                });
                invalidate();
                super::refresh_all();
            } else {
                crate::toast::show(t("toast.sticker_failed"), true, 2);
            }
        }
        C_NEW_GROUP => {
            if let Some(g) = stickers::add_group() {
                stickers::add_to_group(&g, asset);
                switch_to(Tab::Group(g.clone()));
                start_rename(hwnd, &g);
            }
        }
        id if (C_GROUP..C_GROUP + groups.len() as u32).contains(&id) => {
            let g = &groups[(id - C_GROUP) as usize];
            if g.items.iter().any(|a| a == asset) {
                stickers::remove_from_group(&g.id, asset);
            } else {
                stickers::add_to_group(&g.id, asset);
                crate::toast::show(tf("toast.collected", &[("group", &g.name)]), false, 1);
            }
        }
        _ => {}
    }
    invalidate();
}

const G_RENAME: u32 = 1;
const G_LEFT: u32 = 2;
const G_RIGHT: u32 = 3;
const G_DELETE: u32 = 4;

/// 群組標籤的右鍵選單。
fn group_menu(hwnd: HWND, group: &str) {
    let items = vec![
        MenuItem::item(G_RENAME, t("shop.rename_group")),
        MenuItem::item(G_LEFT, t("shop.move_left")),
        MenuItem::item(G_RIGHT, t("shop.move_right")),
        MenuItem::Separator,
        MenuItem::item(G_DELETE, t("shop.delete_group")),
    ];
    match win::popup_menu(hwnd, &items, win::cursor_pos()) {
        G_RENAME => start_rename(hwnd, group),
        G_LEFT => stickers::move_group(group, if is_rtl() { 1 } else { -1 }),
        G_RIGHT => stickers::move_group(group, if is_rtl() { -1 } else { 1 }),
        G_DELETE => {
            let g = stickers::groups().into_iter().find(|g| g.id == group);
            let mine = g.as_ref().map_or(0, |g| g.items.iter().filter(|a| assets::is_custom(a)).count());
            let ok = mine == 0 || unsafe {
                let text = wide(&tf("shop.delete_group_confirm", &[("name", &g.map(|g| g.name).unwrap_or_default()), ("count", &mine.to_string())]));
                let title = wide(crate::base::DISPLAY_NAME);
                MessageBoxW(Some(hwnd), PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OKCANCEL | MB_ICONWARNING) == IDOK
            };
            if ok {
                stickers::delete_group(group);
                switch_to(Tab::Cat(0));
            }
        }
        _ => {}
    }
    invalidate();
}

// ───────────────────────────── 視窗程序 ─────────────────────────────

fn lparam_point(lp: LPARAM) -> POINT {
    POINT { x: (lp.0 & 0xffff) as i16 as i32, y: ((lp.0 >> 16) & 0xffff) as i16 as i32 }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCCALCSIZE if wp.0 != 0 => LRESULT(0),
        WM_NCHITTEST => theme::hit_test(hwnd, lp),
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_SIZE => {
            finish_rename(true);
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lp.0 as *mut MINMAXINFO);
            info.ptMinTrackSize = POINT { x: px(PAD * 2.0 + CARD_W * 2.0 + GAP + 16.0), y: px(360.0) };
            LRESULT(0)
        }
        WM_MOUSEMOVE | WM_MOUSELEAVE => {
            let hot = get(|s| s.close_hot).unwrap_or(false);
            if let Some(now) = theme::close_button_mouse(hwnd, msg, lp, hot) {
                set(|s| s.close_hot = now);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            if msg == WM_MOUSELEAVE {
                set(|s| s.hover = None);
                let _ = InvalidateRect(Some(hwnd), None, false);
                return LRESULT(0);
            }
            let pt = lparam_point(lp);
            let mut tme = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: hwnd, dwHoverTime: 0 };
            let _ = TrackMouseEvent(&mut tme);
            // 拖曳中：影子跟著游標
            let drag = get(|s| s.drag.as_ref().map(|d| (d.asset.clone(), d.start, d.ghost))).flatten();
            if let Some((asset, start, ghost)) = drag {
                let at = win::cursor_pos();
                let ghost = match ghost {
                    Some(g) => Some(g),
                    None if (at.x - start.x).abs().max((at.y - start.y).abs()) > px(5.0) => {
                        let g = make_ghost(&asset);
                        set(|s| {
                            if let Some(d) = s.drag.as_mut() {
                                d.ghost = g;
                            }
                        });
                        g
                    }
                    None => None,
                };
                if let Some((g, w, h)) = ghost {
                    let _ = SetWindowPos(g, Some(HWND_TOPMOST), at.x - w / 2, at.y - h / 2, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
                }
                return LRESULT(0);
            }
            let hover = card_at(hwnd, pt).map(|(i, _)| i);
            if get(|s| s.hover != hover) == Some(true) {
                set(|s| s.hover = hover);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_SETCURSOR if (lp.0 & 0xffff) as u32 == HTCLIENT => {
            let mut pt = win::cursor_pos();
            let _ = ScreenToClient(hwnd, &mut pt);
            if card_at(hwnd, pt).is_some() || tab_at(hwnd, pt).is_some() {
                if let Ok(c) = LoadCursorW(None, IDC_HAND) {
                    SetCursor(Some(c));
                    return LRESULT(1);
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_LBUTTONDOWN => {
            let pt = lparam_point(lp);
            finish_rename(true);
            let banner = banner_rect(client(hwnd).right);
            let on_ads = render::in_rect(&banner, pt.x, pt.y) && !matches!(get(|s| s.tab.clone()), Some(Tab::Group(_)));
            if on_ads {
                match ad_dots(&banner).iter().position(|d| render::in_rect(d, pt.x, pt.y)) {
                    Some(i) => go_ad(hwnd, i),
                    None => ad_click(get(|s| s.ad).unwrap_or(0)),
                }
                return LRESULT(0);
            }
            if let Some((tab, _)) = tab_at(hwnd, pt) {
                if tab == Tab::NewGroup {
                    if let Some(g) = stickers::add_group() {
                        switch_to(Tab::Group(g.clone()));
                        start_rename(hwnd, &g);
                    }
                } else {
                    switch_to(tab);
                }
            } else if let Some((_, card)) = card_at(hwnd, pt) {
                match card {
                    Card::Add => pick_files(hwnd),
                    Card::Paste => paste(hwnd),
                    Card::Custom => win::open_url(&crate::base::custom_url()),
                    Card::Order => win::open_url(&crate::base::order_url()),
                    Card::Asset(id) => {
                        SetCapture(hwnd);
                        set(|s| s.drag = Some(Drag { asset: id, start: win::cursor_pos(), ghost: None }));
                    }
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK => {
            // 雙擊群組標籤改名
            if let Some((Tab::Group(g), _)) = tab_at(hwnd, lparam_point(lp)) {
                start_rename(hwnd, &g);
            }
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            let pt = lparam_point(lp);
            finish_rename(true);
            if let Some((Tab::Group(g), _)) = tab_at(hwnd, pt) {
                group_menu(hwnd, &g);
            } else if let Some((_, Card::Asset(id))) = card_at(hwnd, pt) {
                card_menu(hwnd, &id);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let hot = get(|s| s.close_hot).unwrap_or(false);
            theme::close_button_mouse(hwnd, msg, lp, hot);
            let drag = SHOP.with(|s| s.borrow_mut().as_mut().and_then(|s| s.drag.take()));
            let _ = ReleaseCapture();
            if let Some(d) = drag {
                if let Some((g, _, _)) = d.ghost {
                    let _ = DestroyWindow(g);
                    let at = win::cursor_pos();
                    let over_shop = render::in_rect(&win::window_rect(hwnd), at.x, at.y);
                    if over_shop {
                        // 拖到自己的群組標籤上：收進去
                        let mut pt = at;
                        let _ = ScreenToClient(hwnd, &mut pt);
                        if let Some((Tab::Group(g), _)) = tab_at(hwnd, pt) {
                            stickers::add_to_group(&g, &d.asset);
                            invalidate();
                        }
                    } else if !super::drop_at(&d.asset, at) {
                        crate::toast::show(t("toast.drop_on_note"), false, 2);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DROPFILES => {
            let drop = HDROP(wp.0 as *mut _);
            let files = image::drop_files(drop);
            DragFinish(drop);
            add_files(files);
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wp.0 >> 16) & 0xffff) as i16 as i32;
            let tab = get(|s| s.tab.clone()).unwrap_or(Tab::Cat(0));
            let max = max_scroll(hwnd, &tab);
            finish_rename(true);
            set(|s| s.scroll = (s.scroll - delta.signum() * px(60.0)).clamp(0, max));
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let ctrl = GetKeyState(VK_CONTROL.0 as i32) < 0;
            match VIRTUAL_KEY(wp.0 as u16) {
                VK_ESCAPE => {
                    let _ = DestroyWindow(hwnd);
                }
                VK_V if ctrl => paste(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND if (wp.0 >> 16) as u32 == EN_KILLFOCUS => {
            finish_rename(true);
            LRESULT(0)
        }
        WM_TIMER if wp.0 == AD_TIMER => {
            let banner = banner_rect(client(hwnd).right);
            let mut pt = win::cursor_pos();
            let _ = ScreenToClient(hwnd, &mut pt);
            let hovering = render::in_rect(&banner, pt.x, pt.y);
            if !hovering && !matches!(get(|s| s.tab.clone()), Some(Tab::Group(_))) {
                go_ad(hwnd, get(|s| s.ad + 1).unwrap_or(0));
            }
            LRESULT(0)
        }
        WM_TIMER if wp.0 == SLIDE_TIMER => {
            // 滑完就關掉計時器
            let done = get(|s| s.slide).flatten().is_none_or(|(_, start)| store_now().saturating_sub(start) as f32 >= SLIDE_MS);
            if done {
                set(|s| s.slide = None);
                let _ = KillTimer(Some(hwnd), SLIDE_TIMER);
            }
            let _ = InvalidateRect(Some(hwnd), Some(&banner_rect(client(hwnd).right)), false);
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = KillTimer(Some(hwnd), AD_TIMER);
            let _ = KillTimer(Some(hwnd), SLIDE_TIMER);
            // 縮圖、字型、圖示全部還給系統
            if let Some(s) = SHOP.with(|s| s.borrow_mut().take()) {
                for (_, b) in s.thumbs {
                    let _ = DeleteObject(b.into());
                }
                if let Some(d) = s.drag.and_then(|d| d.ghost) {
                    let _ = DestroyWindow(d.0);
                }
                let _ = DeleteObject(s.font.into());
                let _ = DeleteObject(s.bold.into());
                let _ = DeleteObject(s.small.into());
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe extern "system" fn ghost_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    DefWindowProcW(hwnd, msg, wp, lp)
}
