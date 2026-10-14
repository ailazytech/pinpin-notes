# 發版流程

## 版本號（語意化版本 MAJOR.MINOR.PATCH）

| 改了什麼 | 提高 | 例子 |
|---|---|---|
| 只修 bug、翻譯、效能 | PATCH | 0.1.0 → 0.1.1 |
| 新功能、新樣式、新語言（舊資料照樣能用） | MINOR | 0.1.1 → 0.2.0 |
| 舊資料要轉換才能用、拿掉功能、改 IPC 協議 | MAJOR | 0.9.0 → 1.0.0 |

1.0.0 之前，破壞相容的變更也可以只提高 MINOR，但更新紀錄要寫清楚。

## 平常

- 每個 PR 在 `CHANGELOG.md` 的 `[Unreleased]` 加一行（見 [CONTRIBUTING.md](../CONTRIBUTING.md)）
- `main` 隨時都要能編譯、測試通過

## 發版步驟

以發佈 `0.2.0` 為例：

1. **確認**
   - `cargo test` 通過
   - `cargo build --release` 沒有警告
   - 用隔離的資料（`$env:APPDATA = 暫存資料夾`）跑過 `--render-notes`、`--render-shop`，畫面正常
   - 拿上一版的 `notes.json` 和匯出檔試開，資料都在（尤其改過 `store.rs` 時）
2. **改版本號**：`Cargo.toml` 的 `version = "0.2.0"`
3. **整理更新紀錄**：`CHANGELOG.md`
   - `## [Unreleased]` 底下的內容移到新的 `## [0.2.0] - 2026-11-01`
   - 留一個空的 `## [Unreleased]`
   - 更新檔案最下面的比較連結：
     ```
     [Unreleased]: https://github.com/OWNER/pinpin-notes/compare/v0.2.0...HEAD
     [0.2.0]: https://github.com/OWNER/pinpin-notes/compare/v0.1.0...v0.2.0
     ```
4. **Commit 和 tag**
   ```powershell
   git commit -am "chore(release): v0.2.0"
   git tag -a v0.2.0 -m "PinPin Notes 0.2.0"
   git push origin main --tags
   ```
5. **GitHub Release**：推 `v*` tag 會觸發 `.github/workflows/release.yml`，自動編譯並建立草稿 Release，附上：
   - `PinPin-Notes-0.2.0-x64.exe`
   - `PinPin-Notes-0.2.0-x64.exe.sha256`

   打開草稿，把 `CHANGELOG.md` 這一版的內容貼進說明。重要的修正要讓使用者自動更新的話，先在 `site/public/feed.json` 把 `app.critical` 設成 `true` 並推上 main。
6. **發佈**：按下草稿的「Publish release」。`.github/workflows/site.yml` 會透過部署掛鉤叫 Cloudflare 重新建置官網，建置時 `site/scripts/fetch-release.mjs` 自動：
   - 把這版的 exe 放上官網：`https://pinpin.fast01.net/download/PinPin-Notes-x64.exe`（下載按鈕）和 `…/download/PinPin-Notes-0.2.0-x64.exe`（線上更新用）
   - 把 `feed.json` 的 `app.version`、`url`、`sha256` 換成這一版（`notes`、`critical`、廣告、貼紙包照 repo 裡的）

   部署完，開著自動檢查的使用者會在系統匣看到「更新到 vX」（規則見 [UPDATES.md](UPDATES.md)）
7. **官網內容**：有新功能的話更新功能介紹、截圖（`promo/remotion` 的 `npm run screens`）、問答，推上 main 就會自動部署

## 官網部署（Cloudflare Pages）

`site/` 是純靜態網站：`vite.config.ts` 用相對路徑（`base: "./"`）、hash 路由（`#/custom`），不需要伺服器改寫規則。網址 `https://pinpin.fast01.net`（也寫在 `src/base.rs` 的 `SITE_URL`、`site-config.ts`、`index.html`）。

