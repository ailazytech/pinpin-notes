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
6. **發佈**：按下草稿的「Publish release」。這會觸發 `.github/workflows/site.yml`，自動：
   - 把這版的 exe 放上官網：`https://pinpin.aiverything.app/download/PinPin-Notes-x64.exe`（下載按鈕）和 `…/download/PinPin-Notes-0.2.0-x64.exe`（線上更新用）
   - 把 `feed.json` 的 `app.version`、`url`、`sha256` 換成這一版（`notes`、`critical`、廣告、貼紙包照 repo 裡的）
   - 編譯網站、部署到 Cloudflare Pages

   部署完，開著自動檢查的使用者會在系統匣看到「更新到 vX」（規則見 [UPDATES.md](UPDATES.md)）
7. **官網內容**：有新功能的話更新功能介紹、截圖（`promo/remotion` 的 `npm run screens`）、問答，推上 main 就會自動部署

## 官網部署（Cloudflare Pages）

`site/` 是純靜態網站：`vite.config.ts` 用相對路徑（`base: "./"`）、hash 路由（`#/custom`），不需要伺服器改寫規則。網址 `https://pinpin.aiverything.app`（也寫在 `src/base.rs` 的 `SITE_URL`、`site-config.ts`、`index.html`）。

`.github/workflows/site.yml` 在這些時候部署：推到 main 且改了 `site/`、Release 發佈、在 Actions 頁手動執行。每次部署都會把**最新已發佈 Release** 的 exe 放進 `site/public/download/`（repo 是私人的，GitHub Release 的檔案外人下載不到，所以由官網提供下載）。還沒有任何 Release 時，下載按鈕顯示「準備中」。

第一次設定：

1. Cloudflare → My Profile → API Tokens → Create Token，權限 **Account → Cloudflare Pages → Edit**
2. GitHub repo → Settings → Secrets and variables → Actions，新增
   - `CLOUDFLARE_API_TOKEN`：上一步的 token
   - `CLOUDFLARE_ACCOUNT_ID`：Cloudflare 首頁右側的 Account ID
3. 在 Actions 頁手動跑一次 **Site**（第一次會自動建立 Pages 專案 `pinpin-notes`）
4. Cloudflare → Workers & Pages → pinpin-notes → Custom domains → 加 `pinpin.aiverything.app`（`aiverything.app` 的 DNS 在 Cloudflare 上會自動設好 CNAME）

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
