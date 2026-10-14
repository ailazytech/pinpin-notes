#![allow(dead_code)] // 共用的介面元件，有些這個程式沒用到
//! 便箋、翻譯視窗共用的外觀：跟著 Windows 的深色／淺色模式，用 GDI 自己畫圓角卡片和按鈕。
//! 不用任何介面框架，開視窗時不會多載入東西。
//!
//! 用法：
//! - 建立視窗後呼叫 [`apply_window`]（深色標題列、Windows 11 圓角）
//! - 文字框建立後呼叫 [`style_edit`]（深色捲軸、內距），文字框不要加 WS_EX_CLIENTEDGE
//! - WM_PAINT 用 [`paint_background`] 畫底色，再用 [`paint_card`] 在文字框後面畫圓角卡片
//! - WM_CTLCOLOREDIT／WM_CTLCOLORSTATIC 回傳 [`card_brush`]／[`bg_brush`]
//! - 按鈕用 BS_OWNERDRAW，WM_DRAWITEM 交給 [`draw_button`]

use std::cell::RefCell;

use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::Controls::{SetWindowTheme, DRAWITEMSTRUCT, EM_SETMARGINS, ODS_SELECTED};
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

use crate::win;

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: COLORREF,
    pub card: COLORREF,
    pub border: COLORREF,
    pub text: COLORREF,
    pub muted: COLORREF,
    pub accent: COLORREF,
    pub accent_pressed: COLORREF,
    pub subtle: COLORREF,
    pub subtle_pressed: COLORREF,
}

pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

const LIGHT: Palette = Palette {
    dark: false,
    bg: rgb(0xf3, 0xf4, 0xf8),
    card: rgb(0xff, 0xff, 0xff),
    border: rgb(0xe2, 0xe4, 0xec),
    text: rgb(0x11, 0x18, 0x27),
    muted: rgb(0x6b, 0x72, 0x80),
    accent: rgb(0x63, 0x5b, 0xf0),
    accent_pressed: rgb(0x4f, 0x46, 0xe5),
    subtle: rgb(0xe8, 0xe9, 0xf2),
    subtle_pressed: rgb(0xd9, 0xdb, 0xe8),
};

const DARK: Palette = Palette {
    dark: true,
    bg: rgb(0x1c, 0x1d, 0x22),
    card: rgb(0x26, 0x27, 0x2e),
    border: rgb(0x37, 0x38, 0x41),
    text: rgb(0xe8, 0xe9, 0xee),
    muted: rgb(0x9a, 0x9d, 0xa8),
    accent: rgb(0x7c, 0x6c, 0xf6),
    accent_pressed: rgb(0x63, 0x5b, 0xf0),
    subtle: rgb(0x31, 0x32, 0x3b),
    subtle_pressed: rgb(0x3d, 0x3e, 0x49),
};

struct Brushes {
    dark: bool,
    bg: HBRUSH,
    card: HBRUSH,
}

thread_local! {
    /// 文字框底色的筆刷要一直存在（WM_CTLCOLOR* 回傳後系統會拿去用），每種主題各建一次
    static BRUSHES: RefCell<Option<Brushes>> = const { RefCell::new(None) };
}

/// Windows「應用程式模式」是不是深色。
fn system_dark() -> bool {
    let mut value = 1u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    ok == ERROR_SUCCESS && value == 0
}

/// 目前要用的配色（每次開視窗時讀一次系統設定）。
pub fn palette() -> Palette {
    if system_dark() {
        DARK
    } else {
        LIGHT
    }
}

fn brushes() -> (HBRUSH, HBRUSH) {
    let p = palette();
    BRUSHES.with(|b| {
        let mut b = b.borrow_mut();
        if b.as_ref().is_none_or(|x| x.dark != p.dark) {
            if let Some(old) = b.take() {
                unsafe {
                    let _ = DeleteObject(old.bg.into());
                    let _ = DeleteObject(old.card.into());
                }
            }
            *b = Some(Brushes {
                dark: p.dark,
                bg: unsafe { CreateSolidBrush(p.bg) },
                card: unsafe { CreateSolidBrush(p.card) },
            });
        }
        let x = b.as_ref().expect("brushes");
        (x.bg, x.card)
    })
}

pub fn bg_brush() -> HBRUSH {
    brushes().0
}

pub fn card_brush() -> HBRUSH {
    brushes().1
}

pub fn px(v: f32) -> i32 {
    (v * win::dpi_scale()).round() as i32
}

