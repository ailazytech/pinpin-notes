//! 蕃茄鐘：放在桌面上的小計時器，可以是一顆蕃茄或一個沙漏。
//!
//! 省電的做法：
//! - 沒在跑就完全不跑計時器
//! - 蕃茄：一秒醒來一次，而且對齊到「秒數要變的那一刻」，用可合併的計時器（SetCoalescableTimer）讓系統順便一起叫醒
//! - 沙漏：沙子要流動，跑的時候每 200ms 畫一次（只畫沙漏那一小塊）；暫停就停
//! - 跑的時候只記「幾點結束」，關掉程式、重開機都接得上，不用一直寫檔
//!
//! 時間到：播聲音（內建合成的叮咚、Windows 鬧鐘音效、自己的 .wav，或不出聲）、在原地閃一下、跳提示。整個蕃茄鐘不會晃，只有倒數的地方會變。

use std::sync::OnceLock;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_MEMORY, SND_NODEFAULT};
use windows::Win32::UI::WindowsAndMessaging::*;

use super::render::{noise, smooth, Btn, Canvas, Fx, Hits, Rgb, Scene};
use super::store::{self, Phase, Timer, TimerStyle};
use super::{redraw, start_anim, with, AnimKind};
use crate::i18n::{t as tx, tf};
use crate::win::{self, wide, MenuItem};

/// 走秒用的計時器（和動畫的分開）
pub const TIMER_CLOCK: usize = 2;

/// 兩種樣子的大小（邏輯像素）。
pub fn size_of(style: TimerStyle) -> (i32, i32) {
    match style {
        TimerStyle::Tomato => (150, 156),
        TimerStyle::Hourglass => (124, 200),
        TimerStyle::Led => (184, 96),
        TimerStyle::Alarm => (150, 196),
        TimerStyle::Cat => (166, 196),
        TimerStyle::Dog => (146, 196),
    }
}

/// 所有樣子：(樣子, 名稱的語系 key)。
const STYLES: [(TimerStyle, &str); 6] = [
    (TimerStyle::Tomato, "timer.style.tomato"),
    (TimerStyle::Hourglass, "timer.style.hourglass"),
    (TimerStyle::Led, "timer.style.led"),
    (TimerStyle::Alarm, "timer.style.alarm"),
    (TimerStyle::Cat, "timer.style.cat"),
    (TimerStyle::Dog, "timer.style.dog"),
];

/// 用圖的樣子：(素材, 錶面中心 x、y 和半徑（2 倍圖的像素，`--make-art` 量的）, 主色)。
fn art_of(style: TimerStyle) -> Option<(&'static str, (f32, f32, f32), Rgb)> {
    match style {
        TimerStyle::Alarm => Some(("timer-alarm", (139.2, 206.1, 102.6), Rgb(232, 72, 64))),
        TimerStyle::Cat => Some(("timer-cat", (136.0, 239.9, 82.8), Rgb(236, 128, 44))),
        TimerStyle::Dog => Some(("timer-dog", (117.9, 242.6, 78.8), Rgb(206, 110, 52))),
        _ => None,
    }
}

// ───────────────────────────── 控制 ─────────────────────────────

fn timer_of(hwnd: HWND) -> Option<Timer> {
    with(|m| m.note_of(hwnd).and_then(|n| n.timer.clone())).flatten()
}

/// 改蕃茄鐘的狀態並存檔。
fn update(hwnd: HWND, f: impl FnOnce(&mut Timer)) -> Option<Timer> {
    with(|m| {
        let n = m.note_of(hwnd)?;
        let t = n.timer.as_mut()?;
        f(t);
        let t = t.clone();
        store::save(&m.board);
        Some(t)
    })
    .flatten()
}

/// 依狀態開、關走秒的計時器。
pub fn schedule(hwnd: HWND) {
    let Some(t) = timer_of(hwnd) else { return };
    unsafe {
        if !t.running() {
            let _ = KillTimer(Some(hwnd), TIMER_CLOCK);
            return;
        }
        let left = t.left(store::now_ms());
        let (ms, tolerance) = if left == 0 {
            (10, 0)
        } else if t.style == TimerStyle::Hourglass {
            (200, 50)
        } else {
            // 對齊到顯示的秒數要變的時候（剩下 59.999 秒 → 等 999ms 後顯示 59）
            (((left - 1) % 1000) as u32 + 15, 40)
        };
        SetCoalescableTimer(Some(hwnd), TIMER_CLOCK, ms.max(10), None, tolerance);
    }
}

/// 走秒：時間到就響，否則重畫、排下一次。
pub fn on_clock(hwnd: HWND) {
    let now = store::now_ms();
    match timer_of(hwnd) {
        Some(t) if t.running() && t.left(now) == 0 => {
            // 程式關著的時候就已經到了（例如隔天才開）：安靜地切到下一段就好
            finish(hwnd, now.saturating_sub(t.end_ms) > 60_000);
        }
        Some(t) if t.running() => {
            redraw(hwnd);
            schedule(hwnd);
        }
        _ => unsafe {
            let _ = KillTimer(Some(hwnd), TIMER_CLOCK);
        },
    }
}

fn finish(hwnd: HWND, quiet: bool) {
    let Some(t) = update(hwnd, |t| {
        let was = t.phase;
        t.end_ms = 0;
        if was == Phase::Focus {
            t.done += 1;
            t.phase = Phase::Break;
        } else {
            t.phase = Phase::Focus;
        }
        t.left_ms = t.total_ms();
        t.finished = true;
    }) else {
        return;
    };
    schedule(hwnd);
    if quiet {
        redraw(hwnd);
        return;
    }
    let sound = with(|m| m.board.timer_sound.clone()).unwrap_or_default();
    play(&sound);
    let msg = if t.phase == Phase::Break { tx("timer.focus_done") } else { tx("timer.break_done") };
    crate::toast::show(msg, false, 4);
    start_anim(hwnd, AnimKind::Ring);
}

/// 開始／暫停（點蕃茄鐘、按空白鍵）。
pub fn toggle(hwnd: HWND) {
    let now = store::now_ms();
    let mut flip = false;
    let Some(t) = update(hwnd, |t| {
        if t.running() {
            t.left_ms = t.left(now);
            t.end_ms = 0;
        } else {
            if t.finished {
                // 上一段的沙全在下面：翻過來開始下一段
                flip = true;
                t.finished = false;
                t.left_ms = t.total_ms();
            }
            t.end_ms = now + t.left_ms.max(1000);
        }
    }) else {
        return;
    };
    schedule(hwnd);
    let _ = (flip, t);
    redraw(hwnd);
}

