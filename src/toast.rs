//! 小提示：螢幕下方中間浮出一顆膠囊（「已複製」「已匯出 3 張」），幾秒後自己消失。
//! 分層視窗、不搶焦點、滑鼠穿透；只有顯示時才有計時器。

use std::cell::Cell;

use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::win;

const TIMER_HIDE: usize = 1;

thread_local! {
    static HWND_CELL: Cell<Option<HWND>> = const { Cell::new(None) };
}

fn window() -> Option<HWND> {
    if let Some(h) = HWND_CELL.get() {
        return Some(h);
    }
    unsafe {
        let wc = WNDCLASSW { lpfnWndProc: Some(proc), hInstance: win::hinstance(), lpszClassName: w!("PinPin.Toast"), ..Default::default() };
        RegisterClassW(&wc);
        let h = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            w!("PinPin.Toast"),
            w!(""),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(win::hinstance()),
            None,
        )
        .ok()?;
        HWND_CELL.set(Some(h));
        Some(h)
    }
}

/// 顯示一段提示 `secs` 秒。`error` = 紅色。
pub fn show(text: impl AsRef<str>, error: bool, secs: u32) {
    let Some(hwnd) = window() else { return };
    let text: Vec<u16> = text.as_ref().encode_utf16().collect();
    let s = win::dpi_scale();
    let px = |v: f32| (v * s).round() as i32;
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let font = CreateFontW(-px(14.0), 0, 0, 0, 600, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0, w!("Microsoft JhengHei UI"));
        let old_font = SelectObject(mem, font.into());
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(mem, &text, &mut size);
        let (w, h) = (size.cx + px(36.0), px(38.0));
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
        if let Ok(dib) = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
            let old_bmp = SelectObject(mem, dib.into());
            let buf = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
            // 膠囊底色（不透明），之後 GDI 寫字、再把 alpha 補回
            let bg: [u8; 3] = if error { [48, 48, 200] } else { [58, 46, 40] };
            let r = h as f32 / 2.0;
            let mut cover = vec![0f32; (w * h) as usize];
            for y in 0..h {
                for x in 0..w {
                    let cx = (x as f32 + 0.5).clamp(r, w as f32 - r);
                    let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - r).powi(2)).sqrt();
                    let c = (r - d + 0.5).clamp(0.0, 1.0);
                    cover[(y * w + x) as usize] = c;
                    let i = ((y * w + x) * 4) as usize;
                    buf[i..i + 3].copy_from_slice(&bg);
                    buf[i + 3] = 255;
                }
            }
            SetBkMode(mem, TRANSPARENT);
            SetTextColor(mem, COLORREF(0x00FFFFFF));
            let mut t = text.clone();
            let mut rc = RECT { left: 0, top: 0, right: w, bottom: h };
            let align = if crate::i18n::is_rtl() { DT_RTLREADING } else { DRAW_TEXT_FORMAT(0) };
            DrawTextW(mem, &mut t, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | align);
            // 預乘：顏色乘上圓角的覆蓋率
            for (k, c) in cover.iter().enumerate() {
                let a = c * 0.94;
                let i = k * 4;
                for ch in 0..3 {
                    buf[i + ch] = (buf[i + ch] as f32 * a) as u8;
                }
                buf[i + 3] = (a * 255.0) as u8;
            }
            let wa = win::active_work_area();
            let pos = POINT { x: wa.left + (wa.right - wa.left - w) / 2, y: wa.bottom - h - px(48.0) };
            let sz = SIZE { cx: w, cy: h };
            let src = POINT::default();
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
            let _ = UpdateLayeredWindow(hwnd, Some(screen), Some(&pos), Some(&sz), Some(mem), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
            SelectObject(mem, old_bmp);
            let _ = DeleteObject(dib.into());
        }
        SelectObject(mem, old_font);
        let _ = DeleteObject(font.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        SetTimer(Some(hwnd), TIMER_HIDE, secs.max(1) * 1000, None);
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_TIMER && wp.0 == TIMER_HIDE {
        let _ = KillTimer(Some(hwnd), TIMER_HIDE);
        let _ = ShowWindow(hwnd, SW_HIDE);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
