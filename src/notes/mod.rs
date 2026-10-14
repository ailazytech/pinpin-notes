//! 桌面便利貼。
//!
//! 每張便利貼是一個分層視窗（render.rs 畫好整張交給系統）。平常完全不跑計時器，
//! 只有釘上去、劃線、解鎖這些短動畫時才開 16ms 的計時器，播完就關；拖曳時也只在滑鼠動的時候重畫。
//! 開機時從 notes.json 恢復。
//!
//! 操作：
//! - 拖紙張任何地方移動；靠近別張或螢幕邊緣會輕輕吸住對齊（按住 Alt 不吸），方向鍵一次動 1px（Shift 10px）
//! - 圖釘、夾子、貼紙、紙膠帶都可以拖；紙膠帶拖兩端可以拉長、轉方向；滾輪轉貼紙，Shift+滾輪縮放
//! - 滑過時左邊出現圓圈，點一下用紅筆劃掉那一行
//! - 連點兩下編輯；右下角拖曳改大小；A−／A+ 或 Ctrl+滾輪改字的大小
//! - 右鍵：紙的顏色和紙質、密碼、貼紙小舖、複製、存圖、刪除
//!
//! 狀態放在 thread_local 的 `Manager`，借用規則和主程式一樣：Win32 呼叫會重入視窗程序，
//! 所以會送訊息的呼叫（建立／摧毀視窗、選單、SetWindowPos）都放在借用外面。

mod assets;
mod backup;
mod crypto;
mod edit;
mod image;
mod ink;
mod promo;
mod render;
mod shop;
mod sprites;
mod stickers;
mod store;
mod style;
mod timer;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, ScreenToClient, MONITOR_DEFAULTTONULL};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, GetSaveFileNameW, OFN_ALLOWMULTISELECT, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Shell::{DragFinish, HDROP};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::i18n::t;
use crate::win::{self, wide, MenuItem};
use assets::Cat;
use image::Pixels;
use ink::{InkBtn, InkView};
use render::{AttachAnim, Btn, Fonts, Fx, Handle, Hits, ImageCache, Layout, Scene, Scramble, Shape};
use store::{Attach, Board, Kind, Layer, Line, Note, Paper, Stroke, Texture, PAPERS, STARRED_ID, TEXTURES};

pub use backup::{export_all, import};
/// 主迴圈先問這裡（編輯框、小舖改名框的按鍵）。
pub fn pre_translate(msg: &MSG) -> bool {
    edit::pre_translate(msg) || shop::pre_translate(msg)
}
pub use crypto::sha256_hex;
pub use shop::open as open_shop;
pub use store::now_secs;

/// 線上內容換了（下載完、或使用者切換「接收推送」）：重新載入貼紙包、廣告，小舖重畫。
pub fn reload_remote() {
    assets::load_packs(crate::app::update_flags().1);
    with(|m| m.assets = assets::Cache::new(m.scale));
    shop::refresh_remote();
}
pub use shop::render_png as render_shop;

const CLASS: PCWSTR = w!("PinPin.Note");
const TIMER_ANIM: usize = 1;
/// TrackMouseEvent 送來的「滑鼠離開視窗」訊息
const WM_MOUSELEAVE: u32 = 0x02A3;
/// 拖超過這麼多像素才算移動（不然是點擊）
const DRAG_THRESHOLD: i32 = 4;

// ───────────────────────────── 狀態 ─────────────────────────────

#[derive(Clone)]
enum AnimKind {
    /// 新便利貼出現：滑下來、淡入，再把附件一個個釘上
    Appear,
    /// 附件釘上去（None = 全部）
    Attach(Option<usize>),
    /// 劃線
    Strike(usize),
    /// 解鎖／上鎖：文字亂掉再變成另一份
    Scramble { from: Vec<Line>, to: Vec<Line> },
    /// 密碼錯：左右搖
    Shake,
    /// 時間到
    Ring,
}

impl AnimKind {
    fn ms(&self) -> f32 {
        match self {
            AnimKind::Appear => 640.0,
            AnimKind::Attach(_) => 440.0,
            AnimKind::Strike(_) => 230.0,
            AnimKind::Scramble { .. } => 950.0,
            AnimKind::Shake => 360.0,
            AnimKind::Ring => 1600.0,
        }
    }
}

struct Anim {
    kind: AnimKind,
    start: Instant,
}

impl Anim {
    fn t(&self) -> f32 {
        (self.start.elapsed().as_secs_f32() * 1000.0 / self.kind.ms()).min(1.0)
    }
}

/// 紙膠帶拖哪裡：整條、起點、終點。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemMode {
    Move,
    Start,
    End,
    /// 拖右上角的點旋轉
    Rotate,
    /// 拖右下角的點縮放
    Scale,
}

/// 左鍵按下時按在哪裡。
#[derive(Clone)]
enum Down {
    /// 紙張：準備拖整張（grab = 游標相對紙張左上角）。`click` = 沒拖動就放開的話要選取的附件
    Paper { start: POINT, grab: POINT, moved: bool, click: Option<usize> },
    /// 手寫中：正在寫一筆
    Ink,
    /// 手寫中：用橡皮擦
    Erase,
    /// 附件
    /// center = 附件中心的螢幕座標（旋轉、縮放用）
    Item { idx: usize, mode: ItemMode, start: POINT, orig: Attach, moved: bool, center: (f32, f32) },
    /// 右下角改大小
    Grip { start: POINT, w0: i32, h0: i32 },
    /// 按鈕或圓圈：放開時才處理
    Control,
}

/// 解鎖後放在記憶體裡的東西（關掉便利貼就消失）。
struct Unlocked {
    key: crypto::Key,
    salt: Vec<u8>,
    iterations: u32,
    lines: Vec<Line>,
}

struct NoteWin {
    id: String,
    hwnd: HWND,
    hover: bool,
    hot: Option<Btn>,
    lifted: bool,
    anim: Option<Anim>,
    scroll: i32,
    hits: Hits,
    down: Option<Down>,
    dragging_item: Option<usize>,
    /// 點過的附件：顯示旋轉、縮放的控制點
    selected: Option<usize>,
    image: Option<ImageCache>,
    editing: bool,
    unlocked: Option<Unlocked>,
    /// 手寫模式（工具列開著）
    ink: Option<InkState>,
}

#[derive(Default)]
struct InkState {
    eraser: bool,
    /// 正在寫的那一筆
    cur: Option<Stroke>,
}

struct Manager {
    board: Board,
    wins: Vec<NoteWin>,
    scale: f32,
    /// 依字級倍率（×10）快取的字型
    fonts: HashMap<String, Fonts>,
    assets: assets::Cache,
}

impl Manager {
    fn win(&mut self, hwnd: HWND) -> Option<&mut NoteWin> {
        self.wins.iter_mut().find(|w| w.hwnd == hwnd)
    }
    fn note(&mut self, id: &str) -> Option<&mut Note> {
        self.board.notes.iter_mut().find(|n| n.id == id)
    }
    fn note_of(&mut self, hwnd: HWND) -> Option<&mut Note> {
        let id = self.wins.iter().find(|w| w.hwnd == hwnd)?.id.clone();
        self.note(&id)
    }
    fn hwnd_of(&self, id: &str) -> Option<HWND> {
        self.wins.iter().find(|w| w.id == id).map(|w| w.hwnd)
    }
    /// 這張便利貼要用的字型（字級、字型、粗體）。
    fn fonts_for(&mut self, note: &Note) -> String {
        let st = self.board.style_of(note);
        self.fonts_key(note.font, &st.face, st.bold)
    }
    /// 依字級、字型、粗體快取字型。種類太多（一直換）就整個清掉重建，不會一直累積。
    fn fonts_key(&mut self, zoom: f32, face: &str, bold: bool) -> String {
        let z = (zoom * 20.0).round() as i32;
        let key = format!("{z}|{face}|{bold}");
        if !self.fonts.contains_key(&key) {
            if self.fonts.len() >= 40 {
                self.fonts.clear();
            }
            self.fonts.insert(key.clone(), Fonts::new(self.scale, z as f32 / 20.0, face, bold));
        }
        key
    }
    /// 字的顏色（None = 依紙色自動）。
    fn text_color(&self, note: &Note) -> Option<render::Rgb> {
        self.board.style_of(note).color.filter(|c| *c != style::AUTO).map(|c| render::Rgb((c >> 16) as u8, (c >> 8) as u8, c as u8))
    }
}

thread_local! {
    static M: RefCell<Option<Manager>> = const { RefCell::new(None) };
}

/// 短暫借用狀態；重入時回傳 None。
fn with<R>(f: impl FnOnce(&mut Manager) -> R) -> Option<R> {
    M.with(|cell| cell.try_borrow_mut().ok().and_then(|mut g| g.as_mut().map(f)))
}

fn px(v: f32) -> i32 {
    (v * win::dpi_scale()).round() as i32
}

fn lparam_point(lp: LPARAM) -> POINT {
    POINT { x: (lp.0 & 0xffff) as i16 as i32, y: ((lp.0 >> 16) & 0xffff) as i16 as i32 }
}

fn save_board() {
    with(|m| store::save(&m.board));
}

// ───────────────────────────── 初始化與恢復 ─────────────────────────────

pub fn init() {
    unsafe {
        let wc = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(note_proc),
            hInstance: win::hinstance(),
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
    }
    edit::register();
    shop::register();
    let scale = win::dpi_scale();
    let mut board = store::load();
    let migrated = migrate(&mut board, scale);
    assets::load_packs(crate::app::update_flags().1);
    M.with(|cell| {
        *cell.borrow_mut() = Some(Manager { board, wins: Vec::new(), scale, fonts: HashMap::new(), assets: assets::Cache::new(scale) })
    });
    if migrated {
        save_board();
    }
}

/// 舊版資料：位置是視窗左上角、裝飾是固定的幾種 → 改成紙張左上角和可移動的附件。
fn migrate(board: &mut Board, scale: f32) -> bool {
    let mut changed = false;
    for n in board.notes.iter_mut().chain(board.trash.iter_mut()) {
        if n.v >= store::VERSION {
            continue;
        }
        n.x += (28.0 * scale).round() as i32;
        n.y += (44.0 * scale).round() as i32;
        if n.items.is_empty() {
            n.items = store::items_from_deco(n.deco.unwrap_or_default(), n.w);
        }
        if n.font <= 0.0 {
            n.font = 1.0;
        }
        n.v = store::VERSION;
        changed = true;
    }
    for n in &mut board.notes {
        n.w = n.w.max(render::MIN_W);
        n.h = n.h.max(render::MIN_H);
        n.font = n.font.clamp(0.7, MAX_FONT);
    }
    if board.last_pin.is_empty() {
        board.last_pin = assets::DEFAULT_PIN.into();
    }
    changed
}

/// 開機後把便利貼放回桌面（不播動畫，省電）。
pub fn restore() {
    let Some((notes, hidden)) = with(|m| (m.board.notes.clone(), m.board.hidden)) else { return };
    if hidden {
        return;
    }
    for note in notes.iter().filter(|n| !n.hidden) {
        open_window(note, None);
    }
}

/// 重新開一次全部（匯入之後用）。
fn reopen_all() {
    let hwnds = with(|m| m.wins.iter().map(|w| w.hwnd).collect::<Vec<_>>()).unwrap_or_default();
    for h in hwnds {
        close_window(h);
    }
    restore();
}

/// 建立視窗並顯示。`anim` = 出現時要播的動畫。
fn open_window(note: &Note, anim: Option<AnimKind>) -> Option<HWND> {
    // 位置不在任何螢幕上（例如拔掉外接螢幕、別台電腦匯入）就搬回主螢幕
    let (scale, lay) = with(|m| (m.scale, render::layout(note, m.scale, &mut m.assets)))?;
    unsafe {
        let pw = (note.w as f32 * scale) as i32;
        let center = POINT { x: note.x + pw / 2, y: note.y + 20 };
        if MonitorFromPoint(center, MONITOR_DEFAULTTONULL).is_invalid() {
            let wa = render::work_area_at(POINT { x: 0, y: 0 });
            with(|m| {
                if let Some(n) = m.note(&note.id) {
                    n.x = wa.left + px(80.0);
                    n.y = wa.top + px(80.0);
                }
            });
        }
    }
    let mut ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_ACCEPTFILES;
    if note.layer == Layer::Top {
        ex |= WS_EX_TOPMOST;
    }
    let hwnd = unsafe {
        CreateWindowExW(ex, CLASS, w!("PinPin Notes"), WS_POPUP, note.x - lay.paper.left, note.y - lay.paper.top, lay.w, lay.h, None, None, Some(win::hinstance()), None).ok()?
    };
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, layer_code(note.layer));
    }
    let image = load_image(note);
    let animate = anim.is_some();
    with(|m| {
        m.wins.push(NoteWin {
            id: note.id.clone(),
            hwnd,
            hover: false,
            hot: None,
            lifted: false,
            anim: anim.map(|kind| Anim { kind, start: Instant::now() }),
            scroll: 0,
            hits: Hits::default(),
            down: None,
            dragging_item: None,
            selected: None,
            image,
            editing: false,
            unlocked: None,
            ink: None,
        });
    });
    redraw(hwnd);
    if note.kind == Kind::Timer {
        timer::schedule(hwnd);
    }
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        if note.layer == Layer::Desktop {
            let _ = SetWindowPos(hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        if animate {
            SetTimer(Some(hwnd), TIMER_ANIM, 16, None);
        }
    }
    Some(hwnd)
}

fn layer_code(layer: Layer) -> isize {
    match layer {
        Layer::Normal => 0,
        Layer::Top => 1,
        Layer::Desktop => 2,
    }
}

fn load_image(note: &Note) -> Option<ImageCache> {
    if note.kind != Kind::Image || note.image.is_empty() {
        return None;
    }
    match image::decode_file(&store::image_dir().join(&note.image)) {
        Ok(p) => Some(ImageCache { original: p, scaled: None }),
        Err(e) => {
            crate::base::log_error(format!("便利貼圖片讀取失敗：{e:#}"));
            None
        }
    }
}