/// 深色標題列（Windows 10 1809 以後）、Windows 11 圓角視窗。舊版系統不支援就略過。
pub fn apply_window(hwnd: HWND) {
    let p = palette();
    unsafe {
        let dark: i32 = p.dark as i32;
        // 20 = DWMWA_USE_IMMERSIVE_DARK_MODE；更舊的 Windows 10 是 19
        for attr in [20, 19] {
            if DwmSetWindowAttribute(
                hwnd,
                DWMWINDOWATTRIBUTE(attr),
                &dark as *const i32 as *const _,
                std::mem::size_of::<i32>() as u32,
            )
            .is_ok()
            {
                break;
            }
        }
        // 33 = DWMWA_WINDOW_CORNER_PREFERENCE，2 = 圓角
        let round: u32 = 2;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWINDOWATTRIBUTE(33),
            &round as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// 文字框：深色模式下捲軸也變深色，左右留一點內距。
pub fn style_edit(edit: HWND) {
    let p = palette();
    unsafe {
        if p.dark {
            let _ = SetWindowTheme(edit, w!("DarkMode_Explorer"), None);
        } else {
            let _ = SetWindowTheme(edit, w!("Explorer"), None);
        }
        // EC_LEFTMARGIN | EC_RIGHTMARGIN
        let m = px(4.0) as usize;
        SendMessageW(edit, EM_SETMARGINS, Some(WPARAM(0x1 | 0x2)), Some(LPARAM((m | (m << 16)) as isize)));
    }
}

/// 下拉選單等原生控制項跟著深色模式。
pub fn style_native(control: HWND) {
    if palette().dark {
        unsafe {
            let _ = SetWindowTheme(control, w!("DarkMode_CFD"), None);
        }
    }
}

pub fn paint_background(hdc: HDC, rc: &RECT) {
    unsafe {
        FillRect(hdc, rc, bg_brush());
    }
}

unsafe fn round_rect(hdc: HDC, rc: &RECT, fill: COLORREF, border: Option<COLORREF>, radius: i32) {
    let brush = CreateSolidBrush(fill);
    let pen = match border {
        Some(c) => CreatePen(PS_SOLID, 1, c),
        None => CreatePen(PS_NULL, 0, COLORREF(0)),
    };
    let ob = SelectObject(hdc, brush.into());
    let op = SelectObject(hdc, pen.into());
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, ob);
    SelectObject(hdc, op);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

/// 在控制項後面畫一張圓角卡片：`inner` 是控制項的位置，卡片往外多 `pad`。
pub fn paint_card(hdc: HDC, inner: &RECT, pad: i32) {
    let p = palette();
    let rc = RECT {
        left: inner.left - pad,
        top: inner.top - pad,
        right: inner.right + pad,
        bottom: inner.bottom + pad,
    };
    unsafe {
        round_rect(hdc, &rc, p.card, Some(p.border), px(12.0));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// 主要動作：紫色實心
    Primary,
    /// 其他動作：淡色
    Secondary,
    /// 語言標籤：開的時候是紫色，關的時候是淡色
    Chip { on: bool },
}

/// 畫 BS_OWNERDRAW 按鈕（WM_DRAWITEM）。
pub fn draw_button(dis: &DRAWITEMSTRUCT, kind: ButtonKind, label: &str, font: HFONT) {
    let p = palette();
    let pressed = dis.itemState.0 & ODS_SELECTED.0 != 0;
    let (fill, text) = match kind {
        ButtonKind::Primary | ButtonKind::Chip { on: true } => {
            (if pressed { p.accent_pressed } else { p.accent }, rgb(0xff, 0xff, 0xff))
        }
        ButtonKind::Secondary | ButtonKind::Chip { on: false } => {
            (if pressed { p.subtle_pressed } else { p.subtle }, p.text)
        }
    };
    let rc = dis.rcItem;
    let radius = match kind {
        ButtonKind::Chip { .. } => rc.bottom - rc.top,
        _ => px(10.0),
    };
    unsafe {
        let hdc = dis.hDC;
        // 先用視窗底色清掉圓角外面的區域
        FillRect(hdc, &rc, bg_brush());
        round_rect(hdc, &rc, fill, None, radius);
        let old = SelectObject(hdc, font.into());
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, text);
        let mut label: Vec<u16> = label.encode_utf16().collect();
        let mut r = rc;
        DrawTextW(hdc, &mut label, &mut r, DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX);
        SelectObject(hdc, old);
    }
}

/// 一般文字和粗體兩種字型。
pub fn fonts(size: f32) -> (HFONT, HFONT) {
    let make = |weight: i32| unsafe {
        CreateFontW(
            -px(size),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Microsoft JhengHei UI"),
        )
    };
    (make(FW_NORMAL.0 as i32), make(FW_SEMIBOLD.0 as i32))
}

// ───────────────────────────── 自訂標題列 ─────────────────────────────
//
// 視窗用 WS_POPUP | WS_THICKFRAME，WM_NCCALCSIZE 回傳 0 讓整個視窗都變成工作區，
// 標題列自己畫；WM_NCHITTEST 告訴系統哪裡是標題（可以拖曳、雙擊）、哪裡是邊框（可以調大小），
// 保留系統的陰影、Aero Snap。

/// 標題列高度
pub const TITLE_H: f32 = 38.0;
/// 邊框可以拖曳調整大小的寬度
const GRIP: f32 = 6.0;

/// 建立視窗後呼叫：去掉系統邊框但保留陰影。
pub fn setup_chrome(hwnd: HWND) {
    use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
    use windows::Win32::UI::Controls::MARGINS;
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER};
    unsafe {
        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER);
    }
}

