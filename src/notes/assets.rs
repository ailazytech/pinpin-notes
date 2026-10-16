//! 素材目錄：圖釘、迴紋針、紙膠帶、貼紙（生圖後去背切好，見 assets/make-deco.ps1、make-sheets.ps1）。
//!
//! 圖檔編進執行檔裡，第一次用到才解碼，並依螢幕縮放縮好放在快取，平常不佔記憶體。
//! 檔案是 2 倍大小，邏輯大小 = 檔案大小 / 2。

use std::collections::HashMap;

use super::image::Pixels;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    Sticker,
    Pin,
    Clip,
    Tape,
    /// 蕃茄鐘的圖（不在小舖裡）
    Art,
}

/// (分類, 名稱的語系 key)
pub const CATS: [(Cat, &str); 4] = [
    (Cat::Sticker, "cat.sticker"),
    (Cat::Pin, "cat.pin"),
    (Cat::Clip, "cat.clip"),
    (Cat::Tape, "cat.tape"),
];

/// 貼紙的子分類（SUBS 的第幾個）和名稱的語系 key。
pub const SUBS: [&str; 8] = ["sub.daily", "sub.lucky", "sub.charms", "sub.plants", "sub.spring", "sub.pets", "sub.scooters", "sub.taiwan"];

pub struct Asset {
    pub id: &'static str,
    pub cat: Cat,
    /// 貼紙的子分類（其他分類都是 0）
    pub sub: usize,
    /// 顯示時的最長邊（邏輯像素）；0 = 舊的 2 倍圖，檔案大小的一半
    pub size: f32,
    bytes: &'static [u8],
}

macro_rules! asset {
    ($id:literal, $cat:expr) => {
        asset!($id, $cat, 0)
    };
    ($id:literal, $cat:expr, $sub:expr) => {
        Asset { id: $id, cat: $cat, sub: $sub, size: 0.0, bytes: include_bytes!(concat!("../../assets/deco/", $id, ".png")) }
    };
}

/// 內建貼紙：assets/stickers 裡的 JPEG XR（顯示大小的 4 倍，放大也不糊），`size` = 顯示時的最長邊。
macro_rules! sticker {
    ($id:literal, $sub:expr, $size:expr) => {
        Asset { id: $id, cat: Cat::Sticker, sub: $sub, size: $size, bytes: include_bytes!(concat!("../../assets/stickers/", $id, ".jxr")) }
    };
}

