//! 編輯框和密碼框：一個沒有邊框的小視窗蓋在紙上，底色和紙一樣，看起來像直接在紙上寫字。
//!
//! 分層視窗不能放子控制項，所以另外開一個普通視窗放 EDIT。

use std::cell::{Cell, RefCell};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Controls::{EM_SETMARGINS, EM_SETSEL};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::win::{self, wide};
use super::{px, redraw, render, store, with, Kind, Line};

const CLASS: PCWSTR = w!("PinPin.NoteEdit");
/// 失去焦點、按 Esc、Enter 時送給編輯視窗：收起並處理
const WM_DONE: u32 = WM_APP + 60;

#[derive(Clone, PartialEq, Eq)]
pub enum PromptMode {
    Unlock,
    SetNew,
    /// 再輸入一次（第一次輸入的密碼）
    Confirm(String),
}

#[derive(Clone)]
enum What {
    Text,
    Password(PromptMode),
}

struct Session {
    popup: HWND,
    edit: HWND,
    owner: HWND,
    what: What,
    brush: HBRUSH,
    ink: COLORREF,
    paper: COLORREF,
    /// 密碼框上面的說明
    label: String,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static LABEL_FONT: Cell<Option<HFONT>> = const { Cell::new(None) };
}

pub fn register() {
    unsafe {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: win::hinstance(),
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_IBEAM).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
    }
}

fn session<R>(f: impl FnOnce(&Session) -> R) -> Option<R> {
    SESSION.with(|s| s.try_borrow().ok().and_then(|s| s.as_ref().map(f)))
}

/// 開始編輯文字。
pub fn begin(hwnd: HWND) {
    close();
    let Some((note, lines, area, font)) = with(|m| {
        let w = m.win(hwnd)?;
        w.editing = true;
        let lines = w.unlocked.as_ref().map(|u| u.lines.clone());
        let paper = w.hits.layout.paper;
        let n = m.note_of(hwnd)?.clone();
        if matches!(n.kind, Kind::Sticker | Kind::Timer) {
            return None;
        }
        let key = m.fonts_for(&n);
        let s = m.scale;
        let area = if n.kind == Kind::Image {
            m.win(hwnd)?.hits.text_area
        } else {
            render::edit_area(&n, paper, s)
        };
        let lines = lines.unwrap_or_else(|| n.lines.clone());
        Some((n, lines, area, m.fonts[&key].body))
    })
    .flatten() else {
        return;
    };
    // 有密碼又沒解開：不能直接改偽裝內容，改問密碼
    if note.locked() && with(|m| m.win(hwnd).is_some_and(|w| w.unlocked.is_none())).unwrap_or(true) {
        with(|m| {
            if let Some(w) = m.win(hwnd) {
                w.editing = false;
            }
        });
        prompt(hwnd, PromptMode::Unlock);
        return;
    }
    redraw(hwnd);
    let multi = note.kind != Kind::Image;
    let text = if multi { store::Note::text_of(&lines) } else { lines.first().map(|l| l.text.clone()).unwrap_or_default() };
    open(hwnd, area, What::Text, &text, multi, font, &note);
}

/// 跳出密碼框（蓋在文字區上方）。
pub fn prompt(hwnd: HWND, mode: PromptMode) {
    close();
    let Some((note, paper)) = with(|m| {
        let paper = m.win(hwnd)?.hits.layout.paper;
        Some((m.note_of(hwnd)?.clone(), paper))
    })
    .flatten() else {
        return;
    };
    let s = win::dpi_scale();
    let area = RECT {
        left: paper.left + px(14.0),
        top: paper.top + px(22.0),
        right: paper.right - px(14.0),
        bottom: paper.top + px(22.0) + px(62.0),
    };
    let _ = s;
    let font = with(|m| {
        let key = m.fonts_key(1.0, "", false);
        m.fonts[&key].body
    })
    .unwrap_or_default();
    open(hwnd, area, What::Password(mode), "", false, font, &note);
}

