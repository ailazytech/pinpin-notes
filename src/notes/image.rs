//! 圖片讀寫：全部用 Windows 內建的 WIC（Windows Imaging Component），不另外帶解碼函式庫。
//!
//! 像素一律是「沒有預乘」的 BGRA，每列緊密排列（stride = 寬 × 4）。

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Graphics::Imaging::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Ole::{CF_BITMAP, CF_DIB, CF_HDROP};
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

use crate::win::wide;

#[derive(Clone)]
pub struct Pixels {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

impl Pixels {
    /// 等比例縮到 (w, h)，雙線性取樣。縮小超過一半時先用平均法對半縮，避免鋸齒。
    pub fn resized(&self, w: u32, h: u32) -> Pixels {
        let mut src = std::borrow::Cow::Borrowed(self);
        while src.w >= w * 2 && src.h >= h * 2 && src.w >= 2 && src.h >= 2 {
            src = std::borrow::Cow::Owned(src.halved());
        }
        let (w, h) = (w.max(1), h.max(1));
        let mut out = vec![0u8; (w * h * 4) as usize];
        let sx = src.w as f32 / w as f32;
        let sy = src.h as f32 / h as f32;
        for y in 0..h {
            let fy = ((y as f32 + 0.5) * sy - 0.5).max(0.0);
            let y0 = (fy as u32).min(src.h - 1);
            let y1 = (y0 + 1).min(src.h - 1);
            let ty = fy - y0 as f32;
            for x in 0..w {
                let fx = ((x as f32 + 0.5) * sx - 0.5).max(0.0);
                let x0 = (fx as u32).min(src.w - 1);
                let x1 = (x0 + 1).min(src.w - 1);
                let tx = fx - x0 as f32;
                let o = ((y * w + x) * 4) as usize;
                // 先預乘再內插，透明邊緣才不會出現黑邊
                let p = |xx: u32, yy: u32| {
                    let i = ((yy * src.w + xx) * 4) as usize;
                    let a = src.data[i + 3] as f32 / 255.0;
                    [src.data[i] as f32 * a, src.data[i + 1] as f32 * a, src.data[i + 2] as f32 * a, a]
                };
                let (a, b, c, d) = (p(x0, y0), p(x1, y0), p(x0, y1), p(x1, y1));
                let mut v = [0f32; 4];
                for k in 0..4 {
                    let top = a[k] + (b[k] - a[k]) * tx;
                    let bot = c[k] + (d[k] - c[k]) * tx;
                    v[k] = top + (bot - top) * ty;
                }
                let alpha = v[3];
                if alpha > 0.0 {
                    out[o] = (v[0] / alpha).round().min(255.0) as u8;
                    out[o + 1] = (v[1] / alpha).round().min(255.0) as u8;
                    out[o + 2] = (v[2] / alpha).round().min(255.0) as u8;
                    out[o + 3] = (alpha * 255.0).round() as u8;
                }
            }
        }
        Pixels { w, h, data: out }
    }

    fn halved(&self) -> Pixels {
        let (w, h) = (self.w / 2, self.h / 2);
        let mut out = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0u32; 4];
                let mut alpha_sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let i = (((y * 2 + dy) * self.w + x * 2 + dx) * 4) as usize;
                    let a = self.data[i + 3] as u32;
                    for k in 0..3 {
                        acc[k] += self.data[i + k] as u32 * a;
                    }
                    alpha_sum += a;
                }
                let o = ((y * w + x) * 4) as usize;
                if alpha_sum > 0 {
                    for k in 0..3 {
                        out[o + k] = (acc[k] / alpha_sum) as u8;
                    }
                }
                out[o + 3] = (alpha_sum / 4) as u8;
            }
        }
        Pixels { w, h, data: out }
    }
}

fn factory() -> Result<IWICImagingFactory> {
    thread_local! {
        static FACTORY: std::cell::OnceCell<Option<IWICImagingFactory>> = const { std::cell::OnceCell::new() };
    }
    FACTORY.with(|f| {
        f.get_or_init(|| unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()
        })
        .clone()
        .ok_or_else(|| anyhow!("無法建立 WIC"))
    })
}

unsafe fn read_source(source: &IWICBitmapSource) -> Result<Pixels> {
    let converted = WICConvertBitmapSource(&GUID_WICPixelFormat32bppBGRA, source)?;
    let (mut w, mut h) = (0u32, 0u32);
    converted.GetSize(&mut w, &mut h)?;
    if w == 0 || h == 0 || w > 16384 || h > 16384 {
        return Err(anyhow!("圖片大小不支援：{w}×{h}"));
    }
    let mut data = vec![0u8; (w * h * 4) as usize];
    converted.CopyPixels(std::ptr::null(), w * 4, &mut data)?;
    Ok(Pixels { w, h, data })
}

pub fn decode_file(path: &Path) -> Result<Pixels> {
    unsafe {
        let f = factory()?;
        let name = wide(&path.to_string_lossy());
        let decoder = f
            .CreateDecoderFromFilename(PCWSTR(name.as_ptr()), None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)
            .with_context(|| format!("讀不了圖片 {}", path.display()))?;
        let frame = decoder.GetFrame(0)?;
        read_source(&frame.cast()?)
    }
}

pub fn decode_bytes(bytes: &[u8]) -> Result<Pixels> {
    unsafe {
        let f = factory()?;
        let stream = f.CreateStream()?;
        stream.InitializeFromMemory(bytes)?;
        let decoder = f.CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)?;
        let frame = decoder.GetFrame(0)?;
        read_source(&frame.cast()?)
    }
}

