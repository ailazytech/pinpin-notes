## 改了什麼

<!-- 一兩句話；關聯的 issue 寫 Closes #123 -->

## 畫面

<!-- 改到畫面的話，附 --render-notes / --render-shop 或網站的前後對照 -->

## 檢查

- [ ] `cargo test` 通過、`cargo build --release` 沒有警告
- [ ] 閒置時沒有新增計時器或輪詢
- [ ] 新的介面文字六個語言檔都有 key
- [ ] `CHANGELOG.md` 的 `[Unreleased]` 加了一行（純重構、文件可以不用）
- [ ] 改了資料格式的話：提高 `store::VERSION`、寫了轉換、更新 `docs/DATA.md`
- [ ] 沒有動到 `WM_COPYDATA` 協議（或只加了新的 `cmd`／選用欄位）