pub static ASSETS: &[Asset] = &[
    // 圖釘
    asset!("pin-red", Cat::Pin),
    asset!("pin-teal", Cat::Pin),
    asset!("pin-mint", Cat::Pin),
    asset!("pin-pearl", Cat::Pin),
    asset!("pin-heart", Cat::Pin),
    asset!("pin-black", Cat::Pin),
    asset!("pin-gold", Cat::Pin),
    asset!("pin-silver", Cat::Pin),
    // 夾子
    asset!("clip", Cat::Clip),
    asset!("clip-gold", Cat::Clip),
    asset!("clip-heart", Cat::Clip),
    asset!("clip-mint", Cat::Clip),
    asset!("clip-blue", Cat::Clip),
    asset!("clip-binder", Cat::Clip),
    asset!("clip-clothespin", Cat::Clip),
    // 紙膠帶
    asset!("tape-pink", Cat::Tape),
    asset!("tape-blue", Cat::Tape),
    asset!("tape-kraft", Cat::Tape),
    asset!("tape-gingham", Cat::Tape),
    asset!("tape-grid", Cat::Tape),
    asset!("tape-dusty", Cat::Tape),
    asset!("tape-stars", Cat::Tape),
    asset!("tape-floral", Cat::Tape),
    // 貼紙
    asset!("star", Cat::Sticker),
    // 日常
    sticker!("st-heart", 0, 75.0),
    sticker!("st-star", 0, 75.0),
    sticker!("st-thumbsup", 0, 75.0),
    sticker!("st-check", 0, 75.0),
    sticker!("st-alert", 0, 75.0),
    sticker!("st-fire", 0, 75.0),
    sticker!("st-bolt", 0, 75.0),
    sticker!("st-sparkles", 0, 75.0),
    sticker!("st-coffee", 0, 75.0),
    sticker!("st-cat", 0, 75.0),
    sticker!("st-shiba", 0, 75.0),
    sticker!("st-boba", 0, 75.0),
    sticker!("st-strawberry", 0, 75.0),
    sticker!("st-donut", 0, 75.0),
    sticker!("st-onigiri", 0, 75.0),
    sticker!("st-apple", 0, 75.0),
    sticker!("st-sun", 0, 75.0),
    sticker!("st-rainbow", 0, 75.0),
    sticker!("st-clover", 0, 75.0),
    sticker!("st-cactus", 0, 75.0),
    sticker!("st-monstera", 0, 75.0),
    sticker!("st-sunflower", 0, 75.0),
    sticker!("st-mushroom", 0, 75.0),
    sticker!("st-cloud", 0, 75.0),
    sticker!("st-rain", 0, 75.0),
    sticker!("st-moon", 0, 75.0),
    sticker!("st-sakura", 0, 75.0),
    // 招財祝福（空的聚寶盆、金飾可以放進去）
    sticker!("lk-cat", 1, 75.0),
    sticker!("lk-ingot", 1, 75.0),
    sticker!("lk-goldbars", 1, 75.0),
    sticker!("lk-bag", 1, 75.0),
    sticker!("lk-envelope", 1, 75.0),
    sticker!("lk-koi", 1, 75.0),
    sticker!("lk-bowl", 1, 75.0),
    sticker!("lk-coins", 1, 75.0),
    sticker!("lk-tree", 1, 75.0),
    sticker!("lk-bowl-gold", 1, 100.0),
    sticker!("lk-bowl-red", 1, 100.0),
    sticker!("lk-bowl-box", 1, 100.0),
    sticker!("lk-gold-beans", 1, 40.0),
    sticker!("lk-gold-bean", 1, 22.0),
    sticker!("lk-gold-ring", 1, 30.0),
    sticker!("lk-gold-bracelet", 1, 36.0),
    sticker!("lk-gold-necklace", 1, 36.0),
    sticker!("lk-gold-ingot", 1, 32.0),
    sticker!("lk-gold-coin", 1, 28.0),
    sticker!("lk-gold-pig", 1, 32.0),
    sticker!("lk-gold-lock", 1, 34.0),
    // 各國招財小物
    sticker!("ch-daruma", 2, 75.0),
    sticker!("ch-omamori", 2, 75.0),
    sticker!("ch-horseshoe", 2, 75.0),
    sticker!("ch-pig", 2, 75.0),
    sticker!("ch-nazar", 2, 75.0),
    sticker!("ch-hamsa", 2, 75.0),
    sticker!("ch-bokjori", 2, 75.0),
    sticker!("ch-knot", 2, 75.0),
    sticker!("ch-ladybug", 2, 75.0),
    // 盆栽
    sticker!("pl-monstera", 3, 75.0),
    sticker!("pl-tortoise", 3, 75.0),
    sticker!("pl-succulents", 3, 75.0),
    sticker!("pl-snake", 3, 75.0),
    sticker!("pl-fiddle", 3, 75.0),
    sticker!("pl-pothos", 3, 75.0),
    sticker!("pl-staghorn", 3, 75.0),
    sticker!("pl-pilea", 3, 75.0),
    sticker!("pl-bonsai", 3, 75.0),
    // 春聯：單字斗方（生圖，逐字檢查過）、合體字斗方和直立春條（商家的手寫作品）
    sticker!("cn-fu", 4, 110.0),
    sticker!("cn-chun", 4, 110.0),
    sticker!("cn-an", 4, 110.0),
    sticker!("cn-ji", 4, 110.0),
    sticker!("cn-cai", 4, 110.0),
    sticker!("cn-wang", 4, 110.0),
    sticker!("cn-xi", 4, 110.0),
    sticker!("cn-jin", 4, 110.0),
    sticker!("cn-shuangxi", 4, 110.0),
    sticker!("cn-zhaocai", 4, 110.0),
    sticker!("cn-huangjin", 4, 110.0),
    sticker!("cn-rijin", 4, 110.0),
    sticker!("cn-jianzai", 4, 110.0),
    sticker!("cn-yifan", 4, 110.0),
    sticker!("cn-shengyi", 4, 110.0),
    sticker!("cn-jinyu", 4, 110.0),
    sticker!("cn-ziqi", 4, 110.0),
    sticker!("cn-jinbang", 4, 110.0),
    sticker!("cl-qianxiang", 4, 150.0),
    sticker!("cl-daji", 4, 150.0),
    sticker!("cl-wugu", 4, 150.0),
    sticker!("cl-tianguan", 4, 150.0),
    sticker!("cl-xinxiang", 4, 150.0),
    sticker!("cl-pingan", 4, 150.0),
    sticker!("cl-jixing", 4, 150.0),
    sticker!("cl-jixiang", 4, 150.0),
    sticker!("cl-niannian", 4, 150.0),
    sticker!("cl-shenti", 4, 150.0),
    sticker!("cl-xingfu", 4, 150.0),
    sticker!("cl-xinnian", 4, 150.0),
    sticker!("cl-suisui", 4, 150.0),
    sticker!("cl-fuxing", 4, 150.0),
    sticker!("cl-xueye", 4, 150.0),
    sticker!("cl-jinbang", 4, 150.0),
    // 貓狗品種（每隻眼神不同）
    sticker!("cat-calico", 5, 75.0),
    sticker!("cat-tuxedo", 5, 75.0),
    sticker!("cat-siamese", 5, 75.0),
    sticker!("cat-british", 5, 75.0),
    sticker!("cat-ragdoll", 5, 75.0),
    sticker!("cat-american", 5, 75.0),
    sticker!("cat-black", 5, 75.0),
    sticker!("cat-fold", 5, 75.0),
    sticker!("cat-munchkin", 5, 75.0),
    sticker!("cat-sphynx", 5, 75.0),
    sticker!("dog-corgi", 5, 75.0),
    sticker!("dog-golden", 5, 75.0),
    sticker!("dog-frenchie", 5, 75.0),
    sticker!("dog-dachshund", 5, 75.0),
    sticker!("dog-pom", 5, 75.0),
    sticker!("dog-poodle", 5, 75.0),
    sticker!("dog-husky", 5, 75.0),
    sticker!("dog-formosan", 5, 75.0),
    sticker!("dog-bichon", 5, 75.0),
    sticker!("dog-schnauzer", 5, 75.0),
    // 文青小機車
    sticker!("sc-cream", 6, 85.0),
    sticker!("sc-mint", 6, 85.0),
    sticker!("sc-pink", 6, 85.0),
    sticker!("sc-cub", 6, 85.0),
    sticker!("sc-ev", 6, 85.0),
    sticker!("sc-lemon", 6, 85.0),
    sticker!("sc-lavender", 6, 85.0),
    sticker!("sc-cherry", 6, 85.0),
    sticker!("sc-classic", 6, 85.0),
    // 台灣小物
    sticker!("tw-boba", 7, 75.0),
    sticker!("tw-chicken", 7, 75.0),
    sticker!("tw-greenman", 7, 75.0),
    sticker!("tw-beer", 7, 75.0),
    sticker!("tw-cooker", 7, 75.0),
    sticker!("tw-slippers", 7, 75.0),
    sticker!("tw-bag", 7, 75.0),
    sticker!("tw-pineapplecake", 7, 75.0),
    sticker!("tw-stinkytofu", 7, 75.0),
    sticker!("tw-guabao", 7, 75.0),
    sticker!("tw-porkrice", 7, 75.0),
    sticker!("tw-mangoice", 7, 75.0),
    sticker!("tw-bear", 7, 75.0),
    sticker!("tw-taipei101", 7, 75.0),
    sticker!("tw-douhua", 7, 75.0),
    sticker!("tw-sausage", 7, 75.0),
    sticker!("tw-oyster", 7, 75.0),
    sticker!("tw-pineapple", 7, 75.0),
    // 蕃茄鐘（AI 生圖後用 `--make-art` 去背，原圖在 assets/timer-src）
    Asset { id: "timer-cat", cat: Cat::Art, sub: 0, size: 0.0, bytes: include_bytes!("../../assets/timer/cat.png") },
    Asset { id: "timer-dog", cat: Cat::Art, sub: 0, size: 0.0, bytes: include_bytes!("../../assets/timer/dog.png") },
    Asset { id: "timer-alarm", cat: Cat::Art, sub: 0, size: 0.0, bytes: include_bytes!("../../assets/timer/alarm.png") },
];

