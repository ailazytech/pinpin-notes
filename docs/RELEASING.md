# 發版流程

## 版本號（語意化版本 MAJOR.MINOR.PATCH）

| 改了什麼 | 提高 | 例子 |
|---|---|---|
| 只修 bug、翻譯、效能 | PATCH | 0.1.0 → 0.1.1 |
| 新功能、新樣式、新語言（舊資料照樣能用） | MINOR | 0.1.1 → 0.2.0 |
| 舊資料要轉換才能用、拿掉功能、改 IPC 協議 | MAJOR | 0.9.0 → 1.0.0 |

1.0.0 之前，破壞相容的變更也可以只提高 MINOR，但更新紀錄要寫清楚。

## 分支

| 分支 | 內容 | 誰會用到 |
|---|---|---|
| `main` | 已發佈的版本 | Cloudflare 的正式網站、tag、使用者下載和線上更新 |
| `dev` | 開發中，所有 PR 合進這裡 | 貢獻者、Dependabot |
| `hotfix/…` | 從 `main` 開的緊急修正 | 只有維護者 |

- `main` 有分支保護：只能用 PR 合進來，CI 要過；`pr-checks.yml` 只讓 `dev` 和 `hotfix/*` 開 PR 到 `main`
- tag 一律打在 `main` 上，`release.yml` 會檢查，不在 `main` 上就不編

## 平常

- 每個 PR 開到 `dev`，在 `CHANGELOG.md` 的 `[Unreleased]` 加一行（見 [CONTRIBUTING.zh-TW.md](../CONTRIBUTING.zh-TW.md)）
- `dev` 和 `main` 隨時都要能編譯、測試通過

## 發版步驟

以發佈 `0.2.0` 為例：

1. **確認**（在 `dev` 上）
   - `cargo test` 通過
   - `cargo build --release` 沒有警告
   - 用隔離的資料（`$env:APPDATA = 暫存資料夾`）跑過 `--render-notes`、`--render-shop`，畫面正常
   - 拿上一版的 `notes.json` 和匯出檔試開，資料都在（尤其改過 `store.rs` 時）
   - GitHub 的 Security 分頁沒有未處理的警告（CodeQL、Dependabot、金鑰掃描）
2. **改版本號**：`Cargo.toml` 的 `version = "0.2.0"`
3. **整理更新紀錄**：`CHANGELOG.md`
   - `## [Unreleased]` 底下的內容移到新的 `## [0.2.0] - 2026-11-01`
   - 留一個空的 `## [Unreleased]`
   - 更新檔案最下面的連結：
     ```
     [Unreleased]: https://github.com/ailazytech/pinpin-notes/compare/v0.2.0...HEAD
     [0.2.0]: https://github.com/ailazytech/pinpin-notes/compare/v0.1.0...v0.2.0
     ```
   - 重要的修正要讓使用者自動下載的話，`site/public/feed.json` 的 `app.critical` 設成 `true`
4. **發版 PR**：commit `chore(release): v0.2.0` 推到 `dev`，開 PR `dev → main`，標題 `chore(release): v0.2.0`。CI 過了用 **merge commit** 合併（不要 squash，`dev` 和 `main` 的歷史才接得上）
5. **在 main 打 tag**
   ```powershell
   git switch main
   git pull
   git tag -a v0.2.0 -m "PinPin Notes 0.2.0"
   git push origin v0.2.0
   ```
6. **GitHub Release**：推 `v*` tag 會觸發 `.github/workflows/release.yml`，自動編譯並建立草稿 Release，附上：
   - `PinPin-Notes-0.2.0-x64.exe` 和 `.sha256`（線上更新用，固定版本）
   - `PinPin-Notes-x64.exe`（官網下載按鈕用，`releases/latest/download/…` 永遠指到最新版）

   打開草稿，把 `CHANGELOG.md` 這一版的內容貼進說明（`gh release edit v0.2.0 --notes-file …` 也可以）
7. **發佈**：按下草稿的「Publish release」。`.github/workflows/site.yml` 透過部署掛鉤叫 Cloudflare 重新建置官網，建置時 `site/scripts/fetch-release.mjs` 自動：
   - 下載按鈕指到 `https://github.com/ailazytech/pinpin-notes/releases/latest/download/PinPin-Notes-x64.exe`
   - `feed.json` 的 `app.version`、`url`、`sha256` 換成這一版（`url` 指到 GitHub 上帶版本號的檔案；`notes`、`critical`、廣告、貼紙包照 repo 裡的）

   部署完，開著自動檢查的使用者會在系統匣看到「更新到 vX」（規則見 [UPDATES.md](UPDATES.md)）
