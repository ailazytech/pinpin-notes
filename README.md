# PinPin Notes（貼貼）

<img src="assets/logo/pinpin-256.png" width="112" align="right" alt="PinPin Notes">

> **桌面上的小紙條，一直陪著你。**<br>
> *Little notes that stick with you.*

Windows 桌面便利貼：圖釘、紙膠帶、貼紙拖來拖去，手寫、透明標語、蕃茄鐘，平常幾乎不耗電。

A cozy sticky-notes app for the Windows desktop — pins, washi tape, stickers, handwriting, transparent slogans and pomodoro timers, with almost zero idle CPU.

- 單一 `.exe`（約 11 MB）、免安裝，Windows 10 / 11
- 閒置 0% CPU、約 15 MB 記憶體（沒有任何閒置計時器，只在畫面變動時重畫）
- 六種語言：繁中、简中、English、日本語、한국어、العربية（RTL）
- 開源（MIT），資料只存在自己的電腦

📦 下載：GitHub Releases（準備中）　🌐 官網：`site/`（準備中）　📝 [更新紀錄](CHANGELOG.md)

## 功能

- **便利貼**：開機自動放回桌面；拖曳移動，靠近別張或螢幕邊緣會輕輕吸住對齊（Alt 拖曳不吸，方向鍵 1px 微調）
- **劃掉**：滑過時每行左邊出現圓圈，點一下用紅筆劃掉（有劃線動畫）
- **圖釘、夾子、紙膠帶、貼紙**：從「貼紙小舖」拖到便利貼或桌面上；貼紙 146 張（4 倍解析度，放大不糊）分成日常、招財祝福、各國招財、盆栽、春聯、貓狗、小機車、台灣小物八區；小舖上方是三則輪播的廣告區；點一下出現控制點，右上角旋轉、右下角縮放；紙膠帶拖兩端可以拉長、轉方向。圖釘、夾子、紙膠帶沒選取時，拖它等於拖整張便利貼
- **自製貼紙**：在貼紙小舖開自己的群組，把圖片拖進來、Ctrl+V、點「＋」或「貼上剪貼簿」，自動去背、加圓弧白邊；保留原圖尺寸（最多 4096px），用 JPEG XR 品質 80% 存檔省空間。內建貼紙也可以右鍵收進群組
- **訂製**：貼紙小舖最後一張卡片連到官網的訂製頁（寵物貼紙、計時器角色、主題組）
- **手寫**：鋼筆、螢光筆、粉筆、蠟筆，8 種顏色、3 種粗細，橡皮擦、Ctrl+Z 復原；**透明紙**可以當浮在桌面上的手寫標語
- **文字樣式**：顏色、字型、粗體、大小（最大 6 倍）；系統匣「文字預設樣式」改全部，右鍵「文字樣式（這張）」只改一張
- **蕃茄鐘**：蕃茄、沙漏、LED 數字錶、鬧鐘、抱時鐘的貓、抱時鐘的柴犬；點一下開始／暫停，滾輪調分鐘；時間到會響（內建叮咚、鈴鈴、Windows 鬧鐘音效、自己的 .wav 或不出聲）。沒在跑就完全不耗電
- **紙張**：經典 8 色＋莫蘭迪 8 色＋透明；便利貼、平滑、方格、橫線、點陣、和紙、牛皮紙纖維
- **圖片**：拖圖檔進來或在便利貼上 Ctrl+V，變成拍立得
- **密碼**：每張可以設不同密碼（PBKDF2 + AES-256-GCM）。平常顯示一份偽裝待辦，輸入密碼後文字亂碼再變回原文；關掉或重開自動鎖回
- **星號記事本**：搭配 [Vibotype](../vibotype) 的選取工具列，選字按 ★ 就收進來
- **匯出／匯入 JSON**：換電腦或手動同步（圖片、手寫、自製貼紙和群組一起打包，加密的內容仍然加密）
- **桌面右鍵**「新增便利貼」「新增蕃茄鐘」、**開機啟動**，都可以在系統匣關掉
- **線上更新**：自動檢查新版（不打擾，系統匣選單才顯示）、重要更新自動下載下次啟動套用；新貼紙包和廣告也能線上送來，都可以在系統匣關掉（見 [docs/UPDATES.md](docs/UPDATES.md)）
- **多語系**：可以自己加語言（見 [locales/README.md](locales/README.md)）

