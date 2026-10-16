<!--
PR title: Conventional Commits, e.g. `feat(timer): add the LED style` / `fix(shop): 旋轉控制點 hover 時消失`
Target branch: `dev` (only release PRs go from dev into main).
PR 標題：Conventional Commits 格式。PR 請開到 `dev`（發版 PR 才從 dev 開到 main）。
-->

## What changed · 改了什麼

<!-- One or two sentences. Linked issue: Closes #123 · 一兩句話；關聯的 issue 寫 Closes #123 -->

## Screenshots · 畫面

<!-- Visual change: --render-notes / --render-shop or the website, before and after · 改到畫面的話附前後對照 -->

## Checklist · 檢查

- [ ] Targets `dev`, title follows Conventional Commits · 開到 `dev`，標題符合格式
- [ ] `cargo test` passes, `cargo build --release` has no warnings · 測試通過、編譯沒有警告
- [ ] No new idle timers or polling · 閒置時沒有新增計時器或輪詢
- [ ] New UI strings exist in all six languages · 新的介面文字六個語言都有
- [ ] `CHANGELOG.md` `[Unreleased]` updated (not needed for refactors / docs) · 更新紀錄加了一行（純重構、文件可以不用）
- [ ] Data format changed → `store::VERSION` bumped, migration written, `docs/DATA.md` updated · 改了資料格式：提高版本、寫轉換、更新文件
- [ ] `WM_COPYDATA` protocol untouched (or only a new `cmd` / optional field) · 沒有破壞 `WM_COPYDATA` 協議
- [ ] Touched input parsing, files, network, crypto or `unsafe` → went through the [security checklist](https://github.com/ailazytech/pinpin-notes/blob/dev/AGENTS.md#security-review-checklist) · 動到這些的話跑過安全檢查清單
- [ ] No secrets, local paths or personal names in the diff · 沒有金鑰、本機路徑、個人名字
