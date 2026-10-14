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

   打開草稿，把 `CHANGELOG.md` 這一版的內容貼進說明，確認後發佈。
6. **線上更新**：把版本、執行檔網址、SHA-256（workflow 的 Summary 有）填進 `site/public/feed.json` 的 `app`；重要的修正設 `critical: true`。網站部署之後，開著自動檢查的使用者會在系統匣看到「更新到 vX」（格式和規則見 [UPDATES.md](UPDATES.md)）
7. **官網**
   - `site/src/lib/site-config.ts` 的 `downloadUrl` 指到最新版（可以用 `https://github.com/OWNER/pinpin-notes/releases/latest`，之後就不用每次改）
   - 有新功能的話更新功能介紹、截圖（`promo/remotion` 的 `npm run screens`）、問答
   - `cd site; vp build`，把 `site/dist/` 部署到 Pages

## 官網部署

`site/` 是純靜態網站：`vite.config.ts` 用相對路徑（`base: "./"`）、hash 路由（`#/custom`），所以 `site/dist/` 放在任何靜態主機的任何子路徑都能用，不需要伺服器改寫規則。

```powershell
cd site
vp install
vp build      # 輸出到 site/dist/
vp preview    # 本機預覽
```

部署：把 `site/dist/` 整個資料夾上傳到 Pages（GitHub Pages、Cloudflare Pages、Firebase Hosting 等都可以）。

## 熱修（hotfix）

已發佈的版本有嚴重 bug：

1. 從該版的 tag 開分支：`git switch -c hotfix/0.2.1 v0.2.0`
2. 修好、在 `CHANGELOG.md` 寫 `## [0.2.1]` 的 `修正`
3. 照上面的步驟 2～6 發 `v0.2.1`
4. 把修正合回 `main`
