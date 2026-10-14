// PinPin Notes（貼貼）：Windows 桌面便利貼。
// Release 版不要跳出黑色的主控台視窗；Debug 版保留，方便看 panic 訊息
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod base;
mod i18n;
mod notes;
mod sysreg;
mod theme;
mod toast;
mod tray;
mod update;
mod win;

fn main() {
    app::run();
}
