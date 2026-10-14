//! 文字樣式：顏色、字型、粗體、大小。
//!
//! 兩層：系統匣「文字預設樣式」是通用的（沒有自己設定的便利貼都跟著變），
//! 便利貼右鍵「文字樣式」只改那一張（可以再改回「跟著預設」）。
//! 字型只列這台電腦有裝的，另外可以從系統的字型對話框挑任何字型。

use std::cell::RefCell;
use std::collections::HashSet;

use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Controls::Dialogs::*;

use super::store::{self, Note, TextStyle, DEFAULT_FACE};
use super::{redraw, with};
use crate::i18n::t;
use crate::win::MenuItem;

/// 便利貼自己設成「依紙色自動」（蓋過預設的顏色）
pub const AUTO: u32 = 0xFF00_0000;

/// 調色盤：(0xRRGGBB, 名稱的語系 key)
pub const COLORS: [(u32, &str); 10] = [
    (0x26221E, "color.black"),
    (0x8B5A3C, "color.brown"),
    (0xE21E24, "color.red"),
    (0xF28C28, "color.orange"),
    (0xE8B820, "color.yellow"),
    (0x3DAA5C, "color.green"),
    (0x2F6FDB, "color.blue"),
    (0x8A5CD6, "color.purple"),
    (0xEE78A8, "color.pink"),
    (0xFAFAF7, "color.white"),
];

/// 常用字型（有裝才列出來）：(字型名稱, 顯示名稱)。顯示名稱用字型自己的語言。
const FACES: [(&str, &str); 16] = [
    ("Microsoft JhengHei UI", "微軟正黑體"),
    ("DFKai-SB", "標楷體"),
    ("PMingLiU", "新細明體"),
    ("Microsoft YaHei UI", "微软雅黑"),
    ("SimHei", "黑体"),
    ("Yu Gothic UI", "游ゴシック"),
    ("Meiryo UI", "メイリオ"),
    ("Malgun Gothic", "맑은 고딕"),
    ("Segoe UI", "Segoe UI"),
    ("Segoe Print", "Segoe Print ✎"),
    ("Segoe Script", "Segoe Script ✎"),
    ("Ink Free", "Ink Free ✎"),
    ("Comic Sans MS", "Comic Sans MS"),
    ("Impact", "Impact"),
    ("Arial Black", "Arial Black"),
    ("Georgia", "Georgia"),
];

/// 大小：(倍率, 名稱的語系 key)
const SIZES: [(f32, &str); 7] = [
    (0.85, "size.small"),
    (1.0, "size.normal"),
    (1.4, "size.large"),
    (2.0, "size.xlarge"),
    (3.0, "size.banner"),
    (4.5, "size.huge"),
    (6.0, "size.giant"),
];

thread_local! {
    static INSTALLED: RefCell<Option<HashSet<String>>> = const { RefCell::new(None) };
    static CUSTOM_COLORS: RefCell<[COLORREF; 16]> = const { RefCell::new([COLORREF(0x00FFFFFF); 16]) };
}

unsafe extern "system" fn collect(lf: *const LOGFONTW, _: *const TEXTMETRICW, _: u32, lp: LPARAM) -> i32 {
    let set = &mut *(lp.0 as *mut HashSet<String>);
    let name = &(*lf).lfFaceName;
    let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    set.insert(String::from_utf16_lossy(&name[..len]).to_lowercase());
    1
}

/// 這台電腦有沒有這個字型（第一次問時列一次，之後記著）。
fn installed(face: &str) -> bool {
    INSTALLED.with(|cell| {
        let mut cell = cell.borrow_mut();
        let set = cell.get_or_insert_with(|| {
            let mut set = HashSet::new();
            unsafe {
                let hdc = GetDC(None);
                let lf = LOGFONTW { lfCharSet: DEFAULT_CHARSET, ..Default::default() };
                EnumFontFamiliesExW(hdc, &lf, Some(collect), LPARAM(&mut set as *mut _ as isize), 0);
                ReleaseDC(None, hdc);
            }
            set
        });
        set.contains(&face.to_lowercase())
    })
}

