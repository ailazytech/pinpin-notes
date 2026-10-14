//! 線上更新：程式本身、貼紙小舖的廣告、新的貼紙包。完整流程見 docs/UPDATES.md。
//!
//! - 一份 `feed.json`（放在官網，跟網站一起部署）描述最新版程式、目前的廣告、可以下載的貼紙包
//! - 檢查時機：啟動 1 分鐘後一次、打開貼紙小舖時（距離上次超過一天）、使用者手動「檢查更新」；沒有常駐計時器
//! - 網路在背景執行緒做（WinHTTP，Windows 內建），做完用 PostMessage 交回主執行緒
//! - 自動檢查只默默記下來（系統匣選單出現「更新到 vX」）；手動檢查才跳提示
//! - 推送：feed 把版本標成 critical，或目前版本比 min_version 舊 → 使用者沒關「接收推送」就自動下載，下次啟動時套用；
//!   廣告和貼紙包也算推送內容，關掉之後只用內建的
//! - 下載的每個檔案都要對上 feed 裡的 SHA-256；只走 HTTPS（開發時允許 http://localhost）

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Networking::WinHttp::*;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

use crate::base::{data_dir, log_error, log_info};
use crate::win::wide;

/// 背景工作做完，通知主視窗。
pub const WM_UPDATE: u32 = WM_APP + 0x51;
/// 這一版的版本號。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

