# 線上更新：程式、廣告、貼紙小舖

一份 `feed.json` 控制三件事：

| 內容 | 怎麼到使用者手上 | 使用者能關嗎 |
|---|---|---|
| **程式本身** | 一般更新：系統匣選單出現「⬆ 更新到 vX」，使用者點了才下載、換檔、重新啟動。<br>重要更新（`critical` 或低於 `min_version`）：自動下載，**下次啟動時套用**，並跳一次提示 | 「自動檢查更新」關掉 = 不會自己去查；「接收推送」關掉 = 重要更新也不會自動下載 |
| **貼紙小舖的廣告** | 自動下載底圖，期間內的線上廣告取代內建的三則 | 「接收推送」關掉 = 只顯示內建廣告 |
| **貼紙包** | 自動下載，出現在小舖（加進現有分類或新的分類） | 「接收推送」關掉 = 線上貼紙包不顯示（檔案留著，打開就回來） |

程式碼在 `src/update.rs`；貼紙包的載入在 `src/notes/assets.rs`（`load_packs`），廣告在 `src/notes/shop.rs`（`ads()`）。

## 檢查時機（低耗能）

自動檢查**一天最多一次**：以「上次嘗試」（成功、失敗都算，記在 `last_check`）起算 24 小時，重開程式不會重算，網站連不上也不會一直重試。

1. 程式啟動時排**一個**計時器，等到下次該查的時間（已經到期就等 1 分鐘）；查完再排下一次。程式一直開著也是一天一次
2. 打開**貼紙小舖**時，到期了才查
3. 使用者按系統匣「更新 → 檢查更新…」（隨時可以按，也會更新 `last_check`）

網站掛了或離線：背景安靜失敗、寫進 log，程式照常用上次存下來的 `feed.json`（廣告、貼紙包都還在），不影響任何功能。

開著「傳送匿名使用統計」時，這個請求的網址會帶 `?v=版本&l=語言&os=Windows組建號&id=安裝碼`，不另外送請求（見 [PRIVACY.md](PRIVACY.md)）。

網路在背景執行緒（Windows 內建的 WinHTTP，不加任何套件），做完 `PostMessage(WM_UPDATE)` 交回主執行緒。

## 提示規則

| 情況 | 自動檢查 | 手動檢查 |
|---|---|---|
| 有一般新版 | 不打擾，只在系統匣選單加「⬆ 更新到 vX」 | 提示「有新版本 vX」 |
| 有重要更新（且開著接收推送） | 自動下載完提示「重要更新已下載，下次啟動時套用」 | 同左 |
| 已經是最新 | 不提示 | 提示「已經是最新版」 |
| 失敗（沒網路等） | 不提示，寫進 log | 提示「檢查更新失敗」 |

## 安全

- **只走 HTTPS**（開發時允許 `http://localhost`）
- feed 裡每個檔案都附 **SHA-256**，下載後比對不符就丟掉
- 貼紙包的貼紙代碼一定要 `pk-` 開頭、只能是小寫英數和 `-`，不會蓋掉內建素材，也不能寫到資料夾外面
- 換執行檔：把正在跑的 `pinpin.exe` 改名成 `pinpin.exe.old`，新版放回原位，帶 `--updated` 啟動；新版等舊版結束（最多 10 秒）、刪掉 `.old`。換檔失敗會改回原本的檔名
- 之後可以加強：Authenticode 簽章、feed 用 ECDSA 簽名（Windows CNG 內建就能驗）

## 資料夾

