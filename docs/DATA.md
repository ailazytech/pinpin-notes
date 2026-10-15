# 資料格式

PinPin 的資料全部在使用者自己的電腦上，沒有任何網路傳輸。這份文件是資料格式的正式說明；改 `src/notes/store.rs` 的結構時要一起更新。

## 資料夾

`%APPDATA%\PinPin\`（改名前是 `%APPDATA%\PinBoard\`，第一次啟動會整個搬過來）

| 路徑 | 內容 |
|---|---|
| `notes.json` | 所有便利貼、垃圾桶、偏好（`Board`，見下） |
| `settings.json` | 語言、開機啟動、桌面右鍵（`Settings`） |
| `notes\` | 拍立得圖片（檔名記在 `Note.image`） |
| `stickers\` | 自製貼紙：`<代碼>.jxr`（JPEG XR，品質 80%，含透明）；0.1.0 之前做的是 `<代碼>.png`。`<代碼>.src.jxr` 是做貼紙用的原圖（最長邊 4096），右鍵切換「不去背」時從它重做；沒有它（舊版做的、從備份匯入的）就不能切換，備份不含原圖 |
| `locales\` | 使用者自己加的語言檔（見 [locales/README.md](../locales/README.md)） |
| `pinpin.log` | 錯誤紀錄 |

所有 JSON 都是先寫到 `.tmp` 再改名（`base::write_atomic`），寫到一半當機也不會壞掉。

## notes.json（`Board`）

所有欄位都有預設值（`#[serde(default)]`），缺欄位不會讀取失敗；列舉值一律是 `snake_case` 字串。

| 欄位 | 型別 | 說明 |
|---|---|---|
| `notes` | `Note[]` | 桌面上的便利貼 |
| `trash` | `Note[]` | 最近刪掉的，最多 20 張，可以從系統匣復原 |
| `hidden` | bool | 全部暫時收起來 |
| `last_paper` / `last_texture` / `last_pin` | | 新便利貼沿用上次的選擇 |
| `sticker_groups` | `StickerGroup[]` | 貼紙小舖裡自己分的群組 |
| `whole_stickers` | `string[]` | 選了「不去背（整張照片）」的自製貼紙代碼（空的不寫） |
| `ink` | `InkPrefs` | 上次用的筆：`tool`、`color`（0xRRGGBB）、`size`（0 細／1 中／2 粗） |
| `timer_sound` | string | 時間到的聲音：`""` 內建叮咚、`"none"`、`"alarm:Alarm01"`、`"file:<路徑>"` |
| `text` | `TextStyle` | 文字預設樣式：`face`（"" = 微軟正黑體）、`color`（null = 依紙色）、`bold` |

### `Note`

| 欄位 | 型別 | 說明 |
|---|---|---|
| `id` | string | 唯一代碼；星號記事本固定是 `"starred"` |
| `kind` | `text` \| `image` \| `starred` \| `sticker` \| `timer` | 種類 |
| `lines` | `Line[]` | 每行：`text`、`done`（劃掉了）、`meta`（星號記事本的時間和來源） |
| `image` | string | 圖片檔名（`kind = image`；`lines[0]` 是圖說） |
| `x`, `y` | i32 | 紙張左上角，螢幕座標（實際像素） |
| `w`, `h` | i32 | 紙張大小（邏輯像素，100% 縮放的像素） |
| `angle` | f32 | 整張的角度（度，順時針，繞紙張中心；-180～180）。目前只有拍立得能轉；0 不寫出，舊資料沒有就是 0 |
| `paper` | 見下 | 紙色 |
| `texture` | `sticky` \| `plain` \| `grid` \| `lined` \| `dots` \| `washi` \| `fiber` | 紙質 |
| `font` | f32 | 字的大小倍率，0.7～6 |
| `face`, `color`, `bold` | 可為 null | 這張自己的文字樣式；null = 跟著 `Board.text` |
| `items` | `Attach[]` | 圖釘、夾子、紙膠帶、貼紙 |
| `strokes` | `Stroke[]` | 手寫 |
| `timer` | `Timer` \| null | 番茄鐘設定（`kind = timer`） |
| `layer` | `normal` \| `top` \| `desktop` | 視窗層級：一般、永遠在最上層、貼在桌面底層 |
| `hidden` | bool | 收起來（星號記事本用） |
| `secret` | `Secret` \| null | 有密碼時的加密內容 |
| `created` | u64 | 建立時間（Unix 秒） |
| `v` | u32 | 這張的資料格式版本 |
| `deco` | | 舊版欄位，只讀不寫（轉成 `items`） |

`paper`：`yellow`、`pink`、`blue`、`green`、`lavender`、`white`、`kraft`、`orange`、`morandi_rose`、`morandi_blue`、`sage`、`oat`、`morandi_lilac`、`milk_tea`、`morandi_yellow`、`morandi_grey`、`clear`（透明）。

### `Attach`

座標是相對紙張左上角的邏輯像素。

| 欄位 | 說明 |
|---|---|
| `asset` | 素材代碼：內建的見 `src/notes/assets.rs`（例如 `pin-red`、`tape-dusty`、`st-shiba`）；自製貼紙是 `my-<時間戳><序號>` |
| `x`, `y` | 中心點；紙膠帶是起點 |
| `x2`, `y2` | 紙膠帶的終點 |
| `angle` | 旋轉角度（度） |
| `scale` | 縮放（1 = 原大小） |