type Text = HashMap<String, String>;

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Feed {
    pub schema: u32,
    pub app: Option<Release>,
    pub ads: Vec<Ad>,
    pub packs: Vec<Pack>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Release {
    pub version: String,
    /// 執行檔的下載網址（GitHub Release 的附件）
    pub url: String,
    pub sha256: String,
    /// 更新說明（語言代碼 → 文字）
    pub notes: Text,
    /// 重要更新：開著「接收推送」的使用者會自動下載、下次啟動套用
    pub critical: bool,
    /// 比這個舊的版本一律當成重要更新（例如舊版有安全問題）
    pub min_version: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Ad {
    pub id: String,
    /// 底圖網址（PNG / JPG / JPEG XR / WebP），建議 1040×200，左半邊留給文字
    pub image: String,
    pub sha256: String,
    pub title: Text,
    pub sub: Text,
    /// `site:#/order`（官網的某一頁）、`tab:sub.spring`（切到小舖的分類）、`https://…`
    pub link: String,
    /// 上架期間 YYYY-MM-DD（空的 = 不限）
    pub from: String,
    pub until: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Pack {
    pub id: String,
    /// 改了內容就加一，已經裝過的會重新下載
    pub version: u32,
    /// 需要的最低程式版本
    pub min_app: String,
    /// 放進哪個分類：內建的 key（`sub.spring`）或新的 key（要附 `sub_names`）
    pub sub: String,
    pub sub_names: Text,
    pub items: Vec<PackItem>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct PackItem {
    /// 一定要 `pk-` 開頭，不會和內建的撞名
    pub id: String,
    pub url: String,
    pub sha256: String,
    /// 顯示時的最長邊（邏輯像素），一般貼紙 75；圖檔要是它的 4 倍
    pub size: f32,
    pub names: Text,
}

/// 已經下載好、等著套用的新版程式。
#[derive(Deserialize, Serialize, Clone, Default)]
struct Ready {
    version: String,
    path: PathBuf,
    sha256: String,
}

enum Outcome {
    /// 檢查完：錯誤訊息、是不是已經自動下載了重要更新
    Checked { manual: bool, error: Option<String>, app_ready: bool },
    /// 使用者按了「更新到 vX」之後的下載
    Downloaded { error: Option<String> },
}

#[derive(Default)]
struct State {
    feed: Option<Feed>,
    busy: bool,
    outcome: Option<Outcome>,
}

static STATE: Mutex<State> = Mutex::new(State { feed: None, busy: false, outcome: None });

fn state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

// ───────────────────────────── 路徑、版本 ─────────────────────────────

pub fn dir() -> PathBuf {
    data_dir().join("update")
}

/// feed 的網址：環境變數 PINPIN_FEED（開發用）→ base::FEED_URL → 官網/feed.json；都沒有 = 不檢查。
pub fn feed_url() -> String {
    if let Ok(u) = std::env::var("PINPIN_FEED") {
        return u;
    }
    if !crate::base::FEED_URL.is_empty() {
        return crate::base::FEED_URL.to_string();
    }
    if crate::base::SITE_URL.is_empty() {
        String::new()
    } else {
        format!("{}/feed.json", crate::base::SITE_URL)
    }
}

fn parse_ver(v: &str) -> Vec<u32> {
    v.trim_start_matches('v').split(['.', '-']).map(|p| p.parse().unwrap_or(0)).collect()
}

/// a 比 b 新嗎（1.2.10 > 1.2.9）。
pub fn newer(a: &str, b: &str) -> bool {
    let (a, b) = (parse_ver(a), parse_ver(b));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

/// 今天（UTC）的 YYYY-MM-DD。
fn today() -> String {
    let days = (crate::notes::now_secs() / 86_400) as i64;
    // Howard Hinnant 的 civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() < 64 && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn ext_of(url: &str) -> &'static str {
    let path = url.split(['?', '#']).next().unwrap_or("").to_ascii_lowercase();
    ["png", "jpg", "jpeg", "jxr", "webp"].into_iter().find(|e| path.ends_with(&format!(".{e}"))).unwrap_or("png")
}

// ───────────────────────────── HTTP（WinHTTP） ─────────────────────────────

struct Handle(*mut core::ffi::c_void);
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = WinHttpCloseHandle(self.0);
            }
        }
    }
}

/// GET 一個網址，最多讀 `limit` 位元組。只接受 https（和開發用的 http://localhost）。
pub fn http_get(url: &str, limit: usize) -> Result<Vec<u8>> {
    let (secure, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        let host = r.split(['/', ':']).next().unwrap_or("");
        if host != "localhost" && host != "127.0.0.1" {
            bail!("只接受 https：{url}");
        }
        (false, r)
    } else {
        bail!("不支援的網址：{url}");
    };
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().context("連接埠")?),
        None => (hostport, if secure { 443 } else { 80 }),
    };
    unsafe {
        let ua = wide(&format!("PinPin/{VERSION}"));
        let session = Handle(WinHttpOpen(PCWSTR(ua.as_ptr()), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, PCWSTR::null(), PCWSTR::null(), 0));
        if session.0.is_null() {
            bail!("WinHttpOpen 失敗");
        }
        let _ = WinHttpSetTimeouts(session.0, 10_000, 10_000, 15_000, 30_000);
        let host_w = wide(host);
        let conn = Handle(WinHttpConnect(session.0, PCWSTR(host_w.as_ptr()), port, 0));
        if conn.0.is_null() {
            bail!("連不上 {host}");
        }
        let path_w = wide(path);
        let flags = if secure { WINHTTP_FLAG_SECURE } else { WINHTTP_OPEN_REQUEST_FLAGS(0) };
        let req = Handle(WinHttpOpenRequest(conn.0, w!("GET"), PCWSTR(path_w.as_ptr()), PCWSTR::null(), PCWSTR::null(), std::ptr::null(), flags));
        if req.0.is_null() {
            bail!("WinHttpOpenRequest 失敗");
        }
        WinHttpSendRequest(req.0, None, None, 0, 0, 0).context("送出請求")?;
        WinHttpReceiveResponse(req.0, std::ptr::null_mut()).context("沒有回應")?;
        let mut code = 0u32;
        let mut len = 4u32;
        WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut code as *mut u32 as *mut _),
            &mut len,
            std::ptr::null_mut(),
        )?;
        if code != 200 {
            bail!("HTTP {code}：{url}");
        }
        let mut out = Vec::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let mut read = 0u32;
            WinHttpReadData(req.0, buf.as_mut_ptr().cast(), buf.len() as u32, &mut read).context("讀取資料")?;
            if read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > limit {
                bail!("檔案太大：{url}");
            }
        }
        Ok(out)
    }
}

