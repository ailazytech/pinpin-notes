//! 便利貼密碼：PBKDF2-SHA256 推導金鑰，AES-256-GCM 加密。全部用 Windows 內建的 CNG（bcrypt.dll）。
//!
//! 金鑰只放在記憶體；關掉便利貼或重開程式就要重新輸入密碼。

use anyhow::{anyhow, Result};
use windows::core::PCWSTR;
use windows::Win32::Security::Cryptography::*;

use super::store::Secret;

pub const ITERATIONS: u32 = 200_000;

pub type Key = [u8; 32];

fn check(status: windows::Win32::Foundation::NTSTATUS, what: &str) -> Result<()> {
    if status.is_ok() {
        Ok(())
    } else {
        Err(anyhow!("{what} 失敗（0x{:08X}）", status.0))
    }
}

pub fn random(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    unsafe {
        let _ = BCryptGenRandom(None, &mut buf, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
    }
    buf
}

/// SHA-256（小寫十六進位），線上更新驗證下載的檔案用。
pub fn sha256_hex(data: &[u8]) -> Result<String> {
    unsafe {
        let mut alg = BCRYPT_ALG_HANDLE::default();
        check(BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_SHA256_ALGORITHM, PCWSTR::null(), BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0)), "開啟 SHA256")?;
        let mut out = [0u8; 32];
        let status = BCryptHash(alg, None, data, &mut out);
        let _ = BCryptCloseAlgorithmProvider(alg, 0);
        check(status, "計算 SHA256")?;
        Ok(out.iter().map(|b| format!("{b:02x}")).collect())
    }
}

pub fn derive(password: &str, salt: &[u8], iterations: u32) -> Result<Key> {
    unsafe {
        let mut alg = BCRYPT_ALG_HANDLE::default();
        check(
            BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_SHA256_ALGORITHM, PCWSTR::null(), BCRYPT_ALG_HANDLE_HMAC_FLAG),
            "開啟 SHA256",
        )?;
        let mut key = [0u8; 32];
        let status = BCryptDeriveKeyPBKDF2(alg, Some(password.as_bytes()), Some(salt), iterations as u64, &mut key, 0);
        let _ = BCryptCloseAlgorithmProvider(alg, 0);
        check(status, "推導金鑰")?;
        Ok(key)
    }
}

/// AES-GCM：加密或解密（tag 接在密文後面）。
fn gcm(key: &Key, nonce: &[u8], input: &[u8], encrypt: bool) -> Result<Vec<u8>> {
    unsafe {
        let mut alg = BCRYPT_ALG_HANDLE::default();
        check(BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_AES_ALGORITHM, PCWSTR::null(), BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0)), "開啟 AES")?;
        let mode: Vec<u16> = "ChainingModeGCM\0".encode_utf16().collect();
        let mode_bytes = std::slice::from_raw_parts(mode.as_ptr() as *const u8, mode.len() * 2);
        let result = (|| {
            check(BCryptSetProperty(alg.into(), BCRYPT_CHAINING_MODE, mode_bytes, 0), "設定 GCM")?;
            let mut hkey = BCRYPT_KEY_HANDLE::default();
            check(BCryptGenerateSymmetricKey(alg, &mut hkey, None, key, 0), "建立金鑰")?;
            let (body, mut tag) = if encrypt {
                (input.to_vec(), vec![0u8; 16])
            } else {
                if input.len() < 16 {
                    return Err(anyhow!("密文太短"));
                }
                (input[..input.len() - 16].to_vec(), input[input.len() - 16..].to_vec())
            };
            let info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
                cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
                dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
                pbNonce: nonce.as_ptr() as *mut u8,
                cbNonce: nonce.len() as u32,
                pbTag: tag.as_mut_ptr(),
                cbTag: tag.len() as u32,
                ..Default::default()
            };
            let mut out = vec![0u8; body.len()];
            let mut written = 0u32;
            let status = if encrypt {
                BCryptEncrypt(hkey, Some(&body), Some(&info as *const _ as *const _), None, Some(&mut out), &mut written, BCRYPT_FLAGS(0))
            } else {
                BCryptDecrypt(hkey, Some(&body), Some(&info as *const _ as *const _), None, Some(&mut out), &mut written, BCRYPT_FLAGS(0))
            };
            let _ = BCryptDestroyKey(hkey);
            check(status, if encrypt { "加密" } else { "解密（密碼不對）" })?;
            out.truncate(written as usize);
            if encrypt {
                out.extend_from_slice(&tag);
            }
            Ok(out)
        })();
        let _ = BCryptCloseAlgorithmProvider(alg, 0);
        result
    }
}

/// 用新密碼加密，回傳存檔用的 Secret 和金鑰（留在記憶體，之後編輯直接用）。
pub fn seal_new(password: &str, plain: &[u8]) -> Result<(Secret, Key)> {
    let salt = random(16);
    let key = derive(password, &salt, ITERATIONS)?;
    let secret = seal(&key, &salt, ITERATIONS, plain)?;
    Ok((secret, key))
}

/// 用已經解開的金鑰重新加密（每次用新的 nonce）。
pub fn seal(key: &Key, salt: &[u8], iterations: u32, plain: &[u8]) -> Result<Secret> {
    let nonce = random(12);
    let data = gcm(key, &nonce, plain, true)?;
    Ok(Secret { salt: b64(salt), nonce: b64(&nonce), data: b64(&data), iterations })
}

/// 用密碼解開。密碼錯會回傳錯誤（GCM 驗證失敗）。
pub fn open(password: &str, secret: &Secret) -> Result<(Vec<u8>, Key)> {
    let salt = unb64(&secret.salt)?;
    let key = derive(password, &salt, secret.iterations.max(1))?;
    let plain = gcm(&key, &unb64(&secret.nonce)?, &unb64(&secret.data)?, false)?;
    Ok((plain, key))
}

pub fn salt_of(secret: &Secret) -> Vec<u8> {
    unb64(&secret.salt).unwrap_or_default()
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn b64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub fn unb64(s: &str) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in s.bytes().filter(|c| *c != b'=' && !c.is_ascii_whitespace()) {
        let v = B64.iter().position(|&x| x == c).ok_or_else(|| anyhow!("base64 格式錯誤"))? as u32;
        acc = acc << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        for len in 0..20 {
            let data: Vec<u8> = (0..len as u8).map(|i| i.wrapping_mul(37)).collect();
            assert_eq!(unb64(&b64(&data)).unwrap(), data);
        }
        assert_eq!(b64(b"Man"), "TWFu");
    }

    #[test]
    fn seal_and_open() {
        let (secret, _) = seal_new("密碼123", "秘密內容".as_bytes()).unwrap();
        let (plain, _) = open("密碼123", &secret).unwrap();
        assert_eq!(plain, "秘密內容".as_bytes());
        assert!(open("wrong", &secret).is_err());
    }
}
