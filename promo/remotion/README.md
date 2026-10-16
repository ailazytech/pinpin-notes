# PinPin Notes 宣傳影片（Remotion）

概念：**先排好整片桌面，再一鏡到底走過它；每一段換一種工具接力**。鋼筆劃掉待辦、筆畫變成紙膠帶 → 剪刀沿虛線剪出貼紙 → 蠟筆畫愛心、繞住蕃茄鐘 → 時間到，蕃茄的波紋擴散成夜色（圓形轉場）→ 標語一個字一個字蓋章、粉筆劃底線 → 掛鎖跳出、打開，密碼便利貼亂碼解開（聚光燈引導視線）→ 拉遠，整片桌面（從第一格就在）完整現身 → 紅筆在留好的中央寫下字樣。不是每段都用筆接：轉場依內容選（筆跡變膠帶、剪刀、波紋圓形轉場、快速甩鏡、蓋章、掛鎖、聚光燈）。

版面在 `src/Film.tsx` 的 `BOARD`、`P`、`FILLERS`（4000×2250，中央留給字樣）；有字的段落鏡頭把主角放在右側三分之一，字放左邊。

參考 onetake 的原則（只參考做法，沒有用它的程式碼）：段落之間一定有東西延續、鏡頭長短差 4 倍以上（0.2 秒的連切 vs 2.4 秒的靜止）、有刻意的靜止、快速移動有動態模糊、筆尖就是每個變化的「原因」、音效從畫面事件產生並共用一個空間。

## 檔案

| 路徑 | 內容 |
|---|---|
| `src/timeline.json` | 唯一的時間表：每一段的秒數、音效事件、音樂的安靜段和夜晚段（畫面和聲音都讀它） |
| `src/Film.tsx` | 整支片：世界座標的版面、運鏡（關鍵格＋手持飄動＋落地震動）、紅線的每一段、文字、連切、結尾 |
| `scripts/sound.mjs` | 合成音效和配樂 → `public/audio/score.wav`（全部自己合成，沒有版權問題） |
| `scripts/stills.mjs` | 挑幾個時間點輸出靜態畫面檢查 |
| `public/sprites/` | PinPin 真正的畫面（透明 PNG，3 倍）＋ `sprites.json`（位置、文字行座標） |
| `public/wall-*.png` | 桌布（AI 生成） |

## 步驟

```bash
# 1. 從 PinPin 匯出素材（改了便利貼的樣子之後要重跑）
pinpin.exe --export-sprites promo/remotion/public/sprites promo/corgi.png

# 2. 聲音（改了 timeline.json 的事件之後要重跑）
npm run sfx

# 3. 邊看邊調
npm run studio

# 4. 檢查幾格
node scripts/stills.mjs 1.4 7.0 15.6

# 5. 輸出（1080p30，含動態模糊；約十幾分鐘）
npm run render
```

改時間：只改 `src/timeline.json`，畫面和聲音會一起跟著動。

Remotion 的授權：個人和 3 人以下的公司免費。

## 特點短片（沒有字幕）

`src/Clips.tsx`：五支各 9～13 秒，節奏跟官網那支宣傳片一樣。鏡頭不跟著游標跑：拿東西時推近到介面（小舖、相片視窗），拖出來貼上時拉遠到整個桌面，放大或轉角度時再靠近一點；東西落下時震一下。滑鼠游標是畫上去的。

| 合成 | 內容 |
|---|---|
| `clip-polaroid` | 從相片視窗把寵物照片拖到桌面 → 變成拍立得，拖轉角度鈕轉一下 |
| `clip-stickers` | 從貼紙小舖拖貼紙到便利貼和桌面，拉控制點放大、轉角度，再連續丟四張 |
| `clip-paste` | 把任何圖片（動漫角色、偶像照）丟進小舖 → 自動去背加白邊 → 拖到桌面，再把偶像貼紙放大、轉角度 |
| `clip-lucky` | 春聯分頁往下捲，拖出旺、日進斗金、天官賜福、金榜題名（直立），最後把日進斗金放大、轉角度 |
| `clip-timer` | 六種番茄鐘依序跳出、逐一特寫，番茄倒數 3、2、1，時間到響鈴、波紋 |

- 時間表：`src/clips.json`（每支的秒數、動作時間 `T`、音效事件 `events`、配樂）——畫面和聲音都讀它
- 聲音分兩種：`samples` 列的種類（點擊、咻、翻頁、快門）是錄音，CC0，來自 `@remotion/sfx`（freesound、kenney.nl），放在 `public/sfx/`，由 Remotion 照時間播；其他（啵、閃亮、滴答、鈴、叮）和木琴配樂由 `scripts/clip-sounds.mjs` 合成
- 原圖在 `promo/clips-src/`，都是 AI 生圖：虛構的寵物、原創動漫角色、虛構偶像（不是真實人物，也不是現有角色）
- 畫面比例假裝是 1080p、125% 縮放的螢幕

```bash
# 1. 素材（約 200 MB，不進 git）：拍立得每 1° 一張、桌面貼紙和選取狀態、番茄鐘、貼紙小舖（PINPIN_DPI=2）
bash scripts/clips-assets.sh            # 預設用 ../../target/release/pinpin.exe

# 2. 聲音（改了 clips.json 之後要重跑）→ public/audio/clip-*.wav
node scripts/clip-sounds.mjs

# 3. 輸出
for id in polaroid stickers paste lucky timer; do npx remotion render clip-$id out/clips/pinpin-$id.mp4; done
```

小舖卡片、控制點的位置寫死在 `Clips.tsx`（`tile(...)`、`shopPt(...)`、`BOX`）；小舖版面或貼紙順序改了要跟著調。

## 官網分享圖（Open Graph）

`node scripts/og.mjs`：用宣傳片片尾（`Film.tsx` 的 `EndStill`，合成 `og-<語言>`）輸出六種語言的分享圖到 `site/public/og/<語言>.png`（1200×630），英文版另存一份 `site/public/og.png`。片尾的標語在 `Film.tsx` 的 `END_TEXT`。
