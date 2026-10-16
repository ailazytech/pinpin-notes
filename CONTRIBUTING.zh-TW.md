# 參與 PinPin

[English](CONTRIBUTING.md) · **繁體中文**

謝謝你願意幫忙！這份文件說明怎麼回報問題、怎麼改程式、PR 要符合哪些條件。用 AI agent 開發的話，先讀 [AGENTS.md](AGENTS.md)（重點摘要＋安全檢查清單）。

參與這個專案就表示同意遵守[行為準則](CODE_OF_CONDUCT.md)。

## 回報問題、提建議

- Bug：用 GitHub 的「Bug 回報」範本，附上 Windows 版本、螢幕縮放比例、重現步驟；有的話附 `%APPDATA%\PinPin\pinpin.log`
- 新功能：先開「功能建議」issue 討論，確定方向再動手，免得白做
- 翻譯：用「翻譯」範本，說明是哪個語言、哪些字
- 安全性問題（例如密碼便利貼的加密、更新下載）：**不要開公開 issue**，照 [SECURITY.md](SECURITY.md) 私下回報

## 分支

| 分支 | 用途 |
|---|---|
| `main` | 已發佈的版本。官網和 App 的線上更新都跟著它。只接受 `dev`（或緊急修正 `hotfix/*`）的發版 PR |
| `dev` | 開發中的整合分支。**PR 一律開到 `dev`** |
| `feat/…`、`fix/…`、`docs/…` | 從 `dev` 開出來的工作分支，一個分支做一件事 |

緊急修正：從 `main` 開 `hotfix/…`，PR 到 `main`，發版後再把 `main` 合回 `dev`。

## 開發環境

| 部分 | 需要 | 指令 |
|---|---|---|
| App | Windows 10/11、Rust 穩定版（MSVC） | `cargo test`、`cargo build --release` |
| 官網 `site/` | Node 22+、pnpm | `pnpm install`、`pnpm exec vp dev`、`pnpm run build` |
| 宣傳影片 `promo/remotion/` | Node 22+ | `npm install`、`npm run render` |

## 程式結構（App）

```
src/main.rs          進入點
src/app.rs           單一執行個體、系統匣、設定、開發用參數、WM_COPYDATA
src/update.rs        線上更新（feed.json、下載後核對 SHA-256）
src/i18n.rs          語言檔載入（locales/*.json 編譯時打包，%APPDATA% 可以覆蓋）
src/theme.rs         選單、對話框的配色和外框
src/notes/mod.rs     便利貼視窗：建立、滑鼠、選單、拖曳
src/notes/render.rs  畫一張便利貼（逐像素的 Canvas，layered window）
src/notes/store.rs   資料格式、讀寫 notes.json（見 docs/DATA.md）
src/notes/assets.rs  內建素材表、解碼快取
src/notes/stickers.rs 去背、白邊、自製貼紙與群組
src/notes/shop.rs    貼紙小舖視窗
src/notes/ink.rs     手寫
src/notes/timer.rs   番茄鐘
src/notes/crypto.rs  密碼便利貼（Windows CNG：PBKDF2 + AES-GCM）
src/notes/backup.rs  匯出／匯入
src/notes/promo.rs, sprites.rs  宣傳影片、網站截圖用的離屏繪圖
```

## 一定要守的規矩

這些是 PinPin 的核心承諾，PR 違反就不會合併：