impl Asset {
    /// 顯示名稱：線上貼紙包附的多語名稱，否則看語系檔的 `asset.<代碼>`。
    pub fn name(&self) -> String {
        if let Some(names) = remote(|r| r.names.get(self.id).cloned()) {
            return crate::i18n::pick(&names);
        }
        crate::i18n::t(&format!("asset.{}", self.id))
    }
}

// ───────────────────────────── 線上貼紙包 ─────────────────────────────

/// 線上下載的貼紙包（見 update.rs）：啟動時和下載完之後重新載入。
/// 圖檔讀進來後 leak 成 'static，和內建素材用同一套程式；只在換新包時多佔一點，量很小。
#[derive(Default)]
struct Remote {
    assets: Vec<&'static Asset>,
    names: HashMap<&'static str, HashMap<String, String>>,
    /// 貼紙包新增的分類：(key, 多語名稱)，排在內建分類後面
    subs: Vec<(String, HashMap<String, String>)>,
}

static REMOTE: std::sync::Mutex<Option<Remote>> = std::sync::Mutex::new(None);

fn remote<R>(f: impl FnOnce(&Remote) -> R) -> R {
    let guard = REMOTE.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_ref() {
        Some(r) => f(r),
        None => f(&Remote::default()),
    }
}

/// 重新載入已經裝好的貼紙包。`enabled` = 使用者沒有關掉「接收推送」。
pub fn load_packs(enabled: bool) {
    let mut r = Remote::default();
    if enabled {
        for (pack, dir) in crate::update::installed_packs() {
            let sub = match SUBS.iter().position(|k| *k == pack.sub) {
                Some(i) => i,
                None => match r.subs.iter().position(|(k, _)| *k == pack.sub) {
                    Some(i) => SUBS.len() + i,
                    None => {
                        r.subs.push((pack.sub.clone(), pack.sub_names.clone()));
                        SUBS.len() + r.subs.len() - 1
                    }
                },
            };
            for item in &pack.items {
                if find_builtin(&item.id).is_some() || r.assets.iter().any(|a| a.id == item.id) {
                    continue;
                }
                let Some(file) = std::fs::read_dir(&dir).ok().and_then(|rd| {
                    rd.flatten().map(|e| e.path()).find(|p| p.file_stem().is_some_and(|s| s.to_string_lossy() == item.id))
                }) else {
                    continue;
                };
                let Ok(bytes) = std::fs::read(&file) else { continue };
                let id: &'static str = Box::leak(item.id.clone().into_boxed_str());
                let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                let size = if item.size > 0.0 { item.size } else { 75.0 };
                r.assets.push(Box::leak(Box::new(Asset { id, cat: Cat::Sticker, sub, size, bytes })));
                r.names.insert(id, item.names.clone());
            }
        }
    }
    *REMOTE.lock().unwrap_or_else(|e| e.into_inner()) = Some(r);
}