/// 這一段重來（停在開頭）。
pub fn reset(hwnd: HWND) {
    let flip = timer_of(hwnd).is_some_and(|t| t.style == TimerStyle::Hourglass && (t.finished || t.progress(store::now_ms()) > 0.02));
    update(hwnd, |t| {
        t.end_ms = 0;
        t.finished = false;
        t.left_ms = t.total_ms();
    });
    schedule(hwnd);
    let _ = flip;
    redraw(hwnd);
}

/// 直接跳到下一段（專注 ↔ 休息）。
fn skip(hwnd: HWND) {
    update(hwnd, |t| {
        t.phase = if t.phase == Phase::Focus { Phase::Break } else { Phase::Focus };
        t.end_ms = 0;
        t.finished = false;
        t.left_ms = t.total_ms();
    });
    schedule(hwnd);
    redraw(hwnd);
}

/// 沒在跑的時候滾輪調分鐘（Shift 一次 5 分）。
pub fn adjust(hwnd: HWND, steps: i32, big: bool) {
    let changed = update(hwnd, |t| {
        if t.running() {
            return;
        }
        let d = steps * if big { 5 } else { 1 };
        let m = if t.phase == Phase::Focus { &mut t.focus } else { &mut t.rest };
        *m = (*m as i32 + d).clamp(1, 180) as u32;
        t.finished = false;
        t.left_ms = t.total_ms();
    });
    if changed.is_some() {
        redraw(hwnd);
    }
}

fn set_minutes(hwnd: HWND, phase: Phase, minutes: u32) {
    update(hwnd, |t| {
        if phase == Phase::Focus {
            t.focus = minutes;
        } else {
            t.rest = minutes;
        }
        if t.phase == phase && !t.running() {
            t.finished = false;
            t.left_ms = t.total_ms();
        }
    });
    redraw(hwnd);
}

fn set_style(hwnd: HWND, style: TimerStyle) {
    with(|m| {
        let n = m.note_of(hwnd)?;
        let (w, h) = size_of(style);
        n.w = w;
        n.h = h;
        n.timer.as_mut()?.style = style;
        store::save(&m.board);
        Some(())
    });
    schedule(hwnd);
    redraw(hwnd);
}

// ───────────────────────────── 右鍵選單 ─────────────────────────────

const T_TOGGLE: u32 = 1;
const T_RESET: u32 = 2;
const T_SKIP: u32 = 3;
const T_CLEAR_COUNT: u32 = 4;
const T_PIN_DESKTOP: u32 = 5;
const T_PIN_TOP: u32 = 6;
const T_DELETE: u32 = 7;
const T_STYLE: u32 = 10;
const T_FOCUS: u32 = 20;
const T_REST: u32 = 40;
const T_SOUND: u32 = 60;
const T_SOUND_FILE: u32 = 99;

const FOCUS_CHOICES: [u32; 7] = [15, 20, 25, 30, 45, 50, 60];
const REST_CHOICES: [u32; 5] = [3, 5, 10, 15, 20];

pub fn menu(hwnd: HWND) {
    let Some((t, layer, sound)) = with(|m| {
        let sound = m.board.timer_sound.clone();
        m.note_of(hwnd).and_then(|n| Some((n.timer.clone()?, n.layer, sound)))
    })
    .flatten() else {
        return;
    };
    let sounds = sound_choices();
    let minutes = |base: u32, list: &[u32], cur: u32| -> Vec<MenuItem> {
        let mut v: Vec<MenuItem> = list.iter().enumerate().map(|(i, &m)| MenuItem::checked(base + i as u32, tf("timer.minutes", &[("n", &m.to_string())]), m == cur)).collect();
        if !list.contains(&cur) {
            v.push(MenuItem::checked(0, tf("timer.minutes", &[("n", &cur.to_string())]), true));
        }
        v.push(MenuItem::disabled(tx("timer.wheel_hint")));
        v
    };
    let mut sound_items: Vec<MenuItem> = sounds.iter().enumerate().map(|(i, (code, label))| MenuItem::checked(T_SOUND + i as u32, label.clone(), *code == sound)).collect();
    let custom = sound.strip_prefix("file:").map(|p| std::path::Path::new(p).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default());
    if let Some(name) = custom {
        sound_items.push(MenuItem::checked(0, name, true));
    }
    sound_items.extend([MenuItem::Separator, MenuItem::item(T_SOUND_FILE, tx("timer.sound_file"))]);
    let items = vec![
        MenuItem::item(T_TOGGLE, if t.running() { tx("timer.pause") } else { tx("timer.start") }),
        MenuItem::item(T_RESET, tx("timer.reset")),
        MenuItem::item(T_SKIP, if t.phase == Phase::Focus { tx("timer.skip_to_break") } else { tx("timer.skip_to_focus") }),
        MenuItem::Separator,
        MenuItem::Submenu { text: tx("timer.focus_len"), items: minutes(T_FOCUS, &FOCUS_CHOICES, t.focus) },
        MenuItem::Submenu { text: tx("timer.rest_len"), items: minutes(T_REST, &REST_CHOICES, t.rest) },
        MenuItem::Submenu {
            text: tx("timer.style"),
            items: STYLES.iter().enumerate().map(|(i, (st, key))| MenuItem::checked(T_STYLE + i as u32, tx(key), t.style == *st)).collect(),
        },
        MenuItem::Submenu { text: tx("timer.sound"), items: sound_items },
        MenuItem::Separator,
        MenuItem::checked(T_PIN_DESKTOP, tx("menu.pin_desktop"), layer == store::Layer::Desktop),
        MenuItem::checked(T_PIN_TOP, tx("menu.pin_top"), layer == store::Layer::Top),
        MenuItem::item(T_CLEAR_COUNT, tf("timer.clear_count", &[("count", &t.done.to_string())])),
        MenuItem::Separator,
        MenuItem::item(T_DELETE, tx("menu.delete")),
    ];
    match win::popup_menu(hwnd, &items, win::cursor_pos()) {
        T_TOGGLE => toggle(hwnd),
        T_RESET => reset(hwnd),
        T_SKIP => skip(hwnd),
        T_CLEAR_COUNT => {
            update(hwnd, |t| t.done = 0);
            redraw(hwnd);
        }
        T_PIN_DESKTOP => super::set_layer(hwnd, if layer == store::Layer::Desktop { store::Layer::Normal } else { store::Layer::Desktop }),
        T_PIN_TOP => super::set_layer(hwnd, if layer == store::Layer::Top { store::Layer::Normal } else { store::Layer::Top }),
        T_DELETE => super::delete(hwnd),
        id if (T_STYLE..T_STYLE + STYLES.len() as u32).contains(&id) => set_style(hwnd, STYLES[(id - T_STYLE) as usize].0),
        id if (T_FOCUS..T_FOCUS + FOCUS_CHOICES.len() as u32).contains(&id) => set_minutes(hwnd, Phase::Focus, FOCUS_CHOICES[(id - T_FOCUS) as usize]),
        id if (T_REST..T_REST + REST_CHOICES.len() as u32).contains(&id) => set_minutes(hwnd, Phase::Break, REST_CHOICES[(id - T_REST) as usize]),
        id if (T_SOUND..T_SOUND + sounds.len() as u32).contains(&id) => set_sound(sounds[(id - T_SOUND) as usize].0.clone()),
        T_SOUND_FILE => {
            let filter = format!("{} (*.wav)\0*.wav\0", tx("timer.sound_filter"));
            if let Some(path) = super::open_dialog(hwnd, &filter, false).into_iter().next() {
                set_sound(format!("file:{}", path.display()));
            }
        }
        _ => {}
    }
}