1. **閒置不耗電**：不准有閒置時的計時器或輪詢。只在畫面真的要變時重畫；動畫用 `SetCoalescableTimer`，播完就關。番茄鐘只在跑的時候有計時器
2. **低記憶體**：圖片依顯示大小解碼後快取，不要把原圖整張留在記憶體
3. **不刪使用者的東西**：空白便利貼不會自動刪除；刪除一律先進垃圾桶（可以復原）
4. **劃掉永遠是紅色**，不跟著文字顏色走
5. **`WM_COPYDATA` 協議不能破壞**（視窗類別 `PinPin.Main`、`dwData = 0x50424E31`、JSON 欄位 `cmd/text/source/x/y`），Vibotype 依賴它。要擴充只能加新的 `cmd` 或選用欄位。新的視窗類別一律以 `PinPin.` 開頭（Vibotype 靠這個辨識我們的視窗）
6. **資料格式向下相容**：舊的 `notes.json` 一定要讀得進來；改格式要提高 `store::VERSION` 並寫轉換（見 [docs/DATA.md](docs/DATA.md#版本與轉換)）
7. **測試不碰使用者的桌面**：不要寫模擬鍵盤滑鼠的測試。畫面用離屏繪圖驗證（`--render-notes`、`--render-shop`），並設 `APPDATA` 到暫存資料夾隔開資料
8. **不提交私人資訊**：程式、文件、圖片、測試資料裡都不能有金鑰、本機路徑、個人名字

## 介面文字與翻譯

- 介面上看得到的字一律走 `t("key")`，不要寫死在程式裡
- 新增 key 時**六個語言檔都要加**（`locales/*.json`）；`cargo test` 裡的 `bundled_files_parse_and_have_same_keys` 會檢查 key 是否一致
- 不會的語言可以先填英文，在 PR 裡註明，請懂的人補
- 官網的字在 `site/src/i18n/locales/*.ts`，以 `en.ts` 為型別來源，其他語言少 key 會編譯失敗

## Commit 訊息和 PR 標題

用 [Conventional Commits](https://www.conventionalcommits.org/zh-hant/)，方便整理更新紀錄。**PR 標題也要符合**，CI 會檢查：

```
<類型>(<範圍>): <說明>

feat(timer): 新增 LED 樣式
fix(shop): 旋轉控制點 hover 時消失
docs: 補資料格式說明
refactor(render): 拆出紙質繪製
chore(site): 升級 Vite+
```

| 類型 | 用在 |
|---|---|
| `feat` | 新功能 |
| `fix` | 修 bug |
| `perf` | 效能 |
| `refactor` | 重構（行為不變） |
| `docs` | 文件 |
| `test` | 測試 |
| `build`、`ci` | 編譯、CI |
| `chore` | 其他雜事、升級套件 |
| `revert` | 還原 |

範圍（括號裡，可省略）用模組名稱：`notes`、`render`、`shop`、`stickers`、`ink`、`timer`、`crypto`、`store`、`update`、`i18n`、`site`、`promo`、`ci`。說明中英文都可以，一行講清楚改了什麼。破壞相容的變更在類型後加 `!`（`feat(store)!: …`）並在內文寫 `BREAKING CHANGE:`。

## 送 PR 之前

- [ ] PR 開到 `dev`，標題符合 Conventional Commits
- [ ] `cargo test` 通過、`cargo build --release` 沒有警告
- [ ] 改到畫面的話，附上離屏繪圖（`--render-notes` / `--render-shop`）前後對照
- [ ] 改到官網的話，`pnpm run build` 通過，附截圖
- [ ] `CHANGELOG.md` 的 `[Unreleased]` 加上一行（純重構、文件、CI 可以不用）
- [ ] 改了資料格式的話，更新 `docs/DATA.md` 並提高 `store::VERSION`
- [ ] 動到輸入解析、檔案、網路、加密或 `unsafe` 的話，跑過 [AGENTS.md 的安全檢查清單](AGENTS.md#security-review-checklist)

## 自動檢查

每個 PR 都會跑：

- **CI**：`cargo test`、`cargo build`、官網型別檢查與建置
- **PR 標題**：Conventional Commits 格式；PR 開到 `main` 的話只接受 `dev` 或 `hotfix/*`
- **安全性**：CodeQL、`cargo audit`（RustSec 漏洞資料庫）、`pnpm audit`、gitleaks（掃金鑰）
- **Dependabot**：每週檢查套件更新，PR 開到 `dev`

## 程式風格

- `cargo fmt`；註解用繁體中文，寫「為什麼」而不是「做了什麼」
- 盡量不加新的 crate；Windows 已經內建的（WIC、CNG、GDI、WinHTTP）優先
- 一個 PR 做一件事
