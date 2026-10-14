#![allow(dead_code)] // 共用的介面元件，有些這個程式沒用到
//! 右下角系統匣圖示。直接用 Shell_NotifyIconW，不需要額外的 crate。

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::{HICON, WM_APP};

use crate::win::copy_wide;

/// 系統匣事件會以這個訊息送到主視窗，LPARAM 是滑鼠訊息（WM_LBUTTONUP 等）。
pub const WM_TRAY: u32 = WM_APP + 1;

const TRAY_ID: u32 = 1;

fn base(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

pub fn add(hwnd: HWND, icon: HICON, tip: &str) -> bool {
    let mut nid = base(hwnd);
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = icon;
    copy_wide(&mut nid.szTip, tip);
    unsafe { Shell_NotifyIconW(NIM_ADD, &nid).as_bool() }
}

pub fn set_icon(hwnd: HWND, icon: HICON) {
    let mut nid = base(hwnd);
    nid.uFlags = NIF_ICON;
    nid.hIcon = icon;
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

pub fn set_tip(hwnd: HWND, tip: &str) {
    let mut nid = base(hwnd);
    nid.uFlags = NIF_TIP;
    copy_wide(&mut nid.szTip, tip);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

/// 右下角跳出通知（Windows 10/11 會變成通知中心的訊息）。
pub fn notify(hwnd: HWND, title: &str, text: &str) {
    let mut nid = base(hwnd);
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;
    copy_wide(&mut nid.szInfoTitle, title);
    copy_wide(&mut nid.szInfo, text);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

pub fn remove(hwnd: HWND) {
    let nid = base(hwnd);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}
