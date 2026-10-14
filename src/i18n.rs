//! 多語系。
//!
//! - 內建語言（`locales/*.json`）編進執行檔
//! - `%APPDATA%\PinPin\locales\*.json` 可以新增語言，或覆蓋內建語言的部分字串（同一個 `_meta.code`）
//! - 每個檔是扁平的 key → 字串（`decoys` 是字串陣列），`_meta` 寫語言名稱和是不是由右到左（RTL）
//! - 找不到的 key 依序退回英文、再退回 key 本身
//! - 字串裡的 `{count}` 這類佔位用 [`tf`] 代入
//!
//! 怎麼加新語言見 `locales/README.md`。

use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::Value;

/// 內建語言：(代碼, 檔案內容)。
const BUNDLED: [(&str, &str); 6] = [
    ("zh-TW", include_str!("../locales/zh-TW.json")),
    ("en", include_str!("../locales/en.json")),
    ("zh-CN", include_str!("../locales/zh-CN.json")),
    ("ja", include_str!("../locales/ja.json")),
    ("ko", include_str!("../locales/ko.json")),
    ("ar", include_str!("../locales/ar.json")),
];

#[derive(Default)]
struct Locale {
    code: String,
    rtl: bool,
    strings: HashMap<String, Value>,
    fallback: HashMap<String, Value>,
}

thread_local! {
    static CURRENT: RefCell<Locale> = RefCell::new(Locale::default());
}

fn parse(text: &str) -> HashMap<String, Value> {
    match serde_json::from_str::<HashMap<String, Value>>(text) {
        Ok(map) => map,
        Err(e) => {
            crate::base::log_error(format!("language file error: {e}"));
            HashMap::new()
        }
    }
}

fn meta(map: &HashMap<String, Value>, field: &str) -> Option<Value> {
    map.get("_meta").and_then(|m| m.get(field)).cloned()
}

fn external_dir() -> std::path::PathBuf {
    crate::base::data_dir().join("locales")
}

/// 外部語言檔：(代碼, 內容)。
fn external() -> Vec<(String, HashMap<String, Value>)> {
    let Ok(dir) = std::fs::read_dir(external_dir()) else { return vec![] };
    dir.flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("json")))
        .filter_map(|e| {
            let map = parse(&std::fs::read_to_string(e.path()).ok()?);
            let code = meta(&map, "code").and_then(|v| v.as_str().map(String::from)).or_else(|| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))?;
            Some((code, map))
        })
        .collect()
}

/// 某個語言的完整字串表（內建＋外部覆蓋）。
fn load_map(code: &str) -> HashMap<String, Value> {
    let mut map = BUNDLED.iter().find(|(c, _)| *c == code).map(|(_, t)| parse(t)).unwrap_or_default();
    for (c, ext) in external() {
        if c == code {
            map.extend(ext);
        }
    }
    map
}

/// 可以選的語言：(代碼, 顯示名稱)。
pub fn available() -> Vec<(String, String)> {
    let mut list: Vec<(String, String)> = BUNDLED
        .iter()
        .map(|(c, t)| (c.to_string(), meta(&parse(t), "name").and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| c.to_string())))
        .collect();
    for (code, map) in external() {
        if !list.iter().any(|(c, _)| *c == code) {
            let name = meta(&map, "name").and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| code.clone());
            list.push((code, name));
        }
    }
    list
}

/// Windows 的介面語言對應到我們有的語言。
pub fn system_language() -> String {
    let mut buf = [0u16; 85];
    let n = unsafe { windows::Win32::Globalization::GetUserDefaultLocaleName(&mut buf) };
    let name = String::from_utf16_lossy(&buf[..(n.max(1) - 1) as usize]);
    resolve(&name)
}

/// 「zh-HK」→「zh-TW」、「en-US」→「en」這種對應；都沒有就用英文。
fn resolve(name: &str) -> String {
    let codes: Vec<String> = available().into_iter().map(|(c, _)| c).collect();
    if codes.iter().any(|c| c.eq_ignore_ascii_case(name)) {
        return name.to_string();
    }
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("zh") {
        let traditional = ["zh-tw", "zh-hk", "zh-mo", "zh-hant"].iter().any(|p| lower.starts_with(p));
        return if traditional { "zh-TW".into() } else { "zh-CN".into() };
    }
    let lang = lower.split('-').next().unwrap_or("en");
    codes.into_iter().find(|c| c.to_ascii_lowercase().split('-').next() == Some(lang)).unwrap_or_else(|| "en".into())
}

