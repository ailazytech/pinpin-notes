//! 常駐主程式：看不見的主視窗負責系統匣、單一執行個體、和其他程式（Vibotype、桌面右鍵）的溝通。
//!
//! 其他程式要新增便利貼或加星號：找到 `PinPin.Main` 視窗（舊名 `PinBoard.Main` 也還收得到），送 WM_COPYDATA（dwData = COPYDATA_TAG），
//! 內容是 UTF-8 JSON：`{"cmd":"note"|"star","text":"…","source":"chrome.exe","title":"…","x":100,"y":200}`。

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

use serde::{Deserialize, Serialize};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::base::{data_dir, log_error, write_atomic};
use crate::i18n::{t, tf};
use crate::win::{self, cursor_pos, MenuItem};
use crate::{i18n, notes, sysreg, tray, update};

pub const MAIN_CLASS: PCWSTR = w!("PinPin.Main");
/// 改名前的主視窗類別。Vibotype 舊版用它找我們，留一個隱藏的分身視窗接訊息，等 Vibotype 改完再拿掉
const LEGACY_CLASS: PCWSTR = w!("PinBoard.Main");
/// WM_COPYDATA 的識別碼（"PBN1"）
pub const COPYDATA_TAG: usize = 0x5042_4E31;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    /// 介面語言（空字串 = 跟隨系統）
    lang: String,
    autostart: bool,
    desktop_menu: bool,
    /// 自動檢查更新（啟動後、開貼紙小舖時；有新版只在選單顯示，不跳提示）
    update_auto: bool,
    /// 接收推送：線上廣告、新貼紙包、重要更新（自動下載、下次啟動套用）
    update_push: bool,
    /// 匿名使用統計：每天那一次檢查更新時順便帶上版本、語言、Windows 版本、隨機安裝碼（見 docs/PRIVACY.md）
    usage_stats: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // 開機啟動、桌面右鍵會寫登錄檔，預設關著，使用者在系統匣自己打開
        Settings { lang: String::new(), autostart: false, desktop_menu: false, update_auto: true, update_push: true, usage_stats: true }
    }
}

/// 匿名使用統計開著嗎（update.rs 在主執行緒組網址時問）。
pub fn usage_stats_enabled() -> bool {
    SETTINGS.with(|s| s.borrow().usage_stats)
}

/// 排下一次自動檢查：一天最多一次，用一個長計時器等到時間（不輪詢、重開不重算）。
/// 開程式時已經到期的話，等 60 秒再查（不跟開機搶資源）。關掉自動檢查就取消。
fn schedule_update() {
    let main = main_hwnd();
    unsafe {
        let _ = KillTimer(Some(main), TIMER_UPDATE);
        if update_flags().0 {
            let secs = update::due_in().max(60);
            SetTimer(Some(main), TIMER_UPDATE, (secs * 1000) as u32, None);
        }
    }
}

/// 自動檢查、接收推送兩個開關。
pub fn update_flags() -> (bool, bool) {
    SETTINGS.with(|s| {
        let s = s.borrow();
        (s.update_auto, s.update_push)
    })
}

/// 結束程式（更新時用）。
pub fn quit() {
    unsafe {
        tray::remove(main_hwnd());
        let _ = DestroyWindow(main_hwnd());
    }
}

/// 啟動後多久做第一次自動檢查（只跑一次就關掉）。
const TIMER_UPDATE: usize = 51;

fn settings_path() -> std::path::PathBuf {
    data_dir().join("settings.json")
}