#[allow(clippy::too_many_arguments)]
fn open(owner: HWND, area: RECT, what: What, text: &str, multi: bool, font: HFONT, note: &store::Note) {
    unsafe {
        let rc = win::window_rect(owner);
        let (x, y) = (rc.left + area.left, rc.top + area.top);
        let (w, h) = (area.right - area.left, area.bottom - area.top);
        let Ok(popup) = CreateWindowExW(WS_EX_TOOLWINDOW, CLASS, w!(""), WS_POPUP, x, y, w, h, Some(owner), None, Some(win::hinstance()), None) else {
            return;
        };
        let password = matches!(what, What::Password(_));
        let mut style = WS_CHILD | WS_VISIBLE | WINDOW_STYLE((ES_AUTOVSCROLL | ES_NOHIDESEL) as u32);
        if multi {
            style |= WINDOW_STYLE((ES_MULTILINE | ES_WANTRETURN) as u32);
        } else if password {
            style |= WINDOW_STYLE((ES_PASSWORD | ES_AUTOHSCROLL) as u32);
        } else {
            style |= WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL) as u32);
        }
        let label_h = if password { px(24.0) } else { 0 };
        let edit_h = if multi { h } else { px(24.0) };
        let top = if multi { 0 } else if password { label_h + px(6.0) } else { (h - edit_h).max(0) / 2 };
        let t = wide(text);
        // 由右到左：內容是阿拉伯文、希伯來文，或介面語言是由右到左
        let rtl = crate::i18n::text_is_rtl(text) || (text.trim().is_empty() && crate::i18n::is_rtl());
        let ex = if rtl { WS_EX_RTLREADING | WS_EX_RIGHT | WS_EX_LEFTSCROLLBAR } else { WINDOW_EX_STYLE(0) };
        let Ok(edit) = CreateWindowExW(ex, w!("EDIT"), PCWSTR(t.as_ptr()), style, 0, top, w, edit_h, Some(popup), None, Some(win::hinstance()), None) else {
            let _ = DestroyWindow(popup);
            return;
        };
        SendMessageW(edit, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
        SendMessageW(edit, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM(0)));
        let paper = render::paper_colorref(note);
        let label = match &what {
            What::Password(PromptMode::Unlock) => crate::i18n::t("prompt.unlock"),
            What::Password(PromptMode::SetNew) => crate::i18n::t("prompt.set"),
            What::Password(PromptMode::Confirm(_)) => crate::i18n::t("prompt.confirm"),
            What::Text => String::new(),
        };
        SESSION.with(|s| {
            *s.borrow_mut() = Some(Session {
                popup,
                edit,
                owner,
                what,
                brush: CreateSolidBrush(paper),
                ink: with(|m| m.text_color(note)).flatten().map_or_else(|| render::ink_colorref(note), |c| c.colorref()),
                paper,
                label,
            })
        });
        if LABEL_FONT.get().is_none() {
            LABEL_FONT.set(Some(CreateFontW(-px(12.0), 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0, w!("Microsoft JhengHei UI"))));
        }
        let _ = ShowWindow(popup, SW_SHOW);
        let _ = SetForegroundWindow(popup);
        let _ = SetFocus(Some(edit));
        SendMessageW(edit, EM_SETSEL, Some(WPARAM(usize::MAX)), Some(LPARAM(-1)));
    }
}

/// 有編輯框的話收起（存檔），沒有就什麼都不做。
pub fn close() {
    let Some(s) = SESSION.with(|s| s.borrow_mut().take()) else { return };
    let text = crate::theme::window_text(s.edit);
    unsafe {
        let _ = DestroyWindow(s.popup);
        let _ = DeleteObject(s.brush.into());
    }
    match s.what {
        What::Text => commit_text(s.owner, &text),
        // 密碼框按外面就當取消
        What::Password(_) => {}
    }
}

/// 某張便利貼要關了：它的編輯框一起收。
pub fn close_for(owner: HWND) {
    if session(|s| s.owner == owner) == Some(true) {
        close();
    }
}

fn commit_text(hwnd: HWND, text: &str) {
    let kind = with(|m| {
        if let Some(w) = m.win(hwnd) {
            w.editing = false;
        }
        m.note_of(hwnd).map(|n| n.kind)
    })
    .flatten();
    let Some(kind) = kind else { return };
    super::update_lines(hwnd, |lines| {
        if kind == Kind::Image {
            let caption = text.trim().to_string();
            *lines = if caption.is_empty() { vec![] } else { vec![Line::new(caption)] };
        } else {
            *lines = store::merge_lines(lines, text);
        }
    });
    // 沒寫字也留著，要不要刪由使用者決定
    redraw(hwnd);
}

