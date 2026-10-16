//! 宣傳影片的素材（開發用）：`--export-sprites 資料夾`
//!
//! 用真正的繪圖程式把每一種畫面畫成透明背景的高解析度 PNG（3 倍），連同位置資料寫進 `sprites.json`，
//! 交給 Remotion 組成影片。動畫狀態（手寫一筆一筆、蕃茄鐘每一秒、亂碼）輸出成一串圖。

use std::collections::HashMap;
use std::path::Path;

use serde_json::{json, Value};

use super::assets;
use super::image::{self, Pixels};
use super::ink::InkView;
use super::render::{self, Fonts, Fx, Rgb, Scene, Scramble};
use super::store::{self, Attach, InkPrefs, Kind, Line, Note, Paper, Phase, Stroke, Texture, Timer, TimerStyle, Tool};
use super::stickers;

/// 素材的縮放（影片裡最多放大到這個倍率都還清楚）
const S: f32 = 3.0;

struct Kit {
    assets: assets::Cache,
    fonts: HashMap<String, Fonts>,
    dir: std::path::PathBuf,
    manifest: serde_json::Map<String, Value>,
    /// 上一張的位移（轉角度的拍立得）
    last_off: (i32, i32),
}

#[derive(Default)]
struct Opts<'a> {
    ink: Option<InkView<'a>>,
    scramble: Option<(&'a [Line], &'a [Line], f32)>,
    fx: Fx,
    now_ms: u64,
    /// 圖片便利貼（拍立得）的圖
    image: Option<Pixels>,
    /// 滑過的樣子（按鈕、轉角度鈕）
    hover: bool,
    /// 選取中的附件（外框、控制點）
    selected: Option<usize>,
    /// 整張轉的角度（拍立得）
    angle: f32,
}

/// 預乘的 BGRA → 沒預乘的（存 PNG 用）。
fn unpremultiply(buf: &[u8], w: i32, h: i32) -> Pixels {
    let mut data = buf.to_vec();
    for px in data.chunks_mut(4) {
        let a = px[3] as f32 / 255.0;
        if a > 0.0 {
            for c in px.iter_mut().take(3) {
                *c = (*c as f32 / a).round().min(255.0) as u8;
            }
        }
        // 「收得到滑鼠」用的 alpha 1 在影片裡不要
        if px[3] <= 1 {
            px.copy_from_slice(&[0, 0, 0, 0]);
        }
    }
    Pixels { w: w as u32, h: h as u32, data }
}

impl Kit {
    fn render(&mut self, n: &Note, o: Opts) -> (Pixels, render::Layout, Vec<(i32, i32, i32)>) {
        let face = n.face.clone().unwrap_or_default();
        let bold = n.bold.unwrap_or(false);
        let key = format!("{}|{face}|{bold}", (n.font * 20.0).round());
        self.fonts.entry(key.clone()).or_insert_with(|| Fonts::new(S, n.font, &face, bold));
        let lay = render::layout_ex(n, S, &mut self.assets, o.ink.is_some());
        let rows = render::text_rows(n, &lay, S, &self.fonts[&key]);
        let mut out = Vec::new();
        let mut cache = o.image.map(|p| render::ImageCache { original: p, scaled: None });
        let scene = Scene {
            note: n,
            lines: &n.lines,
            scale: S,
            hover: o.hover,
            hot: None,
            lift: 0.0,
            attach_anim: None,
            dragging_item: None,
            selected: o.selected,
            strike: None,
            scramble: o.scramble.map(|(from, to, t)| Scramble { from, to, t }),
            unlocked: false,
            scroll: 0,
            editing: false,
            alpha: 255,
            image: cache.as_mut(),
            assets: &mut self.assets,
            fonts: &self.fonts[&key],
            now_ms: o.now_ms,
            fx: o.fx,
            angle: o.angle,
            ink: o.ink,
            text_color: n.color.map(|c| Rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)),
        };
        // 轉過角度的話，輸出的大小和位置跟著變（視窗座標 = 原本的座標 + offset）
        let hits = render::present(render::Target::Buffer(&mut out), scene);
        let (w, h) = hits.window_size();
        self.last_off = hits.offset();
        (unpremultiply(&out, w, h), lay, rows)
    }

    fn save(&self, name: &str, p: &Pixels) -> anyhow::Result<String> {
        let file = format!("{name}.png");
        image::save_png(p, &self.dir.join(&file))?;
        Ok(file)
    }

    /// 一張便利貼。
    fn note(&mut self, name: &str, n: &Note, o: Opts) -> anyhow::Result<()> {
        let (p, lay, rows) = self.render(n, o);
        let file = self.save(name, &p)?;
        let r = lay.paper;
        self.manifest.insert(
            name.into(),
            json!({ "file": file, "w": p.w, "h": p.h, "paper": [r.left, r.top, r.right, r.bottom], "rows": rows, "off": [self.last_off.0, self.last_off.1] }),
        );
        Ok(())
    }

    /// 一串動畫（每一格同樣大小）。`make(i)` 給第 i 格的便利貼和選項。
    fn sequence(&mut self, name: &str, count: usize, mut make: impl FnMut(usize) -> (Note, Opts<'static>)) -> anyhow::Result<()> {
        let mut files = Vec::new();
        let mut info = Value::Null;
        for i in 0..count {
            let (n, o) = make(i);
            let (p, lay, rows) = self.render(&n, o);
            files.push(self.save(&format!("{name}-{i:03}"), &p)?);
            if i == 0 {
                let r = lay.paper;
                info = json!({ "w": p.w, "h": p.h, "paper": [r.left, r.top, r.right, r.bottom], "rows": rows });
            }
        }
        if let Value::Object(mut m) = info {
            m.insert("files".into(), json!(files));
            self.manifest.insert(name.into(), Value::Object(m));
        }
        Ok(())
    }

    /// 直接存一張圖（2 倍大小的素材）。
    fn picture(&mut self, name: &str, p: &Pixels, scale_of_file: f32) -> anyhow::Result<()> {
        let file = self.save(name, p)?;
        self.manifest.insert(name.into(), json!({ "file": file, "w": p.w, "h": p.h, "scale": scale_of_file }));
        Ok(())
    }
}

