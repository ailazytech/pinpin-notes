//! 零碎的 Win32 小工具：字串轉換、選單、圖示、開機啟動、螢幕工作區、DPI。


use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Rust 字串 → 以 0 結尾的 UTF-16（Win32 API 要的格式）。
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 把字串複製進固定長度的 UTF-16 陣列（例如系統匣提示文字），太長就截斷。
pub fn copy_wide(dst: &mut [u16], s: &str) {
    let src: Vec<u16> = s.encode_utf16().take(dst.len().saturating_sub(1)).collect();
    dst[..src.len()].copy_from_slice(&src);
    dst[src.len()] = 0;
}

/// 用預設瀏覽器（或郵件程式）開網址。
pub fn open_url(url: &str) {
    let url = wide(url);
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, windows::core::w!("open"), PCWSTR(url.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
    }
}

pub fn hinstance() -> HINSTANCE {
    unsafe { GetModuleHandleW(None).map(|m| m.into()).unwrap_or_default() }
}

pub fn loword(v: usize) -> u32 {
    (v & 0xffff) as u32
}

pub fn cursor_pos() -> POINT {
    let mut pt = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut pt);
    }
    pt
}

pub fn window_rect(hwnd: HWND) -> RECT {
    let mut rc = RECT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut rc);
    }
    rc
}

/// 目前前景視窗所在螢幕的工作區（扣掉工作列）；沒有前景視窗就用滑鼠所在的螢幕。
pub fn active_work_area() -> RECT {
    unsafe {
        let fg = GetForegroundWindow();
        let mon = if fg.is_invalid() {
            MonitorFromPoint(cursor_pos(), MONITOR_DEFAULTTONEAREST)
        } else {
            MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST)
        };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(mon, &mut info).as_bool() {
            info.rcWork
        } else {
            RECT {
                left: 0,
                top: 0,
                right: 1280,
                bottom: 720,
            }
        }
    }
}

/// 系統 DPI 縮放比例（100% = 1.0）。
pub fn dpi_scale() -> f32 {
    unsafe { GetDpiForSystem().max(96) as f32 / 96.0 }
}

// ───────────────────────────── 選單 ─────────────────────────────

pub enum MenuItem {
    Item {
        id: u32,
        text: String,
        enabled: bool,
        checked: bool,
    },
    Separator,
    Submenu {
        text: String,
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    pub fn item(id: u32, text: impl Into<String>) -> Self {
        MenuItem::Item {
            id,
            text: text.into(),
            enabled: true,
            checked: false,
        }
    }
    pub fn checked(id: u32, text: impl Into<String>, checked: bool) -> Self {
        MenuItem::Item {
            id,
            text: text.into(),
            enabled: true,
            checked,
        }
    }
    pub fn disabled(text: impl Into<String>) -> Self {
        MenuItem::Item {
            id: 0,
            text: text.into(),
            enabled: false,
            checked: false,
        }
    }
}

unsafe fn build_menu(items: &[MenuItem]) -> windows::core::Result<HMENU> {
    let menu = CreatePopupMenu()?;
    for it in items {
        match it {
            MenuItem::Item {
                id,
                text,
                enabled,
                checked,
            } => {
                let mut flags = MF_STRING;
                if !*enabled {
                    flags |= MF_GRAYED;
                }
                if *checked {
                    flags |= MF_CHECKED;
                }
                let t = wide(text);
                AppendMenuW(menu, flags, *id as usize, PCWSTR(t.as_ptr()))?;
            }
            MenuItem::Separator => {
                AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            }
            MenuItem::Submenu { text, items } => {
                let sub = build_menu(items)?;
                let t = wide(text);
                AppendMenuW(menu, MF_POPUP, sub.0 as usize, PCWSTR(t.as_ptr()))?;
            }
        }
    }
    Ok(menu)
}

/// 在指定位置跳出選單，回傳使用者點的項目 id（0 = 取消）。
///
/// 注意：選單開著的時候會跑自己的訊息迴圈，呼叫前不要持有 App 的借用。
pub fn popup_menu(owner: HWND, items: &[MenuItem], pt: POINT) -> u32 {
    unsafe {
        let Ok(menu) = build_menu(items) else {
            return 0;
        };
        // 經典的系統匣選單修正：沒有先 SetForegroundWindow，點選單外面時選單不會關掉
        let _ = SetForegroundWindow(owner);
        // 由右到左的語言：選單整個左右鏡像
        let rtl = if crate::i18n::is_rtl() { TPM_LAYOUTRTL } else { TRACK_POPUP_MENU_FLAGS(0) };
        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY | rtl,
            pt.x,
            pt.y,
            None,
            owner,
            None,
        );
        let _ = PostMessageW(Some(owner), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        cmd.0 as u32
    }
}

// ───────────────────────────── 圖示 ─────────────────────────────

/// 執行檔裡的圖示（pinpin.rc 的第 1 號），指定大小。
pub fn app_icon(size: i32) -> HICON {
    unsafe {
        LoadImageW(Some(hinstance()), PCWSTR(1 as *const u16), IMAGE_ICON, size, size, LR_DEFAULTCOLOR)
            .map(|h| HICON(h.0))
            .unwrap_or_else(|_| LoadIconW(None, IDI_APPLICATION).unwrap_or_default())
    }
}