// ───────────────────────────── 畫面 ─────────────────────────────

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// 重畫一張便利貼（位置也一起更新：附件可能讓視窗往左上長）。
fn redraw(hwnd: HWND) {
    with(|m| {
        let Some(wi) = m.wins.iter().position(|w| w.hwnd == hwnd) else { return };
        let id = m.wins[wi].id.clone();
        let Some(note) = m.board.notes.iter().find(|n| n.id == id).cloned() else { return };
        let zoom_key = m.fonts_for(&note);
        let text_color = m.text_color(&note);
        let Manager { wins, scale, fonts, assets, board, .. } = m;
        let scale = *scale;
        let prefs = board.ink;
        let w = &mut wins[wi];

        // 動畫狀態
        let mut lift = if w.lifted { 1.0 } else { 0.0 };
        let mut attach_anim = None;
        let mut strike = None;
        let mut alpha = 255u8;
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        let mut scramble: Option<(Vec<Line>, Vec<Line>, f32)> = None;
        let mut fx = Fx::None;
        if let Some(a) = &w.anim {
            let t = a.t();
            match &a.kind {
                AnimKind::Appear => {
                    alpha = ((t / 0.3).min(1.0) * 255.0) as u8;
                    dy = -14.0 * (1.0 - ease_out(t / 0.45));
                    lift = 1.0 - ease_out(t / 0.5);
                    attach_anim = Some(AttachAnim { which: None, t: ((t - 0.3) / 0.7).clamp(0.0, 1.0) });
                }
                AnimKind::Attach(which) => attach_anim = Some(AttachAnim { which: *which, t }),
                AnimKind::Strike(line) => strike = Some((*line, ease_out(t))),
                AnimKind::Scramble { from, to } => scramble = Some((from.clone(), to.clone(), t)),
                AnimKind::Shake => dx = (t * std::f32::consts::PI * 6.0).sin() * 8.0 * (1.0 - t),
                AnimKind::Ring => fx = Fx::Ring(t),
            }
        }
        let lines: Vec<Line> = w.unlocked.as_ref().map(|u| u.lines.clone()).unwrap_or_else(|| note.lines.clone());
        let lay = render::layout_ex(&note, scale, assets, w.ink.is_some());
        let pos = POINT {
            x: note.x - lay.paper.left + (dx * scale) as i32,
            y: note.y - lay.paper.top + (dy * scale) as i32,
        };
        let scene = Scene {
            note: &note,
            lines: &lines,
            scale,
            hover: w.hover || w.lifted,
            hot: w.hot,
            lift,
            attach_anim,
            dragging_item: w.dragging_item,
            selected: w.selected,
            strike,
            scramble: scramble.as_ref().map(|(from, to, t)| Scramble { from, to, t: *t }),
            unlocked: w.unlocked.is_some(),
            scroll: w.scroll,
            editing: w.editing,
            alpha,
            image: w.image.as_mut(),
            assets,
            fonts: &fonts[&zoom_key],
            text_color,
            now_ms: store::now_ms(),
            fx,
            ink: w.ink.as_ref().map(|i| InkView { prefs, eraser: i.eraser, cur: i.cur.as_ref() }),
        };
        let hits = render::present(render::Target::Window(hwnd, Some(pos)), scene);
        w.scroll = w.scroll.clamp(0, (hits.content_h - hits.view_h).max(0));
        w.hits = hits;
    });
}

fn tick(hwnd: HWND) {
    let done = with(|m| {
        let w = m.win(hwnd)?;
        let finished = w.anim.as_ref()?.t() >= 1.0;
        if finished {
            w.anim = None;
        }
        Some(finished)
    })
    .flatten();
    if done != Some(false) {
        unsafe {
            let _ = KillTimer(Some(hwnd), TIMER_ANIM);
        }
    }
    redraw(hwnd);
}

fn start_anim(hwnd: HWND, kind: AnimKind) {
    with(|m| {
        if let Some(w) = m.win(hwnd) {
            w.anim = Some(Anim { kind, start: Instant::now() });
        }
    });
    unsafe {
        SetTimer(Some(hwnd), TIMER_ANIM, 16, None);
    }
    redraw(hwnd);
}

// ───────────────────────────── 新增 ─────────────────────────────

/// 新便利貼的位置：滑鼠附近（紙張中心對準），不超出螢幕。
fn place_near(note: &mut Note, at: Option<POINT>, scale: f32) {
    let (w, h) = ((note.w as f32 * scale) as i32, (note.h as f32 * scale) as i32);
    let at = at.unwrap_or_else(|| {
        // 沒指定就放在螢幕右上角，一張張錯開
        let wa = render::work_area_at(win::cursor_pos());
        let n = with(|m| m.wins.len()).unwrap_or(0) as i32;
        POINT { x: wa.right - w / 2 - px(70.0) - (n % 6) * px(28.0), y: wa.top + h / 2 + px(60.0) + (n % 6) * px(28.0) }
    });
    let wa = render::work_area_at(at);
    note.x = (at.x - w / 2).clamp(wa.left + px(10.0), (wa.right - w - px(10.0)).max(wa.left));
    note.y = (at.y - h / 2).clamp(wa.top + px(30.0), (wa.bottom - h - px(10.0)).max(wa.top));
}

fn new_note(kind: Kind) -> Note {
    let (paper, texture, pin) = with(|m| (m.board.last_paper, m.board.last_texture, m.board.last_pin.clone())).unwrap_or_default();
    let mut note = Note { id: store::new_id(), kind, paper, texture, created: store::now_secs(), v: store::VERSION, ..Default::default() };
    let w = note.w as f32;
    note.items = match kind {
        Kind::Image => vec![Attach::tape("tape-pink", w / 2.0 - 55.0, 2.0, w / 2.0 + 55.0, -4.0)],
        _ if assets::cat_of(&pin) == Cat::Tape => vec![Attach::tape(&pin, w / 2.0 - 60.0, 0.0, w / 2.0 + 60.0, -3.0)],
        _ if pin.is_empty() => vec![],
        _ => vec![Attach::at(&pin, w / 2.0, 2.0)],
    };
    note
}

/// 新增文字便利貼。文字是空的話直接進入編輯。
pub fn new_text(text: &str, at: Option<POINT>) {
    let mut note = new_note(Kind::Text);
    note.lines = text.trim().lines().map(|l| Line::new(l.trim_end())).collect();
    let Some(scale) = with(|m| m.scale) else { return };
    if !note.lines.is_empty() {
        let key = with(|m| m.fonts_for(&note)).unwrap_or_default();
        note.h = with(|m| render::fit_height(&note.lines, note.w, scale, &m.fonts[&key])).unwrap_or(note.h);
    }
    place_near(&mut note, at, scale);
    let empty = note.is_empty();
    add_and_show(note, empty);
}

fn add_and_show(note: Note, edit: bool) -> Option<HWND> {
    let shown = with(|m| {
        m.board.notes.push(note.clone());
        store::save(&m.board);
        !m.board.hidden
    });
    if shown != Some(true) {
        set_hidden(false);
        return with(|m| m.hwnd_of(&note.id)).flatten();
    }
    let hwnd = open_window(&note, Some(AnimKind::Appear))?;
    if edit {
        edit::begin(hwnd);
    }
    Some(hwnd)
}

/// 新增手寫便利貼。`clear` = 透明紙（浮在桌面上的標語）。開好直接進入手寫。
pub fn new_ink(clear: bool, at: Option<POINT>) {
    let mut note = new_note(Kind::Text);
    if clear {
        note.paper = Paper::Clear;
        note.items.clear();
        note.w = 380;
        note.h = 170;
    }
    let Some(scale) = with(|m| m.scale) else { return };
    place_near(&mut note, at, scale);
    if let Some(hwnd) = add_and_show(note, false) {
        set_ink(hwnd, true);
    }
}

/// 新增蕃茄鐘。
pub fn new_timer(at: Option<POINT>) {
    let t = store::Timer::default();
    let (w, h) = timer::size_of(t.style);
    let mut note = Note { id: store::new_id(), kind: Kind::Timer, w, h, timer: Some(t), created: store::now_secs(), v: store::VERSION, ..Default::default() };
    let Some(scale) = with(|m| m.scale) else { return };
    place_near(&mut note, at, scale);
    add_and_show(note, false);
}

/// 新增圖片便利貼（拍立得樣式）。
pub fn new_image(pixels: Pixels, at: Option<POINT>) {
    let mut note = new_note(Kind::Image);
    note.paper = Paper::White;
    // 原圖太大就先縮，省記憶體也省磁碟
    let pixels = if pixels.w.max(pixels.h) > 1600 {
        let k = 1600.0 / pixels.w.max(pixels.h) as f32;
        pixels.resized((pixels.w as f32 * k) as u32, (pixels.h as f32 * k) as u32)
    } else {
        pixels
    };
    note.image = format!("{}.png", note.id);
    if let Err(e) = image::save_png(&pixels, &store::image_dir().join(&note.image)) {
        crate::base::log_error(format!("儲存便利貼圖片失敗：{e:#}"));
        return;
    }
    let Some(scale) = with(|m| m.scale) else { return };
    let frame_w = (pixels.w as f32 / scale).clamp(140.0, 300.0).round();
    let frame_h = (frame_w * pixels.h as f32 / pixels.w as f32).clamp(60.0, 420.0);
    note.w = (frame_w + 20.0) as i32;
    note.h = (frame_h + 10.0 + 38.0) as i32;
    let w = note.w as f32;
    note.items = vec![Attach::tape("tape-pink", w / 2.0 - 55.0, 2.0, w / 2.0 + 55.0, -4.0)];
    place_near(&mut note, at, scale);
    add_and_show(note, false);
}

/// 從剪貼簿新增：有圖片就是圖片便利貼，否則是文字。
pub fn paste_new(at: Option<POINT>) {
    if let Some(p) = image::clipboard_image(crate::app::main_hwnd()) {
        new_image(p, at);
    } else {
        let text = arboard::Clipboard::new().and_then(|mut c| c.get_text()).unwrap_or_default();
        new_text(&text, at);
    }
}

/// 貼紙直接貼在桌面上。
fn new_sticker(asset: &str, at: POINT) {
    let Some(scale) = with(|m| m.scale) else { return };
    let note = Note {
        id: store::new_id(),
        kind: Kind::Sticker,
        x: at.x,
        y: at.y,
        w: 0,
        h: 0,
        items: vec![Attach { asset: asset.into(), angle: tilt_for(asset), ..Default::default() }],
        created: store::now_secs(),
        v: store::VERSION,
        ..Default::default()
    };
    let _ = scale;
    add_and_show(note, false);
}