fn note(paper: Paper, texture: Texture, lines: &[&str], items: Vec<Attach>, w: i32, h: i32) -> Note {
    Note { paper, texture, w, h, lines: lines.iter().map(|l| Line::new(*l)).collect(), items, v: store::VERSION, ..Default::default() }
}

fn heart(cx: f32, cy: f32, k: f32, p: f32) -> Vec<i32> {
    let n = 80;
    (0..=((n as f32 * p) as usize).min(n))
        .flat_map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let x = cx + 16.0 * t.sin().powi(3) * k;
            let y = cy - (13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos()) * k;
            [(x * 4.0) as i32, (y * 4.0) as i32]
        })
        .collect()
}

fn timer(style: TimerStyle, t: Timer) -> Note {
    let (w, h) = super::timer::size_of(style);
    Note { kind: Kind::Timer, w, h, timer: Some(Timer { style, ..t }), ..Default::default() }
}

/// `picture` = 拿來示範「任何圖片變貼紙」的原圖（可省略）。
pub fn export(dir: &str, picture: Option<&str>) -> anyhow::Result<()> {
    crate::i18n::set("en");
    let dir = Path::new(dir).to_path_buf();
    std::fs::create_dir_all(&dir)?;
    let mut kit = Kit { assets: assets::Cache::new(S), fonts: HashMap::new(), dir: dir.clone(), manifest: serde_json::Map::new(), last_off: (0, 0) };
    let o = Opts::default;

    // ── 便利貼 ──
    kit.note("note-todo", &note(Paper::Yellow, Texture::Sticky, &["Buy milk", "Reply to Amy", "Finish the report"], vec![Attach::at("pin-red", 120.0, 2.0)], 240, 200), o())?;
    kit.note("note-rose", &note(Paper::MorandiRose, Texture::Washi, &["Weekend", "Farmers market", "Call grandma"], vec![], 240, 200), o())?;
    kit.note("note-sage", &note(Paper::Sage, Texture::Grid, &["Ideas", "Plant herbs", "Learn watercolor"], vec![Attach { asset: "clip-heart".into(), x: 200.0, y: 4.0, angle: 6.0, ..Default::default() }], 240, 200), o())?;
    let oat = note(Paper::Oat, Texture::Dots, &["My stickers"], vec![Attach::at("pin-teal", 150.0, 2.0)], 300, 240);
    kit.note("note-oat", &oat, o())?;
    kit.note("note-oat-ink", &oat, Opts { ink: Some(InkView { prefs: InkPrefs { tool: Tool::Crayon, color: 0xE21E24, size: 1 }, eraser: false, cur: None }), ..o() })?;
    // 蠟筆愛心一筆一筆畫出來（透明紙，和 note-oat 一樣大，疊上去剛好對齊）
    kit.sequence("heart", 25, |i| {
        let mut n = note(Paper::Clear, Texture::Plain, &[], vec![], 300, 240);
        n.strokes = vec![Stroke { tool: Tool::Crayon, color: 0xE21E24, size: 7.0, pts: heart(210.0, 150.0, 2.4, i as f32 / 24.0) }];
        (n, Opts::default())
    })?;

    // ── 紙膠帶（紅格子）──
    let tape = Note { kind: Kind::Sticker, items: vec![Attach::tape("tape-gingham", 0.0, 0.0, 170.0, 0.0)], ..Default::default() };
    kit.note("tape", &tape, o())?;

    // ── 貼紙、圖釘、Logo（素材原圖，2 倍大小）──
    for name in ["st-cat", "st-sunflower"] {
        kit.picture(name, &super::assets::picture_2x(name)?, 2.0)?;
    }
    for (name, bytes) in [
        ("pin-gold", &include_bytes!("../../assets/deco/pin-gold.png")[..]),
        ("logo", &include_bytes!("../../assets/logo/pinpin-512.png")[..]),
    ] {
        kit.picture(name, &image::decode_bytes(bytes)?, 2.0)?;
    }
    // 示範用的原圖（淡色背景）和做好的貼紙
    if let Some(path) = picture {
        let src = image::decode_file(Path::new(path))?;
        let shown = if src.w > 800 { src.resized(800, (src.h as f32 * 800.0 / src.w as f32) as u32) } else { src.clone() };
        kit.picture("picture", &shown, 2.0)?;
        kit.picture("picture-sticker", &stickers::make_sticker(&src), 2.0)?;
    }
    // 「截圖」和做好的貼紙
    let src = super::assets::picture_2x("st-shiba")?;
    let mut shot = Pixels { w: src.w + 120, h: src.h + 90, data: vec![0; ((src.w + 120) * (src.h + 90) * 4) as usize] };
    for y in 0..shot.h {
        for x in 0..shot.w {
            let o = ((y * shot.w + x) * 4) as usize;
            let mut c = [232.0f32, 236.0, 238.0];
            if x >= 60 && y >= 45 && x < src.w + 60 && y < src.h + 45 {
                let i = (((y - 45) * src.w + x - 60) * 4) as usize;
                let a = src.data[i + 3] as f32 / 255.0;
                for k in 0..3 {
                    c[k] = src.data[i + k] as f32 * a + c[k] * (1.0 - a);
                }
            }
            shot.data[o..o + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
        }
    }
    kit.picture("shot", &shot, 2.0)?;
    kit.picture("shiba-sticker", &stickers::make_sticker(&shot), 2.0)?;

    // ── 蕃茄鐘 ──
    let base = store::now_ms() / 1000 * 1000;
    // 每一秒一張：24:59 → 24:52
    kit.sequence("tomato", 8, |i| {
        let t = Timer { end_ms: base + (25 * 60 - 1 - i as u64) * 1000 + 500, done: 2, ..Default::default() };
        (timer(TimerStyle::Tomato, t), Opts { now_ms: base, ..Default::default() })
    })?;
    // 時間到：綠蕃茄（休息）原地閃、擴散
    kit.sequence("tomato-ring", 16, |i| {
        let t = Timer { phase: Phase::Break, left_ms: 5 * 60_000, finished: true, done: 3, ..Default::default() };
        (timer(TimerStyle::Tomato, t), Opts { fx: Fx::Ring(i as f32 / 15.0), now_ms: base, ..Default::default() })
    })?;
    for (name, style, left) in [("hourglass", TimerStyle::Hourglass, 552u64), ("led", TimerStyle::Led, 1085), ("alarm", TimerStyle::Alarm, 768), ("cat", TimerStyle::Cat, 1290), ("dog", TimerStyle::Dog, 1499)] {
        let t = Timer { end_ms: base + left * 1000 + 500, done: 2, ..Default::default() };
        kit.note(&format!("timer-{name}"), &timer(style, t), Opts { now_ms: base, ..o() })?;
    }

    // ── 透明標語 ──
    let slogan = Note { paper: Paper::Clear, w: 640, h: 200, font: 3.8, face: Some("Impact".into()), color: Some(0xF7CF3B), lines: vec![Line::new("STAY FOCUSED")], ..Default::default() };
    kit.note("slogan", &slogan, o())?;

    // ── 密碼：偽裝內容亂碼變回真正的內容 ──
    let decoy: &'static [Line] = Box::leak(store::decoy(3).into_boxed_slice());
    let real: &'static [Line] = Box::leak(["Gift ideas for Sam", "Vinyl player", "Concert tickets"].iter().map(|l| Line::new(*l)).collect::<Vec<_>>().into_boxed_slice());
    kit.sequence("secret", 21, |i| {
        let mut n = note(Paper::MorandiLilac, Texture::Plain, &[], vec![Attach::at("pin-heart", 150.0, 2.0)], 300, 220);
        let t = i as f32 / 20.0;
        n.lines = if i == 20 { real.to_vec() } else { decoy.to_vec() };
        let scramble = (i > 0 && i < 20).then_some((decoy, real, t));
        (n, Opts { scramble, ..Default::default() })
    })?;

    // ── 最後整片桌面的填充：星號記事本、拍立得、換字型的便利貼、桌面貼紙 ──
    let mut starred = Note { kind: Kind::Starred, paper: Paper::White, texture: Texture::Lined, w: 270, h: 300, v: store::VERSION, ..Default::default() };
    starred.items = vec![Attach { asset: "star".into(), x: 256.0, y: 4.0, angle: 10.0, ..Default::default() }];
    starred.lines = vec![
        Line { text: "Small steps every day".into(), done: false, meta: "10/12 09:12 · chrome".into() },
        Line { text: "Ship it, then polish".into(), done: false, meta: "10/12 13:40 · notion".into() },
        Line { text: "Drink water".into(), done: true, meta: "10/12 15:05 · slack".into() },
    ];
    kit.note("starred", &starred, o())?;
    if let Some(path) = picture {
        let mut photo = Note { kind: Kind::Image, paper: Paper::White, w: 230, h: 300, v: store::VERSION, ..Default::default() };
        photo.items = vec![Attach::tape("tape-dusty", 60.0, 2.0, 170.0, -4.0)];
        photo.lines = vec![Line::new("Mochi ♡")];
        kit.note("polaroid", &photo, Opts { image: Some(image::decode_file(Path::new(path))?), ..o() })?;
    }
    let mut call = note(Paper::MorandiBlue, Texture::Lined, &["Call mom", "Gym 7pm"], vec![Attach::at("pin-pearl", 120.0, 2.0)], 240, 190);
    call.face = Some("Segoe Print".into());
    call.color = Some(0x2F6FDB);
    call.font = 1.3;
    kit.note("note-call", &call, o())?;
    let mut sale = note(Paper::Yellow, Texture::Sticky, &["SALE!"], vec![Attach::tape("tape-stars", 70.0, 0.0, 170.0, -3.0)], 220, 170);
    sale.face = Some("Impact".into());
    sale.color = Some(0xE21E24);
    sale.font = 3.0;
    kit.note("note-sale", &sale, o())?;
    let mut shop = note(Paper::MilkTea, Texture::Fiber, &["Read 20 pages", "Water the plants"], vec![Attach { asset: "clip-binder".into(), x: 46.0, y: -12.0, ..Default::default() }], 240, 190);
    shop.face = Some("Georgia".into());
    shop.color = Some(0x8A5CD6);
    shop.bold = Some(true);
    kit.note("note-read", &shop, o())?;
    for name in ["st-sakura", "st-coffee", "st-star", "st-rainbow", "st-boba", "st-moon", "st-sparkles"] {
        kit.picture(name, &super::assets::picture_2x(name)?, 2.0)?;
    }

    let manifest = json!({ "scale": S, "sprites": Value::Object(kit.manifest) });
    std::fs::write(dir.join("sprites.json"), serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}

/// `--export-clips 資料夾 原圖資料夾`：特點短片（沒有字幕）的素材，寫進 `clips.json`。
/// 原圖資料夾要有 pet-cat.png、pet-corgi.png（拍立得）、anime.png、idol.png（貼上做貼紙）。
pub fn export_clips(dir: &str, src: &str) -> anyhow::Result<()> {
    crate::i18n::set("en");
    let dir = Path::new(dir).to_path_buf();
    let src = Path::new(src);
    std::fs::create_dir_all(&dir)?;
    let mut kit = Kit { assets: assets::Cache::new(S), fonts: HashMap::new(), dir: dir.clone(), manifest: serde_json::Map::new(), last_off: (0, 0) };

    // ── 寵物照片 → 拍立得（大小、膠帶和 new_image 一樣）；每 1° 一張（滑過的 -h、沒滑過的 -a），轉角度時用 ──
    for pet in ["cat", "corgi"] {
        let img = image::decode_file(&src.join(format!("pet-{pet}.png")))?;
        let frame_w = 300.0f32;
        let frame_h = (frame_w * img.h as f32 / img.w as f32).clamp(60.0, 420.0);
        let mut photo = Note { kind: Kind::Image, paper: Paper::White, w: (frame_w + 20.0) as i32, h: (frame_h + 48.0) as i32, v: store::VERSION, ..Default::default() };
        let w = photo.w as f32;
        photo.items = vec![Attach::tape("tape-pink", w / 2.0 - 55.0, 2.0, w / 2.0 + 55.0, -4.0)];
        kit.note(&format!("polaroid-{pet}"), &photo, Opts { image: Some(img.clone()), ..Default::default() })?;
        for a in -12..=12 {
            kit.note(&format!("polaroid-{pet}-h{a}"), &photo, Opts { image: Some(img.clone()), hover: true, angle: a as f32, ..Default::default() })?;
            kit.note(&format!("polaroid-{pet}-a{a}"), &photo, Opts { image: Some(img.clone()), angle: a as f32, ..Default::default() })?;
        }
        kit.picture(&format!("photo-{pet}"), &img, 2.0)?;
    }

    // ── 貼上任何圖片做成貼紙（和小舖的 Ctrl+V 同一套去背、白邊）──
    for name in ["anime", "idol"] {
        let img = image::decode_file(&src.join(format!("{name}.png")))?;
        kit.assets.insert(&format!("my-{name}"), stickers::make_sticker(&img));
        kit.picture(&format!("photo-{name}"), &img, 2.0)?;
    }

    // ── 直接貼在桌面上的貼紙：一般和選取中（外框、控制點）──
    for name in ["st-cat", "st-shiba", "st-heart", "st-star", "st-thumbsup", "st-fire", "cn-wang", "cn-rijin", "cn-jinbang", "cl-tianguan", "cl-jinbang", "my-anime", "my-idol"] {
        let n = Note { kind: Kind::Sticker, items: vec![Attach { asset: name.into(), ..Default::default() }], v: store::VERSION, ..Default::default() };
        kit.note(&format!("dk-{name}"), &n, Opts::default())?;
        kit.note(&format!("dk-{name}-sel"), &n, Opts { selected: Some(0), ..Default::default() })?;
    }

    // ── 番茄鐘：六種造型各走 8 秒（tm-<造型>-<秒>）；番茄最後 4 秒倒數、時間到的波紋 ──
    let base = store::now_ms() / 1000 * 1000;
    for (name, style, left) in [
        ("tomato", TimerStyle::Tomato, 24 * 60 + 59u64),
        ("hourglass", TimerStyle::Hourglass, 9 * 60 + 12),
        ("led", TimerStyle::Led, 18 * 60 + 5),
        ("alarm", TimerStyle::Alarm, 12 * 60 + 48),
        ("cat", TimerStyle::Cat, 21 * 60 + 30),
        ("dog", TimerStyle::Dog, 14 * 60 + 40),
    ] {
        for i in 0..8u64 {
            let t = Timer { end_ms: base + (left - i) * 1000 + 500, done: 2, ..Default::default() };
            kit.note(&format!("tm-{name}-{i}"), &timer(style, t), Opts { now_ms: base, ..Default::default() })?;
        }
    }
    for left in 0..=3u64 {
        let t = Timer { end_ms: base + left * 1000, done: 2, ..Default::default() };
        kit.note(&format!("tm-end-{left}"), &timer(TimerStyle::Tomato, t), Opts { now_ms: base, ..Default::default() })?;
    }
    for i in 0..16 {
        let t = Timer { phase: Phase::Break, left_ms: 5 * 60_000, finished: true, done: 3, ..Default::default() };
        kit.note(&format!("tm-ring-{i}"), &timer(TimerStyle::Tomato, t), Opts { fx: Fx::Ring(i as f32 / 15.0), now_ms: base, ..Default::default() })?;
    }

    // ── 貼貼紙用的便利貼 ──
    let plan = note(Paper::MilkTea, Texture::Dots, &["Weekend plans", "Picnic at the park", "Bake cookies"], vec![Attach::at("pin-red", 140.0, 2.0)], 280, 230);
    kit.note("note-plan", &plan, Opts::default())?;

    let manifest = json!({ "scale": S, "sprites": Value::Object(kit.manifest) });
    std::fs::write(dir.join("clips.json"), serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}
