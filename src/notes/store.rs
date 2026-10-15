//! 便利貼的資料和存檔：`%APPDATA%\PinPin\notes.json`，圖片放在 `notes\` 資料夾。
//!
//! 每次改動都直接整份寫回（檔案很小），開機時讀回來恢復到桌面上。
//! 有密碼的便利貼：`lines` 存的是偽裝內容，真正的內容加密放在 `secret`，檔案裡看不到原文。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 目前的資料格式版本。
pub const VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// 一般文字便利貼
    #[default]
    Text,
    /// 拍立得樣式的圖片，lines 的第一行是圖說
    Image,
    /// 星號記事本（只有一本）
    Starred,
    /// 直接貼在桌面上的貼紙（沒有紙，只有 items 裡的一張貼紙）
    Sticker,
    /// 蕃茄鐘（沒有紙，畫成蕃茄或沙漏；設定在 `timer`）
    Timer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paper {
    #[default]
    Yellow,
    Pink,
    Blue,
    Green,
    Lavender,
    White,
    Kraft,
    Orange,
    // 莫蘭迪色
    MorandiRose,
    MorandiBlue,
    Sage,
    Oat,
    MorandiLilac,
    MilkTea,
    MorandiYellow,
    MorandiGrey,
    /// 透明（手寫的桌面標語）
    Clear,
}

/// (顏色, 名稱的語系 key, 分組的語系 key)
pub const PAPERS: [(Paper, &str, &str); 17] = [
    (Paper::Yellow, "paper.yellow", "paper.group.classic"),
    (Paper::Pink, "paper.pink", "paper.group.classic"),
    (Paper::Blue, "paper.blue", "paper.group.classic"),
    (Paper::Green, "paper.green", "paper.group.classic"),
    (Paper::Orange, "paper.orange", "paper.group.classic"),
    (Paper::Lavender, "paper.lavender", "paper.group.classic"),
    (Paper::White, "paper.white", "paper.group.classic"),
    (Paper::Kraft, "paper.kraft", "paper.group.classic"),
    (Paper::MorandiRose, "paper.morandi_rose", "paper.group.morandi"),
    (Paper::MorandiBlue, "paper.morandi_blue", "paper.group.morandi"),
    (Paper::Sage, "paper.sage", "paper.group.morandi"),
    (Paper::Oat, "paper.oat", "paper.group.morandi"),
    (Paper::MorandiLilac, "paper.morandi_lilac", "paper.group.morandi"),
    (Paper::MilkTea, "paper.milk_tea", "paper.group.morandi"),
    (Paper::MorandiYellow, "paper.morandi_yellow", "paper.group.morandi"),
    (Paper::MorandiGrey, "paper.morandi_grey", "paper.group.morandi"),
    (Paper::Clear, "paper.clear", "paper.group.special"),
];

/// 紙質。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Texture {
    /// 便利貼：上緣一條黏膠、右下角微翹
    #[default]
    Sticky,
    /// 平滑的紙
    Plain,
    /// 方格紙
    Grid,
    /// 橫線筆記紙
    Lined,
    /// 點陣紙
    Dots,
    /// 和紙（細纖維）
    Washi,
    /// 牛皮紙纖維
    Fiber,
}

/// (紙質, 名稱的語系 key)
pub const TEXTURES: [(Texture, &str); 7] = [
    (Texture::Sticky, "texture.sticky"),
    (Texture::Plain, "texture.plain"),
    (Texture::Grid, "texture.grid"),
    (Texture::Lined, "texture.lined"),
    (Texture::Dots, "texture.dots"),
    (Texture::Washi, "texture.washi"),
    (Texture::Fiber, "texture.fiber"),
];

/// 舊版（v1）的裝飾，只用在轉換舊資料。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Deco {
    None,
    #[default]
    PinRed,
    PinTeal,
    TapePink,
    TapeBlue,
    TapeCorners,
    Clip,
    Star,
}

/// 視窗疊放的層級。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    #[default]
    Normal,
    Top,
    Desktop,
}

/// 貼在便利貼上的東西：圖釘、迴紋針、紙膠帶、貼紙。座標是相對紙張左上角的邏輯像素。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Attach {
    /// 素材代碼，見 assets.rs
    pub asset: String,
    /// 中心點（紙膠帶：起點）
    pub x: f32,
    pub y: f32,
    /// 紙膠帶的終點
    pub x2: f32,
    pub y2: f32,
    /// 旋轉角度（度）
    pub angle: f32,
    pub scale: f32,
}