pub fn save_png(p: &Pixels, path: &Path) -> Result<()> {
    save(p, path, &GUID_ContainerFormatPng, None)
}

/// 存成 JPEG XR（有損但保留透明，quality 0～1）。Windows 內建編碼器裡唯一能同時壓品質又留 alpha 的格式。
pub fn save_jxr(p: &Pixels, path: &Path, quality: f32) -> Result<()> {
    save(p, path, &GUID_ContainerFormatWmp, Some(quality))
}

fn save(p: &Pixels, path: &Path, container: &windows::core::GUID, quality: Option<f32>) -> Result<()> {
    use windows::Win32::System::Com::StructuredStorage::PROPBAG2;
    use windows::Win32::System::Variant::*;
    unsafe {
        let f = factory()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let name = wide(&path.to_string_lossy());
        let stream = f.CreateStream()?;
        stream.InitializeFromFilename(PCWSTR(name.as_ptr()), GENERIC_WRITE.0)?;
        let encoder = f.CreateEncoder(container, std::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
        let mut frame: Option<IWICBitmapFrameEncode> = None;
        let mut props = None;
        encoder.CreateNewFrame(&mut frame, &mut props)?;
        let frame = frame.ok_or_else(|| anyhow!("無法建立圖檔"))?;
        if let (Some(q), Some(bag)) = (quality, props.as_ref()) {
            // 只設 ImageQuality（VT_R4）；透明度維持無損，邊緣才不會糊
            let mut key = wide("ImageQuality");
            let opt = PROPBAG2 { pstrName: windows::core::PWSTR(key.as_mut_ptr()), ..Default::default() };
            let value = VARIANT {
                Anonymous: VARIANT_0 {
                    Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 { vt: VT_R4, Anonymous: VARIANT_0_0_0 { fltVal: q }, ..Default::default() }),
                },
            };
            bag.Write(1, &opt, &value)?;
        }
        frame.Initialize(props.as_ref())?;
        frame.SetSize(p.w, p.h)?;
        let mut format = GUID_WICPixelFormat32bppBGRA;
        frame.SetPixelFormat(&mut format)?;
        frame.WritePixels(p.h, p.w * 4, &p.data)?;
        frame.Commit()?;
        encoder.Commit()?;
        Ok(())
    }
}

/// 檔名看起來是圖片嗎。
pub fn is_image_file(path: &Path) -> bool {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tif" | "tiff" | "ico" | "heic")
}

/// 拖進來或複製的檔案清單。
pub unsafe fn drop_files(drop: HDROP) -> Vec<std::path::PathBuf> {
    let count = DragQueryFileW(drop, u32::MAX, None);
    (0..count)
        .filter_map(|i| {
            let len = DragQueryFileW(drop, i, None) as usize;
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(drop, i, Some(&mut buf));
            let s = String::from_utf16_lossy(&buf[..len]);
            (!s.is_empty()).then(|| s.into())
        })
        .collect()
}

/// 剪貼簿裡的圖片：截圖（點陣圖）或複製的圖片檔。
pub fn clipboard_image(owner: HWND) -> Option<Pixels> {
    unsafe {
        OpenClipboard(Some(owner)).ok()?;
        let result = (|| {
            if IsClipboardFormatAvailable(CF_BITMAP.0 as u32).is_ok() {
                let h = GetClipboardData(CF_BITMAP.0 as u32).ok()?;
                let f = factory().ok()?;
                let bmp = f.CreateBitmapFromHBITMAP(HBITMAP(h.0), HPALETTE::default(), WICBitmapIgnoreAlpha).ok()?;
                return read_source(&bmp.cast().ok()?).ok();
            }
            if IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() {
                let h = GetClipboardData(CF_HDROP.0 as u32).ok()?;
                let file = drop_files(HDROP(h.0)).into_iter().find(|p| is_image_file(p))?;
                return decode_file(&file).ok();
            }
            None
        })();
        let _ = CloseClipboard();
        result
    }
}

/// 剪貼簿裡有沒有文字。
pub fn clipboard_has_text() -> bool {
    unsafe { IsClipboardFormatAvailable(13 /* CF_UNICODETEXT */).is_ok() }
}

/// 圖片放進剪貼簿（CF_DIB，32 位元，貼到小畫家、Word、聊天軟體都可以）。
pub fn set_clipboard_image(owner: HWND, p: &Pixels) -> bool {
    unsafe {
        let header = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: p.w as i32,
            // 正數 = 由下往上存，相容性最好
            biHeight: p.h as i32,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: p.w * p.h * 4,
            ..Default::default()
        };
        let total = header.biSize as usize + p.data.len();
        let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, total) else { return false };
        let ptr = GlobalLock(mem) as *mut u8;
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        std::ptr::copy_nonoverlapping(&header as *const _ as *const u8, ptr, header.biSize as usize);
        let row = (p.w * 4) as usize;
        for y in 0..p.h as usize {
            let src = &p.data[y * row..(y + 1) * row];
            let dst = ptr.add(header.biSize as usize + (p.h as usize - 1 - y) * row);
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst, row);
        }
        let _ = GlobalUnlock(mem);
        if OpenClipboard(Some(owner)).is_err() {
            let _ = GlobalFree(Some(mem));
            return false;
        }
        let _ = EmptyClipboard();
        let ok = SetClipboardData(CF_DIB.0 as u32, Some(HANDLE(mem.0))).is_ok();
        let _ = CloseClipboard();
        if !ok {
            let _ = GlobalFree(Some(mem));
        }
        ok
    }
}