/// 系統的字型對話框，回傳 (字型名稱, 粗體)。
fn pick_font(owner: HWND, current: &TextStyle) -> Option<(String, bool)> {
    let mut lf = LOGFONTW { lfCharSet: DEFAULT_CHARSET, lfWeight: if current.bold { 700 } else { 400 }, ..Default::default() };
    let face: Vec<u16> = (if current.face.is_empty() { DEFAULT_FACE } else { &current.face }).encode_utf16().take(31).collect();
    lf.lfFaceName[..face.len()].copy_from_slice(&face);
    let mut cf = CHOOSEFONTW {
        lStructSize: std::mem::size_of::<CHOOSEFONTW>() as u32,
        hwndOwner: owner,
        lpLogFont: &mut lf,
        Flags: CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT | CF_NOSIZESEL | CF_NOSCRIPTSEL,
        ..Default::default()
    };
    if !unsafe { ChooseFontW(&mut cf) }.as_bool() {
        return None;
    }
    let len = lf.lfFaceName.iter().position(|&c| c == 0).unwrap_or(32);
    Some((String::from_utf16_lossy(&lf.lfFaceName[..len]), lf.lfWeight >= 600))
}

/// 系統的顏色對話框，回傳 0xRRGGBB。
fn pick_color(owner: HWND, current: Option<u32>) -> Option<u32> {
    let init = current.filter(|c| *c != AUTO).unwrap_or(0x26221E);
    let rgb = |c: u32| COLORREF(((c >> 16) & 0xff) | (c & 0xff00) | ((c & 0xff) << 16));
    CUSTOM_COLORS.with(|custom| {
        let mut custom = custom.borrow_mut();
        let mut cc = CHOOSECOLORW {
            lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
            hwndOwner: owner,
            rgbResult: rgb(init),
            lpCustColors: custom.as_mut_ptr(),
            Flags: CC_FULLOPEN | CC_RGBINIT,
            ..Default::default()
        };
        if !unsafe { ChooseColorW(&mut cc) }.as_bool() {
            return None;
        }
        let c = cc.rgbResult.0;
        Some(((c & 0xff) << 16) | (c & 0xff00) | ((c >> 16) & 0xff))
    })
}

// ───────────────────────────── 選單 ─────────────────────────────

const C_FOLLOW: u32 = 0;
const C_AUTO: u32 = 1;
const C_COLOR: u32 = 2; // ～11
const C_CUSTOM: u32 = 12;
const F_FOLLOW: u32 = 20;
const F_FACE: u32 = 21; // ～36
const F_CURRENT: u32 = 38;
const F_MORE: u32 = 39;
const B_BOLD: u32 = 40;
const S_SIZE: u32 = 50; // ～56
const N_MAKE_DEFAULT: u32 = 60;
const N_RESET: u32 = 61;
/// 這組選單的 id 範圍（從 base 開始）
pub const SPAN: u32 = 70;

/// 文字樣式的子選單。`note` = 改某一張（None = 改預設）。id 從 `base` 開始。
pub fn menu(base: u32, note: Option<&Note>, default: &TextStyle) -> Vec<MenuItem> {
    let own = |f: fn(&Note) -> bool| note.is_some_and(f);
    let effective = match note {
        Some(n) => with(|m| m.board.style_of(n)).unwrap_or_default(),
        None => default.clone(),
    };
    // 顏色
    let cur_color = effective.color.filter(|c| *c != AUTO);
    let mut colors = Vec::new();
    if note.is_some() {
        colors.push(MenuItem::checked(base + C_FOLLOW, t("style.follow"), !own(|n| n.color.is_some())));
        colors.push(MenuItem::Separator);
    }
    let auto = match note {
        Some(n) => n.color == Some(AUTO),
        None => default.color.is_none(),
    };
    colors.push(MenuItem::checked(base + C_AUTO, t("color.auto"), auto));
    for (i, (c, key)) in COLORS.iter().enumerate() {
        colors.push(MenuItem::checked(base + C_COLOR + i as u32, format!("●  {}", t(key)), cur_color == Some(*c)));
    }
    if let Some(c) = cur_color.filter(|c| !COLORS.iter().any(|(x, _)| x == c)) {
        colors.push(MenuItem::checked(0, format!("●  #{c:06X}"), true));
    }
    colors.push(MenuItem::item(base + C_CUSTOM, t("color.custom")));
    // 字型
    let cur_face = if effective.face.is_empty() { DEFAULT_FACE.to_string() } else { effective.face.clone() };
    let mut faces = Vec::new();
    if note.is_some() {
        faces.push(MenuItem::checked(base + F_FOLLOW, t("style.follow"), !own(|n| n.face.is_some())));
        faces.push(MenuItem::Separator);
    }
    for (i, (face, label)) in FACES.iter().enumerate() {
        if installed(face) {
            faces.push(MenuItem::checked(base + F_FACE + i as u32, *label, cur_face.eq_ignore_ascii_case(face)));
        }
    }
    if !FACES.iter().any(|(f, _)| cur_face.eq_ignore_ascii_case(f)) {
        faces.push(MenuItem::checked(base + F_CURRENT, cur_face.clone(), true));
    }
    faces.extend([MenuItem::Separator, MenuItem::item(base + F_MORE, t("style.more_fonts"))]);
    let mut items = vec![
        MenuItem::Submenu { text: t("style.color"), items: colors },
        MenuItem::Submenu { text: t("style.font"), items: faces },
        MenuItem::checked(base + B_BOLD, t("style.bold"), effective.bold),
    ];
    if let Some(n) = note {
        items.push(MenuItem::Submenu {
            text: t("style.size"),
            items: SIZES.iter().enumerate().map(|(i, (z, key))| MenuItem::checked(base + S_SIZE + i as u32, t(key), (n.font - z).abs() < 0.03)).chain([MenuItem::disabled(t("style.size_hint"))]).collect(),
        });
        items.extend([
            MenuItem::Separator,
            MenuItem::item(base + N_MAKE_DEFAULT, t("style.make_default")),
            MenuItem::item(base + N_RESET, t("style.reset")),
        ]);
    }
    items
}