/// 全部素材：內建＋線上貼紙包。
pub fn all() -> Vec<&'static Asset> {
    ASSETS.iter().chain(remote(|r| r.assets.clone())).collect()
}

/// 貼紙的子分類數量（內建＋貼紙包新增的）。
pub fn sub_count() -> usize {
    SUBS.len() + remote(|r| r.subs.len())
}

/// 子分類的 key（`sub.spring`…）。
pub fn sub_key(i: usize) -> String {
    match SUBS.get(i) {
        Some(k) => k.to_string(),
        None => remote(|r| r.subs.get(i - SUBS.len()).map(|(k, _)| k.clone())).unwrap_or_default(),
    }
}

/// 子分類的顯示名稱。
pub fn sub_name(i: usize) -> String {
    match SUBS.get(i) {
        Some(k) => crate::i18n::t(k),
        None => remote(|r| r.subs.get(i - SUBS.len()).map(|(_, n)| crate::i18n::pick(n))).unwrap_or_default(),
    }
}

fn find_builtin(id: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|a| a.id == id)
}

pub fn find(id: &str) -> Option<&'static Asset> {
    find_builtin(id).or_else(|| remote(|r| r.assets.iter().copied().find(|a| a.id == id)))
}

/// 自己做的貼紙（代碼以 `my-` 開頭，圖檔在 stickers 資料夾）。
pub fn is_custom(id: &str) -> bool {
    id.starts_with("my-")
}