用 Cloudflare Pages 的 **Git 整合**：推到 `main` 就由 Cloudflare 自己建置、部署。每次建置前，`site/scripts/fetch-release.mjs` 會把**最新已發佈 Release** 的 exe 放進 `site/public/download/`（repo 是私人的，GitHub Release 的檔案外人下載不到，所以由官網提供下載）。還沒有 Release、或沒設 `GITHUB_TOKEN` 時就跳過，下載按鈕顯示「準備中」。發佈 Release 不會推 main，所以由 `site.yml` 呼叫部署掛鉤補一次建置。

`site/functions/` 是 Pages Functions：`feed.json.ts` 記匿名使用統計（見 [PRIVACY.md](PRIVACY.md)），`api/stats.ts` 給統計頁 `/stats.html` 用。`site/wrangler.toml` 設定輸出資料夾和 Analytics Engine 的綁定。

第一次設定（Cloudflare → Workers & Pages → 建立 → Pages → 連接到 Git → 選 `ailazytech/pinpin-notes`）：

| 欄位 | 值 |
|---|---|
| 專案名稱 | `pinpin-notes` |
| 生產分支 | `main` |
| Framework 預設 | 無 |
| 組建命令 | `node scripts/fetch-release.mjs && pnpm run build`（Cloudflare 看到 `pnpm-lock.yaml` 會先自己 `pnpm install`） |
| 組建輸出目錄 | `dist` |
| 根目錄 → 路徑 | `site` |
| 環境變數 | `NODE_VERSION` = `22`；`GITHUB_TOKEN` = 下面第 1 步的 token（按「加密」） |

1. GitHub → Settings → Developer settings → **Fine-grained tokens** → 新增：Resource owner `ailazytech`、只選 `pinpin-notes`、權限 **Contents: Read-only**。填進上表的 `GITHUB_TOKEN`
2. 建好後：pinpin-notes → **自訂網域** → 加 `pinpin.fast01.net`（`fast01.net` 的 DNS 在 Cloudflare 上會自動設 CNAME；不在的話到 DNS 商加 CNAME 指向 `pinpin-notes.pages.dev`）
3. pinpin-notes → 設定 → 組建 → **部署掛鉤** → 新增（分支 `main`），把網址存成 GitHub repo 的 Actions secret `CLOUDFLARE_DEPLOY_HOOK`
4. 使用統計（選用）：
   - Workers & Pages → Analytics Engine 第一次用要按啟用；`wrangler.toml` 的綁定會自動接上資料集 `pinpin_usage`
   - My Profile → API Tokens → 建一個權限 **Account Analytics: Read** 的 token
   - pinpin-notes → 設定 → 變數和祕密，新增（加密）：`STATS_KEY`（自己訂一串長密碼）、`CF_ACCOUNT_ID`、`CF_STATS_TOKEN`（上面的 token）
   - 打開 `https://pinpin.fast01.net/stats.html`，輸入 `STATS_KEY` 就看得到每日、7 日、30 日不重複使用者和版本、語言、Windows、國家分布

本機預覽：

```powershell
cd site
pnpm install
pnpm run build    # 輸出到 site/dist/（npm 會因 packageManager 設定報錯，要用 pnpm）
pnpm exec vp preview
```

圖示：`public/` 的 `favicon.ico`（16～256 多尺寸）、`favicon-32.png`、`icon-192.png`、`icon-512.png`（PWA）、`apple-touch-icon.png`（iOS 主畫面，要不透明底，用 `scripts/apple-touch-icon.ps1` 從 icon-512 產生）、`og.png`（社群分享預覽 1200×630）。

## 熱修（hotfix）

已發佈的版本有嚴重 bug：

1. 從該版的 tag 開分支：`git switch -c hotfix/0.2.1 v0.2.0`
2. 修好、在 `CHANGELOG.md` 寫 `## [0.2.1]` 的 `修正`
3. 照上面的步驟 2～6 發 `v0.2.1`
4. 把修正合回 `main`
