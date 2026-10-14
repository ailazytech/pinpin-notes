# Privacy

**Your notes never leave your computer.** Notes, pictures, handwriting, passwords and stickers are stored only in `%APPDATA%\PinPin\`. Nothing is uploaded or synced.

## What goes over the network

PinPin makes **at most one request a day**: it fetches `https://pinpin.fast01.net/feed.json` to check for a new version, new sticker packs and shop banners. It waits a minute after start, never retries more than once a day (restarts don't count), and if the site is unreachable it simply carries on offline with what it already has.

Turn it off: tray menu → Updates → *Check for updates automatically*. Then PinPin makes no requests at all (except when you click *Check for updates* yourself).

## Anonymous usage stats

So we know roughly how many people use PinPin and which versions and languages to support, that same daily request carries four values:

| Value | Example | Why |
|---|---|---|
| App version | `0.1.0` | Which versions are still in use |
| UI language | `zh-TW` | Which translations matter |
| Windows build | `19045` | Windows 10 vs 11 |
| Install id | `bb4b962f…` (32 random hex digits) | Count each install once per day |

- The install id is generated randomly on your computer the first time (`%APPDATA%\PinPin\update\install_id`). It is not derived from your hardware, account or files. Delete the file and you get a new one.
- The website records these four values plus the country Cloudflare reports for the request. It does **not** store your IP address, user agent or anything from your notes.
- Data is kept in the project's own Cloudflare account (Workers Analytics Engine, kept for about 3 months) and is only looked at as totals.
- No extra request is made for this, and nothing is sent more than once a day.

Turn it off: tray menu → Updates → *Send anonymous usage stats*. The daily update check then carries no parameters.

## The website

`pinpin.fast01.net` is a static site on Cloudflare Pages. It uses no cookies and no third-party trackers.

---

## 隱私（繁體中文摘要）

- **便利貼的內容永遠不會離開你的電腦**，只存在 `%APPDATA%\PinPin\`。
- 程式**一天最多連線一次**，抓 `feed.json` 檢查新版、新貼紙包；重開不會重算，網站連不上就照常離線使用。系統匣 → 更新 → 關掉「自動檢查更新」就完全不連線。
- **匿名使用統計**：同一個請求順便帶上程式版本、介面語言、Windows 組建號、一組本機隨機產生的安裝碼（和電腦、帳號、檔案無關）。網站只記這四項和 Cloudflare 判斷的國家，不存 IP、不存任何便利貼內容，只看總數。系統匣 → 更新 → 關掉「傳送匿名使用統計」即可。