/// 顯示名稱：內建的看語系檔，自己做的叫「我的貼紙」。
pub fn name_of(id: &str) -> String {
    match find(id) {
        Some(a) => a.name(),
        None if is_custom(id) => crate::i18n::t("asset.custom"),
        None => String::new(),
    }
}

pub fn cat_of(id: &str) -> Cat {
    find(id).map_or(Cat::Sticker, |a| a.cat)
}

/// 開發用的離屏繪圖（宣傳影片、網站截圖）：內建貼紙縮回舊的 2 倍圖大小，畫面和以前一樣。
pub fn picture_2x(id: &str) -> anyhow::Result<Pixels> {
    let a = find(id).ok_or_else(|| anyhow::anyhow!("沒有素材 {id}"))?;
    let p = super::image::decode_bytes(a.bytes)?;
    if a.size <= 0.0 {
        return Ok(p);
    }
    let k = a.size * 2.0 / p.w.max(p.h) as f32;
    Ok(p.resized(((p.w as f32 * k).round() as u32).max(1), ((p.h as f32 * k).round() as u32).max(1)))
}

/// 預設的圖釘（新便利貼用）。
pub const DEFAULT_PIN: &str = "pin-red";

/// 依目前螢幕縮放解碼好的素材。
pub struct Cache {
    scale: f32,
    /// 1 倍的圖（代碼）和放大用的圖（代碼@倍數）
    items: HashMap<String, Option<Pixels>>,
    /// 開發用的離屏預覽直接塞進來的原圖
    raw: HashMap<String, Pixels>,
}

impl Cache {
    pub fn new(scale: f32) -> Cache {
        Cache { scale, items: HashMap::new(), raw: HashMap::new() }
    }

    /// 解碼原圖（內建的從執行檔、自己做的從 stickers 資料夾）。
    fn decode(&self, id: &str) -> Option<Pixels> {
        if let Some(p) = self.raw.get(id) {
            return Some(p.clone());
        }
        let decoded = match find(id) {
            Some(asset) => super::image::decode_bytes(asset.bytes),
            None if is_custom(id) && !id.contains(['/', '\\', '.']) => super::image::decode_file(&super::stickers::file_of(id)),
            None => return None,
        };
        decoded.map_err(|e| crate::base::log_error(format!("讀不到素材 {id}：{e:#}"))).ok()
    }

