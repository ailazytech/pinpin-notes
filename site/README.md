# PinPin 官網

[Vite+](https://viteplus.dev)（`vp`）＋ React 19 ＋ Tailwind 4 ＋ framer-motion 的靜態網站，六語系。

```powershell
vp install
vp dev        # 開發
vp build      # 輸出到 dist/
vp preview    # 預覽 dist/
```

## 部署

`base: "./"`＋hash 路由（`#/`、`#/custom`、`#/qa`、`#/donate`），`dist/` 放到任何靜態主機的任何路徑都能用。發版時的步驟見 [docs/RELEASING.md](../docs/RELEASING.md#官網部署)。

## 要改的地方

| 檔案 | 內容 |
|---|---|
| `src/lib/site-config.ts` | 下載連結、GitHub、贊助連結、聯絡信箱、影片和截圖路徑（**空白的連結上線前要填**） |
| `src/i18n/locales/*.ts` | 所有文字；`en.ts` 是型別來源，其他語言少 key 會編譯失敗 |
| `src/components/site/views/custom-view.tsx` | 訂製頁（方案、價格在字典的 `commission`） |
| `public/` | 圖片、貼紙、影片、favicon；路徑一律用 `asset("img/…")`，不要寫 `/img/…` |

## 素材從哪來

| 檔案 | 來源 |
|---|---|
| `public/img/stickers/*.webp` | `assets/make-stickers.ps1` 產生的 `target/stickers-png/*.png` 轉 WebP |
| `public/img/screens/*.webp` | `promo/remotion`：`npm run screens` |
| `public/promo/pinpin-promo.mp4` | `promo/remotion`：`npm run render` |
| `public/favicon.ico`、`icon-*.png`、`apple-touch-icon.png` | `assets/logo/`（`pinpin.exe --make-logo`） |
| `public/og.png` | 宣傳影片 21.5 秒的畫面，1200×630 |

## 截圖檢查

無頭 Chrome 不會跑完 framer-motion 的透明度動畫，截圖時網址加 `?static` 讓動畫直接跳到結束狀態：

```
http://localhost:4173/?lang=zh-TW&static#/custom
```