8. **合回 dev**：`git switch dev; git merge main; git push`（讓 `dev` 帶著發版 commit）
9. **官網內容**：有新功能的話更新功能介紹、截圖（`promo/remotion` 的 `npm run screens`）、問答，照一般 PR 流程走，下次發版一起上

## 官網部署（Cloudflare Pages）

`site/` 是純靜態網站：`vite.config.ts` 用相對路徑（`base: "./"`）、hash 路由（`#/custom`），不需要伺服器改寫規則。網址 `https://pinpin.fast01.net`（也寫在 `src/base.rs` 的 `SITE_URL`、`site-config.ts`、`index.html`）。

用 Cloudflare Pages 的 **Git 整合**：`main` 是正式網站，推到 `main` 就由 Cloudflare 自己建置、部署；其他分支（例如 `dev`）會有預覽網址。每次建置前，`site/scripts/fetch-release.mjs` 會讀 GitHub 上**最新已發佈的 Release**，把下載按鈕和 `feed.json` 指過去（repo 是公開的，檔案直接從 GitHub 下載，下載次數也算在 GitHub 上）。還沒有 Release 就跳過，下載按鈕顯示「準備中」。發佈 Release 不會推 main，所以由 `site.yml` 呼叫部署掛鉤補一次建置。

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
| 環境變數 | `NODE_VERSION` = `22`；`GITHUB_TOKEN`（選用，只是提高 GitHub API 的次數上限） |

1. 建好後：pinpin-notes → **自訂網域** → 加 `pinpin.fast01.net`（`fast01.net` 的 DNS 在 Cloudflare 上會自動設 CNAME；不在的話到 DNS 商加 CNAME 指向 `pinpin-notes.pages.dev`）。**App 的線上更新讀的是這個網域的 `feed.json`，沒設好就收不到更新**
2. pinpin-notes → 設定 → 組建 → **部署掛鉤** → 新增（分支 `main`），把網址存成 GitHub repo 的 Actions secret `CLOUDFLARE_DEPLOY_HOOK`
3. 使用統計（選用）：
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

圖示：`public/` 的 `favicon.ico`（16～256 多尺寸）、`favicon-32.png`、`icon-192.png`、`icon-512.png`（PWA）、`apple-touch-icon.png`（iOS 主畫面，要不透明底，用 `scripts/apple-touch-icon.ps1` 從 icon-512 產生）。

社群分享預覽：`public/og/<語言>.png`（1200×630，六種語言各一張；`og.png` 是英文版的備份），用 `promo/remotion` 的 `node scripts/og.mjs` 從宣傳片片尾產生。`functions/_middleware.ts` 依 `?lang=` → 網站上選過的語言（cookie）→ 瀏覽器語言，在伺服器端換掉首頁的標題、描述、Open Graph／Twitter 標籤、canonical 和 hreflang，網址用請求本身的網域。要分享特定語言就用 `https://pinpin.fast01.net/?lang=ja` 這種網址。

## 熱修（hotfix）

已發佈的版本有嚴重 bug：

1. 從 `main` 開分支：`git switch -c hotfix/0.2.1 main`
2. 修好、在 `CHANGELOG.md` 寫 `## [0.2.1]` 的 `修正`，`Cargo.toml` 改成 `0.2.1`
3. PR `hotfix/0.2.1 → main`，合併後照上面的步驟 5～7 發 `v0.2.1`
4. 把 `main` 合回 `dev`

## GitHub repo 設定（第一次公開時做一次）

- Settings → General：預設分支 `main`；Pull Requests 允許 merge commit 和 squash；勾「Automatically delete head branches」
- Settings → Branches：`main` 需要 PR、需要 CI（`App (cargo test)`、`Site (vp build)`）通過
- Settings → Code security：Dependabot alerts、Dependabot security updates、Secret scanning、Push protection、Private vulnerability reporting 全開
- About：描述、網站 `https://pinpin.fast01.net`、topics（`windows`、`sticky-notes`、`rust`、`win32`、`desktop-app`、`pomodoro`）