/// 換聲音，順便試聽。
fn set_sound(code: String) {
    play(&code);
    with(|m| {
        m.board.timer_sound = code;
        store::save(&m.board);
    });
}

// ───────────────────────────── 聲音 ─────────────────────────────

/// 可以選的聲音：(代碼, 名稱)。Windows 內建的鬧鐘音效有幾個就列幾個。
fn sound_choices() -> Vec<(String, String)> {
    let mut list = vec![(String::new(), tx("timer.sound.chime")), ("bell".into(), tx("timer.sound.bell"))];
    let media = windows_media();
    for i in 1..=10 {
        let name = format!("Alarm{i:02}");
        if media.join(format!("{name}.wav")).exists() {
            list.push((format!("alarm:{name}"), tf("timer.sound.windows", &[("n", &i.to_string())])));
        }
    }
    list.push(("none".into(), tx("timer.sound.none")));
    list
}

fn windows_media() -> std::path::PathBuf {
    let root = std::env::var_os("SystemRoot").map(std::path::PathBuf::from).unwrap_or_else(|| "C:\\Windows".into());
    root.join("Media")
}

/// 播放（非同步，不卡畫面）。
pub fn play(code: &str) {
    unsafe {
        match code {
            "none" => {}
            "bell" | "" => {
                static CHIME: OnceLock<Vec<u8>> = OnceLock::new();
                static BELL: OnceLock<Vec<u8>> = OnceLock::new();
                let wav = if code == "bell" { BELL.get_or_init(bell_wav) } else { CHIME.get_or_init(chime_wav) };
                let _ = PlaySoundW(PCWSTR(wav.as_ptr() as *const u16), None, SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
            }
            _ => {
                let path = match code.split_once(':') {
                    Some(("alarm", name)) => windows_media().join(format!("{name}.wav")),
                    Some(("file", p)) => p.into(),
                    _ => return,
                };
                let name = wide(&path.to_string_lossy());
                if !PlaySoundW(PCWSTR(name.as_ptr()), None, SND_FILENAME | SND_ASYNC | SND_NODEFAULT).as_bool() {
                    // 檔案不見了：退回內建的叮咚
                    play("");
                }
            }
        }
    }
}

const RATE: u32 = 22050;

/// 合成一段聲音：每個音 = (頻率, 開始秒數, 長度, 音量)，帶一點泛音、快起慢收，像敲鐘。
fn synth(notes: &[(f32, f32, f32, f32)], total: f32) -> Vec<u8> {
    let n = (total * RATE as f32) as usize;
    let mut buf = vec![0f32; n];
    for &(freq, start, dur, vol) in notes {
        let s0 = (start * RATE as f32) as usize;
        let len = (dur * RATE as f32) as usize;
        for i in 0..len.min(n.saturating_sub(s0)) {
            let t = i as f32 / RATE as f32;
            let attack = (t / 0.006).min(1.0);
            let env = attack * (-t * 3.2 / dur.max(0.2)).exp();
            let ph = std::f32::consts::TAU * freq * t;
            let v = ph.sin() + 0.32 * (ph * 2.0).sin() * (-t * 6.0).exp() + 0.12 * (ph * 3.01).sin() * (-t * 9.0).exp();
            buf[s0 + i] += v * env * vol;
        }
    }
    let peak = buf.iter().fold(0.0f32, |a, v| a.max(v.abs())).max(0.001);
    let mut wav = Vec::with_capacity(44 + n * 2);
    let data_len = (n * 2) as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // 單聲道
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for v in buf {
        wav.extend_from_slice(&((v / peak * 0.7 * 32767.0) as i16).to_le_bytes());
    }
    wav
}

/// 叮—咚、叮—咚（溫和）。
fn chime_wav() -> Vec<u8> {
    synth(&[(1318.5, 0.0, 1.1, 1.0), (1046.5, 0.42, 1.4, 0.9), (1318.5, 1.7, 1.1, 1.0), (1046.5, 2.12, 1.6, 0.9)], 3.8)
}

/// 鈴鈴鈴（比較醒）。
fn bell_wav() -> Vec<u8> {
    let mut notes = Vec::new();
    for round in 0..3 {
        for k in 0..8 {
            let t0 = round as f32 * 1.1 + k as f32 * 0.075;
            notes.push((1760.0, t0, 0.35, 0.8));
            notes.push((2637.0, t0, 0.25, 0.35));
        }
    }
    synth(&notes, 3.6)
}

// ───────────────────────────── 畫面 ─────────────────────────────

/// 畫蕃茄鐘（paper = 整個蕃茄鐘的範圍）。
pub unsafe fn draw(hdc: HDC, canvas: &mut Canvas, scene: &mut Scene, paper: RECT, hits: &mut Hits) {
    let t = scene.note.timer.clone().unwrap_or_default();
    let s = scene.scale;
    // 整塊都收得到滑鼠（不然滑到兩個控制點中間就會以為離開了）
    canvas.catch_mouse(paper);
    match t.style {
        TimerStyle::Tomato => draw_tomato(hdc, canvas, scene, &t, paper),
        TimerStyle::Hourglass => draw_hourglass(hdc, canvas, scene, &t, paper),
        TimerStyle::Led => draw_led(hdc, canvas, scene, &t, paper),
        style => {
            if let Some((id, face, accent)) = art_of(style) {
                draw_art(hdc, canvas, scene, &t, paper, id, face, accent);
            }
        }
    }
    // 滑過時右上角：重來、選單
    if scene.hover && scene.lift <= 0.0 {
        let x = paper.right as f32 - 13.0 * s;
        for (i, (btn, glyph)) in [(Btn::Reset, "\u{E72C}"), (Btn::Menu, "\u{E712}")].into_iter().enumerate() {
            let y = paper.top as f32 + (13.0 + i as f32 * 27.0) * s;
            let hot = scene.hot == Some(btn);
            canvas.disc(x, y + 1.0 * s, 11.5 * s, Rgb(0, 0, 0), 0.18);
            canvas.disc(x, y, 11.0 * s, if hot { Rgb(70, 66, 62) } else { Rgb(252, 251, 248) }, 0.97);
            SelectObject(hdc, scene.fonts.icons.into());
            SetTextColor(hdc, if hot { Rgb(255, 255, 255) } else { Rgb(70, 66, 62) }.colorref());
            let r = (11.0 * s) as i32;
            let mut rc = RECT { left: x as i32 - r, top: y as i32 - r, right: x as i32 + r, bottom: y as i32 + r };
            hits.buttons.push((btn, rc));
            let mut text: Vec<u16> = glyph.encode_utf16().collect();
            DrawTextW(hdc, &mut text, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            let _ = GdiFlush();
            canvas.opaque_disc(x, y, 10.0 * s);
        }
    }
}

/// 剩下的時間 → "24:59"（超過一小時 "1:05:00"）。
fn clock_text(ms: u64) -> String {
    let secs = ms.div_ceil(1000);
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// 狀態那一行：專注・第 3 個／休息／點一下開始。
fn status_text(t: &Timer, now: u64) -> String {
    let phase = if t.phase == Phase::Focus { tx("timer.focus") } else { tx("timer.rest") };
    if t.running() {
        phase
    } else if t.left(now) < t.total_ms() && !t.finished {
        tf("timer.paused", &[("phase", &phase)])
    } else {
        tf("timer.click_to_start", &[("phase", &phase)])
    }
}

/// 字（GDI）。GDI 會把畫到的地方 alpha 清成 0，所以先記下原本的 alpha、畫完放回去（字要畫在不透明的東西上）。
unsafe fn text_at(hdc: HDC, canvas: &mut Canvas, text: &str, rc: RECT, font: HFONT, color: Rgb) {
    let (l, t, r, b) = (rc.left.max(0), rc.top.max(0), rc.right.min(canvas.w), rc.bottom.min(canvas.h));
    let mut saved = Vec::new();
    for y in t..b {
        for x in l..r {
            saved.push(canvas.buf[((y * canvas.w + x) * 4 + 3) as usize]);
        }
    }
    SelectObject(hdc, font.into());
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, color.colorref());
    let mut w: Vec<u16> = text.encode_utf16().collect();
    let mut rr = rc;
    DrawTextW(hdc, &mut w, &mut rr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    let _ = GdiFlush();
    let mut k = 0;
    for y in t..b {
        for x in l..r {
            let i = ((y * canvas.w + x) * 4) as usize;
            let a = saved[k];
            k += 1;
            canvas.buf[i + 3] = a;
            for c in 0..3 {
                canvas.buf[i + c] = canvas.buf[i + c].min(a);
            }
        }
    }
}

fn make_font(size: f32, weight: i32) -> HFONT {
    unsafe {
        CreateFontW(-(size.round() as i32), 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, ANTIALIASED_QUALITY, 0, w!("Segoe UI"))
    }
}

fn ui_font(size: f32) -> HFONT {
    unsafe {
        CreateFontW(-(size.round() as i32), 0, 0, 0, 600, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, ANTIALIASED_QUALITY, 0, w!("Microsoft JhengHei UI"))
    }
}

/// 動畫：(水平晃動 px, 縮放, 旋轉弧度)。
/// 蕃茄鐘整個不動（使用者要求：只有倒數的地方會變）。時間到的提示只在原地閃、擴散，不搖。
fn wobble(_fx: Fx) -> (f32, f32, f32) {
    (0.0, 1.0, 0.0)
}

unsafe fn draw_tomato(hdc: HDC, canvas: &mut Canvas, scene: &Scene, t: &Timer, paper: RECT) {
    let s = scene.scale;
    let now = scene.now_ms;
    let (_, k, rot) = wobble(scene.fx);
    let cx = (paper.left + paper.right) as f32 / 2.0;
    let cy = paper.top as f32 + 86.0 * s;
    let (rx, ry) = (62.0, 55.0);
    let focus = t.phase == Phase::Focus;
    let base = if focus { Rgb(226, 66, 52) } else { Rgb(122, 180, 86) };
    let left = t.left(now);
    let frac = if t.finished && !t.running() { 1.0 } else { (left as f32 / t.total_ms() as f32).clamp(0.0, 1.0) };
    let secs = left.div_ceil(1000);
    let (sr, cr) = rot.sin_cos();

    // 時間到：一圈圈擴散
    if let Fx::Ring(p) = scene.fx {
        for wave in 0..2 {
            let q = (p * 1.6 - wave as f32 * 0.35).clamp(0.0, 1.0);
            if q > 0.0 && q < 1.0 {
                canvas.ring(cx, cy, (rx + 4.0 + q * 16.0) * s, 2.5 * s, base);
            }
        }
    }
    // 底下的影子
    let (sx0, sx1) = ((cx - 70.0 * s) as i32, (cx + 70.0 * s) as i32);
    for y in ((cy + 40.0 * s) as i32).max(0)..((cy + 72.0 * s) as i32).min(canvas.h) {
        for x in sx0.max(0)..sx1.min(canvas.w) {
            let ex = (x as f32 + 0.5 - cx) / (rx * 0.85 * s);
            let ey = (y as f32 + 0.5 - (cy + ry * s * k + 3.0 * s)) / (9.0 * s);
            let e = ex * ex + ey * ey;
            if e < 1.0 {
                canvas.over(x, y, [0.0, 0.0, 0.0, 1.0], 0.22 * smooth(1.0 - e));
            }
        }
    }
    // 蕃茄本體、進度圈、蒂頭：逐點算形狀（可以旋轉、壓扁，邊緣反鋸齒）
    let reach = 80.0 * s;
    for y in ((cy - reach) as i32).max(0)..((cy + reach) as i32).min(canvas.h) {
        for x in ((cx - reach) as i32).max(0)..((cx + reach) as i32).min(canvas.w) {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / s, (y as f32 + 0.5 - cy) / s);
            let (lx, ly) = ((dx * cr + dy * sr) / (2.0 - k), (-dx * sr + dy * cr) / k);
            // 上面中間凹一點（蒂頭的地方）
            let dent = 7.0 * (-(lx * lx) / (2.0 * 15.0 * 15.0)).exp() * (-ly / ry).max(0.0);
            let (nx, ny) = (lx / rx, (ly - dent) / ry);
            let e = (nx * nx + ny * ny).sqrt();
            let d = (e - 1.0) * rx.min(ry) * s;
            let cov = (0.5 - d).clamp(0.0, 1.0);
            if cov > 0.0 {
                // 左上打光、邊緣暗一點、淡淡的直紋
                let ang = lx.atan2(-ly);
                let mut shade = 1.0 - 0.16 * (nx * 0.5 + ny * 0.85) - 0.2 * smooth((e - 0.72) / 0.28);
                shade *= 1.0 - 0.035 * (1.0 + (ang * 6.0).cos()) * smooth((e - 0.2) / 0.6);
                let mut c = [base.2 as f32 * shade, base.1 as f32 * shade, base.0 as f32 * shade];
                // 亮點
                let (hx, hy) = ((lx + 26.0) / 15.0, (ly + 22.0) / 8.0);
                let hl = smooth(1.0 - (hx * hx + hy * hy)) * 0.6;
                // 進度圈：還剩的部分亮、走掉的部分淡
                let band = 1.0 - ((e - 0.8) / 0.045).abs();
                let band = (band * 2.5).clamp(0.0, 1.0);
                let turn = (ang.rem_euclid(std::f32::consts::TAU)) / std::f32::consts::TAU;
                let ring = if turn <= frac { 0.62 } else { 0.16 };
                let white = (hl + band * ring).min(1.0);
                for v in c.iter_mut() {
                    *v = (*v + (255.0 - *v) * white).min(255.0);
                }
                canvas.over(x, y, [c[0], c[1], c[2], 1.0], cov);
            }
            // 蒂頭：五片葉子＋梗
            let (vx, vy) = (lx, ly + ry - 6.0);
            let r = (vx * vx + vy * vy).sqrt();
            let a = vy.atan2(vx);
            let petal = 17.0 * (0.42 + 0.58 * ((a * 5.0 / 2.0).cos().abs()).powf(1.6));
            let leaf = (0.5 - (r - petal) * s).clamp(0.0, 1.0);
            if leaf > 0.0 {
                let g = 1.0 - 0.25 * (r / 17.0);
                canvas.over(x, y, [62.0 * g, 150.0 * g, 76.0 * g, 1.0], leaf);
            }
            let stem_x = (lx - 1.0 - (ly + ry + 6.0) * 0.15).abs();
            if stem_x < 2.6 && ly > -ry - 12.0 && ly < -ry + 2.0 {
                canvas.over(x, y, [50.0, 112.0, 58.0, 1.0], (2.6 - stem_x).clamp(0.0, 1.0));
            }
        }
    }
    // 秒針：跑的時候一顆小白點每秒沿著圈跳一格
    if t.running() {
        let a = (secs % 60) as f32 / 60.0 * std::f32::consts::TAU;
        let (lx, ly) = (a.sin() * rx * 0.8, -a.cos() * ry * 0.8);
        let (px, py) = (cx + (lx * cr - ly * sr) * s, cy + (lx * sr + ly * cr) * s * k);
        canvas.disc(px, py, 3.4 * s, Rgb(255, 255, 255), 1.0);
    }
    // 字
    {
        let big = make_font(31.0 * s, 700);
        let small = ui_font(11.5 * s);
        let time_rc = RECT { left: (cx - 50.0 * s) as i32, top: (cy - 22.0 * s) as i32, right: (cx + 50.0 * s) as i32, bottom: (cy + 14.0 * s) as i32 };
        let label_rc = RECT { left: (cx - 46.0 * s) as i32, top: (cy + 14.0 * s) as i32, right: (cx + 46.0 * s) as i32, bottom: (cy + 30.0 * s) as i32 };
        let shown = if t.finished && !t.running() { t.total_ms() } else { left };
        text_at(hdc, canvas, &clock_text(shown), time_rc, big, Rgb(255, 255, 255));
        text_at(hdc, canvas, &status_text(t, now), label_rc, small, Rgb(255, 246, 240));
        let _ = DeleteObject(big.into());
        let _ = DeleteObject(small.into());
    }
    // 完成的蕃茄數：底下一排小點
    let count = t.done.min(8);
    for i in 0..count {
        let x = cx + (i as f32 - (count as f32 - 1.0) / 2.0) * 11.0 * s;
        canvas.disc(x, paper.bottom as f32 - 6.0 * s, 3.6 * s, Rgb(226, 66, 52), 1.0);
        canvas.disc(x - 0.8 * s, paper.bottom as f32 - 7.6 * s, 1.3 * s, Rgb(120, 190, 100), 1.0);
    }
}

unsafe fn draw_hourglass(hdc: HDC, canvas: &mut Canvas, scene: &Scene, t: &Timer, paper: RECT) {
    let s = scene.scale;
    let now = scene.now_ms;
    let (_, k, rot) = wobble(scene.fx);
    let p = t.progress(now);

    let cx = (paper.left + paper.right) as f32 / 2.0;
    let cy = paper.top as f32 + 84.0 * s;
    let (sr, cr) = rot.sin_cos();
    let flowing = t.running() && p > 0.0 && p < 1.0;
    let phase_ms = (now % 100_000) as f32;
    let focus = t.phase == Phase::Focus;
    let sand = if focus { Rgb(236, 192, 120) } else { Rgb(150, 200, 170) };

    // 玻璃的半寬（y = -66～66，0 是瓶頸）
    let glass_w = |y: f32| {
        let u = (y.abs() / 66.0).min(1.0);
        3.0 + 25.0 * (u * std::f32::consts::FRAC_PI_2).sin().powf(0.9) * (1.0 - 0.32 * smooth((u - 0.8) / 0.2))
    };
    let lower = 64.0 * p.powf(0.85);
    let pile = |x: f32| 64.0 - lower + 10.0 * (x / 24.0).powi(2) * (lower / 22.0).min(1.0);
    let top_level = -1.5 - 60.0 * (1.0 - p).powf(0.8);

    // 影子
    for y in ((cy + 74.0 * s) as i32).max(0)..((cy + 92.0 * s) as i32).min(canvas.h) {
        for x in ((cx - 56.0 * s) as i32).max(0)..((cx + 56.0 * s) as i32).min(canvas.w) {
            let ex = (x as f32 + 0.5 - cx) / (48.0 * s);
            let ey = (y as f32 + 0.5 - (cy + 82.0 * s)) / (6.0 * s);
            let e = ex * ex + ey * ey;
            if e < 1.0 {
                canvas.over(x, y, [0.0, 0.0, 0.0, 1.0], 0.22 * smooth(1.0 - e));
            }
        }
    }
    let reach = 90.0 * s;
    let aa = |d: f32| (0.5 - d * s).clamp(0.0, 1.0);
    for y in ((cy - reach) as i32).max(0)..((cy + reach) as i32).min(canvas.h) {
        for x in ((cx - reach) as i32).max(0)..((cx + reach) as i32).min(canvas.w) {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / s, (y as f32 + 0.5 - cy) / s);
            let (lx, ly) = ((dx * cr + dy * sr) / (2.0 - k), (-dx * sr + dy * cr) / k);
            // 玻璃
            if ly.abs() <= 67.0 {
                let gw = glass_w(ly);
                let d = lx.abs() - gw;
                let inside = aa(d);
                if inside > 0.0 {
                    canvas.over(x, y, [250.0, 248.0, 240.0, 1.0], 0.2 * inside);
                    // 沙
                    let inner = aa(lx.abs() - (gw - 1.6));
                    let mut amount = 0.0;
                    if ly < -1.0 && ly > top_level + 3.5 * (1.0 - (lx / gw).powi(2)).max(0.0) * (p > 0.01) as u8 as f32 {
                        amount = inner;
                    }
                    if ly > 0.0 && ly < 64.5 && ly > pile(lx) {
                        amount = inner;
                    }
                    // 流下來的細沙：一顆顆往下掉
                    if flowing && ly >= -1.0 && ly <= pile(0.0) && lx.abs() < 1.6 {
                        let g = noise((ly * 0.9 - phase_ms * 0.09) as i32, 7);
                        amount = amount.max(if g > 0.3 { 0.95 } else { 0.25 } * (1.6 - lx.abs()).min(1.0));
                    }
                    if amount > 0.0 {
                        let grain = 0.9 + 0.14 * noise((lx * 2.0) as i32, (ly * 2.0) as i32);
                        canvas.over(x, y, [sand.2 as f32 * grain, sand.1 as f32 * grain, (sand.0 as f32 * grain).min(255.0), 1.0], amount);
                    }
                    // 玻璃的邊和左邊的反光
                    let edge = (1.0 - (d + 0.8).abs() / 1.0).clamp(0.0, 1.0);
                    if edge > 0.0 {
                        canvas.over(x, y, [245.0, 236.0, 222.0, 1.0], 0.75 * edge);
                    }
                    let shine = lx + gw - 5.0;
                    if shine.abs() < 1.4 && ly.abs() > 10.0 && ly.abs() < 58.0 {
                        canvas.over(x, y, [255.0, 255.0, 255.0, 1.0], 0.5 * (1.4 - shine.abs()).min(1.0));
                    }
                }
            }
            // 木頭柱子
            let post = (lx.abs() - 34.0).abs() - 2.6;
            if ly.abs() < 68.0 && post < 0.5 {
                let wood = 0.9 + 0.12 * noise(lx as i32, (ly * 0.3) as i32);
                canvas.over(x, y, [62.0 * wood, 100.0 * wood, 146.0 * wood, 1.0], aa(post));
            }
            // 上下的木蓋（圓角）
            let cap_y = ly.abs() - 73.0;
            let qx = (lx.abs() - 40.0 + 4.0).max(0.0);
            let qy = (cap_y.abs() - 5.5 + 4.0).max(0.0);
            let cap = (qx * qx + qy * qy).sqrt() - 4.0;
            if cap < 0.5 {
                let light = 1.08 - 0.14 * ((cap_y + 5.5) / 11.0);
                let wood = light * (0.92 + 0.1 * noise((lx * 0.25) as i32, (ly * 2.0) as i32 + 50));
                canvas.over(x, y, [72.0 * wood, 116.0 * wood, (168.0 * wood).min(255.0), 1.0], aa(cap));
            }
        }
    }
    // 底下的時間牌子
    let pill = RECT { left: (cx - 54.0 * s) as i32, top: paper.bottom - (28.0 * s) as i32, right: (cx + 54.0 * s) as i32, bottom: paper.bottom - (4.0 * s) as i32 };
    let (pl, pt, pr, pb) = (pill.left as f32, pill.top as f32, pill.right as f32, pill.bottom as f32);
    let pr_r = (pb - pt) / 2.0;
    for y in pill.top.max(0)..pill.bottom.min(canvas.h) {
        for x in pill.left.max(0)..pill.right.min(canvas.w) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let qx = (px - px.clamp(pl + pr_r, pr - pr_r)).abs();
            let qy = py - (pt + pb) / 2.0;
            let d = (qx * qx + qy * qy).sqrt() - pr_r;
            let cov = (0.5 - d).clamp(0.0, 1.0);
            if cov > 0.0 {
                let c = if focus { [60.0, 56.0, 64.0] } else { [70.0, 110.0, 60.0] };
                canvas.over(x, y, [c[0], c[1], c[2], 1.0], cov);
            }
        }
    }
    let left = if t.finished && !t.running() { t.total_ms() } else { t.left(now) };
    let label = format!("{}  {}", clock_text(left), if t.running() { if focus { tx("timer.focus") } else { tx("timer.rest") } } else { "▶".into() });
    let font = ui_font(13.0 * s);
    text_at(hdc, canvas, &label, pill, font, Rgb(255, 255, 255));
    let _ = DeleteObject(font.into());
    // 完成的蕃茄數
    let count = t.done.min(8);
    for i in 0..count {
        let x = cx + (i as f32 - (count as f32 - 1.0) / 2.0) * 9.0 * s;
        canvas.disc(x, pill.top as f32 - 6.0 * s, 3.0 * s, Rgb(226, 66, 52), 1.0);
    }
}

