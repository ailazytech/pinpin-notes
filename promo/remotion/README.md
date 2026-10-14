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