/// 自然一點：貼紙隨手貼會有一點點歪。
fn tilt_for(seed: &str) -> f32 {
    let h = seed.bytes().fold(store::now_secs() as u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
    (h % 17) as f32 - 8.0
}

/// 從貼紙小舖拖到某張便利貼上：在放開的位置貼上去，播動畫。
fn drop_asset(hwnd: HWND, asset: &str, at: POINT) {
    let cat = assets::cat_of(asset);
    let idx = with(|m| {
        let scale = m.scale;
        let note = m.note_of(hwnd)?;
        if matches!(note.kind, Kind::Sticker | Kind::Timer) {
            return None;
        }
        let (lx, ly) = ((at.x - note.x) as f32 / scale, (at.y - note.y) as f32 / scale);
        let a = match cat {
            Cat::Tape => Attach::tape(asset, lx - 55.0, ly + 3.0, lx + 55.0, ly - 3.0),
            Cat::Sticker => Attach { asset: asset.into(), x: lx, y: ly, angle: tilt_for(asset), ..Default::default() },
            _ => Attach::at(asset, lx, ly),
        };
        note.items.push(a);
        let idx = note.items.len() - 1;
        if matches!(cat, Cat::Pin | Cat::Clip | Cat::Tape) {
            m.board.last_pin = asset.into();
        }
        store::save(&m.board);
        Some(idx)
    })
    .flatten();
    if let Some(i) = idx {
        start_anim(hwnd, AnimKind::Attach(Some(i)));
    }
}

/// 拖曳放開：找游標底下的便利貼；沒有的話，貼紙就直接貼在桌面上。
fn drop_at(asset: &str, at: POINT) -> bool {
    let target = unsafe {
        let h = WindowFromPoint(at);
        let root = GetAncestor(h, GA_ROOT);
        let mut name = [0u16; 32];
        let n = GetClassNameW(root, &mut name) as usize;
        (String::from_utf16_lossy(&name[..n]) == "PinPin.Note").then_some(root)
    };
    let on_paper = target.filter(|h| with(|m| m.note_of(*h).is_some_and(|n| !matches!(n.kind, Kind::Sticker | Kind::Timer))).unwrap_or(false));
    match on_paper {
        Some(h) => {
            drop_asset(h, asset, at);
            true
        }
        None if assets::cat_of(asset) == Cat::Sticker => {
            new_sticker(asset, at);
            true
        }
        None => false,
    }
}

// ───────────────────────────── 星號記事本 ─────────────────────────────

fn now_label() -> String {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!("{}/{} {:02}:{:02}", t.wMonth, t.wDay, t.wHour, t.wMinute)
}

fn ensure_starred() -> Option<()> {
    if with(|m| m.note(STARRED_ID).is_some())? {
        return Some(());
    }
    let scale = with(|m| m.scale)?;
    let mut note = Note {
        id: STARRED_ID.into(),
        kind: Kind::Starred,
        paper: Paper::White,
        texture: Texture::Lined,
        w: 300,
        h: 380,
        hidden: true,
        created: store::now_secs(),
        v: store::VERSION,
        items: vec![Attach { asset: "star".into(), x: 286.0, y: 4.0, angle: 10.0, ..Default::default() }],
        ..Default::default()
    };
    let wa = render::work_area_at(win::cursor_pos());
    place_near(&mut note, Some(POINT { x: wa.right - px(190.0), y: wa.top + px(260.0) }), scale);
    with(|m| m.board.notes.push(note));
    Some(())
}

/// 把選取的文字加進星號記事本（最新的在最上面）。回傳目前共幾則。
pub fn star(text: &str, source: &str) -> usize {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() || ensure_starred().is_none() {
        return 0;
    }
    let source = source.trim_end_matches(".exe").trim_end_matches(".EXE");
    let meta = if source.is_empty() { now_label() } else { format!("{} · {}", now_label(), source) };
    let (count, hwnd) = with(|m| {
        let note = m.note(STARRED_ID)?;
        note.lines.insert(0, Line { text, done: false, meta });
        let count = note.lines.len();
        store::save(&m.board);
        Some((count, m.hwnd_of(STARRED_ID)))
    })
    .flatten()
    .unwrap_or((0, None));
    if let Some(hwnd) = hwnd {
        with(|m| {
            if let Some(w) = m.win(hwnd) {
                w.scroll = 0;
            }
        });
        start_anim(hwnd, AnimKind::Attach(None));
    }
    count
}

pub fn starred_visible() -> bool {
    with(|m| m.hwnd_of(STARRED_ID).is_some_and(|h| unsafe { IsWindowVisible(h).as_bool() })).unwrap_or(false)
}

/// 打開或收起星號記事本。
pub fn toggle_starred() {
    if starred_visible() {
        if let Some(h) = with(|m| m.hwnd_of(STARRED_ID)).flatten() {
            close_window(h);
        }
        with(|m| {
            if let Some(n) = m.note(STARRED_ID) {
                n.hidden = true;
            }
            store::save(&m.board);
        });
        return;
    }
    if ensure_starred().is_none() {
        return;
    }
    let note = with(|m| {
        let n = m.note(STARRED_ID)?;
        n.hidden = false;
        let n = n.clone();
        store::save(&m.board);
        Some(n)
    })
    .flatten();
    if let Some(note) = note {
        if is_hidden() {
            set_hidden(false);
        } else {
            open_window(&note, Some(AnimKind::Appear));
        }
    }
}

// ───────────────────────────── 全部顯示／收起、刪除、復原 ─────────────────────────────

pub fn is_hidden() -> bool {
    with(|m| m.board.hidden).unwrap_or(false)
}

pub fn trash_count() -> usize {
    with(|m| m.board.trash.len()).unwrap_or(0)
}

/// 全部收起來（不刪除；有密碼的會重新鎖上）或放回桌面。
pub fn set_hidden(hidden: bool) {
    let Some(wins) = with(|m| {
        m.board.hidden = hidden;
        store::save(&m.board);
        m.wins.iter().map(|w| w.hwnd).collect::<Vec<_>>()
    }) else {
        return;
    };
    if hidden {
        for h in wins {
            close_window(h);
        }
    } else if wins.is_empty() {
        let notes = with(|m| m.board.notes.clone()).unwrap_or_default();
        for n in notes.iter().filter(|n| !n.hidden) {
            open_window(n, Some(AnimKind::Appear));
        }
    }
}

fn close_window(hwnd: HWND) {
    edit::close_for(hwnd);
    with(|m| m.wins.retain(|w| w.hwnd != hwnd));
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

/// 刪掉（放進回收，可以從系統匣復原）。星號記事本只會收起來。
fn delete(hwnd: HWND) {
    let Some(id) = with(|m| m.win(hwnd).map(|w| w.id.clone())).flatten() else { return };
    if id == STARRED_ID {
        toggle_starred();
        return;
    }
    close_window(hwnd);
    with(|m| {
        if let Some(i) = m.board.notes.iter().position(|n| n.id == id) {
            let note = m.board.notes.remove(i);
            if !note.is_empty() {
                m.board.trash.push(note);
            }
            while m.board.trash.len() > store::TRASH_LIMIT {
                let old = m.board.trash.remove(0);
                if !old.image.is_empty() {
                    let _ = std::fs::remove_file(store::image_dir().join(&old.image));
                }
            }
        }
        store::save(&m.board);
    });
}

/// 復原最近刪掉的一張。
pub fn undo_delete() {
    let Some(note) = with(|m| {
        let n = m.board.trash.pop()?;
        m.board.notes.push(n.clone());
        store::save(&m.board);
        Some(n)
    })
    .flatten() else {
        return;
    };
    if is_hidden() {
        set_hidden(false);
    } else {
        open_window(&note, Some(AnimKind::Appear));
    }
}

// ───────────────────────────── 文字內容（含密碼） ─────────────────────────────

/// 目前顯示的內容（解鎖時是真正的內容）。
fn current_lines(hwnd: HWND) -> Option<Vec<Line>> {
    with(|m| {
        let w = m.win(hwnd)?;
        if let Some(u) = &w.unlocked {
            return Some(u.lines.clone());
        }
        Some(m.note_of(hwnd)?.lines.clone())
    })
    .flatten()
}

/// 改內容：解鎖中就改真正的內容並重新加密，否則改 note.lines。
fn update_lines(hwnd: HWND, f: impl FnOnce(&mut Vec<Line>)) -> Option<()> {
    with(|m| {
        let wi = m.wins.iter().position(|w| w.hwnd == hwnd)?;
        let id = m.wins[wi].id.clone();
        if let Some(u) = m.wins[wi].unlocked.as_mut() {
            f(&mut u.lines);
            let plain = serde_json::to_vec(&u.lines).ok()?;
            let secret = crypto::seal(&u.key, &u.salt, u.iterations, &plain).ok()?;
            m.note(&id)?.secret = Some(secret);
        } else {
            f(&mut m.note(&id)?.lines);
        }
        store::save(&m.board);
        Some(())
    })
    .flatten()
}

/// 設定新密碼：真正的內容加密，紙上換成偽裝內容（播亂碼動畫）。
fn set_password(hwnd: HWND, password: &str) {
    let Some(real) = current_lines(hwnd) else { return };
    let Ok(plain) = serde_json::to_vec(&real) else { return };
    let (secret, _) = match crypto::seal_new(password, &plain) {
        Ok(v) => v,
        Err(e) => {
            crate::base::log_error(format!("設定密碼失敗：{e:#}"));
            return;
        }
    };
    let decoy = with(|m| {
        let n = m.note_of(hwnd)?;
        let decoy = store::decoy(n.created ^ store::now_secs());
        n.lines = decoy.clone();
        n.secret = Some(secret);
        if let Some(w) = m.win(hwnd) {
            w.unlocked = None;
        }
        store::save(&m.board);
        Some(decoy)
    })
    .flatten();
    if let Some(decoy) = decoy {
        start_anim(hwnd, AnimKind::Scramble { from: real, to: decoy });
    }
}

/// 用密碼解開；密碼錯就搖一搖。
fn unlock(hwnd: HWND, password: &str) -> bool {
    let Some((secret, decoy)) = with(|m| m.note_of(hwnd).and_then(|n| Some((n.secret.clone()?, n.lines.clone())))).flatten() else {
        return false;
    };
    match crypto::open(password, &secret) {
        Ok((plain, key)) => {
            let lines: Vec<Line> = serde_json::from_slice(&plain).unwrap_or_default();
            with(|m| {
                if let Some(w) = m.win(hwnd) {
                    w.unlocked = Some(Unlocked {
                        key,
                        salt: crypto::salt_of(&secret),
                        iterations: secret.iterations,
                        lines: lines.clone(),
                    });
                }
            });
            start_anim(hwnd, AnimKind::Scramble { from: decoy, to: lines });
            true
        }
        Err(_) => {
            start_anim(hwnd, AnimKind::Shake);
            false
        }
    }
}

/// 鎖回去：亂碼動畫變回偽裝內容。
fn lock(hwnd: HWND) {
    let pair = with(|m| {
        let real = m.win(hwnd)?.unlocked.take()?.lines;
        Some((real, m.note_of(hwnd)?.lines.clone()))
    })
    .flatten();
    if let Some((real, decoy)) = pair {
        start_anim(hwnd, AnimKind::Scramble { from: real, to: decoy });
    }
}

fn remove_password(hwnd: HWND) {
    with(|m| {
        let real = m.win(hwnd)?.unlocked.take()?.lines;
        let n = m.note_of(hwnd)?;
        n.lines = real;
        n.secret = None;
        store::save(&m.board);
        Some(())
    });
    redraw(hwnd);
}

fn change_decoy(hwnd: HWND) {
    with(|m| {
        let n = m.note_of(hwnd)?;
        let all = store::decoys();
        let next = (all.iter().position(|d| d.lines().next() == n.lines.first().map(|l| l.text.as_str())).unwrap_or(0) + 1) as u64;
        n.lines = store::decoy(next);
        store::save(&m.board);
        Some(())
    });
}

// ───────────────────────────── 滑鼠與鍵盤 ─────────────────────────────

fn hit_button(hwnd: HWND, pt: POINT) -> Option<Btn> {
    with(|m| m.win(hwnd).and_then(|w| w.hits.buttons.iter().find(|(_, r)| render::in_rect(r, pt.x, pt.y)).map(|(b, _)| *b))).flatten()
}

fn hit_bullet(hwnd: HWND, pt: POINT) -> Option<usize> {
    with(|m| m.win(hwnd).and_then(|w| w.hits.bullets.iter().find(|(_, r)| render::in_rect(r, pt.x, pt.y)).map(|(i, _)| *i))).flatten()
}

/// 游標底下最上面的附件，和紙膠帶拖的是哪個部位。
fn hit_item(hwnd: HWND, pt: POINT) -> Option<(usize, ItemMode)> {
    with(|m| {
        let w = m.win(hwnd)?;
        let (x, y) = (pt.x as f32 + 0.5, pt.y as f32 + 0.5);
        w.hits.items.iter().rev().find(|(_, s)| s.contains(x, y)).map(|(i, s)| {
            let mode = match s.tape_pos(x, y) {
                Some(t) if t < 0.22 => ItemMode::Start,
                Some(t) if t > 0.78 => ItemMode::End,
                _ => ItemMode::Move,
            };
            (*i, mode)
        })
    })
    .flatten()
}

/// 選取中附件的控制點（游標 9px 內）。
fn hit_handle(hwnd: HWND, pt: POINT) -> Option<(usize, Handle)> {
    let reach = px(9.0) as f32;
    with(|m| {
        let w = m.win(hwnd)?;
        w.selected?;
        w.hits
            .handles
            .iter()
            .find(|(_, _, x, y)| (x - pt.x as f32).powi(2) + (y - pt.y as f32).powi(2) <= reach * reach)
            .map(|(i, h, _, _)| (*i, *h))
    })
    .flatten()
}

/// 附件中心（視窗座標）。
fn item_center(hwnd: HWND, idx: usize) -> Option<(f32, f32)> {
    with(|m| {
        let w = m.win(hwnd)?;
        w.hits.items.iter().find(|(i, _)| *i == idx).map(|(_, s)| match *s {
            Shape::Box { cx, cy, .. } => (cx, cy),
            Shape::Tape { x1, y1, x2, y2, .. } => ((x1 + x2) / 2.0, (y1 + y2) / 2.0),
        })
    })
    .flatten()
}

fn on_grip(hwnd: HWND, pt: POINT) -> bool {
    with(|m| m.win(hwnd).is_some_and(|w| w.hover && render::in_rect(&w.hits.grip, pt.x, pt.y))).unwrap_or(false)
        && with(|m| m.note_of(hwnd).is_some_and(|n| !matches!(n.kind, Kind::Sticker | Kind::Timer))).unwrap_or(false)
}

fn kind_of(hwnd: HWND) -> Option<Kind> {
    with(|m| m.note_of(hwnd).map(|n| n.kind)).flatten()
}

fn ink_on(hwnd: HWND) -> bool {
    with(|m| m.win(hwnd).is_some_and(|w| w.ink.is_some())).unwrap_or(false)
}

// ───────────────────────────── 手寫 ─────────────────────────────

/// 進入或離開手寫模式（紙張下面出現工具列）。
fn set_ink(hwnd: HWND, on: bool) {
    if on {
        edit::close_for(hwnd);
    }
    with(|m| {
        if let Some(w) = m.win(hwnd) {
            w.ink = on.then(InkState::default);
            w.selected = None;
            w.down = None;
        }
    });
    redraw(hwnd);
    if on {
        unsafe {
            // 要收得到 Esc、Ctrl+Z
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

fn hit_ink_btn(hwnd: HWND, pt: POINT) -> Option<InkBtn> {
    with(|m| m.win(hwnd).and_then(|w| w.hits.ink_buttons.iter().find(|(_, r)| render::in_rect(r, pt.x, pt.y)).map(|(b, _)| *b))).flatten()
}

/// 螢幕座標 → 紙上的座標（邏輯像素 × 4），超出紙張就貼齊邊緣。
fn ink_point(hwnd: HWND, screen: POINT) -> Option<(i32, i32)> {
    with(|m| {
        let s = m.scale;
        let n = m.note_of(hwnd)?;
        let x = ((screen.x - n.x) as f32 / s).clamp(0.0, n.w as f32);
        let y = ((screen.y - n.y) as f32 / s).clamp(0.0, n.h as f32);
        Some(((x * 4.0).round() as i32, (y * 4.0).round() as i32))
    })
    .flatten()
}

fn ink_start(hwnd: HWND, screen: POINT) {
    let Some((x, y)) = ink_point(hwnd, screen) else { return };
    with(|m| {
        let p = m.board.ink;
        let w = m.win(hwnd)?;
        w.ink.as_mut()?.cur = Some(Stroke { tool: p.tool, color: p.color, size: ink::width_of(p.tool, p.size), pts: vec![x, y] });
        Some(())
    });
    redraw(hwnd);
}

fn ink_move(hwnd: HWND, screen: POINT) {
    let Some((x, y)) = ink_point(hwnd, screen) else { return };
    let changed = with(|m| {
        let st = m.win(hwnd)?.ink.as_mut()?.cur.as_mut()?;
        let n = st.pts.len();
        let (lx, ly) = (st.pts[n - 2], st.pts[n - 1]);
        // 太近的點不收；滑鼠的抖動稍微抹平一點
        if (x - lx).abs().max((y - ly).abs()) < 3 {
            return Some(false);
        }
        st.pts.push(lx + ((x - lx) as f32 * 0.7).round() as i32);
        st.pts.push(ly + ((y - ly) as f32 * 0.7).round() as i32);
        Some(true)
    })
    .flatten();
    if changed == Some(true) {
        redraw(hwnd);
    }
}

fn ink_end(hwnd: HWND, screen: POINT) {
    let end = ink_point(hwnd, screen);
    with(|m| {
        let mut st = m.win(hwnd)?.ink.as_mut()?.cur.take()?;
        if let Some((x, y)) = end {
            if st.pts.len() >= 2 && (st.pts[st.pts.len() - 2], st.pts[st.pts.len() - 1]) != (x, y) {
                st.pts.extend([x, y]);
            }
        }
        st.pts = ink::simplify(&st.pts);
        m.note_of(hwnd)?.strokes.push(st);
        store::save(&m.board);
        Some(())
    });
    redraw(hwnd);
}

/// 橡皮擦：碰到的整筆拿掉。
fn ink_erase(hwnd: HWND, screen: POINT) {
    let Some((x, y)) = ink_point(hwnd, screen) else { return };
    let removed = with(|m| {
        let n = m.note_of(hwnd)?;
        let before = n.strokes.len();
        n.strokes.retain(|st| !ink::stroke_hit(st, x as f32 / 4.0, y as f32 / 4.0, 6.0));
        let removed = n.strokes.len() != before;
        if removed {
            store::save(&m.board);
        }
        Some(removed)
    })
    .flatten();
    if removed == Some(true) {
        redraw(hwnd);
    }
}

fn ink_undo(hwnd: HWND) {
    with(|m| {
        m.note_of(hwnd)?.strokes.pop()?;
        store::save(&m.board);
        Some(())
    });
    redraw(hwnd);
}

fn on_ink_button(hwnd: HWND, btn: InkBtn) {
    match btn {
        InkBtn::Done => return set_ink(hwnd, false),
        InkBtn::Undo => return ink_undo(hwnd),
        _ => {}
    }
    with(|m| {
        let p = &mut m.board.ink;
        match btn {
            InkBtn::Tool(t) => p.tool = t,
            InkBtn::Color(i) => p.color = ink::COLORS[i.min(ink::COLORS.len() - 1)],
            InkBtn::Size(k) => p.size = k.min(2),
            _ => {}
        }
        store::save(&m.board);
        if let Some(i) = m.win(hwnd).and_then(|w| w.ink.as_mut()) {
            i.eraser = match btn {
                InkBtn::Eraser => !i.eraser,
                InkBtn::Size(_) => i.eraser,
                _ => false,
            };
        }
    });
    redraw(hwnd);
}

/// 劃掉或取消劃掉一行。
fn toggle_line(hwnd: HWND, line: usize) {
    let mut now_done = None;
    update_lines(hwnd, |lines| {
        if let Some(l) = lines.get_mut(line) {
            l.done = !l.done;
            now_done = Some(l.done);
        }
    });
    match now_done {
        Some(true) => start_anim(hwnd, AnimKind::Strike(line)),
        Some(false) => redraw(hwnd),
        None => {}
    }
}

/// 磁吸：紙張靠近別張的邊或螢幕邊緣幾個像素內就對齊。吸的範圍很小，繼續拖就會離開，方便微調。
fn snap(hwnd: HWND, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    if unsafe { GetKeyState(VK_MENU.0 as i32) } < 0 {
        return (x, y);
    }
    let reach = px(7.0);
    let gap = px(10.0);
    let near = px(80.0);
    let others: Vec<RECT> = with(|m| {
        let scale = m.scale;
        m.wins
            .iter()
            .filter(|ow| ow.hwnd != hwnd)
            .filter_map(|ow| m.board.notes.iter().find(|n| n.id == ow.id))
            .filter(|n| n.kind != Kind::Sticker)
            .map(|n| RECT { left: n.x, top: n.y, right: n.x + (n.w as f32 * scale) as i32, bottom: n.y + (n.h as f32 * scale) as i32 })
            .collect()
    })
    .unwrap_or_default();
    let (mut best_x, mut best_y): (Option<i32>, Option<i32>) = (None, None);
    let consider = |best: &mut Option<i32>, d: i32| {
        if d.abs() <= reach && best.is_none_or(|b| d.abs() < b.abs()) {
            *best = Some(d);
        }
    };
    for o in &others {
        if y < o.bottom + near && y + h > o.top - near {
            for d in [o.left - x, o.right - (x + w), o.right + gap - x, o.left - gap - (x + w)] {
                consider(&mut best_x, d);
            }
        }
        if x < o.right + near && x + w > o.left - near {
            for d in [o.top - y, o.bottom - (y + h), o.bottom + gap - y, o.top - gap - (y + h)] {
                consider(&mut best_y, d);
            }
        }
    }
    let wa = render::work_area_at(POINT { x: x + w / 2, y: y + h / 2 });
    for d in [wa.left + gap - x, wa.right - gap - (x + w)] {
        consider(&mut best_x, d);
    }
    for d in [wa.top + gap - y, wa.bottom - gap - (y + h)] {
        consider(&mut best_y, d);
    }
    (x + best_x.unwrap_or(0), y + best_y.unwrap_or(0))
}

/// 把視窗搬到紙張的新位置（只動位置，不重畫）。
fn move_paper(hwnd: HWND, x: i32, y: i32) {
    let off = with(|m| {
        let lay: Layout = m.win(hwnd)?.hits.layout;
        let n = m.note_of(hwnd)?;
        n.x = x;
        n.y = y;
        Some((lay.paper.left, lay.paper.top))
    })
    .flatten();
    if let Some((ox, oy)) = off {
        unsafe {
            let _ = SetWindowPos(hwnd, None, x - ox, y - oy, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
    }
}

fn on_button(hwnd: HWND, btn: Btn) {
    match btn {
        Btn::New => {
            let rc = win::window_rect(hwnd);
            new_text("", Some(POINT { x: rc.right + px(100.0), y: rc.top + px(140.0) }));
        }
        Btn::Menu if kind_of(hwnd) == Some(Kind::Timer) => timer::menu(hwnd),
        Btn::Menu => context_menu(hwnd),
        Btn::Delete => delete(hwnd),
        Btn::Lock => lock(hwnd),
        Btn::Smaller => zoom(hwnd, -0.1),
        Btn::Bigger => zoom(hwnd, 0.1),
        Btn::Ink => set_ink(hwnd, true),
        Btn::Reset => timer::reset(hwnd),
    }
}

/// 字的最大倍率（寫標語用）
const MAX_FONT: f32 = 6.0;

/// A−／A+、Ctrl+滾輪：每次放大縮小 12%（字越大跳越多，不用按很久）。
fn zoom(hwnd: HWND, delta: f32) {
    let Some(cur) = with(|m| m.note_of(hwnd).map(|n| n.font)).flatten() else { return };
    let next = if delta > 0.0 { cur * 1.12 } else { cur / 1.12 };
    set_font_size(hwnd, next);
}

/// 設定字的大小（字變大放不下時，紙自動長高）。
fn set_font_size(hwnd: HWND, size: f32) {
    with(|m| {
        if let Some(n) = m.note_of(hwnd) {
            n.font = ((size * 20.0).round() / 20.0).clamp(0.7, MAX_FONT);
        }
        store::save(&m.board);
    });
    grow_to_fit(hwnd);
    redraw(hwnd);
}

/// 字放不下就把紙拉高（只長不縮）。
fn grow_to_fit(hwnd: HWND) {
    with(|m| {
        let n = m.note_of(hwnd)?.clone();
        if !matches!(n.kind, Kind::Text) || n.lines.is_empty() {
            return None;
        }
        let key = m.fonts_for(&n);
        let need = render::fit_height(&n.lines, n.w, m.scale, &m.fonts[&key]);
        let note = m.note_of(hwnd)?;
        if need > note.h {
            note.h = need;
            store::save(&m.board);
        }
        Some(())
    });
}

/// 滾輪轉貼紙（Shift = 縮放）。
fn turn_item(hwnd: HWND, idx: usize, steps: i32, scale_mode: bool) {
    with(|m| {
        let n = m.note_of(hwnd)?;
        let a = n.items.get_mut(idx)?;
        if assets::cat_of(&a.asset) == Cat::Tape {
            return None;
        }
        if scale_mode {
            a.scale = (a.scale + steps as f32 * 0.08).clamp(0.4, 2.5);
        } else {
            a.angle = (a.angle + steps as f32 * 6.0) % 360.0;
        }
        store::save(&m.board);
        Some(())
    });
    redraw(hwnd);
}

/// 拖附件時依游標更新位置（邏輯像素，相對紙張）。
fn drag_item(hwnd: HWND, idx: usize, mode: ItemMode, orig: &Attach, start: POINT, now: POINT, center: (f32, f32)) {
    with(|m| {
        let scale = m.scale;
        let n = m.note_of(hwnd)?;
        let (w, h) = (n.w as f32, n.h as f32);
        let sticker = n.kind == Kind::Sticker;
        let a = n.items.get_mut(idx)?;
        let (dx, dy) = ((now.x - start.x) as f32 / scale, (now.y - start.y) as f32 / scale);
        let clamp = |v: f32, max: f32| if sticker { v } else { v.clamp(-70.0, max + 70.0) };
        match mode {
            ItemMode::Move => {
                a.x = clamp(orig.x + dx, w);
                a.y = clamp(orig.y + dy, h);
                a.x2 = orig.x2 + (a.x - orig.x);
                a.y2 = orig.y2 + (a.y - orig.y);
            }
            ItemMode::Start => {
                a.x = clamp(orig.x + dx, w);
                a.y = clamp(orig.y + dy, h);
            }
            ItemMode::End => {
                a.x2 = clamp(orig.x2 + dx, w);
                a.y2 = clamp(orig.y2 + dy, h);
            }
            ItemMode::Rotate => {
                // 以中心為軸，游標轉了多少角度就轉多少；按住 Shift 每 15° 一格
                let (cx, cy) = center;
                let a0 = (start.y as f32 - cy).atan2(start.x as f32 - cx);
                let a1 = (now.y as f32 - cy).atan2(now.x as f32 - cx);
                let mut deg = orig.angle + (a1 - a0).to_degrees();
                if unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0 {
                    deg = (deg / 15.0).round() * 15.0;
                }
                a.angle = deg.rem_euclid(360.0);
            }
            ItemMode::Scale => {
                // 離中心的距離比例 = 縮放比例
                let (cx, cy) = center;
                let d0 = ((start.x as f32 - cx).powi(2) + (start.y as f32 - cy).powi(2)).sqrt().max(4.0);
                let d1 = ((now.x as f32 - cx).powi(2) + (now.y as f32 - cy).powi(2)).sqrt();
                a.scale = (orig.scale * d1 / d0).clamp(0.3, 3.0);
            }
        }
        Some(())
    });
    redraw(hwnd);
}

unsafe extern "system" fn note_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_SETCURSOR if (lp.0 & 0xffff) as u32 == HTCLIENT => {
            let mut pt = win::cursor_pos();
            let _ = ScreenToClient(hwnd, &mut pt);
            let ink = with(|m| m.win(hwnd).filter(|w| w.ink.is_some()).map(|w| (w.hits.layout.paper, w.hits.bar))).flatten();
            let cursor = if let Some((paper, bar)) = ink {
                // 手寫中：紙上是十字、工具列按鈕是手指、工具列空白處可以拖
                if hit_ink_btn(hwnd, pt).is_some() {
                    Some(IDC_HAND)
                } else if render::in_rect(&bar, pt.x, pt.y) {
                    Some(IDC_SIZEALL)
                } else if render::in_rect(&paper, pt.x, pt.y) {
                    Some(IDC_CROSS)
                } else {
                    None
                }
            } else if let Some((_, h)) = hit_handle(hwnd, pt) {
                Some(if h == Handle::Scale { IDC_SIZENWSE } else { IDC_HAND })
            } else if hit_button(hwnd, pt).is_some() || hit_bullet(hwnd, pt).is_some() {
                Some(IDC_HAND)
            } else if on_grip(hwnd, pt) {
                Some(IDC_SIZENWSE)
            } else if hit_item(hwnd, pt).is_some() {
                Some(IDC_SIZEALL)
            } else {
                None
            };
            if let Some(c) = cursor.and_then(|c| LoadCursorW(None, c).ok()) {
                SetCursor(Some(c));
                return LRESULT(1);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_MOUSEMOVE => {
            let pt = lparam_point(lp);
            let mut tme = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: hwnd, dwHoverTime: 0 };
            let _ = TrackMouseEvent(&mut tme);
            on_mouse_move(hwnd, pt);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            let changed = with(|m| {
                let w = m.win(hwnd)?;
                if w.down.is_some() {
                    return Some(false);
                }
                let changed = w.hover || w.hot.is_some() || w.selected.is_some();
                w.hover = false;
                w.hot = None;
                // 滑鼠離開就收起控制點
                w.selected = None;
                Some(changed)
            })
            .flatten();
            if changed == Some(true) {
                redraw(hwnd);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let pt = lparam_point(lp);
            edit::close_for(hwnd);
            SetCapture(hwnd);
            let screen = win::cursor_pos();
            let item_down = |idx: usize, mode: ItemMode| {
                let orig = with(|m| m.note_of(hwnd).and_then(|n| n.items.get(idx).cloned())).flatten().unwrap_or_default();
                let rc = win::window_rect(hwnd);
                let center = item_center(hwnd, idx).map_or((screen.x as f32, screen.y as f32), |(x, y)| (rc.left as f32 + x, rc.top as f32 + y));
                Down::Item { idx, mode, start: screen, orig, moved: false, center }
            };
            let paper_grab = || {
                let (nx, ny) = with(|m| m.note_of(hwnd).map(|n| (n.x, n.y))).flatten().unwrap_or((0, 0));
                Down::Paper { start: screen, grab: POINT { x: screen.x - nx, y: screen.y - ny }, moved: false, click: None }
            };
            // 手寫中：工具列按鈕、工具列空白（拖整張）、紙上（寫字或擦掉）
            if let Some((paper, bar, eraser)) = with(|m| m.win(hwnd).and_then(|w| Some((w.hits.layout.paper, w.hits.bar, w.ink.as_ref()?.eraser)))).flatten() {
                let down = if hit_ink_btn(hwnd, pt).is_some() {
                    Down::Control
                } else if render::in_rect(&paper, pt.x, pt.y) && !render::in_rect(&bar, pt.x, pt.y) {
                    if eraser {
                        ink_erase(hwnd, screen);
                        Down::Erase
                    } else {
                        ink_start(hwnd, screen);
                        Down::Ink
                    }
                } else {
                    paper_grab()
                };
                with(|m| {
                    if let Some(w) = m.win(hwnd) {
                        w.down = Some(down);
                    }
                });
                return LRESULT(0);
            }
            // 先看控制點（選取中的附件），再看按鈕、圓圈、附件、右下角、紙張
            let handle = hit_handle(hwnd, pt);
            let mut item = if handle.is_none() { hit_item(hwnd, pt) } else { None };
            // 圖釘、夾子、紙膠帶是「固定紙張用的」：沒選取時拖它就是拖整張（點一下才選取，選取後才能單獨移動）
            let prev = with(|m| m.win(hwnd).and_then(|w| w.selected)).flatten();
            let mut click = None;
            if let Some((idx, _)) = item {
                let fastener = with(|m| m.note_of(hwnd).and_then(|n| Some(n.kind != Kind::Sticker && assets::cat_of(&n.items.get(idx)?.asset) != Cat::Sticker))).flatten().unwrap_or(false);
                if fastener && prev != Some(idx) {
                    click = Some(idx);
                    item = None;
                }
            }
            let selected = handle.map(|(i, _)| i).or(item.map(|(i, _)| i));
            let changed = with(|m| {
                let w = m.win(hwnd)?;
                let changed = w.selected != selected;
                w.selected = selected;
                Some(changed)
            })
            .flatten();
            if changed == Some(true) {
                redraw(hwnd);
            }
            let down = if let Some((idx, h)) = handle {
                item_down(
                    idx,
                    match h {
                        Handle::Rotate => ItemMode::Rotate,
                        Handle::Scale => ItemMode::Scale,
                        Handle::TapeStart => ItemMode::Start,
                        Handle::TapeEnd => ItemMode::End,
                    },
                )
            } else if hit_button(hwnd, pt).is_some() || hit_bullet(hwnd, pt).is_some() {
                Down::Control
            } else if let Some((idx, mode)) = item {
                item_down(idx, mode)
            } else if on_grip(hwnd, pt) {
                let (w0, h0) = with(|m| m.note_of(hwnd).map(|n| (n.w, n.h))).flatten().unwrap_or((200, 200));
                Down::Grip { start: screen, w0, h0 }
            } else {
                match paper_grab() {
                    Down::Paper { start, grab, moved, .. } => Down::Paper { start, grab, moved, click },
                    d => d,
                }
            };
            with(|m| {
                if let Some(w) = m.win(hwnd) {
                    w.down = Some(down);
                }
            });
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let pt = lparam_point(lp);
            let _ = ReleaseCapture();
            on_mouse_up(hwnd, pt);
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK => {
            let pt = lparam_point(lp);
            if kind_of(hwnd) == Some(Kind::Timer) {
                // 蕃茄鐘：連點兩下 = 第二次點（開始又暫停）
                if hit_button(hwnd, pt).is_none() {
                    timer::toggle(hwnd);
                }
                return LRESULT(0);
            }
            if ink_on(hwnd) {
                // 手寫中連點：當成再點一下（畫一個點）
                return note_proc(hwnd, WM_LBUTTONDOWN, wp, lp);
            }
            if let Some(line) = hit_bullet(hwnd, pt) {
                toggle_line(hwnd, line);
            } else if hit_button(hwnd, pt).is_none() && hit_item(hwnd, pt).is_none() {
                let (locked, unlocked, kind) = with(|m| {
                    let u = m.win(hwnd).is_some_and(|w| w.unlocked.is_some());
                    m.note_of(hwnd).map(|n| (n.locked(), u, n.kind))
                })
                .flatten()
                .unwrap_or((false, false, Kind::Text));
                if kind == Kind::Sticker {
                } else if locked && !unlocked {
                    edit::prompt(hwnd, edit::PromptMode::Unlock);
                } else {
                    edit::begin(hwnd);
                }
            }
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            let pt = lparam_point(lp);
            if kind_of(hwnd) == Some(Kind::Timer) {
                timer::menu(hwnd);
                return LRESULT(0);
            }
            match hit_item(hwnd, pt).filter(|_| !ink_on(hwnd)) {
                Some((idx, _)) => item_menu(hwnd, idx),
                None => context_menu(hwnd),
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wp.0 >> 16) & 0xffff) as i16 as i32;
            let steps = delta.signum();
            let mut pt = win::cursor_pos();
            let _ = ScreenToClient(hwnd, &mut pt);
            if kind_of(hwnd) == Some(Kind::Timer) {
                timer::adjust(hwnd, steps, GetKeyState(VK_SHIFT.0 as i32) < 0);
            } else if GetKeyState(VK_CONTROL.0 as i32) < 0 {
                zoom(hwnd, steps as f32 * 0.1);
            } else if let Some((idx, _)) = hit_item(hwnd, pt) {
                turn_item(hwnd, idx, steps, GetKeyState(VK_SHIFT.0 as i32) < 0);
            } else {
                let changed = with(|m| {
                    let step = (render::ROW_H * 2.0 * m.scale) as i32;
                    let w = m.win(hwnd)?;
                    let max = (w.hits.content_h - w.hits.view_h).max(0);
                    let old = w.scroll;
                    w.scroll = (w.scroll - steps * step).clamp(0, max);
                    Some(old != w.scroll)
                })
                .flatten();
                if changed == Some(true) {
                    redraw(hwnd);
                }
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let ctrl = GetKeyState(VK_CONTROL.0 as i32) < 0;
            let step = if GetKeyState(VK_SHIFT.0 as i32) < 0 { 10 } else { 1 };
            let key = VIRTUAL_KEY(wp.0 as u16);
            let nudge = match key {
                VK_LEFT => Some((-step, 0)),
                VK_RIGHT => Some((step, 0)),
                VK_UP => Some((0, -step)),
                VK_DOWN => Some((0, step)),
                _ => None,
            };
            if let Some((dx, dy)) = nudge {
                // 方向鍵微調位置（不吸附）
                if let Some((x, y)) = with(|m| m.note_of(hwnd).map(|n| (n.x, n.y))).flatten() {
                    move_paper(hwnd, x + dx, y + dy);
                    save_board();
                }
                return LRESULT(0);
            }
            if ink_on(hwnd) {
                match key {
                    VK_ESCAPE | VK_RETURN => set_ink(hwnd, false),
                    VK_Z if ctrl => ink_undo(hwnd),
                    VK_E => on_ink_button(hwnd, InkBtn::Eraser),
                    _ => {}
                }
                return LRESULT(0);
            }
            if kind_of(hwnd) == Some(Kind::Timer) {
                match key {
                    VK_SPACE => timer::toggle(hwnd),
                    VK_R => timer::reset(hwnd),
                    VK_DELETE => delete(hwnd),
                    _ => {}
                }
                return LRESULT(0);
            }
            match key {
                VK_DELETE => {
                    // 有選取附件就拿掉附件，否則刪掉整張
                    match with(|m| m.win(hwnd).and_then(|w| w.selected.take())).flatten() {
                        Some(idx) => remove_item(hwnd, idx),
                        None => delete(hwnd),
                    }
                }
                VK_F2 | VK_RETURN => edit::begin(hwnd),
                VK_V if ctrl => {
                    let rc = win::window_rect(hwnd);
                    paste_new(Some(POINT { x: rc.right + px(120.0), y: (rc.top + rc.bottom) / 2 }));
                }
                VK_C if ctrl => copy(hwnd),
                VK_N if ctrl => on_button(hwnd, Btn::New),
                VK_L if ctrl => lock(hwnd),
                VK_OEM_PLUS | VK_ADD if ctrl => zoom(hwnd, 0.1),
                VK_OEM_MINUS | VK_SUBTRACT if ctrl => zoom(hwnd, -0.1),
                _ => {}
            }
            LRESULT(0)
        }
        WM_DROPFILES => {
            let drop = HDROP(wp.0 as *mut _);
            let files = image::drop_files(drop);
            DragFinish(drop);
            let mut at = win::cursor_pos();
            for f in files.into_iter().filter(|f| image::is_image_file(f)) {
                match image::decode_file(&f) {
                    Ok(p) => new_image(p, Some(at)),
                    Err(e) => crate::base::log_error(format!("{e:#}")),
                }
                at.x += px(36.0);
                at.y += px(36.0);
            }
            LRESULT(0)
        }
        WM_TIMER if wp.0 == TIMER_ANIM => {
            tick(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wp.0 == timer::TIMER_CLOCK => {
            timer::on_clock(hwnd);
            LRESULT(0)
        }
        WM_WINDOWPOSCHANGING => {
            // 貼在桌面：永遠排在最底下
            if GetWindowLongPtrW(hwnd, GWLP_USERDATA) == 2 {
                let pos = &mut *(lp.0 as *mut WINDOWPOS);
                pos.hwndInsertAfter = HWND_BOTTOM;
                pos.flags &= !SWP_NOZORDER;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn on_mouse_move(hwnd: HWND, pt: POINT) {
    let screen = win::cursor_pos();
    let down = with(|m| m.win(hwnd).and_then(|w| w.down.clone())).flatten();
    match down {
        Some(Down::Ink) => ink_move(hwnd, screen),
        Some(Down::Erase) => ink_erase(hwnd, screen),
        Some(Down::Paper { start, grab, moved, click }) => {
            if !moved {
                if (start.x - screen.x).abs().max((start.y - screen.y).abs()) <= DRAG_THRESHOLD {
                    return;
                }
                // 拿起來：陰影變大
                with(|m| {
                    if let Some(w) = m.win(hwnd) {
                        w.lifted = true;
                        w.anim = None;
                        w.down = Some(Down::Paper { start, grab, moved: true, click });
                    }
                });
                redraw(hwnd);
            }
            let Some((sticker, pw, ph)) = with(|m| {
                let s = m.scale;
                m.note_of(hwnd).map(|n| (n.kind == Kind::Sticker, (n.w as f32 * s) as i32, (n.h as f32 * s) as i32))
            })
            .flatten() else {
                return;
            };
            let (x, y) = (screen.x - grab.x, screen.y - grab.y);
            let (x, y) = if sticker { (x, y) } else { snap(hwnd, x, y, pw, ph) };
            move_paper(hwnd, x, y);
        }
        Some(Down::Item { idx, mode, start, orig, moved, center }) => {
            if !moved {
                if (start.x - screen.x).abs().max((start.y - screen.y).abs()) <= DRAG_THRESHOLD {
                    return;
                }
                let sticker_note = with(|m| m.note_of(hwnd).is_some_and(|n| n.kind == Kind::Sticker)).unwrap_or(false);
                if sticker_note && mode == ItemMode::Move {
                    // 桌面上的貼紙：拖的是整個視窗
                    let (nx, ny) = with(|m| m.note_of(hwnd).map(|n| (n.x, n.y))).flatten().unwrap_or((0, 0));
                    with(|m| {
                        if let Some(w) = m.win(hwnd) {
                            w.down = Some(Down::Paper { start, grab: POINT { x: start.x - nx, y: start.y - ny }, moved: true, click: None });
                            w.dragging_item = Some(idx);
                        }
                    });
                    redraw(hwnd);
                    return;
                }
                with(|m| {
                    if let Some(w) = m.win(hwnd) {
                        w.down = Some(Down::Item { idx, mode, start, orig: orig.clone(), moved: true, center });
                        w.dragging_item = Some(idx);
                        w.anim = None;
                    }
                });
            }
            drag_item(hwnd, idx, mode, &orig, start, screen, center);
        }
        Some(Down::Grip { start, w0, h0 }) => {
            let changed = with(|m| {
                let s = m.scale;
                let n = m.note_of(hwnd)?;
                let nw = (w0 + render::logical(screen.x - start.x, s)).max(render::MIN_W);
                let nh = (h0 + render::logical(screen.y - start.y, s)).max(render::MIN_H);
                let changed = (nw, nh) != (n.w, n.h);
                n.w = nw;
                n.h = nh;
                Some(changed)
            })
            .flatten();
            if changed == Some(true) {
                redraw(hwnd);
            }
        }
        Some(Down::Control) => {}
        None => {
            let hot = hit_button(hwnd, pt);
            let changed = with(|m| {
                let w = m.win(hwnd)?;
                let changed = !w.hover || w.hot != hot;
                w.hover = true;
                w.hot = hot;
                Some(changed)
            })
            .flatten();
            if changed == Some(true) {
                redraw(hwnd);
            }
        }
    }
}

fn on_mouse_up(hwnd: HWND, pt: POINT) {
    let down = with(|m| {
        let w = m.win(hwnd)?;
        w.dragging_item = None;
        w.down.take()
    })
    .flatten();
    match down {
        Some(Down::Paper { moved: true, .. }) => {
            with(|m| {
                if let Some(w) = m.win(hwnd) {
                    w.lifted = false;
                }
            });
            save_board();
            // 放下：附件重新「釘」一次
            start_anim(hwnd, AnimKind::Attach(None));
        }
        Some(Down::Item { idx, moved: true, .. }) => {
            save_board();
            start_anim(hwnd, AnimKind::Attach(Some(idx)));
        }
        Some(Down::Grip { .. }) => save_board(),
        Some(Down::Ink) => ink_end(hwnd, win::cursor_pos()),
        Some(Down::Erase) => {}
        Some(Down::Paper { moved: false, click, .. }) => {
            if kind_of(hwnd) == Some(Kind::Timer) {
                timer::toggle(hwnd);
            } else if let Some(idx) = click {
                // 點一下圖釘、紙膠帶：選取（出現控制點），之後才能單獨拖它
                with(|m| {
                    if let Some(w) = m.win(hwnd) {
                        w.selected = Some(idx);
                    }
                });
                redraw(hwnd);
            } else {
                redraw(hwnd);
            }
        }
        Some(Down::Control) => {
            if let Some(btn) = hit_ink_btn(hwnd, pt) {
                on_ink_button(hwnd, btn);
            } else if let Some(btn) = hit_button(hwnd, pt) {
                on_button(hwnd, btn);
            } else if let Some(line) = hit_bullet(hwnd, pt) {
                toggle_line(hwnd, line);
            }
        }
        _ => redraw(hwnd),
    }
}

// ───────────────────────────── 右鍵選單 ─────────────────────────────

const M_EDIT: u32 = 1;
const M_DONE_ALL: u32 = 10;
const M_UNDONE_ALL: u32 = 11;
const M_CLEAR_DONE: u32 = 12;
const M_COPY: u32 = 20;
const M_SAVE_IMAGE: u32 = 22;
const M_NEW: u32 = 30;
const M_PASTE: u32 = 31;
const M_SHOP: u32 = 32;
const M_SET_PW: u32 = 50;
const M_UNLOCK: u32 = 51;
const M_LOCK: u32 = 52;
const M_REMOVE_PW: u32 = 53;
const M_DECOY: u32 = 54;
const M_CLEAR_ITEMS: u32 = 60;
const M_INK: u32 = 61;
const M_CLEAR_INK: u32 = 62;
/// 文字樣式（佔 style::SPAN 個 id）
const M_STYLE: u32 = 400;
const M_DELETE: u32 = 40;
const M_PAPER: u32 = 100;
const M_TEXTURE: u32 = 150;
const M_PIN_DESKTOP: u32 = 300;
const M_PIN_TOP: u32 = 301;

fn context_menu(hwnd: HWND) {
    let Some((note, unlocked)) = with(|m| {
        let u = m.win(hwnd).is_some_and(|w| w.unlocked.is_some());
        m.note_of(hwnd).cloned().map(|n| (n, u))
    })
    .flatten() else {
        return;
    };
    let image = note.kind == Kind::Image;
    let starred = note.kind == Kind::Starred;
    let sticker = note.kind == Kind::Sticker;
    // 最常用的放最上面：釘在桌面、永遠在最上層（再點一次取消）
    let mut items = vec![
        MenuItem::checked(M_PIN_DESKTOP, t("menu.pin_desktop"), note.layer == Layer::Desktop),
        MenuItem::checked(M_PIN_TOP, t("menu.pin_top"), note.layer == Layer::Top),
        MenuItem::Separator,
    ];
    if !sticker {
        let locked_view = note.locked() && !unlocked;
        items.push(if locked_view {
            MenuItem::item(M_UNLOCK, t("menu.unlock"))
        } else {
            MenuItem::item(M_EDIT, if image { t("menu.edit_caption") } else { t("menu.edit") })
        });
        if note.kind == Kind::Text && !note.locked() {
            items.push(MenuItem::checked(M_INK, t("menu.ink"), ink_on(hwnd)));
        }
        if !note.strokes.is_empty() {
            items.push(MenuItem::item(M_CLEAR_INK, t("menu.clear_ink")));
        }
        if !image && !locked_view {
            items.extend([
                MenuItem::Separator,
                MenuItem::item(M_DONE_ALL, t("menu.done_all")),
                MenuItem::item(M_UNDONE_ALL, t("menu.undone_all")),
                MenuItem::item(M_CLEAR_DONE, t("menu.clear_done")),
            ]);
        }
        items.push(MenuItem::Separator);
        if !image {
            let mut papers = Vec::new();
            let mut group = "";
            for (i, (p, name, g)) in PAPERS.iter().enumerate() {
                if *g != group {
                    if !group.is_empty() {
                        papers.push(MenuItem::Separator);
                    }
                    papers.push(MenuItem::disabled(t(g)));
                    group = g;
                }
                papers.push(MenuItem::checked(M_PAPER + i as u32, format!("　{}", t(name)), note.paper == *p));
            }
            items.push(MenuItem::Submenu { text: t("menu.paper"), items: papers });
            if note.paper != Paper::Clear {
                items.push(MenuItem::Submenu {
                text: t("menu.texture"),
                items: TEXTURES.iter().enumerate().map(|(i, (tx, name))| MenuItem::checked(M_TEXTURE + i as u32, t(name), note.texture == *tx)).collect(),
            });
            }
        }
        if !locked_view {
            let default = with(|m| m.board.text.clone()).unwrap_or_default();
            items.push(MenuItem::Submenu { text: t("menu.text_style"), items: style::menu(M_STYLE, Some(&note), &default) });
        }
        items.push(MenuItem::item(M_SHOP, t("menu.shop")));
        if !note.items.is_empty() {
            items.push(MenuItem::item(M_CLEAR_ITEMS, t("menu.clear_items")));
        }
        if !image && !starred {
            let pw = if !note.locked() {
                vec![MenuItem::item(M_SET_PW, t("menu.set_password"))]
            } else if unlocked {
                vec![
                    MenuItem::item(M_LOCK, t("menu.lock")),
                    MenuItem::item(M_DECOY, t("menu.decoy")),
                    MenuItem::item(M_REMOVE_PW, t("menu.remove_password")),
                ]
            } else {
                vec![MenuItem::item(M_UNLOCK, t("menu.unlock"))]
            };
            items.push(MenuItem::Submenu { text: t("menu.password"), items: pw });
        }
    }
    items.push(MenuItem::Separator);
    if image {
        items.push(MenuItem::item(M_COPY, t("menu.copy_image")));
        items.push(MenuItem::item(M_SAVE_IMAGE, t("menu.save_image")));
    } else if !sticker {
        items.push(MenuItem::item(M_COPY, t("menu.copy_text")));
    }
    if !starred {
        items.push(MenuItem::item(M_NEW, t("menu.new")));
    }
    items.push(MenuItem::item(M_PASTE, t("menu.paste")));
    items.push(MenuItem::Separator);
    items.push(MenuItem::item(M_DELETE, if starred { t("menu.hide_starred") } else if sticker { t("menu.peel_sticker") } else { t("menu.delete") }));

    let cmd = win::popup_menu(hwnd, &items, win::cursor_pos());
    if style::handle(hwnd, M_STYLE, cmd, Some(hwnd)) {
        return;
    }
    match cmd {
        M_EDIT => edit::begin(hwnd),
        M_INK => set_ink(hwnd, !ink_on(hwnd)),
        M_CLEAR_INK => {
            with(|m| {
                if let Some(n) = m.note_of(hwnd) {
                    n.strokes.clear();
                }
                store::save(&m.board);
            });
            redraw(hwnd);
        }
        M_DONE_ALL => set_all_done(hwnd, true),
        M_UNDONE_ALL => set_all_done(hwnd, false),
        M_CLEAR_DONE => {
            update_lines(hwnd, |lines| lines.retain(|l| !l.done));
            redraw(hwnd);
        }
        M_COPY => copy(hwnd),
        M_SAVE_IMAGE => save_image_as(hwnd),
        M_NEW => on_button(hwnd, Btn::New),
        M_PASTE => {
            let rc = win::window_rect(hwnd);
            paste_new(Some(POINT { x: rc.right + px(120.0), y: (rc.top + rc.bottom) / 2 }));
        }
        M_SHOP => shop::open(),
        M_SET_PW => edit::prompt(hwnd, edit::PromptMode::SetNew),
        M_UNLOCK => edit::prompt(hwnd, edit::PromptMode::Unlock),
        M_LOCK => lock(hwnd),
        M_REMOVE_PW => remove_password(hwnd),
        M_DECOY => change_decoy(hwnd),
        M_CLEAR_ITEMS => {
            with(|m| {
                if let Some(n) = m.note_of(hwnd) {
                    n.items.clear();
                }
                store::save(&m.board);
            });
            redraw(hwnd);
        }
        M_DELETE => delete(hwnd),
        id if (M_PAPER..M_PAPER + PAPERS.len() as u32).contains(&id) => {
            let paper = PAPERS[(id - M_PAPER) as usize].0;
            with(|m| {
                if let Some(n) = m.note_of(hwnd) {
                    n.paper = paper;
                }
                // 透明紙不當成新便利貼的預設
                if !starred && paper != Paper::Clear {
                    m.board.last_paper = paper;
                }
                store::save(&m.board);
            });
            redraw(hwnd);
        }
        id if (M_TEXTURE..M_TEXTURE + TEXTURES.len() as u32).contains(&id) => {
            let texture = TEXTURES[(id - M_TEXTURE) as usize].0;
            with(|m| {
                if let Some(n) = m.note_of(hwnd) {
                    n.texture = texture;
                }
                if !starred {
                    m.board.last_texture = texture;
                }
                store::save(&m.board);
            });
            redraw(hwnd);
        }
        M_PIN_DESKTOP => set_layer(hwnd, if note.layer == Layer::Desktop { Layer::Normal } else { Layer::Desktop }),
        M_PIN_TOP => set_layer(hwnd, if note.layer == Layer::Top { Layer::Normal } else { Layer::Top }),
        _ => {}
    }
}

const I_LEFT: u32 = 1;
const I_RIGHT: u32 = 2;
const I_BIGGER: u32 = 3;
const I_SMALLER: u32 = 4;
const I_TOP: u32 = 5;
const I_REMOVE: u32 = 6;
const I_DEFAULT: u32 = 7;

/// 附件的右鍵選單。
fn item_menu(hwnd: HWND, idx: usize) {
    let Some((asset, sticker_note)) = with(|m| m.note_of(hwnd).and_then(|n| Some((n.items.get(idx)?.asset.clone(), n.kind == Kind::Sticker)))).flatten() else {
        return;
    };
    let cat = assets::cat_of(&asset);
    let name = assets::name_of(&asset);
    let mut items = vec![MenuItem::disabled(name)];
    if cat != Cat::Tape {
        items.extend([
            MenuItem::item(I_LEFT, t("item.rotate_left")),
            MenuItem::item(I_RIGHT, t("item.rotate_right")),
            MenuItem::item(I_BIGGER, t("item.bigger")),
            MenuItem::item(I_SMALLER, t("item.smaller")),
        ]);
    }
    if !sticker_note {
        items.push(MenuItem::item(I_TOP, t("item.to_top")));
        if cat != Cat::Sticker {
            items.push(MenuItem::item(I_DEFAULT, t("item.default")));
        }
    }
    items.extend([MenuItem::Separator, MenuItem::item(I_REMOVE, t("item.remove"))]);
    match win::popup_menu(hwnd, &items, win::cursor_pos()) {
        I_LEFT => turn_item(hwnd, idx, -3, false),
        I_RIGHT => turn_item(hwnd, idx, 3, false),
        I_BIGGER => turn_item(hwnd, idx, 2, true),
        I_SMALLER => turn_item(hwnd, idx, -2, true),
        I_TOP => {
            with(|m| {
                let n = m.note_of(hwnd)?;
                if idx < n.items.len() {
                    let a = n.items.remove(idx);
                    n.items.push(a);
                }
                store::save(&m.board);
                Some(())
            });
            redraw(hwnd);
        }
        I_DEFAULT => {
            with(|m| {
                m.board.last_pin = asset.clone();
                store::save(&m.board);
            });
        }
        I_REMOVE => remove_item(hwnd, idx),
        _ => {}
    }
}

/// 拿掉一個附件（桌面上的貼紙就是撕掉整張）。
fn remove_item(hwnd: HWND, idx: usize) {
    if with(|m| m.note_of(hwnd).is_some_and(|n| n.kind == Kind::Sticker)).unwrap_or(false) {
        delete(hwnd);
        return;
    }
    with(|m| {
        if let Some(w) = m.win(hwnd) {
            w.selected = None;
        }
        let n = m.note_of(hwnd)?;
        if idx < n.items.len() {
            n.items.remove(idx);
        }
        store::save(&m.board);
        Some(())
    });
    redraw(hwnd);
}

fn set_all_done(hwnd: HWND, done: bool) {
    update_lines(hwnd, |lines| {
        for l in lines {
            l.done = done;
        }
    });
    redraw(hwnd);
}

fn set_layer(hwnd: HWND, layer: Layer) {
    if layer == Layer::Desktop {
        // 釘上去的感覺：附件重新釘一次
        start_anim(hwnd, AnimKind::Attach(None));
    }
    with(|m| {
        if let Some(n) = m.note_of(hwnd) {
            n.layer = layer;
        }
        store::save(&m.board);
    });
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, layer_code(layer));
        let after = match layer {
            Layer::Top => HWND_TOPMOST,
            Layer::Normal => HWND_NOTOPMOST,
            Layer::Desktop => HWND_BOTTOM,
        };
        if layer == Layer::Desktop {
            let _ = SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        let _ = SetWindowPos(hwnd, Some(after), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

fn copy(hwnd: HWND) {
    let Some(kind) = with(|m| m.note_of(hwnd).map(|n| n.kind)).flatten() else { return };
    if kind == Kind::Image {
        let pixels = with(|m| m.win(hwnd).and_then(|w| w.image.as_ref().map(|c| c.original.clone()))).flatten();
        if let Some(p) = pixels {
            if image::set_clipboard_image(hwnd, &p) {
                crate::toast::show(t("toast.copied_image"), false, 1);
            }
        }
        return;
    }
    let text = Note::text_of(&current_lines(hwnd).unwrap_or_default());
    if arboard::Clipboard::new().and_then(|mut c| c.set_text(text)).is_ok() {
        crate::toast::show(t("toast.copied"), false, 1);
    }
}

/// 開檔對話框。`filter` 是「說明\0樣式\0」；`multi` = 可以選好幾個。
fn open_dialog(owner: HWND, filter: &str, multi: bool) -> Vec<std::path::PathBuf> {
    let mut buf = vec![0u16; if multi { 32768 } else { 1024 }];
    let filter: Vec<u16> = filter.encode_utf16().chain([0]).collect();
    let mut flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER;
    if multi {
        flags |= OFN_ALLOWMULTISELECT;
    }
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(buf.as_mut_ptr()),
        nMaxFile: buf.len() as u32,
        Flags: flags,
        ..Default::default()
    };
    if !unsafe { GetOpenFileNameW(&mut ofn) }.as_bool() {
        return vec![];
    }
    // 多選：資料夾\0檔名1\0檔名2\0\0；單選：完整路徑\0
    let parts: Vec<String> = buf.split(|&c| c == 0).take_while(|p| !p.is_empty()).map(String::from_utf16_lossy).collect();
    match parts.as_slice() {
        [] => vec![],
        [one] => vec![one.into()],
        [dir, files @ ..] => files.iter().map(|f| std::path::Path::new(dir).join(f)).collect(),
    }
}

/// 存檔對話框，回傳選好的路徑。
fn save_dialog(owner: HWND, default_name: &str, filter: &str, ext: &str) -> Option<String> {
    let mut buf = [0u16; 520];
    for (i, c) in default_name.encode_utf16().enumerate().take(500) {
        buf[i] = c;
    }
    let filter: Vec<u16> = filter.encode_utf16().chain([0, 0]).collect();
    let ext = wide(ext);
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(buf.as_mut_ptr()),
        nMaxFile: buf.len() as u32,
        lpstrDefExt: PCWSTR(ext.as_ptr()),
        Flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST,
        ..Default::default()
    };
    unsafe {
        if GetSaveFileNameW(&mut ofn).as_bool() {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..len]))
        } else {
            None
        }
    }
}

fn save_image_as(hwnd: HWND) {
    let Some(file) = with(|m| m.note_of(hwnd).map(|n| n.image.clone())).flatten() else { return };
    if let Some(dest) = save_dialog(hwnd, &t("file.image_name"), &format!("{}\0*.png", t("file.image_filter")), "png") {
        if let Err(e) = std::fs::copy(store::image_dir().join(&file), &dest) {
            crate::base::log_error(format!("另存圖片失敗：{e}"));
        }
    }
}

// ───────────────────────────── 開發用 ─────────────────────────────

/// `--demo-notes`：放幾張範例便利貼（各種顏色、紙質、附件、劃線、星號記事本），檢查畫面用。
pub fn demo() {
    let wa = render::work_area_at(POINT { x: 0, y: 0 });
    for (i, mut note) in sample_notes().into_iter().enumerate() {
        note.x = wa.left + px(60.0) + (i as i32 % 4) * px(290.0);
        note.y = wa.top + px(90.0) + (i as i32 / 4) * px(300.0);
        add_and_show(note, false);
    }
    star("Rust 的所有權讓記憶體安全不需要垃圾回收", "vivaldi.exe");
    star("便利貼要低耗能：只在改變時重畫", "Telegram.exe");
    if !starred_visible() {
        toggle_starred();
    }
}

fn sample_notes() -> Vec<Note> {
    let text = "買牛奶\n回覆 Amy 的信\n週五前交報告，記得附上上個月的數字和圖表";
    let specs: [(Paper, Texture, Vec<Attach>); 8] = [
        (Paper::Yellow, Texture::Sticky, vec![Attach::at("pin-red", 120.0, 2.0)]),
        (Paper::MorandiRose, Texture::Washi, vec![Attach::tape("tape-floral", 60.0, 2.0, 180.0, -4.0), Attach { asset: "st-cat".into(), x: 205.0, y: 165.0, angle: -8.0, ..Default::default() }]),
        (Paper::Sage, Texture::Grid, vec![Attach::tape("tape-gingham", -12.0, 28.0, 34.0, -14.0), Attach::tape("tape-gingham", 206.0, -14.0, 252.0, 28.0)]),
        (Paper::Oat, Texture::Lined, vec![Attach { asset: "clip-binder".into(), x: 46.0, y: -12.0, ..Default::default() }]),
        (Paper::MorandiBlue, Texture::Dots, vec![Attach::at("pin-pearl", 30.0, 10.0), Attach::at("pin-pearl", 210.0, 10.0)]),
        (Paper::Kraft, Texture::Fiber, vec![Attach::tape("tape-stars", 50.0, -2.0, 190.0, 6.0), Attach { asset: "st-sakura".into(), x: 200.0, y: 160.0, angle: 12.0, scale: 0.8, ..Default::default() }]),
        (Paper::MilkTea, Texture::Sticky, vec![Attach { asset: "clip-heart".into(), x: 200.0, y: 4.0, angle: 6.0, ..Default::default() }]),
        (Paper::MorandiLilac, Texture::Plain, vec![Attach::at("pin-heart", 120.0, 2.0), Attach { asset: "st-boba".into(), x: 30.0, y: 170.0, angle: -10.0, scale: 0.7, ..Default::default() }]),
    ];
    specs
        .into_iter()
        .enumerate()
        .map(|(i, (paper, texture, items))| {
            let mut n = Note { id: store::new_id(), paper, texture, items, w: 240, h: 200, v: store::VERSION, created: store::now_secs(), ..Default::default() };
            n.lines = text.lines().map(Line::new).collect();
            n.lines[0].done = i % 2 == 0;
            n
        })
        .collect()
}

/// `--render-notes out.png`：不開視窗，把各種樣式畫在一張圖上（離屏，開發時看畫面用）。
pub fn render_preview(path: &str) {
    let Some(scale) = with(|m| m.scale) else { return };
    let mut tiles: Vec<(Note, &str)> = sample_notes().into_iter().zip(["", "hover", "", "strike", "anim", "", "lift", "scramble"]).collect();
    let mut starred = Note { kind: Kind::Starred, paper: Paper::White, texture: Texture::Lined, w: 280, h: 230, ..Default::default() };
    starred.items = vec![Attach { asset: "star".into(), x: 266.0, y: 4.0, angle: 10.0, ..Default::default() }];
    starred.lines = vec![
        Line { text: "Rust 的所有權讓記憶體安全不需要垃圾回收".into(), done: false, meta: "10/12 14:03 · vivaldi".into() },
        Line { text: "便利貼要低耗能：只在改變時重畫".into(), done: true, meta: "10/12 13:40 · Telegram".into() },
    ];
    tiles.push((starred, ""));
    let mut photo = Note { kind: Kind::Image, paper: Paper::White, w: 220, h: 238, ..Default::default() };
    photo.items = vec![Attach::tape("tape-dusty", 55.0, 2.0, 165.0, -4.0)];
    photo.lines = vec![Line::new("Star sticker")];
    tiles.push((photo, ""));
    let mut arabic = Note { paper: Paper::MorandiYellow, texture: Texture::Lined, w: 240, h: 170, ..Default::default() };
    arabic.items = vec![Attach::at("pin-gold", 120.0, 2.0)];
    arabic.lines = vec![Line { text: "شراء الحليب".into(), done: true, meta: String::new() }, Line::new("الرد على البريد"), Line::new("Mixed: موعد 3:00 meeting")];
    tiles.push((arabic, ""));
    let sticker = Note { kind: Kind::Sticker, items: vec![Attach { asset: "st-shiba".into(), angle: -6.0, ..Default::default() }], ..Default::default() };
    tiles.push((sticker, ""));
    // 放大 2.4 倍的貼紙：要從原圖縮，不能糊
    let big = Note { kind: Kind::Sticker, items: vec![Attach { asset: "cn-zhaocai".into(), scale: 2.4, ..Default::default() }], ..Default::default() };
    tiles.push((big, ""));

    // 蕃茄鐘：跑到一半的蕃茄、沙漏、休息中的蕃茄
    let now = store::now_ms();
    let mut tomato = store::Timer { end_ms: now + 754_400, done: 3, ..Default::default() };
    let (w, h) = timer::size_of(tomato.style);
    tiles.push((Note { kind: Kind::Timer, w, h, timer: Some(tomato.clone()), ..Default::default() }, "hover"));
    let glass = store::Timer { style: store::TimerStyle::Hourglass, end_ms: now + 600_000, done: 2, ..Default::default() };
    let (w, h) = timer::size_of(glass.style);
    tiles.push((Note { kind: Kind::Timer, w, h, timer: Some(glass), ..Default::default() }, ""));
    tomato.phase = store::Phase::Break;
    tomato.end_ms = 0;
    tomato.left_ms = 5 * 60_000;
    tomato.finished = true;
    let (w, h) = timer::size_of(tomato.style);
    tiles.push((Note { kind: Kind::Timer, w, h, timer: Some(tomato), ..Default::default() }, ""));
    for (style, running) in [(store::TimerStyle::Led, true), (store::TimerStyle::Alarm, true), (store::TimerStyle::Cat, true), (store::TimerStyle::Dog, false)] {
        let t = store::Timer { style, end_ms: if running { now + 1_234_000 } else { 0 }, done: 2, ..Default::default() };
        let (w, h) = timer::size_of(style);
        tiles.push((Note { kind: Kind::Timer, w, h, timer: Some(t), ..Default::default() }, ""));
    }
    // 文字樣式：紅色粗體大字標語、手寫風字型
    let mut banner = Note { paper: Paper::Clear, w: 320, h: 150, font: 3.0, color: Some(0xE21E24), bold: Some(true), face: Some("Impact".into()), ..Default::default() };
    banner.lines = vec![Line::new("FOCUS!")];
    tiles.push((banner, ""));
    let mut fancy = Note { paper: Paper::MorandiBlue, texture: Texture::Lined, w: 240, h: 180, font: 1.4, color: Some(0x2F6FDB), face: Some("Segoe Print".into()), ..Default::default() };
    fancy.items = vec![Attach::at("pin-teal", 120.0, 2.0)];
    fancy.lines = vec![Line::new("Call mom"), Line { text: "Gym 7pm".into(), done: true, meta: String::new() }];
    tiles.push((fancy, ""));

    // 手寫：透明紙（深色桌布上）、手寫中的工具列
    let wave = |tool: store::Tool, color: u32, size: f32, x0: f32, y0: f32, len: f32, amp: f32| Stroke {
        tool,
        color,
        size,
        pts: (0..=40)
            .flat_map(|k| {
                let u = k as f32 / 40.0;
                [((x0 + u * len) * 4.0) as i32, ((y0 + (u * 9.0).sin() * amp) * 4.0) as i32]
            })
            .collect(),
    };
    let heart = |tool: store::Tool, color: u32, size: f32, cx: f32, cy: f32, k: f32| Stroke {
        tool,
        color,
        size,
        pts: (0..=60)
            .flat_map(|i| {
                let t = i as f32 / 60.0 * std::f32::consts::TAU;
                let x = 16.0 * t.sin().powi(3);
                let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
                [((cx + x * k) * 4.0) as i32, ((cy + y * k) * 4.0) as i32]
            })
            .collect(),
    };
    use store::Tool::*;
    let mut slogan = Note { paper: Paper::Clear, w: 300, h: 150, ..Default::default() };
    slogan.lines = vec![Line::new("今天也要加油")];
    slogan.strokes = vec![
        wave(Chalk, 0xFAFAF7, 9.0, 20.0, 60.0, 180.0, 10.0),
        wave(Crayon, 0xF7CF3B, 8.0, 20.0, 95.0, 180.0, 8.0),
        heart(Chalk, 0xEE78A8, 7.0, 250.0, 80.0, 1.6),
        wave(Marker, 0x3DAA5C, 13.0, 30.0, 128.0, 240.0, 2.0),
    ];
    tiles.push((slogan, "dark"));
    let mut inked = Note { paper: Paper::Oat, texture: Texture::Dots, w: 240, h: 170, ..Default::default() };
    inked.items = vec![Attach::tape("tape-kraft", 60.0, 0.0, 180.0, -3.0)];
    inked.lines = vec![Line::new("週末計畫")];
    inked.strokes = vec![
        wave(Pen, 0x26221E, 2.6, 30.0, 70.0, 170.0, 6.0),
        wave(Crayon, 0xE21E24, 7.0, 30.0, 105.0, 150.0, 9.0),
        heart(Pen, 0xE21E24, 2.6, 200.0, 125.0, 1.1),
        wave(Marker, 0xF7CF3B, 13.0, 30.0, 140.0, 120.0, 1.0),
    ];
    tiles.push((inked, "ink"));
    // 自己做的貼紙：把內建的柴犬墊上淡灰底（像截圖），再去背加白邊
    if let Ok(src) = assets::picture_2x("st-cat") {
        let mut shot = image::Pixels { w: src.w + 80, h: src.h + 60, data: vec![0; ((src.w + 80) * (src.h + 60) * 4) as usize] };
        for y in 0..shot.h {
            for x in 0..shot.w {
                let o = ((y * shot.w + x) * 4) as usize;
                let mut c = [236.0f32, 240.0, 241.0];
                if x >= 40 && y >= 30 && x < src.w + 40 && y < src.h + 30 {
                    let i = (((y - 30) * src.w + x - 40) * 4) as usize;
                    let a = src.data[i + 3] as f32 / 255.0;
                    for k in 0..3 {
                        c[k] = src.data[i + k] as f32 * a + c[k] * (1.0 - a);
                    }
                }
                shot.data[o..o + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        }
        let made = stickers::make_sticker(&shot);
        with(|m| m.assets.insert("my-preview", made));
        let mine = Note { kind: Kind::Sticker, items: vec![Attach { asset: "my-preview".into(), angle: 5.0, ..Default::default() }], ..Default::default() };
        tiles.push((mine, ""));
        let mut pinned = Note { paper: Paper::MorandiBlue, texture: Texture::Grid, w: 220, h: 170, ..Default::default() };
        pinned.items = vec![Attach::at("pin-red", 110.0, 2.0), Attach { asset: "my-preview".into(), x: 150.0, y: 110.0, angle: -8.0, scale: 0.8, ..Default::default() }];
        pinned.lines = vec![Line::new("自製貼紙")];
        tiles.push((pinned, "select-pin"));
    }

    let cols = 4;
    let (cell_w, cell_h) = ((340.0 * scale) as i32, (330.0 * scale) as i32);
    let rows = tiles.len().div_ceil(cols) as i32;
    let (out_w, out_h) = (cell_w * cols as i32, cell_h * rows);
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];
    for p in out.chunks_mut(4) {
        p.copy_from_slice(&[233, 228, 220, 255]);
    }
    let photo_px = image::decode_bytes(include_bytes!("../../assets/deco-src/star.png")).ok();
    let decoy = store::decoy(1);
    for (i, (note, state)) in tiles.iter().enumerate() {
        let mut buf = Vec::new();
        let mut cache = photo_px.clone().map(|p| ImageCache { original: p, scaled: None });
        let mut lay = Layout::default();
        with(|m| {
            let key = m.fonts_for(note);
            let text_color = m.text_color(note);
            let Manager { fonts, assets, board, .. } = m;
            let ink_view = (*state == "ink").then(|| InkView { prefs: store::InkPrefs { tool: store::Tool::Crayon, color: 0xE21E24, size: 1 }, eraser: false, cur: None });
            let _ = board;
            lay = render::layout_ex(note, scale, assets, ink_view.is_some());
            let scene = Scene {
                note,
                lines: &note.lines,
                scale,
                hover: matches!(*state, "hover" | "lift" | "dark"),
                hot: (*state == "hover").then_some(Btn::Menu),
                lift: if *state == "lift" { 1.0 } else { 0.0 },
                attach_anim: (*state == "anim").then_some(AttachAnim { which: None, t: 0.3 }),
                dragging_item: None,
                selected: match *state {
                    "hover" => Some(1),
                    "select-pin" => Some(0),
                    _ => None,
                },
                strike: (*state == "strike").then_some((1, 0.6)),
                scramble: (*state == "scramble").then(|| Scramble { from: &decoy, to: &note.lines, t: 0.6 }),
                unlocked: false,
                scroll: 0,
                editing: false,
                alpha: 255,
                image: if note.kind == Kind::Image { cache.as_mut() } else { None },
                assets,
                fonts: &fonts[&key],
                text_color,
                now_ms: store::now_ms(),
                fx: Fx::None,
                ink: ink_view,
            };
            let started = std::time::Instant::now();
            render::present(render::Target::Buffer(&mut buf), scene);
            crate::base::log_info(format!("render-notes: tile {i} {state} {:.2} ms", started.elapsed().as_secs_f64() * 1000.0));
        });
        let (w, h) = (lay.w, lay.h);
        let (ox, oy) = ((i % cols) as i32 * cell_w + (cell_w - w) / 2, (i / cols) as i32 * cell_h + (cell_h - h) / 2);
        if *state == "dark" {
            // 深色桌布：看透明紙上的粉筆
            for y in (i / cols) as i32 * cell_h..((i / cols) as i32 + 1) * cell_h {
                for x in (i % cols) as i32 * cell_w..((i % cols) as i32 + 1) * cell_w {
                    let d = ((y * out_w + x) * 4) as usize;
                    let g = (x + y) as f32 / (cell_w + cell_h) as f32;
                    out[d..d + 3].copy_from_slice(&[(38.0 + 20.0 * g) as u8, (52.0 + 18.0 * g) as u8, (70.0 + 16.0 * g) as u8]);
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = (ox + x, oy + y);
                if dx < 0 || dy < 0 || dx >= out_w || dy >= out_h {
                    continue;
                }
                let s = ((y * w + x) * 4) as usize;
                let d = ((dy * out_w + dx) * 4) as usize;
                let a = buf[s + 3] as f32 / 255.0;
                // 預乘 BGRA 疊到背景，輸出 RGBA
                out[d] = (buf[s + 2] as f32 + out[d] as f32 * (1.0 - a)) as u8;
                out[d + 1] = (buf[s + 1] as f32 + out[d + 1] as f32 * (1.0 - a)) as u8;
                out[d + 2] = (buf[s] as f32 + out[d + 2] as f32 * (1.0 - a)) as u8;
            }
        }
    }
    let png = crate::base::png_rect(out_w as u32, out_h as u32, &out);
    match std::fs::write(path, png) {
        Ok(()) => crate::base::log_info(format!("render-notes：{path}")),
        Err(e) => crate::base::log_error(format!("render-notes：{e}")),
    }
}

/// `--make-art 原圖 輸出.png`：蕃茄鐘素材（去背、細白邊、挖掉包住的白色空隙），印出錶面的位置（以 2 倍圖的像素計）。
pub fn make_art(src: &str, dst: &str) {
    let recipe = stickers::Recipe { out_max: 400, border: 2.6, round: 5.2, holes: true, upscale: 1.6, shadows: false, smooth: false };
    let result = image::decode_file(std::path::Path::new(src)).and_then(|p| {
        let art = stickers::make(&p, &recipe);
        image::save_png(&art, std::path::Path::new(dst))?;
        Ok(art)
    });
    let msg = match result {
        Ok(art) => format!("make-art: {}x{} face={:?}", art.w, art.h, stickers::find_face(&art)),
        Err(e) => format!("make-art: {e:#}"),
    };
    println!("{msg}");
    crate::base::log_info(msg);
}

/// `--make-sticker 原圖 輸出.png`：用和「貼上做貼紙」一樣的做法做一張，開發用（看去背的效果）。
pub fn make_user_sticker(src: &str, dst: &str) {
    let raw = std::env::var("PINPIN_NO_SMOOTH").is_ok();
    let result = image::decode_file(std::path::Path::new(src)).and_then(|p| {
        let t0 = std::time::Instant::now();
        let s = if raw { stickers::make(&p, &stickers::Recipe { smooth: false, ..stickers::USER }) } else { stickers::make_sticker(&p) };
        println!("{} ms", t0.elapsed().as_millis());
        image::save_png(&s, std::path::Path::new(dst))?;
        Ok(s)
    });
    match result {
        Ok(s) => println!("make-sticker: {}x{}", s.w, s.h),
        Err(e) => println!("make-sticker: {e:#}"),
    }
}

/// `--split-grid 素材表.png 資料夾 前綴 名稱,名稱,… 欄 列 最大寬 最大高`：把生圖的貼紙表（透明背景、大致排成格子）切成一張張。
/// 先在整張圖找連通區域，依中心點歸到格子裡，區域本身完整保留（生圖不會剛好對齊格線，用格線切會切掉邊）；
/// 只有和隔壁黏在一起、跨過好幾格的區域才用格線切開並提出警告。名稱「_」開頭的格子跳過；前綴「_」表示名稱就是檔名。
pub fn split_grid(args: &[String]) {
    struct Comp {
        area: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        sx: f64,
        sy: f64,
    }
    let run = || -> anyhow::Result<String> {
        let [src, dir, prefix, names, cols, rows, max_w, max_h] = args else { anyhow::bail!("參數不對") };
        let sheet = image::decode_file(std::path::Path::new(src))?;
        let names: Vec<&str> = names.split(',').collect();
        let (cols, rows): (usize, usize) = (cols.parse()?, rows.parse()?);
        let (max_w, max_h): (f32, f32) = (max_w.parse()?, max_h.parse()?);
        let (w, h) = (sheet.w as usize, sheet.h as usize);
        let (cell_w, cell_h) = (w as f32 / cols as f32, h as f32 / rows as f32);
        let solid = |i: usize| sheet.data[i * 4 + 3] >= 128;
        // 整張圖的連通區域
        let mut label = vec![0u32; w * h];
        let mut comps = vec![Comp { area: 0, x0: 0, y0: 0, x1: 0, y1: 0, sx: 0.0, sy: 0.0 }];
        for start in 0..w * h {
            if label[start] != 0 || !solid(start) {
                continue;
            }
            let id = comps.len() as u32;
            let mut c = Comp { area: 0, x0: w, y0: h, x1: 0, y1: 0, sx: 0.0, sy: 0.0 };
            let mut stack = vec![start];
            label[start] = id;
            while let Some(i) = stack.pop() {
                let (x, y) = (i % w, i / w);
                c.area += 1;
                (c.x0, c.y0, c.x1, c.y1) = (c.x0.min(x), c.y0.min(y), c.x1.max(x), c.y1.max(y));
                (c.sx, c.sy) = (c.sx + x as f64, c.sy + y as f64);
                for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                    if ok && label[j] == 0 && solid(j) {
                        label[j] = id;
                        stack.push(j);
                    }
                }
            }
            comps.push(c);
        }
        let cell_of = |c: &Comp| {
            let (cx, cy) = (c.sx / c.area as f64, c.sy / c.area as f64);
            ((cy / cell_h as f64) as usize).min(rows - 1) * cols + ((cx / cell_w as f64) as usize).min(cols - 1)
        };
        let mut report = String::new();
        for (n, name) in names.iter().enumerate().take(cols * rows) {
            if name.starts_with('_') {
                continue;
            }
            let file = if prefix == "_" { name.to_string() } else { format!("{prefix}-{name}") };
            // 跨過好幾格的區域（和隔壁黏住）：碰到這格就算進來，等一下用格線切
            let (gx0, gy0) = ((n % cols) as f32 * cell_w, (n / cols) as f32 * cell_h);
            let merged = |c: &Comp| (c.x1 - c.x0) as f32 > cell_w * 1.25 || (c.y1 - c.y0) as f32 > cell_h * 1.25;
            let overlaps = |c: &Comp| (c.x0 as f32) < gx0 + cell_w && (c.x1 as f32) > gx0 && (c.y0 as f32) < gy0 + cell_h && (c.y1 as f32) > gy0;
            let mine: Vec<usize> = (1..comps.len()).filter(|&i| cell_of(&comps[i]) == n || (merged(&comps[i]) && overlaps(&comps[i]))).collect();
            let biggest = mine.iter().map(|&i| comps[i].area).max().unwrap_or(0);
            let keep: Vec<usize> = mine.iter().copied().filter(|&i| comps[i].area * 25 >= biggest && biggest > 0).collect();
            if keep.is_empty() {
                report += &format!("{file}: 空的\n");
                continue;
            }
            let mut warn = String::new();
            // 範圍：留下的區域合起來；有黏住的就限制在格子裡
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
            for &i in &keep {
                let c = &comps[i];
                (x0, y0, x1, y1) = (x0.min(c.x0), y0.min(c.y0), x1.max(c.x1), y1.max(c.y1));
            }
            if keep.iter().any(|&i| merged(&comps[i])) {
                warn += "  ⚠ 和隔壁黏在一起，用格線切";
                x0 = x0.max(gx0 as usize);
                y0 = y0.max(gy0 as usize);
                x1 = x1.min((gx0 + cell_w) as usize - 1);
                y1 = y1.min((gy0 + cell_h) as usize - 1);
            }
            if x0 == 0 || y0 == 0 || x1 + 1 >= w || y1 + 1 >= h {
                warn += "  ⚠ 碰到圖的邊緣，生圖時可能就被切到了";
            }
            // 邊緣往外 3px 的半透明像素一起帶走
            let (x0, y0) = (x0.saturating_sub(3), y0.saturating_sub(3));
            let (x1, y1) = ((x1 + 3).min(w - 1), (y1 + 3).min(h - 1));
            let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
            let kept = |x: usize, y: usize| {
                let l = label[y * w + x];
                l != 0 && keep.contains(&(l as usize))
            };
            let mut crop = image::Pixels { w: bw as u32, h: bh as u32, data: vec![0; bw * bh * 4] };
            for y in 0..bh {
                for x in 0..bw {
                    let (sx, sy) = (x0 + x, y0 + y);
                    let near = (sy.saturating_sub(3)..(sy + 4).min(h)).any(|yy| (sx.saturating_sub(3)..(sx + 4).min(w)).any(|xx| kept(xx, yy)));
                    if !near {
                        continue;
                    }
                    let s = (sy * w + sx) * 4;
                    let a = sheet.data[s + 3] as u32;
                    // 先預乘，縮小時透明的地方不會把邊緣染黑
                    let d = (y * bw + x) * 4;
                    for c in 0..3 {
                        crop.data[d + c] = ((sheet.data[s + c] as u32 * a + 127) / 255) as u8;
                    }
                    crop.data[d + 3] = a as u8;
                }
            }
            let k = (max_w / bw as f32).min(max_h / bh as f32).min(1.0);
            let (ow, oh) = (((bw as f32 * k).round() as u32).max(1), ((bh as f32 * k).round() as u32).max(1));
            let mut small = crop.resized(ow, oh);
            for px in small.data.chunks_mut(4) {
                let a = px[3] as u32;
                if a > 0 {
                    for c in px.iter_mut().take(3) {
                        *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                    }
                }
            }
            image::save_png(&small, &std::path::Path::new(dir).join(format!("{file}.png")))?;
            report += &format!("{file} {ow}x{oh}（{} 塊）{warn}\n", keep.len());
        }
        Ok(report)
    };
    match run() {
        Ok(r) => print!("{r}"),
        Err(e) => println!("split-grid: {e:#}"),
    }
}

/// `--unborder 資料夾 前綴,前綴`：拿掉貼紙外圈的白邊（春聯不要白邊）。
/// 從透明的地方往內灌，經過的白色、淺灰都變透明；碰到紅紙停下，交界處依「多白」給半透明，邊緣不會有白毛。
pub fn unborder(args: &[String]) {
    let [dir, prefixes] = args else {
        println!("unborder: 參數不對");
        return;
    };
    let prefixes: Vec<&str> = prefixes.split(',').collect();
    let mut count = 0;
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if !name.ends_with(".png") || !prefixes.iter().any(|p| name.starts_with(p)) {
            continue;
        }
        let Ok(mut p) = image::decode_file(&path) else { continue };
        let (w, h) = (p.w as usize, p.h as usize);
        let px = |d: &[u8], i: usize| (d[i * 4], d[i * 4 + 1], d[i * 4 + 2], d[i * 4 + 3]);
        let whitish = |(b, g, r, a): (u8, u8, u8, u8)| a > 0 && b.min(g).min(r) > 170 && b.max(g).max(r) - b.min(g).min(r) < 45;
        let mut gone = vec![false; w * h];
        let mut stack: Vec<usize> = (0..w * h).filter(|&i| p.data[i * 4 + 3] < 30).collect();
        for &i in &stack {
            gone[i] = true;
        }
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            for (ok, j) in [(x > 0, i.wrapping_sub(1)), (x + 1 < w, i + 1), (y > 0, i.wrapping_sub(w)), (y + 1 < h, i + w)] {
                if ok && !gone[j] && whitish(px(&p.data, j)) {
                    gone[j] = true;
                    stack.push(j);
                }
            }
        }
        for i in 0..w * h {
            if gone[i] {
                p.data[i * 4 + 3] = 0;
            }
        }
        // 交界：緊鄰被拿掉的地方，越白越透明
        let edge: Vec<usize> = (0..w * h)
            .filter(|&i| !gone[i] && p.data[i * 4 + 3] > 0)
            .filter(|&i| {
                let (x, y) = (i % w, i / w);
                (x > 0 && gone[i - 1]) || (x + 1 < w && gone[i + 1]) || (y > 0 && gone[i - w]) || (y + 1 < h && gone[i + w])
            })
            .collect();
        for i in edge {
            let (b, g, _, a) = px(&p.data, i);
            let white = ((g.min(b) as f32 - 60.0) / 160.0).clamp(0.0, 1.0);
            p.data[i * 4 + 3] = (a as f32 * (1.0 - white)) as u8;
        }
        if image::save_png(&p, &path).is_ok() {
            count += 1;
        }
    }
    println!("unborder: {count} 張");
}

/// `--pack-jxr 來源資料夾 目的資料夾 品質`：把切好的 PNG 貼紙轉成 JPEG XR（內建貼紙用，檔案小又保留透明）。
pub fn pack_jxr(args: &[String]) {
    let [src, dst, quality] = args else {
        println!("pack-jxr: 參數不對");
        return;
    };
    let q: f32 = quality.parse().unwrap_or(0.75);
    let (mut count, mut bytes) = (0, 0u64);
    for entry in std::fs::read_dir(src).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "png") {
            continue;
        }
        let out = std::path::Path::new(dst).join(path.with_extension("jxr").file_name().unwrap_or_default());
        match image::decode_file(&path).and_then(|p| image::save_jxr(&p, &out, q)) {
            Ok(()) => {
                count += 1;
                bytes += std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
            }
            Err(e) => println!("{}: {e:#}", path.display()),
        }
    }
    println!("pack-jxr: {count} 張，共 {} KB", bytes / 1024);
}

/// 換語言之後全部重畫一次（星號記事本的標題等）。
pub fn refresh_all() {
    let hwnds = with(|m| m.wins.iter().map(|w| w.hwnd).collect::<Vec<_>>()).unwrap_or_default();
    for h in hwnds {
        redraw(h);
    }
}

/// 系統匣的「文字預設樣式」子選單（id 從 base 開始）。
pub fn default_style_menu(base: u32) -> Vec<MenuItem> {
    let default = with(|m| m.board.text.clone()).unwrap_or_default();
    style::menu(base, None, &default)
}

/// 處理「文字預設樣式」的選單結果。
pub fn default_style_cmd(owner: HWND, base: u32, cmd: u32) -> bool {
    style::handle(owner, base, cmd, None)
}

/// `--render-video out.mp4 淺色桌布 深色桌布`／`--render-stills 資料夾 淺色桌布 深色桌布`（開發用：宣傳影片、網站截圖）。
pub fn render_promo(video: bool, out: &str, light: &str, dark: &str) {
    let (light, dark) = (std::path::Path::new(light), std::path::Path::new(dark));
    let result = if video { promo::render_video(out, light, dark) } else { promo::render_stills(out, light, dark) };
    match result {
        Ok(()) => crate::base::log_info(format!("render-promo：{out}")),
        Err(e) => crate::base::log_error(format!("render-promo：{e:#}")),
    }
}

/// `--export-sprites 資料夾`：宣傳影片（Remotion）用的透明素材和 sprites.json（開發用）。
pub fn export_sprites(dir: &str, picture: Option<&str>) {
    match sprites::export(dir, picture) {
        Ok(()) => crate::base::log_info(format!("export-sprites：{dir}")),
        Err(e) => crate::base::log_error(format!("export-sprites：{e:#}")),
    }
}

/// `--make-logo 原圖 資料夾`：Logo（去背、去掉灰影子、細白邊）→ 各尺寸 PNG 和多尺寸 .ico（開發用）。
pub fn make_logo(src: &str, dir: &str) {
    let result = (|| -> anyhow::Result<String> {
        let dir = std::path::Path::new(dir);
        std::fs::create_dir_all(dir)?;
        let recipe = stickers::Recipe { out_max: 1100, border: 1.2, round: 3.0, holes: false, upscale: 1.0, shadows: true, smooth: false };
        let art = stickers::make(&image::decode_file(std::path::Path::new(src))?, &recipe);
        // 正方形畫布、置中
        let side = art.w.max(art.h);
        let mut sq = image::Pixels { w: side, h: side, data: vec![0; (side * side * 4) as usize] };
        let (ox, oy) = ((side - art.w) / 2, (side - art.h) / 2);
        for y in 0..art.h {
            let s = (y * art.w * 4) as usize;
            let d = (((y + oy) * side + ox) * 4) as usize;
            sq.data[d..d + (art.w * 4) as usize].copy_from_slice(&art.data[s..s + (art.w * 4) as usize]);
        }
        let mut ico_entries = Vec::new();
        for size in [1024u32, 512, 256, 192, 180, 128, 96, 64, 48, 40, 32, 24, 20, 16] {
            let p = if size == side { sq.clone() } else { sq.resized(size, size) };
            let path = dir.join(format!("pinpin-{size}.png"));
            image::save_png(&p, &path)?;
            if size <= 256 && ![192, 180, 40, 20].contains(&size) {
                ico_entries.push((size, std::fs::read(&path)?));
            }
        }
        // .ico：每個尺寸直接放 PNG
        ico_entries.sort_by_key(|e| e.0);
        let mut ico = Vec::new();
        ico.extend_from_slice(&[0, 0, 1, 0]);
        ico.extend_from_slice(&(ico_entries.len() as u16).to_le_bytes());
        let mut offset = 6 + 16 * ico_entries.len() as u32;
        for (size, png) in &ico_entries {
            let b = if *size >= 256 { 0 } else { *size as u8 };
            ico.extend_from_slice(&[b, b, 0, 0, 1, 0, 32, 0]);
            ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
            ico.extend_from_slice(&offset.to_le_bytes());
            offset += png.len() as u32;
        }
        for (_, png) in &ico_entries {
            ico.extend_from_slice(png);
        }
        std::fs::write(dir.join("pinpin.ico"), ico)?;
        Ok(format!("make-logo: {side}px → {}", dir.display()))
    })();
    let msg = result.unwrap_or_else(|e| format!("make-logo: {e:#}"));
    println!("{msg}");
    crate::base::log_info(msg);
}

/// 安全的亂數（CNG），給 update.rs 產生匿名安裝碼用。
pub fn random_bytes(n: usize) -> Vec<u8> {
    crypto::random(n)
}