/// 下載並對 SHA-256，存到 `path`（已經有、而且內容對的就不重抓）。
fn fetch_verified(url: &str, sha256: &str, path: &Path, limit: usize) -> Result<()> {
    let want = sha256.trim().to_ascii_lowercase();
    if want.len() != 64 {
        bail!("feed 裡缺少 sha256：{url}");
    }
    if let Ok(existing) = std::fs::read(path) {
        if crate::notes::sha256_hex(&existing)? == want {
            return Ok(());
        }
    }
    let bytes = http_get(url, limit)?;
    let got = crate::notes::sha256_hex(&bytes)?;
    if got != want {
        bail!("SHA-256 不符：{url}");
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    crate::base::write_atomic(path, &bytes)?;
    Ok(())
}

// ───────────────────────────── 背景工作 ─────────────────────────────

fn post_done() {
    let main = crate::app::main_hwnd();
    unsafe {
        let _ = PostMessageW(Some(main), WM_UPDATE, WPARAM(0), LPARAM(0));
    }
}

/// 檢查更新（背景）。`manual` = 使用者自己按的；`push` = 允許推送內容和重要更新。
pub fn check(manual: bool, push: bool) {
    if state(|s| std::mem::replace(&mut s.busy, true)) {
        if manual {
            crate::toast::show(crate::i18n::t("toast.update_checking"), false, 2);
        }
        return;
    }
    std::thread::spawn(move || {
        let result = sync(push);
        let (error, app_ready) = match result {
            Ok(r) => (None, r),
            Err(e) => {
                log_error(format!("檢查更新：{e:#}"));
                (Some(format!("{e:#}")), false)
            }
        };
        state(|s| {
            s.busy = false;
            s.outcome = Some(Outcome::Checked { manual, error, app_ready });
        });
        post_done();
    });
}

/// 開發用：在目前的執行緒同步跑一次，回傳結果文字。
pub fn check_now(push: bool) -> String {
    match sync(push) {
        Ok(app_ready) => {
            let feed = state(|s| s.feed.clone()).unwrap_or_default();
            format!(
                "feed ok：app {}（目前 {VERSION}，可更新 {}），重要更新已下載 {app_ready}；廣告 {} 則（有效 {}）；已裝貼紙包 {}",
                feed.app.as_ref().map(|a| a.version.as_str()).unwrap_or("-"),
                available().is_some(),
                feed.ads.len(),
                active_ads(push).len(),
                installed_packs().iter().map(|(p, _)| format!("{} v{}（{} 張）", p.id, p.version, p.items.len())).collect::<Vec<_>>().join("、"),
            )
        }
        Err(e) => format!("失敗：{e:#}"),
    }
}

/// 打開貼紙小舖時呼叫：距離上次檢查超過一天才查。
pub fn maybe_check(push: bool) {
    let fresh = std::fs::metadata(dir().join("feed.json"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age.as_secs() < 86_400);
    if !fresh {
        check(false, push);
    }
}

/// 抓 feed、下載推送內容；回傳是不是自動下載好了重要更新。
fn sync(push: bool) -> Result<bool> {
    let url = feed_url();
    if url.is_empty() {
        bail!("off");
    }
    let bytes = http_get(&url, 2 << 20)?;
    let feed: Feed = serde_json::from_slice(&bytes).context("feed.json 格式不對")?;
    std::fs::create_dir_all(dir())?;
    crate::base::write_atomic(&dir().join("feed.json"), &bytes)?;
    state(|s| s.feed = Some(feed.clone()));
    if !push {
        return Ok(false);
    }
    let today = today();
    // 廣告的底圖（過期的不抓）
    for ad in &feed.ads {
        if !safe_id(&ad.id) || (!ad.until.is_empty() && ad.until < today) {
            continue;
        }
        if let Err(e) = fetch_verified(&ad.image, &ad.sha256, &ad_image_path(ad), 4 << 20) {
            log_error(format!("廣告 {}：{e:#}", ad.id));
        }
    }
    // 貼紙包（版本比裝過的新才抓）
    for pack in &feed.packs {
        if let Err(e) = install_pack(pack) {
            log_error(format!("貼紙包 {}：{e:#}", pack.id));
        }
    }
    // 重要更新：自動下載，下次啟動套用
    if let Some(app) = &feed.app {
        let forced = app.critical || (!app.min_version.is_empty() && newer(&app.min_version, VERSION));
        if newer(&app.version, VERSION) && forced {
            download_app(app)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn install_pack(pack: &Pack) -> Result<()> {
    if !safe_id(&pack.id) {
        bail!("代碼不合法");
    }
    if !pack.min_app.is_empty() && newer(&pack.min_app, VERSION) {
        return Ok(());
    }
    let pdir = dir().join("packs").join(&pack.id);
    let installed: Option<Pack> = std::fs::read(pdir.join("pack.json")).ok().and_then(|b| serde_json::from_slice(&b).ok());
    if installed.is_some_and(|p| p.version >= pack.version) {
        return Ok(());
    }
    for item in &pack.items {
        if !item.id.starts_with("pk-") || !safe_id(&item.id) {
            bail!("貼紙代碼要 pk- 開頭：{}", item.id);
        }
        fetch_verified(&item.url, &item.sha256, &pdir.join(format!("{}.{}", item.id, ext_of(&item.url))), 4 << 20)?;
    }
    // 全部下載完才寫 pack.json：沒寫就表示還沒裝好
    crate::base::write_atomic(&pdir.join("pack.json"), &serde_json::to_vec_pretty(pack)?)?;
    log_info(format!("installed sticker pack {} v{}", pack.id, pack.version));
    Ok(())
}

fn download_app(app: &Release) -> Result<Ready> {
    if !safe_id(&app.version.replace('.', "-")) {
        bail!("版本號不合法");
    }
    let path = dir().join("app").join(format!("pinpin-{}.exe", app.version));
    fetch_verified(&app.url, &app.sha256, &path, 80 << 20)?;
    let ready = Ready { version: app.version.clone(), path, sha256: app.sha256.to_ascii_lowercase() };
    crate::base::write_atomic(&dir().join("ready.json"), &serde_json::to_vec_pretty(&ready)?)?;
    Ok(ready)
}

fn ready() -> Option<Ready> {
    let r: Ready = serde_json::from_slice(&std::fs::read(dir().join("ready.json")).ok()?).ok()?;
    if !newer(&r.version, VERSION) {
        return None;
    }
    let bytes = std::fs::read(&r.path).ok()?;
    (crate::notes::sha256_hex(&bytes).ok()? == r.sha256).then_some(r)
}

// ───────────────────────────── 主執行緒 ─────────────────────────────

/// 啟動時讀上次抓到的 feed（廣告、貼紙包離線也能用）。
pub fn load_cached() {
    let feed = std::fs::read(dir().join("feed.json")).ok().and_then(|b| serde_json::from_slice::<Feed>(&b).ok());
    state(|s| s.feed = feed);
}

/// 有比較新的版本嗎（看最後一次抓到的 feed）。
pub fn available() -> Option<Release> {
    state(|s| s.feed.as_ref().and_then(|f| f.app.clone())).filter(|a| newer(&a.version, VERSION))
}

/// 現在要輪播的線上廣告（期間內、底圖已經下載好的）。`push` 關掉時一律空的。
pub fn active_ads(push: bool) -> Vec<(Ad, PathBuf)> {
    if !push {
        return vec![];
    }
    let today = today();
    state(|s| s.feed.as_ref().map(|f| f.ads.clone()).unwrap_or_default())
        .into_iter()
        .filter(|a| safe_id(&a.id) && (a.from.is_empty() || a.from <= today) && (a.until.is_empty() || a.until >= today))
        .map(|a| {
            let p = ad_image_path(&a);
            (a, p)
        })
        .filter(|(_, p)| p.exists())
        .collect()
}

fn ad_image_path(ad: &Ad) -> PathBuf {
    dir().join("ads").join(format!("{}.{}", ad.id, ext_of(&ad.image)))
}

/// 已經裝好的貼紙包：(pack.json 的內容, 資料夾)。
pub fn installed_packs() -> Vec<(Pack, PathBuf)> {
    let Ok(rd) = std::fs::read_dir(dir().join("packs")) else { return vec![] };
    rd.flatten()
        .filter_map(|e| {
            let p: Pack = serde_json::from_slice(&std::fs::read(e.path().join("pack.json")).ok()?).ok()?;
            Some((p, e.path()))
        })
        .collect()
}

/// 背景工作做完（WM_UPDATE）：換上新內容、需要的話提示使用者。
pub fn on_done() {
    let Some(outcome) = state(|s| s.outcome.take()) else { return };
    let t = crate::i18n::t;
    let tf = crate::i18n::tf;
    match outcome {
        Outcome::Checked { manual, error, app_ready } => {
            crate::notes::reload_remote();
            match (error, manual) {
                (Some(e), true) => crate::toast::show(if e == "off" { t("toast.update_off") } else { t("toast.update_failed") }, true, 3),
                (Some(_), false) => {}
                (None, _) if app_ready => {
                    let v = available().map(|a| a.version).unwrap_or_default();
                    crate::toast::show(tf("toast.update_ready", &[("version", &v)]), false, 5);
                }
                (None, true) => match available() {
                    Some(a) => crate::toast::show(tf("toast.update_found", &[("version", &a.version)]), false, 4),
                    None => crate::toast::show(tf("toast.update_latest", &[("version", VERSION)]), false, 2),
                },
                (None, false) => {}
            }
        }
        Outcome::Downloaded { error } => match error {
            None => install_now(),
            Some(_) => crate::toast::show(t("toast.update_bad"), true, 3),
        },
    }
}

/// 使用者按了「更新到 vX」：還沒下載就先下載，好了就換掉執行檔並重新啟動。
pub fn install_now() {
    if let Some(r) = ready() {
        match swap_and_restart(&r.path) {
            Ok(()) => crate::app::quit(),
            Err(e) => {
                log_error(format!("套用更新：{e:#}"));
                crate::toast::show(crate::i18n::t("toast.update_bad"), true, 3);
            }
        }
        return;
    }
    let Some(app) = available() else { return };
    if state(|s| std::mem::replace(&mut s.busy, true)) {
        return;
    }
    crate::toast::show(crate::i18n::t("toast.update_downloading"), false, 3);
    std::thread::spawn(move || {
        let error = download_app(&app).err().map(|e| {
            log_error(format!("下載更新：{e:#}"));
            format!("{e:#}")
        });
        state(|s| {
            s.busy = false;
            s.outcome = Some(Outcome::Downloaded { error });
        });
        post_done();
    });
}

/// 啟動時：有已經下載好的新版（重要更新）就直接換上去並重新啟動。回傳 true = 這個程序要結束。
pub fn apply_pending() -> bool {
    if cfg!(debug_assertions) {
        return false;
    }
    let Some(r) = ready() else { return false };
    match swap_and_restart(&r.path) {
        Ok(()) => true,
        Err(e) => {
            log_error(format!("啟動時套用更新：{e:#}"));
            false
        }
    }
}

/// 把正在跑的執行檔改名成 .old（Windows 允許），新版放回原位，帶 --updated 啟動。
fn swap_and_restart(new: &Path) -> Result<()> {
    let exe = std::env::current_exe()?;
    let old = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).context("沒辦法改名（資料夾可能沒有寫入權限）")?;
    if let Err(e) = std::fs::copy(new, &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(anyhow!(e).context("沒辦法放上新版"));
    }
    std::process::Command::new(&exe).arg("--updated").spawn().context("啟動新版")?;
    let _ = std::fs::remove_file(dir().join("ready.json"));
    log_info(format!("updating to {}", new.display()));
    Ok(())
}

/// 更新後第一次啟動：把舊的執行檔、下載過的安裝檔清掉（舊程序可能還沒完全結束，背景多試幾次）。
pub fn cleanup_after_update() {
    std::thread::spawn(|| {
        let Ok(exe) = std::env::current_exe() else { return };
        let old = exe.with_extension("exe.old");
        for _ in 0..20 {
            if !old.exists() || std::fs::remove_file(&old).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let _ = std::fs::remove_dir_all(dir().join("app"));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare() {
        assert!(newer("0.2.0", "0.1.9"));
        assert!(newer("1.10.0", "1.9.3"));
        assert!(newer("v1.0.1", "1.0"));
        assert!(!newer("1.0.0", "1.0.0"));
        assert!(!newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn today_is_a_date() {
        let d = today();
        assert_eq!(d.len(), 10);
        assert!(d.as_str() > "2020-01-01");
    }
}