/// 切換語言（空字串 = 跟隨系統）。
pub fn set(code: &str) {
    let code = if code.is_empty() { system_language() } else { resolve(code) };
    let strings = load_map(&code);
    let rtl = meta(&strings, "rtl").and_then(|v| v.as_bool()).unwrap_or(false);
    let fallback = if code == "en" { HashMap::new() } else { load_map("en") };
    CURRENT.with(|c| *c.borrow_mut() = Locale { code, rtl, strings, fallback });
}

/// 目前的語言代碼（例如 zh-TW）。
pub fn current() -> String {
    CURRENT.with(|c| c.borrow().code.clone())
}

/// 從「語言代碼 → 文字」的表挑出目前語言的（線上的廣告、貼紙包用）：完全符合 → 同語系 → 英文 → 隨便一個。
pub fn pick(map: &HashMap<String, String>) -> String {
    let code = current();
    let lang = code.split('-').next().unwrap_or("en").to_string();
    map.get(&code)
        .or_else(|| map.iter().find(|(k, _)| k.split('-').next() == Some(lang.as_str())).map(|(_, v)| v))
        .or_else(|| map.get("en"))
        .or_else(|| map.values().next())
        .cloned()
        .unwrap_or_default()
}

/// 目前的語言是不是由右到左（阿拉伯文、希伯來文…）。
pub fn is_rtl() -> bool {
    CURRENT.with(|c| c.borrow().rtl)
}

fn lookup(key: &str) -> Option<Value> {
    CURRENT.with(|c| {
        let c = c.borrow();
        c.strings.get(key).or_else(|| c.fallback.get(key)).cloned()
    })
}

/// 翻譯一個字串。
pub fn t(key: &str) -> String {
    match lookup(key) {
        Some(Value::String(s)) => s,
        _ => key.to_string(),
    }
}

/// 翻譯並代入參數：`tf("toast.exported", &[("count", "3")])`。
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

/// 字串陣列（例如偽裝內容）。
pub fn list(key: &str) -> Vec<String> {
    match lookup(key) {
        Some(Value::Array(items)) => items.into_iter().filter_map(|v| v.as_str().map(String::from)).collect(),
        _ => vec![],
    }
}

/// 一段文字是不是由右到左（看第一個有方向的字：阿拉伯文、希伯來文…）。內容方向和介面語言無關。
pub fn text_is_rtl(text: &str) -> bool {
    for c in text.chars() {
        let u = c as u32;
        if matches!(u, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF) {
            return true;
        }
        if c.is_alphabetic() {
            return false;
        }
    }
    false
}

/// 打開自訂語言檔的資料夾（沒有就建立，順便放一份英文範本）。
pub fn open_folder() {
    let dir = external_dir();
    let _ = std::fs::create_dir_all(&dir);
    let sample = dir.join("_template-en.json.txt");
    if !sample.exists() {
        let _ = std::fs::write(&sample, BUNDLED[1].1);
    }
    let _ = std::process::Command::new("explorer").arg(&dir).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_files_parse_and_have_same_keys() {
        let en = parse(BUNDLED[1].1);
        assert!(en.len() > 100);
        for (code, text) in BUNDLED {
            let map = parse(text);
            assert!(meta(&map, "name").is_some(), "{code} 缺 _meta.name");
            let missing: Vec<_> = en.keys().filter(|k| !map.contains_key(*k)).collect();
            assert!(missing.is_empty(), "{code} 缺少：{missing:?}");
        }
    }

    #[test]
    fn rtl_detection() {
        assert!(text_is_rtl("مرحبا"));
        assert!(text_is_rtl("123 שלום"));
        assert!(!text_is_rtl("Hello"));
        assert!(!text_is_rtl("你好"));
    }
}