// ───────────────────────────── LED 數字錶 ─────────────────────────────

/// 七段顯示器每個數字亮哪幾段（a b c d e f g）。
const SEGMENTS: [u8; 10] = [0b1111110, 0b0110000, 0b1101101, 0b1111001, 0b0110011, 0b1011011, 0b1011111, 0b1110000, 0b1111111, 0b1111011];

fn seg_distance(px: f32, py: f32, (ax, ay, bx, by): (f32, f32, f32, f32)) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = (dx * dx + dy * dy).max(0.0001);
    let u = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    ((px - ax - u * dx).powi(2) + (py - ay - u * dy).powi(2)).sqrt()
}

unsafe fn draw_led(hdc: HDC, canvas: &mut Canvas, scene: &Scene, t: &Timer, paper: RECT) {
    let s = scene.scale;
    let now = scene.now_ms;
    let focus = t.phase == Phase::Focus;
    let glow = if focus { Rgb(255, 78, 58) } else { Rgb(70, 232, 170) };
    let left = if t.finished && !t.running() { t.total_ms() } else { t.left(now) };
    let secs = left.div_ceil(1000);
    // 時間到：左右抖、數字一閃一閃；開始／暫停：暗一下
    let (shake, lit) = match scene.fx {
        Fx::Ring(q) => (0.0, if (q * 10.0) as i32 % 2 == 0 { 1.0 } else { 0.35 }),
        _ => (0.0, 1.0),
    };
    let (l, tp, r, b) = (paper.left as f32 + 4.0 * s + shake, paper.top as f32 + 4.0 * s, paper.right as f32 - 4.0 * s + shake, paper.bottom as f32 - 4.0 * s);
    canvas.shadow(RECT { left: l as i32, top: tp as i32, right: r as i32, bottom: b as i32 }, 0.0, 3.0 * s, 8.0 * s, 0.3);
    super::ink::round_rect(canvas, (l, tp, r, b), 14.0 * s, Rgb(44, 46, 54), 1.0);
    // 上緣一點反光
    super::ink::round_rect(canvas, (l + 2.0 * s, tp + 1.5 * s, r - 2.0 * s, tp + 12.0 * s), 11.0 * s, Rgb(255, 255, 255), 0.07);
    let screen = (l + 7.0 * s, tp + 7.0 * s, r - 7.0 * s, b - 7.0 * s);
    super::ink::round_rect(canvas, screen, 8.0 * s, Rgb(12, 13, 17), 1.0);

    // 數字：分鐘可能超過 99（最多 180）
    let text = format!("{:02}:{:02}", secs / 60, secs % 60);
    let (dw, dh, th, gap, colon_w) = (21.0 * s, 38.0 * s, 4.6 * s, 6.0 * s, 10.0 * s);
    let digits = text.chars().filter(|c| c.is_ascii_digit()).count() as f32;
    let total = digits * dw + (digits - 1.0) * gap + colon_w + gap;
    let mut x = (screen.0 + screen.2) / 2.0 - total / 2.0;
    let y0 = screen.1 + 8.0 * s;
    let slant = 0.1;
    let colon_on = !t.running() || secs % 2 == 0;
    let color = [glow.2 as f32, glow.1 as f32, glow.0 as f32, 1.0];
    for ch in text.chars() {
        if ch == ':' {
            if colon_on {
                for fy in [0.32, 0.7] {
                    let cy = y0 + dh * fy;
                    canvas.disc(x + colon_w / 2.0 + (y0 + dh - cy) * slant, cy, th * 0.62, glow, lit);
                }
            }
            x += colon_w + gap;
            continue;
        }
        let bits = SEGMENTS[ch.to_digit(10).unwrap_or(0) as usize];
        let i = th * 0.75;
        let (x0, x1, ym, y1) = (x, x + dw, y0 + dh / 2.0, y0 + dh);
        let segs = [
            (x0 + i, y0, x1 - i, y0),
            (x1, y0 + i, x1, ym - i),
            (x1, ym + i, x1, y1 - i),
            (x0 + i, y1, x1 - i, y1),
            (x0, ym + i, x0, y1 - i),
            (x0, y0 + i, x0, ym - i),
            (x0 + i, ym, x1 - i, ym),
        ];
        // 往右斜一點，像真的 LED
        let lean = |(ax, ay, bx, by): (f32, f32, f32, f32)| (ax + (y1 - ay) * slant, ay, bx + (y1 - by) * slant, by);
        let reach = th * 2.4;
        for py in ((y0 - reach) as i32).max(0)..((y1 + reach) as i32).min(canvas.h) {
            for px in ((x0 - reach) as i32).max(0)..((x1 + dh * slant + reach) as i32).min(canvas.w) {
                let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                let (mut on, mut off) = (f32::MAX, f32::MAX);
                for (k, seg) in segs.iter().enumerate() {
                    let d = seg_distance(fx, fy, lean(*seg));
                    if bits & (1 << (6 - k)) != 0 {
                        on = on.min(d);
                    } else {
                        off = off.min(d);
                    }
                }
                let core = (th / 2.0 - on + 0.5).clamp(0.0, 1.0);
                let halo = (-(on / (th * 1.3)).powi(2)).exp() * 0.32;
                let ghost = (th / 2.0 - off + 0.5).clamp(0.0, 1.0) * 0.07;
                let a = (core.max(halo) * lit).max(ghost);
                if a > 0.003 {
                    canvas.over(px, py, color, a);
                }
            }
        }
        x += dw + gap;
    }
    // 底下一排小燈：剩下多少
    let frac = (left as f32 / t.total_ms() as f32).clamp(0.0, 1.0);
    let n = 14;
    let row_y = screen.3 - 9.0 * s;
    let dots_right = screen.2 - 10.0 * s;
    for k in 0..n {
        let cx = dots_right - (n - 1 - k) as f32 * 7.0 * s;
        let on = (k as f32) < (frac * n as f32).ceil();
        canvas.disc(cx, row_y, 2.0 * s, glow, if on { 0.9 * lit.max(0.4) } else { 0.12 });
    }
    // 左下角：專注／休息
    let font = ui_font(10.5 * s);
    let label = if focus { tx("timer.focus") } else { tx("timer.rest") };
    let rc = RECT { left: (screen.0 + 8.0 * s) as i32, top: (row_y - 8.0 * s) as i32, right: (screen.0 + 70.0 * s) as i32, bottom: (row_y + 8.0 * s) as i32 };
    let dim = |v: u8| (v as f32 * 0.8) as u8;
    text_left(hdc, canvas, &label, rc, font, Rgb(dim(glow.0), dim(glow.1), dim(glow.2)));
    let _ = DeleteObject(font.into());
    // 完成的蕃茄數：右上角小紅點
    for i in 0..t.done.min(8) {
        canvas.disc(screen.2 - 8.0 * s - i as f32 * 7.0 * s, screen.1 + 6.0 * s, 2.2 * s, Rgb(255, 92, 70), 0.85);
    }
}

