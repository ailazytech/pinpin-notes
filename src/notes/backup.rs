//! 匯出／匯入：全部便利貼（含圖片）打包成一個 JSON，換電腦或手動同步用。
//!
//! 有密碼的便利貼照樣是加密的（匯出檔裡也看不到原文），到另一台電腦用同一組密碼解開。
//! 匯入時同一張（id 相同）用匯入檔的版本取代，其他的加進來。
//! 自己做的貼紙和小舖的群組也一起打包；匯入時同一個群組合併，貼紙檔補進來。

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;

use super::{assets, crypto, stickers, store, with};
use crate::i18n::{t, tf};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Bundle {
    app: String,
    version: u32,
    exported_at: u64,
    notes: Vec<store::Note>,
    /// 圖檔名 → PNG 的 base64
    images: std::collections::BTreeMap<String, String>,
    /// 貼紙小舖自己分的群組
    sticker_groups: Vec<store::StickerGroup>,
    /// 自己做的貼紙：代碼 → 圖檔（PNG 或 JPEG XR）的 base64
    stickers: std::collections::BTreeMap<String, String>,
}

/// 匯出到使用者選的檔案。
pub fn export_all(owner: HWND) {
    let Some((notes, sticker_groups)) = with(|m| (m.board.notes.clone(), m.board.sticker_groups.clone())) else { return };
    let mut bundle = Bundle { app: crate::base::DISPLAY_NAME.into(), version: store::VERSION, exported_at: store::now_secs(), notes, sticker_groups, ..Default::default() };
    // 自己做的貼紙：群組裡的、便利貼上貼著的
    let used = bundle.sticker_groups.iter().flat_map(|g| g.items.iter().cloned()).chain(bundle.notes.iter().flat_map(|n| n.items.iter().map(|a| a.asset.clone())));
    for id in used.filter(|a| assets::is_custom(a)) {
        if !bundle.stickers.contains_key(&id) {
            if let Ok(bytes) = std::fs::read(stickers::file_of(&id)) {
                bundle.stickers.insert(id, crypto::b64(&bytes));
            }
        }
    }
    for n in &bundle.notes {
        if !n.image.is_empty() {
            if let Ok(bytes) = std::fs::read(store::image_dir().join(&n.image)) {
                bundle.images.insert(n.image.clone(), crypto::b64(&bytes));
            }
        }
    }
    let Some(path) = super::save_dialog(owner, &t("file.backup_name"), &format!("{} (*.json)\0*.json", t("file.backup_filter")), "json") else { return };
    let result = serde_json::to_vec_pretty(&bundle).map_err(std::io::Error::other).and_then(|json| std::fs::write(&path, json));
    match result {
        Ok(()) => crate::toast::show(tf("toast.exported", &[("count", &bundle.notes.len().to_string())]), false, 2),
        Err(e) => crate::base::log_error(format!("匯出便利貼失敗：{e}")),
    }
}

/// 從使用者選的檔案匯入。
pub fn import(owner: HWND) {
    let filter = format!("{} (*.json)\0*.json\0", t("file.backup_filter"));
    let Some(path) = super::open_dialog(owner, &filter, false).into_iter().next() else { return };
    let bundle: Bundle = match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string())) {
        Ok(b) => b,
        Err(e) => {
            crate::base::log_error(format!("匯入便利貼失敗：{e}"));
            crate::toast::show(t("toast.bad_backup"), true, 2);
            return;
        }
    };
    let _ = std::fs::create_dir_all(store::image_dir());
    for (name, data) in &bundle.images {
        // 只接受單純的檔名，避免寫到資料夾外面
        if name.contains(['/', '\\']) || name.contains("..") {
            continue;
        }
        if let Ok(bytes) = crypto::unb64(data) {
            let _ = std::fs::write(store::image_dir().join(name), bytes);
        }
    }
    let _ = std::fs::create_dir_all(store::sticker_dir());
    for (id, data) in &bundle.stickers {
        if !stickers::valid_id(id) {
            continue;
        }
        if let Ok(bytes) = crypto::unb64(data) {
            let _ = std::fs::remove_file(stickers::file_of(id));
            let _ = std::fs::write(stickers::file_for_bytes(id, &bytes), bytes);
            with(|m| m.assets.forget(id));
        }
    }
    let count = bundle.notes.len();
    with(|m| {
        for g in bundle.sticker_groups {
            match m.board.sticker_groups.iter_mut().find(|x| x.id == g.id) {
                Some(x) => {
                    for a in g.items {
                        if !x.items.contains(&a) {
                            x.items.push(a);
                        }
                    }
                }
                None => m.board.sticker_groups.push(g),
            }
        }
        for mut n in bundle.notes {
            if n.v < store::VERSION {
                // 舊格式的備份：交給下次啟動時的轉換（這裡先補附件）
                if n.items.is_empty() {
                    n.items = store::items_from_deco(n.deco.unwrap_or_default(), n.w);
                }
                n.v = store::VERSION;
            }
            match m.board.notes.iter_mut().find(|x| x.id == n.id) {
                Some(x) => *x = n,
                None => m.board.notes.push(n),
            }
        }
        m.board.hidden = false;
        store::save(&m.board);
    });
    super::reopen_all();
    crate::toast::show(tf("toast.imported", &[("count", &count.to_string())]), false, 2);
}

