// 內嵌 Windows 應用程式資訊清單（Per-Monitor V2 DPI，高解析度螢幕上畫面才清楚）和執行檔圖示。
use embed_manifest::manifest::DpiAwareness;
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("PinPin").dpi_awareness(DpiAwareness::PerMonitorV2)).expect("無法內嵌 manifest");
        embed_resource::compile("pinpin.rc", embed_resource::NONE).manifest_optional().expect("無法內嵌圖示");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=pinpin.rc");
    println!("cargo:rerun-if-changed=assets/logo/pinpin.ico");
}