impl Default for Attach {
    fn default() -> Self {
        Attach { asset: String::new(), x: 0.0, y: 0.0, x2: 0.0, y2: 0.0, angle: 0.0, scale: 1.0 }
    }
}

impl Attach {
    pub fn at(asset: &str, x: f32, y: f32) -> Attach {
        Attach { asset: asset.into(), x, y, ..Default::default() }
    }
    pub fn tape(asset: &str, x: f32, y: f32, x2: f32, y2: f32) -> Attach {
        Attach { asset: asset.into(), x, y, x2, y2, ..Default::default() }
    }
}

/// 手寫的筆。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    #[default]
    Pen,
    /// 螢光筆：半透明、粗
    Marker,
    Chalk,
    Crayon,
}

/// 一筆手寫。座標是相對紙張左上角的邏輯像素 × 4（整數比較省空間），x、y 交錯。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stroke {
    pub tool: Tool,
    /// 0xRRGGBB
    pub color: u32,
    /// 筆寬（邏輯像素）
    pub size: f32,
    pub pts: Vec<i32>,
}

/// 上次用的筆（新的手寫沿用）。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InkPrefs {
    pub tool: Tool,
    pub color: u32,
    /// 0 細、1 中、2 粗
    pub size: u8,
}

impl Default for InkPrefs {
    fn default() -> Self {
        InkPrefs { tool: Tool::Pen, color: 0x26221E, size: 1 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerStyle {
    #[default]
    Tomato,
    Hourglass,
    /// 七段顯示器的 LED 數字錶
    Led,
    /// 雙鈴鬧鐘
    Alarm,
    /// 抱著時鐘的貓
    Cat,
    /// 抱著時鐘的柴犬
    Dog,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Focus,
    Break,
}

/// 蕃茄鐘的狀態。跑的時候記「幾點結束」，所以重開程式也接得上。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Timer {
    pub style: TimerStyle,
    /// 專注、休息各幾分鐘
    pub focus: u32,
    pub rest: u32,
    pub phase: Phase,
    /// 跑的時候：結束的時間（毫秒）；0 = 沒在跑
    pub end_ms: u64,
    /// 沒在跑的時候：還剩幾毫秒
    pub left_ms: u64,
    /// 上一段剛結束、還沒開始下一段（沙漏的沙全在下面）
    pub finished: bool,
    /// 今天完成了幾個蕃茄
    pub done: u32,
}

impl Default for Timer {
    fn default() -> Self {
        Timer { style: TimerStyle::Tomato, focus: 25, rest: 5, phase: Phase::Focus, end_ms: 0, left_ms: 25 * 60_000, finished: false, done: 0 }
    }
}

impl Timer {
    /// 這一段的總長（毫秒）。
    pub fn total_ms(&self) -> u64 {
        (if self.phase == Phase::Focus { self.focus } else { self.rest }).max(1) as u64 * 60_000
    }
    pub fn running(&self) -> bool {
        self.end_ms > 0
    }
    /// 剩下的毫秒。
    pub fn left(&self, now: u64) -> u64 {
        if self.running() { self.end_ms.saturating_sub(now) } else { self.left_ms }
    }
    /// 進度 0～1（沙漏流下去的比例）。
    pub fn progress(&self, now: u64) -> f32 {
        if self.finished && !self.running() {
            return 1.0;
        }
        1.0 - (self.left(now) as f32 / self.total_ms() as f32).clamp(0.0, 1.0)
    }
}

/// 貼紙小舖裡自己分的群組（裡面可以有自己做的貼紙，也可以收內建的）。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StickerGroup {
    pub id: String,
    pub name: String,
    /// 素材代碼
    pub items: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Line {
    pub text: String,
    /// 劃掉了
    pub done: bool,
    /// 附註（星號記事本：收藏時間和來源程式）
    #[serde(skip_serializing_if = "String::is_empty")]
    pub meta: String,
}

impl Line {
    pub fn new(text: impl Into<String>) -> Self {
        Line { text: text.into(), ..Default::default() }
    }
}

/// 加密的內容（PBKDF2-SHA256 + AES-256-GCM，全部 base64）。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Secret {
    pub salt: String,
    pub nonce: String,
    pub data: String,
    pub iterations: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Note {
    pub id: String,
    pub kind: Kind,
    pub lines: Vec<Line>,
    /// 圖片檔名（在 notes 資料夾裡）
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image: String,
    /// 紙張左上角（螢幕座標，實際像素）
    pub x: i32,
    pub y: i32,
    /// 紙張大小（邏輯像素）
    pub w: i32,
    pub h: i32,
    /// 整張的角度（度，順時針，繞紙張中心；目前只有拍立得能轉）。舊資料沒有就是 0
    #[serde(skip_serializing_if = "is_zero")]
    pub angle: f32,
    pub paper: Paper,
    pub texture: Texture,
    /// 字的大小（1.0 = 預設，最大 6 倍，寫標語用）
    pub font: f32,
    /// 這張自己的字型、顏色（0xRRGGBB）、粗體；None = 跟著預設樣式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    pub items: Vec<Attach>,
    /// 手寫
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strokes: Vec<Stroke>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer: Option<Timer>,
    pub layer: Layer,
    /// 收起來（星號記事本用）
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<Secret>,
    pub created: u64,
    /// 資料格式版本（舊資料是 0）
    pub v: u32,
    /// 舊版欄位，讀進來轉成 items 後就不再寫出
    #[serde(skip_serializing)]
    pub deco: Option<Deco>,
}

impl Default for Note {
    fn default() -> Self {
        Note {
            id: String::new(),
            kind: Kind::Text,
            lines: Vec::new(),
            image: String::new(),
            x: 200,
            y: 200,
            w: 240,
            h: 220,
            angle: 0.0,
            paper: Paper::Yellow,
            texture: Texture::Sticky,
            font: 1.0,
            face: None,
            color: None,
            bold: None,
            items: Vec::new(),
            strokes: Vec::new(),
            timer: None,
            layer: Layer::Normal,
            hidden: false,
            secret: None,
            created: 0,
            v: 0,
            deco: None,
        }
    }
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl Note {
    /// 編輯框裡的文字（一行一列）。
    pub fn text_of(lines: &[Line]) -> String {
        lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\r\n")
    }

    pub fn is_empty(&self) -> bool {
        self.image.is_empty()
            && self.secret.is_none()
            && self.strokes.is_empty()
            && !matches!(self.kind, Kind::Sticker | Kind::Timer)
            && self.lines.iter().all(|l| l.text.trim().is_empty())
    }

    pub fn locked(&self) -> bool {
        self.secret.is_some()
    }
}

/// 舊版裝飾換成新的附件（位置相對紙張）。
pub fn items_from_deco(deco: Deco, w: i32) -> Vec<Attach> {
    let w = w as f32;
    match deco {
        Deco::None => vec![],
        Deco::PinRed => vec![Attach::at("pin-red", w / 2.0, 2.0)],
        Deco::PinTeal => vec![Attach::at("pin-teal", w / 2.0, 2.0)],
        Deco::TapePink => vec![Attach::tape("tape-pink", w / 2.0 - 60.0, 0.0, w / 2.0 + 60.0, -3.0)],
        Deco::TapeBlue => vec![Attach::tape("tape-blue", w / 2.0 - 60.0, -2.0, w / 2.0 + 60.0, 2.0)],
        Deco::TapeCorners => vec![
            Attach::tape("tape-pink", -14.0, 26.0, 30.0, -14.0),
            Attach::tape("tape-pink", w - 30.0, -14.0, w + 14.0, 26.0),
        ],
        Deco::Clip => vec![Attach { asset: "clip".into(), x: 34.0, y: 8.0, angle: 8.0, ..Default::default() }],
        Deco::Star => vec![Attach { asset: "star".into(), x: w - 14.0, y: 4.0, angle: 10.0, ..Default::default() }],
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Board {
    pub notes: Vec<Note>,
    /// 最近刪掉的便利貼，可以從系統匣復原（最多留 20 張）
    pub trash: Vec<Note>,
    /// 全部暫時收起來
    pub hidden: bool,
    /// 新便利貼沿用上次選的顏色、紙質和釘法
    pub last_paper: Paper,
    pub last_texture: Texture,
    pub last_pin: String,
    /// 貼紙小舖裡自己分的群組
    pub sticker_groups: Vec<StickerGroup>,
    /// 自己做的貼紙裡，選了「不去背（整張照片）」的
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub whole_stickers: Vec<String>,
    pub ink: InkPrefs,
    /// 蕃茄鐘時間到的聲音：""（內建叮咚）、"none"、"alarm:Alarm01"、"file:路徑"
    pub timer_sound: String,
    /// 文字的預設樣式（沒有自己設定的便利貼都跟著它）
    pub text: TextStyle,
}

/// 文字樣式。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextStyle {
    /// 字型名稱（"" = 微軟正黑體）
    pub face: String,
    /// 0xRRGGBB；None = 依紙色自動
    pub color: Option<u32>,
    pub bold: bool,
}

/// 預設字型。
pub const DEFAULT_FACE: &str = "Microsoft JhengHei UI";

impl Board {
    /// 這張便利貼實際用的樣式（自己的設定蓋過預設）。
    pub fn style_of(&self, n: &Note) -> TextStyle {
        TextStyle {
            face: n.face.clone().unwrap_or_else(|| self.text.face.clone()),
            color: n.color.or(self.text.color),
            bold: n.bold.unwrap_or(self.text.bold),
        }
    }
}

pub const TRASH_LIMIT: usize = 20;

/// 自己做的貼紙放這裡（`<代碼>.jxr`，舊版做的是 `.png`）。
pub fn sticker_dir() -> PathBuf {
    crate::base::data_dir().join("stickers")
}
pub const STARRED_ID: &str = "starred";

pub fn image_dir() -> PathBuf {
    crate::base::data_dir().join("notes")
}

fn board_path() -> PathBuf {
    crate::base::data_dir().join("notes.json")
}

pub fn load() -> Board {
    std::fs::read(board_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn save(board: &Board) {
    let result = serde_json::to_vec_pretty(board)
        .map_err(std::io::Error::other)
        .and_then(|json| crate::base::write_atomic(&board_path(), &json));
    if let Err(e) = result {
        crate::base::log_error(format!("儲存便利貼失敗：{e}"));
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// 不重複的代碼（毫秒時間戳＋流水號）。
pub fn new_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("n{ms}{}", SEQ.fetch_add(1, Ordering::Relaxed) % 100)
}

/// 編輯完把新文字拆成行，沿用舊行的「劃掉」和附註（內容一樣的行一對一對應）。
pub fn merge_lines(old: &[Line], text: &str) -> Vec<Line> {
    let mut pool: Vec<Option<&Line>> = old.iter().map(Some).collect();
    let mut lines: Vec<Line> = text
        .split('\n')
        .map(|t| t.trim_end_matches('\r').trim_end())
        .map(|t| {
            let found = pool.iter_mut().find(|o| o.is_some_and(|l| l.text == t)).and_then(|o| o.take());
            match found {
                Some(l) => Line { text: t.to_string(), done: l.done, meta: l.meta.clone() },
                None => Line::new(t),
            }
        })
        .collect();
    while lines.last().is_some_and(|l| l.text.is_empty()) {
        lines.pop();
    }
    lines
}

/// 有密碼的便利貼平常顯示的偽裝內容：看起來就是普通的待辦（語系檔的 decoys，每份用換行分隔）。
pub fn decoys() -> Vec<String> {
    let list = crate::i18n::list("decoys");
    if list.is_empty() {
        vec!["To do
Laundry
Groceries".into()]
    } else {
        list
    }
}

pub fn decoy(seed: u64) -> Vec<Line> {
    let all = decoys();
    all[(seed as usize) % all.len()].lines().map(Line::new).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_done_and_meta() {
        let old = vec![
            Line { text: "買牛奶".into(), done: true, meta: String::new() },
            Line { text: "寄信".into(), done: false, meta: "m".into() },
        ];
        let lines = merge_lines(&old, "寄信\r\n買牛奶\r\n新的一行\r\n\r\n");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].meta, "m");
        assert!(lines[1].done);
        assert!(!lines[2].done);
    }

    #[test]
    fn angle_defaults_to_zero_and_round_trips() {
        // 舊資料沒有角度 = 0°；0° 不寫出來，有角度才寫
        let old: Note = serde_json::from_str(r#"{"id":"a","kind":"image","x":10,"y":20,"w":200,"h":240}"#).unwrap();
        assert_eq!(old.angle, 0.0);
        assert!(!serde_json::to_string(&old).unwrap().contains("angle"));
        let turned = Note { angle: -12.5, ..old };
        let back: Note = serde_json::from_str(&serde_json::to_string(&turned).unwrap()).unwrap();
        assert_eq!(back.angle, -12.5);
    }

    #[test]
    fn old_deco_converts() {
        let json = r#"{"id":"a","deco":"tape_corners","w":240}"#;
        let n: Note = serde_json::from_str(json).unwrap();
        assert_eq!(n.deco, Some(Deco::TapeCorners));
        assert_eq!(items_from_deco(n.deco.unwrap(), n.w).len(), 2);
        let out = serde_json::to_string(&n).unwrap();
        assert!(!out.contains("deco"));
    }
}
