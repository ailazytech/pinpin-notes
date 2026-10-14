//! 基本工具：資料夾、寫檔、紀錄、PNG 輸出。
//!
//! 資料放在 `%APPDATA%\PinPin\`：notes.json、notes\（圖片）、locales\（自訂語言）、pinpin.log。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 資料夾名稱
pub const APP_NAME: &str = "PinPin";
/// 顯示的名稱（視窗標題、匯出檔）
pub const DISPLAY_NAME: &str = "PinPin Notes";
/// 改名前的資料夾（PinBoard），第一次啟動時搬過來
const OLD_APP_NAME: &str = "PinBoard";

/// 官網（部署好再填，結尾不要斜線）。空的時候「訂製」改成直接寫信。
pub const SITE_URL: &str = "";
/// 線上更新的 feed（空的 = 用官網/feed.json）。也可以直接指到 GitHub 上的檔案，例如
/// https://raw.githubusercontent.com/OWNER/pinpin-notes/main/site/public/feed.json
pub const FEED_URL: &str = "";
/// 聯絡信箱（訂製、問題回報）
pub const CONTACT_EMAIL: &str = "ailazytech@gmail.com";

/// 訂製頁的網址；官網還沒上線就開新郵件。
pub fn custom_url() -> String {
    if SITE_URL.is_empty() {
        format!("mailto:{CONTACT_EMAIL}?subject=PinPin%20custom%20stickers")
    } else {
        format!("{SITE_URL}/#/custom")
    }
}

/// 官網的某一頁（`#/order` 這種）；官網還沒上線就開新郵件。
pub fn site_page(path: &str) -> String {
    if SITE_URL.is_empty() {
        let subject = match path {
            "#/order" => "PinPin%20couplet%20order",
            "#/donate" => "PinPin%20support",
            _ => "PinPin%20custom%20stickers",
        };
        format!("mailto:{CONTACT_EMAIL}?subject={subject}")
    } else {
        format!("{SITE_URL}/{}", path.trim_start_matches('/'))
    }
}

/// 春聯訂製（接單頁）的網址；官網還沒上線就開新郵件。
pub fn order_url() -> String {
    if SITE_URL.is_empty() {
        format!("mailto:{CONTACT_EMAIL}?subject=PinPin%20couplet%20order")
    } else {
        format!("{SITE_URL}/#/order")
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join(APP_NAME)
}

/// 改名（PinBoard → PinPin）之後第一次啟動：把舊資料夾整個搬過來。搬不動（被鎖住）就複製，舊的留著。
pub fn migrate_data_dir() {
    let new = data_dir();
    let Some(old) = new.parent().map(|p| p.join(OLD_APP_NAME)) else { return };
    if new.exists() || !old.is_dir() {
        return;
    }
    if fs::rename(&old, &new).is_err() {
        if let Err(e) = copy_dir(&old, &new) {
            log_error(format!("搬移舊資料夾失敗：{e}"));
            return;
        }
    }
    log_info("moved data from PinBoard");
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn log_path() -> PathBuf {
    data_dir().join("pinpin.log")
}

/// 先寫到暫存檔再改名，避免寫到一半當掉時把檔案弄壞。
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

pub fn log_error(msg: impl AsRef<str>) {
    log_line(msg.as_ref());
}

pub fn log_info(msg: impl AsRef<str>) {
    log_line(msg.as_ref());
}

fn log_line(msg: &str) {
    use std::io::Write;
    let _ = fs::create_dir_all(data_dir());
    // 超過 1 MB 就換一個新檔，避免越長越大
    if fs::metadata(log_path()).is_ok_and(|m| m.len() > 1024 * 1024) {
        let _ = fs::rename(log_path(), data_dir().join("pinpin.old.log"));
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_path()) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = writeln!(f, "[{secs}] {msg}");
    }
    #[cfg(debug_assertions)]
    eprintln!("{msg}");
}

/// RGBA 像素存成 PNG（不壓縮，給開發用的離屏截圖）。
pub fn png_rect(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    fn crc32(data: &[u8]) -> u32 {
        let mut c = !0u32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 {
                c = (c >> 1) ^ (0xedb8_8320 & (!(c & 1)).wrapping_add(1));
            }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    }
    let stride = width as usize * 4;
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);
    for row in 0..height as usize {
        raw.push(0); // 不做濾波
        raw.extend_from_slice(&rgba[row * stride..(row + 1) * stride]);
    }
    // zlib：標頭 + 不壓縮區塊 + Adler-32
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, block) in blocks.iter().enumerate() {
        z.push(if i + 1 == blocks.len() { 1 } else { 0 });
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in &raw {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8 位元 RGBA
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn old_data_dir_moves_over() {
        let root = std::env::temp_dir().join(format!("pinpin-migrate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("PinBoard").join("stickers")).unwrap();
        std::fs::write(root.join("PinBoard").join("notes.json"), b"{}").unwrap();
        std::fs::write(root.join("PinBoard").join("stickers").join("my-1.png"), b"x").unwrap();
        // 只有這個測試會動 APPDATA
        std::env::set_var("APPDATA", &root);
        super::migrate_data_dir();
        assert!(root.join("PinPin").join("notes.json").exists());
        assert!(root.join("PinPin").join("stickers").join("my-1.png").exists());
        assert!(!root.join("PinBoard").exists(), "搬走而不是複製");
        // 已經有新資料夾就不再動
        std::fs::create_dir_all(root.join("PinBoard")).unwrap();
        std::fs::write(root.join("PinBoard").join("notes.json"), b"old").unwrap();
        super::migrate_data_dir();
        assert_eq!(std::fs::read(root.join("PinPin").join("notes.json")).unwrap(), b"{}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
