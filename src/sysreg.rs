//! 登錄檔（只寫目前使用者，不需要系統管理員權限）：開機啟動、桌面右鍵選單的「新增便利貼」「新增蕃茄鐘」。

use std::ffi::c_void;
use std::path::Path;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::*;

use crate::win::wide;

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE: PCWSTR = w!("PinPin");
/// 改名前（PinBoard）寫的開機啟動
const OLD_RUN_VALUE: PCWSTR = w!("PinBoard");
const MENU_ROOT: &str = "Software\\Classes\\DesktopBackground\\Shell\\";
/// 舊版寫的選單：還在 Vibotype 裡的時候、改名前（PinBoard）
const OLD_MENU_KEYS: [PCWSTR; 3] = [
    w!("Software\\Classes\\DesktopBackground\\Shell\\VibotypeNewNote"),
    w!("Software\\Classes\\DesktopBackground\\Shell\\PinBoardNewNote"),
    w!("Software\\Classes\\DesktopBackground\\Shell\\PinBoardNewTimer"),
];

fn set_string(key: PCWSTR, name: PCWSTR, value: &str) -> Result<(), String> {
    let value: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let err = unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, key, name, REG_SZ.0, Some(value.as_ptr() as *const c_void), (value.len() * 2) as u32) };
    if err != ERROR_SUCCESS {
        return Err(format!("registry write failed ({err:?})"));
    }
    Ok(())
}

pub fn set_autostart(enable: bool, exe: &Path) -> Result<(), String> {
    unsafe {
        let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, OLD_RUN_VALUE);
    }
    if enable {
        set_string(RUN_KEY, RUN_VALUE, &format!("\"{}\"", exe.display()))
    } else {
        unsafe {
            let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE);
        }
        Ok(())
    }
}

/// 桌面空白處的右鍵選單（Windows 11 在「顯示更多選項」裡）。
/// `items` = (登錄檔的鍵名, 顯示文字（跟著介面語言）, 命令列參數)。
pub fn set_desktop_menu(enable: bool, exe: &Path, items: &[(&str, String, &str)]) -> Result<(), String> {
    unsafe {
        for key in OLD_MENU_KEYS {
            let _ = RegDeleteTreeW(HKEY_CURRENT_USER, key);
            let _ = RegDeleteKeyW(HKEY_CURRENT_USER, key);
        }
    }
    for (name, label, arg) in items {
        let key = wide(&format!("{MENU_ROOT}{name}"));
        let key = PCWSTR(key.as_ptr());
        if enable {
            let command = wide(&format!("{MENU_ROOT}{name}\\command"));
            set_string(key, PCWSTR::null(), label)?;
            set_string(key, w!("Icon"), &format!("\"{}\",0", exe.display()))?;
            set_string(PCWSTR(command.as_ptr()), PCWSTR::null(), &format!("\"{}\" {arg}", exe.display()))?;
        } else {
            unsafe {
                let _ = RegDeleteTreeW(HKEY_CURRENT_USER, key);
                let _ = RegDeleteKeyW(HKEY_CURRENT_USER, key);
            }
        }
    }
    Ok(())
}