fn load_settings() -> Settings {
    std::fs::read(settings_path()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn save_settings(s: &Settings) {
    if let Ok(json) = serde_json::to_vec_pretty(s) {
        if let Err(e) = write_atomic(&settings_path(), &json) {
            log_error(format!("save settings: {e}"));
        }
    }
}

thread_local! {
    static SETTINGS: RefCell<Settings> = RefCell::new(Settings::default());
}

static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static MSG_TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
static MSG_NEW_NOTE: AtomicU32 = AtomicU32::new(0);
static SMALL_ICON: AtomicIsize = AtomicIsize::new(0);

pub fn main_hwnd() -> HWND {
    HWND(MAIN_HWND.load(Ordering::Relaxed) as *mut c_void)
}

/// 標題列、系統匣用的小圖示。
pub fn small_icon() -> HICON {
    HICON(SMALL_ICON.load(Ordering::Relaxed) as *mut c_void)
}

fn arg(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

/// 第一次執行：把還在 Vibotype 時的便利貼資料複製過來（原檔留著）。
fn migrate_from_vibotype() {
    let target = data_dir().join("notes.json");
    if target.exists() {
        return;
    }
    let Some(appdata) = std::env::var_os("APPDATA") else { return };
    let old = std::path::PathBuf::from(appdata).join("Vibotype");
    if !old.join("notes.json").exists() {
        return;
    }
    let _ = std::fs::create_dir_all(data_dir().join("notes"));
    let _ = std::fs::copy(old.join("notes.json"), &target);
    if let Ok(dir) = std::fs::read_dir(old.join("notes")) {
        for e in dir.flatten() {
            let _ = std::fs::copy(e.path(), data_dir().join("notes").join(e.file_name()));
        }
    }
    crate::base::log_info("migrated notes from Vibotype");
}

pub fn run() {
    unsafe {
        // `--render-notes out.png`：不開視窗，把便利貼各種樣式畫成一張圖（開發用，不佔單一執行個體）
        if let Some(path) = std::env::args().skip_while(|a| a != "--render-notes").nth(1) {
            i18n::set(&load_settings().lang);
            notes::init();
            notes::render_preview(&path);
            return;
        }

        // `--render-shop out.png [分類] [語言]`：貼紙小舖畫成圖（開發用，不顯示視窗）
        if let Some(i) = std::env::args().position(|a| a == "--render-shop") {
            let args: Vec<String> = std::env::args().collect();
            i18n::set(args.get(i + 3).unwrap_or(&load_settings().lang));
            update::load_cached();
            notes::init();
            if let Some(out) = args.get(i + 1) {
                notes::render_shop(out, args.get(i + 2).and_then(|t| t.parse().ok()).unwrap_or(0));
            }
            return;
        }

        // `--render-video out.mp4 淺色桌布 深色桌布`、`--render-stills 資料夾 …`：宣傳影片和網站截圖（開發用）
        for (flag, video) in [("--render-video", true), ("--render-stills", false)] {
            if let Some(i) = std::env::args().position(|a| a == flag) {
                let args: Vec<String> = std::env::args().collect();
                if let (Some(out), Some(light), Some(dark)) = (args.get(i + 1), args.get(i + 2), args.get(i + 3)) {
                    notes::render_promo(video, out, light, dark);
                }
                return;
            }
        }
        // `--export-sprites 資料夾`：宣傳影片的素材（開發用）
        if let Some(i) = std::env::args().position(|a| a == "--export-sprites") {
            let args: Vec<String> = std::env::args().collect();
            if let Some(dir) = args.get(i + 1) {
                notes::export_sprites(dir, args.get(i + 2).map(|s| s.as_str()));
            }
            return;
        }
        // `--make-logo 原圖 資料夾`：Logo 各尺寸和 .ico（開發用）
        if let Some(i) = std::env::args().position(|a| a == "--make-logo") {
            let args: Vec<String> = std::env::args().collect();
            if let (Some(src), Some(dir)) = (args.get(i + 1), args.get(i + 2)) {
                notes::make_logo(src, dir);
            }
            return;
        }
        // `--split-grid 素材表 資料夾 前綴 名稱,… 欄 列 最大寬 最大高`：切生圖的貼紙表，開發用
        if let Some(i) = std::env::args().position(|a| a == "--split-grid") {
            let args: Vec<String> = std::env::args().skip(i + 1).collect();
            notes::split_grid(&args);
            return;
        }
        // `--check-update [--push]`：前景跑一次完整的更新同步（feed、廣告、貼紙包、重要更新），印出結果，開發用
        // `--apply-update`：把已經下載好的更新換上去（開發用，測試換檔流程）
        if arg("--apply-update") {
            println!("applied: {}", update::apply_pending());
            return;
        }
        if arg("--check-update") {
            println!("{}", update::check_now(arg("--push")));
            return;
        }
        // `--unborder 資料夾 前綴,前綴`：拿掉貼紙的白邊，開發用
        if let Some(i) = std::env::args().position(|a| a == "--unborder") {
            let args: Vec<String> = std::env::args().skip(i + 1).collect();
            notes::unborder(&args);
            return;
        }
        // `--pack-jxr 來源 目的 品質`：PNG 貼紙轉 JPEG XR，開發用
        if let Some(i) = std::env::args().position(|a| a == "--pack-jxr") {
            let args: Vec<String> = std::env::args().skip(i + 1).collect();
            notes::pack_jxr(&args);
            return;
        }
        // `--make-sticker 原圖 輸出.png`：和貼上做貼紙同樣的做法，開發用
        if let Some(i) = std::env::args().position(|a| a == "--make-sticker") {
            let args: Vec<String> = std::env::args().collect();
            if let (Some(src), Some(dst)) = (args.get(i + 1), args.get(i + 2)) {
                notes::make_user_sticker(src, dst);
            }
            return;
        }
        // `--make-art 原圖 輸出.png`：做蕃茄鐘的素材（去背、白邊、量錶面位置），開發用
        if let Some(i) = std::env::args().position(|a| a == "--make-art") {
            let args: Vec<String> = std::env::args().collect();
            if let (Some(src), Some(dst)) = (args.get(i + 1), args.get(i + 2)) {
                notes::make_art(src, dst);
            }
            return;
        }

        let new_note_msg = RegisterWindowMessageW(w!("PinPin.NewNote"));
        // 只允許一個執行個體。已經在跑的話：桌面右鍵的「新增便利貼」交給它，其他情況叫它新增一張
        let mut _mutex = CreateMutexW(None, true, w!("Local\\PinPin.SingleInstance"));
        if arg("--updated") {
            for _ in 0..50 {
                if GetLastError() != ERROR_ALREADY_EXISTS {
                    break;
                }
                if let Ok(h) = &_mutex {
                    let _ = CloseHandle(*h);
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
                _mutex = CreateMutexW(None, true, w!("Local\\PinPin.SingleInstance"));
            }
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // 更新後的新版等不到舊版結束：不要把工作交給它，安靜結束
            if arg("--updated") {
                log_error("updated instance gave up waiting for the old one");
                return;
            }
            if let Ok(main) = FindWindowW(MAIN_CLASS, None) {
                // 讓主程式可以把新便利貼的編輯框叫到前景
                let _ = AllowSetForegroundWindow(ASFW_ANY);
                // wParam：0 = 新增便利貼、1 = 新增蕃茄鐘（桌面右鍵）
                let what = if arg("--new-timer") { 1 } else { 0 };
                let _ = PostMessageW(Some(main), new_note_msg, WPARAM(what), LPARAM(0));
            }
            return;
        }
        MSG_NEW_NOTE.store(new_note_msg, Ordering::Relaxed);
        MSG_TASKBAR_CREATED.store(RegisterWindowMessageW(w!("TaskbarCreated")), Ordering::Relaxed);

        crate::base::migrate_data_dir();
        let settings = load_settings();
        i18n::set(&settings.lang);
        SETTINGS.with(|s| *s.borrow_mut() = settings);
        migrate_from_vibotype();
        // 上次自動下載好的重要更新：換上新版後重新啟動
        if update::apply_pending() {
            return;
        }
        if arg("--updated") {
            update::cleanup_after_update();
        }
        update::load_cached();

        let wc = WNDCLASSW { lpfnWndProc: Some(main_proc), hInstance: win::hinstance(), lpszClassName: MAIN_CLASS, ..Default::default() };
        RegisterClassW(&wc);
        let title = win::wide(crate::base::DISPLAY_NAME);
        let Ok(main) = CreateWindowExW(WS_EX_TOOLWINDOW, MAIN_CLASS, PCWSTR(title.as_ptr()), WS_POPUP, 0, 0, 0, 0, None, None, Some(win::hinstance()), None) else {
            log_error("create main window failed");
            return;
        };
        // 舊名的分身：只轉送 WM_COPYDATA（Vibotype 舊版還在找 PinBoard.Main）
        let legacy = WNDCLASSW { lpfnWndProc: Some(legacy_proc), hInstance: win::hinstance(), lpszClassName: LEGACY_CLASS, ..Default::default() };
        RegisterClassW(&legacy);
        let _ = CreateWindowExW(WS_EX_TOOLWINDOW, LEGACY_CLASS, PCWSTR(title.as_ptr()), WS_POPUP, 0, 0, 0, 0, None, None, Some(win::hinstance()), None);
        MAIN_HWND.store(main.0 as isize, Ordering::Relaxed);
        let icon = win::app_icon(GetSystemMetrics(SM_CXSMICON).max(16));
        SMALL_ICON.store(icon.0 as isize, Ordering::Relaxed);
        tray::add(main, icon, &t("tray.tip"));

        notes::init();
        apply_settings();
        notes::restore();
        if arg("--updated") {
            crate::toast::show(i18n::tf("toast.updated", &[("version", update::VERSION)]), false, 3);
        }
        schedule_update();
        if arg("--new-note") {
            notes::new_text("", Some(cursor_pos()));
        }
        if arg("--new-timer") {
            notes::new_timer(Some(cursor_pos()));
        }
        // 開發用：放幾張範例、直接打開貼紙小舖
        if arg("--demo-notes") {
            notes::demo();
        }
        if arg("--shop") {
            notes::open_shop();
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if notes::pre_translate(&msg) {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        tray::remove(main);
    }
}

/// 開機啟動、桌面右鍵選單照設定寫進登錄檔（debug 版不寫，免得指到開發中的執行檔）。
fn apply_settings() {
    let s = SETTINGS.with(|s| s.borrow().clone());
    tray::set_tip(main_hwnd(), &t("tray.tip"));
    if cfg!(debug_assertions) {
        return;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Err(e) = sysreg::set_autostart(s.autostart, &exe) {
            log_error(e);
        }
        if let Err(e) = sysreg::set_desktop_menu(
            s.desktop_menu,
            &exe,
            &[("PinPinNewNote", t("desktop_menu.label"), "--new-note"), ("PinPinNewTimer", t("desktop_menu.timer"), "--new-timer")],
        ) {
            log_error(e);
        }
    }
}

fn update_settings(change: impl FnOnce(&mut Settings)) {
    let s = SETTINGS.with(|s| {
        let mut s = s.borrow_mut();
        change(&mut s);
        s.clone()
    });
    save_settings(&s);
    apply_settings();
}

/// Vibotype 這類程式送來的指令。
#[derive(Deserialize, Default)]
#[serde(default)]
struct Command {
    cmd: String,
    text: String,
    source: String,
    /// 來源視窗的標題（選填，舊版 Vibotype 不會送）
    title: String,
    x: Option<i32>,
    y: Option<i32>,
}

fn on_copydata(data: &COPYDATASTRUCT) {
    if data.dwData != COPYDATA_TAG || data.lpData.is_null() {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data.lpData as *const u8, data.cbData as usize) };
    let Ok(cmd) = serde_json::from_slice::<Command>(bytes) else { return };
    let at = match (cmd.x, cmd.y) {
        (Some(x), Some(y)) => Some(POINT { x, y }),
        _ => Some(cursor_pos()),
    };
    match cmd.cmd.as_str() {
        "note" => notes::new_text(&cmd.text, at),
        "star" => {
            let count = notes::star(&cmd.text, &cmd.source, &cmd.title);
            if count > 0 && !notes::starred_visible() {
                crate::toast::show(tf("toast.starred", &[("count", &count.to_string())]), false, 2);
            }
        }
        _ => {}
    }
}

/// 舊名 `PinBoard.Main` 的分身：只收 WM_COPYDATA。
unsafe extern "system" fn legacy_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_COPYDATA {
        on_copydata(&*(lp.0 as *const COPYDATASTRUCT));
        return LRESULT(1);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

unsafe extern "system" fn main_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        tray::WM_TRAY => {
            match win::loword(lp.0 as usize) {
                WM_LBUTTONUP => notes::new_text("", None),
                WM_RBUTTONUP | WM_CONTEXTMENU => tray_menu(),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COPYDATA => {
            on_copydata(&*(lp.0 as *const COPYDATASTRUCT));
            LRESULT(1)
        }
        WM_TIMER if wp.0 == TIMER_UPDATE => {
            let _ = KillTimer(Some(hwnd), TIMER_UPDATE);
            let (auto, push) = update_flags();
            // 期間手動查過、或開小舖時查過，就不用再查
            if auto && update::due_in() == 0 {
                update::check(false, push);
            }
            schedule_update();
            LRESULT(0)
        }
        update::WM_UPDATE => {
            update::on_done();
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        m if m != 0 && m == MSG_NEW_NOTE.load(Ordering::Relaxed) => {
            if wp.0 == 1 {
                notes::new_timer(Some(cursor_pos()));
            } else {
                notes::new_text("", Some(cursor_pos()));
            }
            LRESULT(0)
        }
        // Explorer 重啟後系統匣圖示會不見，要重新加
        m if m != 0 && m == MSG_TASKBAR_CREATED.load(Ordering::Relaxed) => {
            tray::add(main_hwnd(), small_icon(), &t("tray.tip"));
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

const M_NEW: u32 = 1;
const M_PASTE: u32 = 2;
const M_STARRED: u32 = 3;
const M_SHOP: u32 = 4;
const M_HIDE: u32 = 5;
const M_UNDO: u32 = 6;
const M_EXPORT: u32 = 7;
const M_IMPORT: u32 = 8;
const M_DESKTOP_MENU: u32 = 9;
const M_AUTOSTART: u32 = 10;
const M_FOLDER: u32 = 11;
const M_LANG_FOLDER: u32 = 12;
const M_NEW_INK: u32 = 13;
const M_NEW_SLOGAN: u32 = 14;
const M_NEW_TIMER: u32 = 15;
/// 文字預設樣式（佔一段 id）
const M_STYLE: u32 = 500;
const M_EXIT: u32 = 19;
const M_UPDATE_CHECK: u32 = 20;
const M_UPDATE_NOW: u32 = 21;
const M_UPDATE_AUTO: u32 = 22;
const M_UPDATE_PUSH: u32 = 23;
const M_USAGE_STATS: u32 = 24;
const M_LANG_SYSTEM: u32 = 99;
const M_LANG: u32 = 100;

fn tray_menu() {
    let s = SETTINGS.with(|s| s.borrow().clone());
    let langs = i18n::available();
    let mut lang_items = vec![MenuItem::checked(M_LANG_SYSTEM, t("tray.language.system"), s.lang.is_empty()), MenuItem::Separator];
    for (i, (code, name)) in langs.iter().enumerate() {
        lang_items.push(MenuItem::checked(M_LANG + i as u32, name.clone(), s.lang == *code));
    }
    lang_items.extend([MenuItem::Separator, MenuItem::item(M_LANG_FOLDER, t("tray.language.folder"))]);
    let trash = notes::trash_count();
    let mut items = vec![
        MenuItem::item(M_NEW, t("tray.new")),
        MenuItem::item(M_PASTE, t("tray.paste")),
        MenuItem::item(M_NEW_INK, t("tray.new_ink")),
        MenuItem::item(M_NEW_SLOGAN, t("tray.new_slogan")),
        MenuItem::item(M_NEW_TIMER, t("tray.new_timer")),
        MenuItem::checked(M_STARRED, t("tray.starred"), notes::starred_visible()),
        MenuItem::item(M_SHOP, t("tray.shop")),
        MenuItem::Separator,
        MenuItem::checked(M_HIDE, t("tray.hide_all"), notes::is_hidden()),
    ];
    if trash > 0 {
        items.push(MenuItem::item(M_UNDO, tf("tray.undo", &[("count", &trash.to_string())])));
    }
    items.extend([
        MenuItem::Separator,
        MenuItem::item(M_EXPORT, t("tray.export")),
        MenuItem::item(M_IMPORT, t("tray.import")),
        MenuItem::Separator,
        MenuItem::Submenu { text: t("tray.text_style"), items: notes::default_style_menu(M_STYLE) },
        MenuItem::Submenu { text: format!("{} / Language", t("tray.language")), items: lang_items },
        MenuItem::checked(M_DESKTOP_MENU, t("tray.desktop_menu"), s.desktop_menu),
        MenuItem::checked(M_AUTOSTART, t("tray.autostart"), s.autostart),
        MenuItem::item(M_FOLDER, t("tray.folder")),
        MenuItem::Separator,
    ]);
    // 更新：有新版時多一個最上層的「更新到 vX」
    let new_version = update::available();
    if let Some(a) = &new_version {
        items.push(MenuItem::item(M_UPDATE_NOW, i18n::tf("tray.update_now", &[("version", &a.version)])));
    }
    items.extend([
        MenuItem::Submenu {
            text: format!("{} ({})", t("tray.update"), update::VERSION),
            items: vec![
                MenuItem::item(M_UPDATE_CHECK, t("tray.update_check")),
                MenuItem::Separator,
                MenuItem::checked(M_UPDATE_AUTO, t("tray.update_auto"), s.update_auto),
                MenuItem::checked(M_UPDATE_PUSH, t("tray.update_push"), s.update_push),
                MenuItem::checked(M_USAGE_STATS, t("tray.usage_stats"), s.usage_stats),
            ],
        },
        MenuItem::Separator,
        MenuItem::item(M_EXIT, t("tray.exit")),
    ]);
    let cmd = win::popup_menu(main_hwnd(), &items, cursor_pos());
    if notes::default_style_cmd(main_hwnd(), M_STYLE, cmd) {
        return;
    }
    match cmd {
        M_NEW => notes::new_text("", None),
        M_PASTE => notes::paste_new(None),
        M_NEW_INK => notes::new_ink(false, None),
        M_NEW_SLOGAN => notes::new_ink(true, None),
        M_NEW_TIMER => notes::new_timer(None),
        M_STARRED => notes::toggle_starred(),
        M_SHOP => notes::open_shop(),
        M_HIDE => notes::set_hidden(!notes::is_hidden()),
        M_UNDO => notes::undo_delete(),
        M_EXPORT => notes::export_all(main_hwnd()),
        M_IMPORT => notes::import(main_hwnd()),
        M_DESKTOP_MENU => update_settings(|s| s.desktop_menu = !s.desktop_menu),
        M_AUTOSTART => update_settings(|s| s.autostart = !s.autostart),
        M_FOLDER => {
            let _ = std::fs::create_dir_all(data_dir());
            let _ = std::process::Command::new("explorer").arg(data_dir()).spawn();
        }
        M_LANG_FOLDER => i18n::open_folder(),
        M_LANG_SYSTEM => set_language(""),
        id if (M_LANG..M_LANG + langs.len() as u32).contains(&id) => set_language(&langs[(id - M_LANG) as usize].0),
        M_UPDATE_CHECK => update::check(true, s.update_push),
        M_UPDATE_NOW => update::install_now(),
        M_UPDATE_AUTO => {
            update_settings(|s| s.update_auto = !s.update_auto);
            schedule_update();
        }
        M_UPDATE_PUSH => {
            update_settings(|s| s.update_push = !s.update_push);
            notes::reload_remote();
        }
        M_USAGE_STATS => update_settings(|s| s.usage_stats = !s.usage_stats),
        M_EXIT => unsafe {
            tray::remove(main_hwnd());
            let _ = DestroyWindow(main_hwnd());
        },
        _ => {}
    }
}

/// 換語言：選單、提示、桌面右鍵選單的文字都跟著換；便利貼重畫一次。
fn set_language(code: &str) {
    i18n::set(code);
    update_settings(|s| s.lang = code.to_string());
    notes::refresh_all();
    notes::reload_remote();
}