完整的功能規格在 [docs/FEATURES.md](docs/FEATURES.md)。

## 使用

- 系統匣圖示：**左鍵**新增便利貼、**右鍵**選單（新增手寫、透明標語、蕃茄鐘也在這裡）
- 手寫：便利貼右上角的筆、或右鍵「手寫」；Esc／Enter 結束，E 切換橡皮擦
- 蕃茄鐘：空白鍵開始／暫停、R 重來；右鍵換樣式、時間、聲音
- 便利貼上：連點兩下編輯、右鍵選單、Delete 刪除（可以從系統匣復原）、Ctrl+L 鎖上
- 資料在 `%APPDATA%\PinPin\`，格式見 [docs/DATA.md](docs/DATA.md)

## 編譯

需要 Windows 10/11 和 Rust 穩定版（MSVC 工具鏈）。

```powershell
cargo test
cargo build --release
.\target\release\pinpin.exe
```

開發用參數（測試時用 `$env:APPDATA = "某個暫存資料夾"` 隔開資料，不要碰真實的便利貼）：

| 參數 | 用途 |
|---|---|
| `--render-notes out.png` | 不開視窗，把各種樣式畫成一張圖 |
| `--render-shop out.png [分類] [語言]` | 不開視窗，把貼紙小舖畫成圖 |
| `--render-stills 資料夾 淺色桌布 深色桌布` | 網站截圖 |
| `--export-sprites 資料夾` | 宣傳影片用的素材（每張便利貼、貼紙各一張 PNG） |
| `--make-art 原圖 輸出.png` | 做蕃茄鐘的圖：去背、細白邊、挖掉包住的白色空隙 |
| `--make-logo 原圖 資料夾` | Logo 各尺寸 PNG 和 `pinpin.ico` |
| `--check-update [--push]`、`--apply-update` | 測試線上更新（搭配 `PINPIN_FEED=網址`） |
| `--unborder 資料夾 前綴` | 拿掉貼紙的白邊（春聯） |
| `--demo-notes` | 放幾張範例便利貼 |
| `--shop` | 啟動時打開貼紙小舖 |
| `--new-note`、`--new-timer` | 新增便利貼／蕃茄鐘（桌面右鍵用；已經在跑的話交給它） |

## 專案結構

```
src/              Rust 原始碼（app.rs 進入點與系統匣、notes/ 便利貼本體）
locales/          介面翻譯（六語系 JSON，編譯時打包）
assets/           圖釘、紙膠帶、貼紙、蕃茄鐘圖、Logo（-src 是 AI 生圖原稿）
site/             官網（Vite+、React、Tailwind，靜態輸出、可放任何 Pages）
promo/remotion/   宣傳影片和網站截圖（Remotion）
docs/             功能規格、資料格式、發版流程
```

## 和其他程式溝通

找 `PinPin.Main` 視窗，送 `WM_COPYDATA`（`dwData = 0x50424E31`），內容是 UTF-8 JSON：

```json
{ "cmd": "note", "text": "要記下來的文字", "x": 100, "y": 200 }
{ "cmd": "star", "text": "收藏這段", "source": "chrome.exe" }
```

改名前的 `PinBoard.Main` 暫時還收得到。這個協議 Vibotype 在用，改的話兩邊要一起改（見 [docs/DATA.md](docs/DATA.md#ipc)）。

## 參與

歡迎 issue 和 PR，流程與規矩見 [CONTRIBUTING.md](CONTRIBUTING.md)；發版見 [docs/RELEASING.md](docs/RELEASING.md)。

想要自己的寵物貼紙或計時器角色？官網有訂製服務，或寄信到 ailazytech@gmail.com。

## 授權

[MIT](LICENSE)。內建的圖釘、紙膠帶、貼紙、蕃茄鐘圖片是用 AI 生圖後再處理，一樣以 MIT 釋出。