/// 標題列右邊的關閉按鈕位置（工作區座標）。
pub fn close_rect(hwnd: HWND) -> RECT {
    let mut rc = RECT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rc);
    }
    RECT {
        left: rc.right - px(46.0),
        top: 0,
        right: rc.right,
        bottom: px(TITLE_H),
    }
}

fn contains(rc: &RECT, x: i32, y: i32) -> bool {
    x >= rc.left && x < rc.right && y >= rc.top && y < rc.bottom
}

/// WM_NCHITTEST：邊框 → 調大小；標題列（關閉按鈕以外）→ 拖曳；其他 → 工作區。
pub fn hit_test(hwnd: HWND, lp: LPARAM) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::*;
    let pt = POINT {
        x: (lp.0 & 0xffff) as i16 as i32,
        y: ((lp.0 >> 16) & 0xffff) as i16 as i32,
    };
    let rc = win::window_rect(hwnd);
    let g = px(GRIP);
    let (l, r, t, b) = (pt.x < rc.left + g, pt.x >= rc.right - g, pt.y < rc.top + g, pt.y >= rc.bottom - g);
    let code = match (l, r, t, b) {
        (true, _, true, _) => HTTOPLEFT,
        (_, true, true, _) => HTTOPRIGHT,
        (true, _, _, true) => HTBOTTOMLEFT,
        (_, true, _, true) => HTBOTTOMRIGHT,
        (true, _, _, _) => HTLEFT,
        (_, true, _, _) => HTRIGHT,
        (_, _, true, _) => HTTOP,
        (_, _, _, true) => HTBOTTOM,
        _ => {
            let (x, y) = (pt.x - rc.left, pt.y - rc.top);
            if y < px(TITLE_H) && !contains(&close_rect(hwnd), x, y) {
                HTCAPTION
            } else {
                HTCLIENT
            }
        }
    };
    LRESULT(code as isize)
}

/// 畫標題列：小圖示、標題文字、關閉按鈕（滑過時變紅）。
pub fn paint_title(hdc: HDC, hwnd: HWND, title: &str, icon: windows::Win32::UI::WindowsAndMessaging::HICON, font: HFONT, close_hot: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};
    let p = palette();
    let close = close_rect(hwnd);
    unsafe {
        let size = px(18.0);
        let top = (px(TITLE_H) - size) / 2;
        let _ = DrawIconEx(hdc, px(12.0), top, icon, size, size, 0, None, DI_NORMAL);
        let old = SelectObject(hdc, font.into());
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, p.text);
        let mut text: Vec<u16> = title.encode_utf16().collect();
        let mut r = RECT {
            left: px(38.0),
            top: 0,
            right: close.left,
            bottom: px(TITLE_H),
        };
        DrawTextW(hdc, &mut text, &mut r, DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
        SelectObject(hdc, old);

        if close_hot {
            let red = CreateSolidBrush(rgb(0xe8, 0x11, 0x23));
            FillRect(hdc, &close, red);
            let _ = DeleteObject(red.into());
        }
        // 關閉的 ✕ 用兩條線畫，任何字型都一樣
        let color = if close_hot { rgb(0xff, 0xff, 0xff) } else { p.muted };
        let pen = CreatePen(PS_SOLID, px(1.0).max(1), color);
        let op = SelectObject(hdc, pen.into());
        let (cx, cy, h) = ((close.left + close.right) / 2, (close.top + close.bottom) / 2, px(5.0));
        let _ = MoveToEx(hdc, cx - h, cy - h, None);
        let _ = LineTo(hdc, cx + h + 1, cy + h + 1);
        let _ = MoveToEx(hdc, cx + h, cy - h, None);
        let _ = LineTo(hdc, cx - h - 1, cy + h + 1);
        SelectObject(hdc, op);
        let _ = DeleteObject(pen.into());
    }
}

/// 處理標題列關閉按鈕的滑鼠事件。回傳 Some(新的 hover 狀態) 表示要重畫；點到關閉時送 WM_CLOSE。
pub fn close_button_mouse(hwnd: HWND, msg: u32, lp: LPARAM, hot: bool) -> Option<bool> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE, WM_LBUTTONUP, WM_MOUSEMOVE};
    const WM_MOUSELEAVE: u32 = 0x02A3;
    let (x, y) = ((lp.0 & 0xffff) as i16 as i32, ((lp.0 >> 16) & 0xffff) as i16 as i32);
    let inside = contains(&close_rect(hwnd), x, y);
    match msg {
        WM_MOUSEMOVE => {
            if inside {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                unsafe {
                    let _ = TrackMouseEvent(&mut tme);
                }
            }
            (inside != hot).then_some(inside)
        }
        WM_MOUSELEAVE => hot.then_some(false),
        WM_LBUTTONUP if inside => {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            None
        }
        _ => None,
    }
}

/// 按鈕上的文字（WM_DRAWITEM 時從控制項讀回來）。
pub fn window_text(hwnd: HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        let mut buf = vec![0u16; len as usize + 1];
        let n = GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}
