# 語言檔 / Language files

PinPin Notes（貼貼）的介面文字都在這裡，每個語言一個 JSON 檔。
All UI text lives here, one JSON file per language.

## 自己加一個語言（不用重新編譯）/ Add a language without rebuilding

1. 系統匣 → 語言 → 「開啟自訂語言檔資料夾…」（`%APPDATA%\PinPin\locales\`）
   Tray → Language → "Open custom language folder…"
2. 複製裡面的 `_template-en.json.txt`，改名成例如 `fr.json`
   Copy `_template-en.json.txt` to e.g. `fr.json`
3. 改 `_meta`，再把右邊的字串翻譯掉（左邊的 key 不要動）
   Edit `_meta` and translate the values (keep the keys)
4. 重開選單就會看到新語言 / It appears in the Language menu right away

```json
{
  "_meta": { "code": "fr", "name": "Français", "rtl": false, "author": "you" },
  "tray.new": "Nouvelle note",
  "…": "…"
}
```

- `code`：語言代碼（BCP 47，例如 `fr`、`pt-BR`、`he`）
- `name`：選單裡顯示的名稱，用該語言自己的寫法
- `rtl`：由右到左的語言（阿拉伯文、希伯來文、波斯文…）設 `true`，選單和貼紙小舖會左右鏡像
- 沒翻到的 key 會自動用英文
- `{count}` 這種大括號是數字之類的佔位，保留原樣
- `\t` 後面是快捷鍵提示（選單靠右顯示）
- `decoys` 是有密碼的便利貼平常顯示的「偽裝內容」，每一份用 `\n` 換行，寫得越像普通待辦越好
- 同一個 `code` 的外部檔會**覆蓋**內建語言，只想改幾個字也可以只寫那幾個 key

## 貢獻到專案 / Contributing

把檔案放進這個資料夾，並在 `src/i18n.rs` 的 `BUNDLED` 加一行，就會編進執行檔。
`cargo test` 會檢查每個內建語言是不是都有英文版的所有 key。

Put the file here and add it to `BUNDLED` in `src/i18n.rs`. `cargo test` checks that every bundled language has all the keys of `en.json`.

便利貼的**內容**不管介面語言，每一行會自動判斷方向（阿拉伯文、希伯來文的行靠右）。
Note **content** direction is detected per line, independent of the UI language.