### `Stroke`

| 欄位 | 說明 |
|---|---|
| `tool` | `pen` \| `marker` \| `chalk` \| `crayon` |
| `color` | 0xRRGGBB |
| `size` | 筆寬（邏輯像素） |
| `pts` | `[x0, y0, x1, y1, …]`，相對紙張左上角的邏輯像素 × 4，取整數（省空間） |

### `Timer`

| 欄位 | 說明 |
|---|---|
| `style` | `tomato` \| `hourglass` \| `led` \| `alarm` \| `cat` \| `dog` |
| `focus`, `rest` | 專注、休息各幾分鐘 |
| `phase` | `focus` \| `break` |
| `end_ms` | 跑的時候：結束的時間（Unix 毫秒）；0 = 沒在跑 |
| `left_ms` | 沒在跑的時候：還剩幾毫秒 |
| `finished` | 上一段剛結束、還沒開始下一段 |
| `done` | 今天完成了幾個番茄 |

用結束時間而不是剩餘秒數存，所以關掉程式再開，倒數還是對的。

### `Secret`（密碼便利貼）

| 欄位 | 說明 |
|---|---|
| `salt` | 16 bytes，base64 |
| `nonce` | 12 bytes，base64 |
| `data` | AES-256-GCM 密文（含驗證標籤），base64 |
| `iterations` | PBKDF2-SHA256 次數（目前 200 000） |

明文是真正內容的 `Line[]` JSON；鎖著的時候 `lines` 存的是偽裝內容。實作用 Windows CNG（`src/notes/crypto.rs`），沒有第三方加密函式庫。

### `StickerGroup`

`id`、`name`、`items`（素材代碼陣列，可以混內建和自製）。

## settings.json（`Settings`）

| 欄位 | 預設 | 說明 |
|---|---|---|
| `lang` | `""` | 介面語言代碼；空字串 = 跟隨系統 |
| `autostart` | `false` | 開機啟動（HKCU `Run\PinPin`） |
| `desktop_menu` | `false` | 桌面右鍵選單（HKCU `DesktopBackground\Shell\PinPinNewNote`、`PinPinNewTimer`） |

## 匯出檔（`Bundle`）

系統匣「匯出」產生的 JSON，換電腦或備份用：

| 欄位 | 說明 |
|---|---|
| `app` | 固定 `"PinPin"` |
| `version` | 匯出時的 `store::VERSION` |
| `exported_at` | Unix 秒 |
| `notes` | `Note[]`（加密的仍然加密） |
| `sticker_groups` | `StickerGroup[]` |
| `images` | 圖檔名 → 圖片 base64 |
| `stickers` | 自製貼紙代碼 → 圖檔 base64（PNG 或 JPEG XR，匯入時依檔頭判斷副檔名） |

匯入是合併：同 `id` 的便利貼用匯入的版本取代，其他的保留；群組依 `id` 合併、貼紙不重複；圖檔名和貼紙代碼會先檢查，不接受路徑。

## 版本與轉換

- `store::VERSION`（目前是 **2**）是每張便利貼的資料格式版本，存在 `Note.v`
- 啟動時（`notes::migrate`）和匯入時，`v` 比較小的便利貼會被轉成新格式再存回去
- 規則：
  - **只加欄位、有合理預設值** → 不用提高版本（`serde(default)` 會處理）
  - **改了欄位意義、單位或座標系** → 提高 `VERSION`，在 `migrate` 和 `backup::import` 寫轉換，並在 `CHANGELOG.md` 標註「資料格式 vN」
  - 舊欄位改成只讀（`skip_serializing`），至少保留兩個主要版本再移除
- 歷史（v2 之前的資料一律直接轉成 v2）：
  - `deco`（單一裝飾）轉成 `items`（多個裝飾）
  - 視窗加了陰影邊距，`x`、`y` 往右下補 (28, 44) × 螢幕縮放，讓紙張留在原位
  - `font` 沒有值的補成 1.0

## IPC

其他程式（目前是 Vibotype）用 `WM_COPYDATA` 傳東西給 PinPin：

- 找視窗：`FindWindowW("PinPin.Main", null)`；改名前叫 `PinBoard.Main`，目前還留著一個同名的隱藏分身視窗接同樣的訊息，下一個主要版本移除（Vibotype 已改成先找新名、找不到才找舊名）
- 所有頂層視窗的類別都以 `PinPin.` 開頭：Vibotype 靠這個判斷游標底下是不是我們的視窗（拖便利貼時不跳出選取工具列），新增視窗類別要照這個規則
- `COPYDATASTRUCT.dwData = 0x50424E31`（"PBN1"）
- `lpData`：UTF-8 JSON

```json
{ "cmd": "note", "text": "要記下來的文字", "x": 100, "y": 200 }
{ "cmd": "star", "text": "收藏這段", "source": "chrome.exe" }
```

| 欄位 | 說明 |
|---|---|
| `cmd` | `note`：新增一張便利貼；`star`：收進星號記事本 |
| `text` | 內容 |
| `source` | 選用，來源程式（星號記事本的附註） |
| `x`, `y` | 選用，便利貼放的位置（螢幕座標） |

這是對外的合約：不能改名、不能移除欄位；要擴充就加新的 `cmd` 或選用欄位。