/// 改樣式：某一張或預設。
enum Target {
    Note(HWND),
    Default,
}

fn apply(target: &Target, f: impl FnOnce(&mut Option<String>, &mut Option<u32>, &mut Option<bool>)) {
    with(|m| {
        match target {
            Target::Note(h) => {
                if let Some(n) = m.note_of(*h) {
                    f(&mut n.face, &mut n.color, &mut n.bold);
                }
            }
            Target::Default => {
                let d = &mut m.board.text;
                let (mut face, mut color, mut bold) = (Some(d.face.clone()), d.color, Some(d.bold));
                f(&mut face, &mut color, &mut bold);
                d.face = face.unwrap_or_default();
                d.color = color.filter(|c| *c != AUTO);
                d.bold = bold.unwrap_or(false);
            }
        }
        store::save(&m.board);
    });
}

/// 處理選單結果。回傳 true = 是這組選單的項目。
pub fn handle(owner: HWND, base: u32, cmd: u32, note: Option<HWND>) -> bool {
    if cmd < base || cmd >= base + SPAN {
        return false;
    }
    let id = cmd - base;
    let target = match note {
        Some(h) => Target::Note(h),
        None => Target::Default,
    };
    let effective = with(|m| match note {
        Some(h) => m.note_of(h).cloned().map(|n| m.board.style_of(&n)),
        None => Some(m.board.text.clone()),
    })
    .flatten()
    .unwrap_or_default();
    match id {
        C_FOLLOW => apply(&target, |_, c, _| *c = None),
        C_AUTO => apply(&target, |_, c, _| *c = Some(AUTO)),
        _ if (C_COLOR..C_COLOR + COLORS.len() as u32).contains(&id) => apply(&target, |_, c, _| *c = Some(COLORS[(id - C_COLOR) as usize].0)),
        C_CUSTOM => {
            if let Some(color) = pick_color(owner, effective.color) {
                apply(&target, |_, c, _| *c = Some(color));
            }
        }
        F_FOLLOW => apply(&target, |f, _, _| *f = None),
        _ if (F_FACE..F_FACE + FACES.len() as u32).contains(&id) => {
            let face = FACES[(id - F_FACE) as usize].0;
            apply(&target, |f, _, _| *f = Some(if face == DEFAULT_FACE { String::new() } else { face.to_string() }));
        }
        F_MORE => {
            if let Some((face, bold)) = pick_font(owner, &effective) {
                apply(&target, |f, _, b| {
                    *f = Some(face);
                    *b = Some(bold);
                });
            }
        }
        B_BOLD => apply(&target, |_, _, b| *b = Some(!effective.bold)),
        _ if (S_SIZE..S_SIZE + SIZES.len() as u32).contains(&id) => {
            if let Some(h) = note {
                super::set_font_size(h, SIZES[(id - S_SIZE) as usize].0);
            }
        }
        N_MAKE_DEFAULT => {
            // 這張的樣式變成預設，自己的設定清掉（之後跟著預設走）
            with(|m| {
                m.board.text = TextStyle { color: effective.color.filter(|c| *c != AUTO), ..effective.clone() };
                if let Some(n) = note.and_then(|h| m.note_of(h)) {
                    (n.face, n.color, n.bold) = (None, None, None);
                }
                store::save(&m.board);
            });
            super::refresh_all();
        }
        N_RESET => apply(&target, |f, c, b| (*f, *c, *b) = (None, None, None)),
        _ => {}
    }
    match note {
        Some(h) => {
            super::grow_to_fit(h);
            redraw(h);
        }
        None => super::refresh_all(),
    }
    true
}