/// 密碼框按 Enter。
fn submit_password() {
    let Some((owner, mode, edit)) = session(|s| match &s.what {
        What::Password(m) => Some((s.owner, m.clone(), s.edit)),
        _ => None,
    })
    .flatten() else {
        return;
    };
    let pw = crate::theme::window_text(edit);
    if pw.is_empty() {
        return;
    }
    match mode {
        PromptMode::Unlock => {
            if super::unlock(owner, &pw) {
                close();
            } else {
                unsafe {
                    SetWindowTextW(edit, w!("")).ok();
                }
            }
        }
        PromptMode::SetNew => {
            close();
            prompt(owner, PromptMode::Confirm(pw));
        }
        PromptMode::Confirm(first) => {
            close();
            if first == pw {
                super::set_password(owner, &pw);
            } else {
                super::start_anim(owner, super::AnimKind::Shake);
                prompt(owner, PromptMode::SetNew);
            }
        }
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CTLCOLOREDIT => {
            let hdc = HDC(wp.0 as *mut _);
            if let Some((brush, ink, paper)) = session(|s| (s.brush, s.ink, s.paper)) {
                SetTextColor(hdc, ink);
                SetBkColor(hdc, paper);
                return LRESULT(brush.0 as isize);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            if let Some((brush, ink, label)) = session(|s| (s.brush, s.ink, s.label.clone())) {
                FillRect(hdc, &rc, brush);
                if !label.is_empty() {
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, ink);
                    if let Some(f) = LABEL_FONT.get() {
                        SelectObject(hdc, f.into());
                    }
                    let mut t: Vec<u16> = label.encode_utf16().collect();
                    let mut r = RECT { left: px(3.0), top: 0, right: rc.right, bottom: px(24.0) };
                    let align = if crate::i18n::is_rtl() { DT_RIGHT | DT_RTLREADING } else { DT_LEFT };
                    DrawTextW(hdc, &mut t, &mut r, align | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                    // 密碼框底下一條線，看起來像紙上的填空欄
                    let line = RECT { left: 0, top: rc.bottom - px(4.0), right: rc.right, bottom: rc.bottom - px(4.0) + 1 };
                    let pen_brush = CreateSolidBrush(ink);
                    FillRect(hdc, &line, pen_brush);
                    let _ = DeleteObject(pen_brush.into());
                }
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_COMMAND if (wp.0 >> 16) as u32 == EN_KILLFOCUS => {
            let _ = PostMessageW(Some(hwnd), WM_DONE, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }
        WM_DONE => {
            // 焦點跑到別的地方：收起（密碼框就是取消）
            if session(|s| s.popup == hwnd) == Some(true) && GetFocus() != session(|s| s.edit).unwrap_or_default() {
                close();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// 主迴圈先問這裡：Esc 收起、Enter（密碼）送出、Ctrl+Enter（文字）收起、Ctrl+A 全選、Ctrl+V 貼圖片。
pub fn pre_translate(msg: &MSG) -> bool {
    let Some((edit, owner, is_password)) = session(|s| (s.edit, s.owner, matches!(s.what, What::Password(_)))) else { return false };
    if msg.hwnd != edit || msg.message != WM_KEYDOWN {
        return false;
    }
    let ctrl = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
    match VIRTUAL_KEY(msg.wParam.0 as u16) {
        VK_ESCAPE => {
            close();
            true
        }
        VK_RETURN if is_password => {
            submit_password();
            true
        }
        VK_RETURN if ctrl => {
            close();
            true
        }
        VK_A if ctrl => {
            unsafe {
                SendMessageW(edit, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
            }
            true
        }
        VK_V if ctrl && !is_password && !super::image::clipboard_has_text() => {
            // 剪貼簿是圖片：在旁邊另外貼一張圖片便利貼
            if let Some(p) = super::image::clipboard_image(owner) {
                let rc = win::window_rect(owner);
                super::new_image(p, Some(POINT { x: rc.right + px(120.0), y: (rc.top + rc.bottom) / 2 }));
                return true;
            }
            false
        }
        _ => false,
    }
}