/// 靠左的字（和 text_at 一樣會保留 alpha）。
unsafe fn text_left(hdc: HDC, canvas: &mut Canvas, text: &str, rc: RECT, font: HFONT, color: Rgb) {
    let (l, t, r, b) = (rc.left.max(0), rc.top.max(0), rc.right.min(canvas.w), rc.bottom.min(canvas.h));
    let mut saved = Vec::new();
    for y in t..b {
        for x in l..r {
            saved.push(canvas.buf[((y * canvas.w + x) * 4 + 3) as usize]);
        }
    }
    SelectObject(hdc, font.into());
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, color.colorref());
    let mut w: Vec<u16> = text.encode_utf16().collect();
    let mut rr = rc;
    DrawTextW(hdc, &mut w, &mut rr, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    let _ = GdiFlush();
    let mut k = 0;
    for y in t..b {
        for x in l..r {
            let i = ((y * canvas.w + x) * 4) as usize;
            let a = saved[k];
            k += 1;
            canvas.buf[i + 3] = a;
            for c in 0..3 {
                canvas.buf[i + c] = canvas.buf[i + c].min(a);
            }
        }
    }
}

// ───────────────────────────── 圖：鬧鐘、貓、狗 ─────────────────────────────

/// 畫一張抱著時鐘的圖：圖本身（跑的時候像鐘擺一樣一秒換邊輕輕搖），錶面上畫進度圈、時間。
#[allow(clippy::too_many_arguments)]
unsafe fn draw_art(hdc: HDC, canvas: &mut Canvas, scene: &mut Scene, t: &Timer, paper: RECT, id: &str, face: (f32, f32, f32), accent: Rgb) {
    let s = scene.scale;
    let now = scene.now_ms;
    let Some(img) = scene.assets.get(id).cloned() else { return };
    let (_, k, rot) = wobble(scene.fx);
    let left = if t.finished && !t.running() { t.total_ms() } else { t.left(now) };
    let secs = left.div_ceil(1000);
    let (iw, ih) = (img.w as f32, img.h as f32);
    let cx = (paper.left + paper.right) as f32 / 2.0;
    let top = paper.top as f32 + 2.0 * s;
    // 以底部中間為支點轉、縮
    let pivot = (cx, top + ih);
    let (sn, cs) = rot.sin_cos();
    let place = |x: f32, y: f32| {
        let (dx, dy) = ((x - pivot.0) * k, (y - pivot.1) * k);
        (pivot.0 + dx * cs - dy * sn, pivot.1 + dx * sn + dy * cs)
    };
    // 影子、圖
    let (icx, icy) = place(cx, top + ih / 2.0);
    canvas.blit_xf(&img, icx + 1.5 * s, icy + 3.0 * s, rot, k, 0.2, true);
    canvas.blit_xf(&img, icx, icy, rot, k, 1.0, false);
    // 錶面
    let (fx, fy) = place(cx - iw / 2.0 + face.0 / 2.0 * s, top + face.1 / 2.0 * s);
    let fr = face.2 / 2.0 * s * k;
    let frac = (left as f32 / t.total_ms() as f32).clamp(0.0, 1.0);
    let alarm = id == "timer-alarm";
    let tau = std::f32::consts::TAU;
    for y in ((fy - fr) as i32).max(0)..((fy + fr) as i32 + 1).min(canvas.h) {
        for x in ((fx - fr) as i32).max(0)..((fx + fr) as i32 + 1).min(canvas.w) {
            let (dx, dy) = (x as f32 + 0.5 - fx, y as f32 + 0.5 - fy);
            let d = (dx * dx + dy * dy).sqrt();
            if d > fr {
                continue;
            }
            // 錶面跟著圖一起轉
            let (ux, uy) = (dx * cs + dy * sn, -dx * sn + dy * cs);
            let turn = ux.atan2(-uy).rem_euclid(tau) / tau;
            let e = d / fr;
            // 進度圈
            let band = ((1.0 - (e - 0.86).abs() / 0.055) * 2.0).clamp(0.0, 1.0);
            if band > 0.0 {
                if turn <= frac {
                    canvas.over(x, y, [accent.2 as f32, accent.1 as f32, accent.0 as f32, 1.0], band * 0.92);
                } else {
                    canvas.over(x, y, [210.0, 212.0, 214.0, 1.0], band * 0.55);
                }
            }
            if alarm {
                // 鬧鐘：12 格刻度，還剩的時間是一塊淡淡的扇形
                let hour = (turn * 12.0).round() / 12.0;
                let off = (turn - hour).abs() * tau * d;
                if e > 0.66 && e < 0.75 && off < 1.1 * s {
                    canvas.over(x, y, [70.0, 62.0, 58.0, 1.0], ((1.1 * s - off) / s).clamp(0.0, 1.0) * 0.85);
                }
                if e < 0.64 && turn <= frac {
                    canvas.over(x, y, [accent.2 as f32, accent.1 as f32, accent.0 as f32, 1.0], 0.13);
                }
            }
        }
    }
    // 秒針（鬧鐘）：跑的時候一秒一格
    if alarm && t.running() {
        let a = (secs % 60) as f32 / 60.0 * tau + rot;
        let (ex, ey) = (fx + a.sin() * fr * 0.62, fy - a.cos() * fr * 0.62);
        canvas.line(fx, fy, ex, ey, 1.4 * s, accent, 0.9);
        canvas.disc(fx, fy, 2.6 * s, accent, 1.0);
    }
    // 時間和狀態
    let big = make_font(fr * 0.5, 700);
    let small = ui_font((fr * 0.2).max(10.0 * s));
    let ty = if alarm { fy + fr * 0.08 } else { fy - fr * 0.08 };
    let time_rc = RECT { left: (fx - fr * 0.86) as i32, top: (ty - fr * 0.3) as i32, right: (fx + fr * 0.86) as i32, bottom: (ty + fr * 0.22) as i32 };
    let label_rc = RECT { left: (fx - fr * 0.72) as i32, top: (ty + fr * 0.2) as i32, right: (fx + fr * 0.72) as i32, bottom: (ty + fr * 0.52) as i32 };
    text_at(hdc, canvas, &clock_text(left), time_rc, big, Rgb(76, 56, 44));
    text_at(hdc, canvas, &status_text(t, now), label_rc, small, Rgb(150, 120, 100));
    let _ = DeleteObject(big.into());
    let _ = DeleteObject(small.into());
    // 完成的蕃茄數
    let count = t.done.min(8);
    for i in 0..count {
        let x = cx + (i as f32 - (count as f32 - 1.0) / 2.0) * 9.0 * s;
        canvas.disc(x, paper.bottom as f32 - 5.0 * s, 3.0 * s, Rgb(226, 66, 52), 1.0);
    }
}