    /// 原圖要縮多少倍才是 1 倍顯示大小（邏輯大小 × 螢幕縮放）。
    fn base_k(&self, id: &str, p: &Pixels) -> f32 {
        let long = p.w.max(p.h) as f32;
        let logical = match find(id) {
            // 內建貼紙：表裡寫好的大小；舊的素材檔是 2 倍圖
            Some(a) if a.size > 0.0 => a.size,
            Some(_) => long / 2.0,
            // 自己做的貼紙保留原圖解析度（最多 4096），貼上去時最長邊大約 110（和內建貼紙差不多大）
            None => (long / 2.0).min(110.0),
        };
        logical * self.scale / long
    }

    /// 取得素材（1 倍：邏輯大小 × 縮放）。紙膠帶保留原尺寸，畫的時候再取樣。
    pub fn get(&mut self, id: &str) -> Option<&Pixels> {
        if !self.items.contains_key(id) {
            let entry = self.decode(id).map(|p| {
                // 不夾到 1：高 DPI 時舊的 2 倍圖要放大才會是原本的邏輯大小
                let k = if cat_of(id) == Cat::Tape { 0.5 * self.scale } else { self.base_k(id, &p) };
                let (w, h) = ((p.w as f32 * k).round().max(1.0) as u32, (p.h as f32 * k).round().max(1.0) as u32);
                if (w, h) == (p.w, p.h) {
                    p
                } else {
                    p.resized(w, h)
                }
            });
            self.items.insert(id.to_string(), entry);
        }
        self.items.get(id)?.as_ref()
    }

    /// 放大 `k` 倍時用的圖：從原圖重新縮，不從 1 倍的小圖拉大，所以不會糊。
    /// 回傳 (圖, 這張圖是 1 倍的幾倍)；畫的時候再縮 `k / 倍數`。只有真的放大過才會多佔記憶體。
    pub fn get_at(&mut self, id: &str, k: f32) -> Option<(&Pixels, f32)> {
        const STEPS: [f32; 5] = [1.0, 1.5, 2.0, 3.0, 4.0];
        let step = STEPS.iter().copied().find(|&s| s >= k - 0.02).unwrap_or(4.0);
        let base_w = self.get(id)?.w;
        if step <= 1.0 || cat_of(id) == Cat::Tape {
            return self.get(id).map(|p| (p, 1.0));
        }
        let key = format!("{id}@{step}");
        if !self.items.contains_key(&key) {
            let entry = self.decode(id).map(|p| {
                // 原圖不夠大就用原圖
                let k = (self.base_k(id, &p) * step).min(1.0);
                let (w, h) = ((p.w as f32 * k).round().max(1.0) as u32, (p.h as f32 * k).round().max(1.0) as u32);
                if (w, h) == (p.w, p.h) {
                    p
                } else {
                    p.resized(w, h)
                }
            });
            self.items.insert(key.clone(), entry);
        }
        let img = self.items.get(&key)?.as_ref()?;
        Some((img, img.w as f32 / base_w.max(1) as f32))
    }

    /// 直接放一張原圖進快取（開發用的離屏預覽，不寫檔）。
    pub fn insert(&mut self, id: &str, p: Pixels) {
        self.forget(id);
        self.raw.insert(id.to_string(), p);
    }

    /// 刪掉自己做的貼紙之後，把快取也丟掉（含放大用的）。
    pub fn forget(&mut self, id: &str) {
        let prefix = format!("{id}@");
        self.items.retain(|k, _| k != id && !k.starts_with(&prefix));
        self.raw.remove(id);
    }

    /// 邏輯大小（寬, 高）。
    pub fn logical_size(&mut self, id: &str) -> (f32, f32) {
        let scale = self.scale;
        self.get(id).map_or((40.0, 40.0), |p| (p.w as f32 / scale, p.h as f32 / scale))
    }
}