`%APPDATA%\PinPin\update\`

| 檔案 | 內容 |
|---|---|
| `feed.json` | 最後一次抓到的 feed（離線時廣告、貼紙包照樣能用） |
| `last_check` | 上次嘗試檢查的時間（Unix 秒數），自動檢查以它起算 24 小時 |
| `install_id` | 匿名使用統計的隨機安裝碼（32 個十六進位字元），刪掉就換一個 |
| `ads/<id>.<ext>` | 廣告底圖 |
| `packs/<id>/pack.json`＋圖檔 | 已安裝的貼紙包（`pack.json` 最後才寫，沒有它就表示沒裝完） |
| `app/pinpin-<版本>.exe`、`ready.json` | 下載好、等著套用的新版 |

## feed.json 格式

放在官網（`site/public/feed.json`，跟網站一起部署），網址在 `src/base.rs`：`FEED_URL`（空的 = `SITE_URL/feed.json`）。

```json
{
  "schema": 1,
  "app": {
    "version": "0.2.0",
    "url": "https://github.com/ailazytech/pinpin-notes/releases/download/v0.2.0/PinPin-Notes-0.2.0-x64.exe",
    "sha256": "（64 個小寫十六進位）",
    "critical": false,
    "min_version": "",
    "notes": { "zh-TW": "新增台灣小物", "en": "New Taiwan stickers" }
  },
  "ads": [
    {
      "id": "cny-2027",
      "image": "https://example.com/ads/cny-2027.png",
      "sha256": "…",
      "title": { "zh-TW": "✦ 春聯訂製開跑", "en": "✦ Couplet orders are open" },
      "sub": { "zh-TW": "金字毛筆、全球寄送，點這裡下單 →", "en": "Gold calligraphy, shipped worldwide →" },
      "link": "site:#/order",
      "from": "2027-01-01",
      "until": "2027-02-10"
    }
  ],
  "packs": [
    {
      "id": "xmas-2026",
      "version": 1,
      "min_app": "0.2.0",
      "sub": "sub.xmas",
      "sub_names": { "zh-TW": "聖誕節", "en": "Christmas" },
      "items": [
        { "id": "pk-xmas-tree", "url": "https://…/pk-xmas-tree.png", "sha256": "…", "size": 75, "names": { "zh-TW": "聖誕樹", "en": "Christmas tree" } }
      ]
    }
  ]
}
```

- **多語文字**（`title`、`sub`、`names`、`notes`、`sub_names`）：語言代碼 → 文字；挑目前語言 → 同語系 → 英文 → 隨便一個
- **廣告**
  - 底圖建議 **1040×200**，左半邊留給程式畫的文字（程式會在左邊蓋一層淡白）
  - `link`：`site:#/order`（官網某頁）、`tab:sub.spring`（切到小舖分類）、`https://…`
  - `from`／`until` 是 UTC 日期；有任何一則有效的線上廣告，就整組取代內建的三則
- **貼紙包**
  - `sub` 用內建的 key（`sub.daily`、`sub.lucky`、`sub.charms`、`sub.plants`、`sub.spring`、`sub.pets`、`sub.scooters`、`sub.taiwan`）就加進那一區；用新的 key 要附 `sub_names`，會多一個分類
  - 圖檔要是顯示大小（`size`，邏輯像素）的 **4 倍**，PNG／JPEG XR，透明背景、白邊自己做好
  - 改了內容就把 `version` 加一，使用者會重新下載
  - feed 拿掉某個包不會刪掉使用者已經裝的（便利貼上已經貼著的不會不見）

## 發版時要做的事

見 [RELEASING.md](RELEASING.md)：Release 的 workflow 會印出執行檔的 SHA-256，把版本、網址、SHA-256 填進 `site/public/feed.json`，網站重新部署就生效。

- 一般更新：`critical: false`
- 修了安全性或資料遺失的 bug：`critical: true`，或把 `min_version` 設成修好的版本
- 只更新廣告或貼紙包：改 `feed.json`、放好圖檔、部署網站即可，不用發新版

## 開發與測試

| 參數 | 用途 |
|---|---|
| `PINPIN_FEED=網址` | 改用別的 feed（例如本機 `vp preview` 的 `http://localhost:4318/_test/feed.json`） |
| `--check-update [--push]` | 前景同步一次並印出結果（不開視窗） |
| `--apply-update` | 套用已下載的更新（測試換檔） |
| `--render-shop` | 看廣告和貼紙包在小舖裡的樣子 |

測試一律搭配 `APPDATA=<暫存資料夾>`，並在暫存資料夾裡的執行檔副本上測換檔；更新後的新版如果等不到舊版結束（例如你平常用的那份正開著），會安靜結束、不會打擾正在跑的那份。
