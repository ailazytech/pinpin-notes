# PinPin Notes（貼貼）

<img src="assets/logo/pinpin-256.png" width="112" align="right" alt="PinPin Notes">

[English](README.md) · **繁體中文** · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [العربية](README.ar.md)

[![GitHub stars](https://img.shields.io/github/stars/ailazytech/pinpin-notes?style=flat&logo=github&label=Stars)](https://github.com/ailazytech/pinpin-notes/stargazers) [![GitHub watchers](https://img.shields.io/github/watchers/ailazytech/pinpin-notes?style=flat&logo=github&label=Watchers)](https://github.com/ailazytech/pinpin-notes/watchers) [![GitHub followers](https://img.shields.io/github/followers/ailazytech?style=flat&logo=github&label=Followers)](https://github.com/ailazytech) [![Latest release](https://img.shields.io/github/v/release/ailazytech/pinpin-notes?style=flat&label=Release)](https://pinpin.fast01.net) [![CI](https://github.com/ailazytech/pinpin-notes/actions/workflows/ci.yml/badge.svg)](https://github.com/ailazytech/pinpin-notes/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE) [![Windows 10 | 11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)](https://pinpin.fast01.net) [![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> **桌面上的小紙條，一直陪著你。**

Windows 桌面便利貼：圖釘、紙膠帶、貼紙拖來拖去，手寫、透明標語、蕃茄鐘，平常幾乎不耗電。

- 單一 `.exe`（約 11 MB）、免安裝，Windows 10 / 11
- 閒置 0% CPU、約 15 MB 記憶體（沒有任何閒置計時器，只在畫面變動時重畫）
- 六種語言：English、繁中、简中、日本語、한국어、العربية（由右到左）
- 開源（MIT），資料只存在自己的電腦

🌐 官網：**[pinpin.fast01.net](https://pinpin.fast01.net)**　📦 下載：[官網](https://pinpin.fast01.net)（準備中）　📝 [更新紀錄](CHANGELOG.md)

## 功能

- **便利貼**：開機自動放回桌面；拖曳移動，靠近別張或螢幕邊緣會輕輕吸住對齊（Alt 拖曳不吸，方向鍵 1px 微調）
- **劃掉**：滑過時每行左邊出現圓圈，點一下用紅筆劃掉（有劃線動畫）
- **圖釘、夾子、紙膠帶、貼紙**：從「貼紙小舖」拖到便利貼或桌面上；貼紙 146 張（4 倍解析度，放大不糊）分成日常、招財祝福、各國招財、盆栽、春聯、貓狗、小機車、台灣小物八區；點一下出現控制點，右上角旋轉、右下角縮放；紙膠帶拖兩端可以拉長、轉方向。圖釘、夾子、紙膠帶沒選取時，拖它等於拖整張便利貼
- **自製貼紙**：在貼紙小舖開自己的群組，把圖片拖進來、Ctrl+V 或點「＋」，自動去背、加圓弧白邊；保留原圖尺寸（最多 4096px），用 JPEG XR 存檔省空間。內建貼紙也可以收進群組
- **訂製**：貼紙小舖最後一張卡片連到官網的訂製頁（寵物貼紙、計時器角色、主題組）
- **手寫**：鋼筆、螢光筆、粉筆、蠟筆，8 種顏色、3 種粗細，橡皮擦、Ctrl+Z 復原；**透明紙**可以當浮在桌面上的手寫標語
- **文字樣式**：顏色、字型、粗體、大小（最大 6 倍）；系統匣「文字預設樣式」改全部，右鍵「文字樣式（這張）」只改一張
- **蕃茄鐘**：蕃茄、沙漏、LED 數字錶、鬧鐘、抱時鐘的貓、抱時鐘的柴犬；點一下開始／暫停，滾輪調分鐘；時間到會響（內建鈴聲、Windows 鬧鐘音效、自己的 .wav 或不出聲）。沒在跑就完全不耗電
- **紙張**：經典 8 色＋莫蘭迪 8 色＋透明；便利貼、平滑、方格、橫線、點陣、和紙、牛皮紙纖維
- **圖片**：拖圖檔進來或在便利貼上 Ctrl+V，變成拍立得
- **密碼**：每張可以設不同密碼（PBKDF2 + AES-256-GCM）。平常顯示一份偽裝待辦，輸入密碼後文字亂碼再變回原文；關掉或重開自動鎖回
- **星號記事本**：搭配 Vibotype 的選取工具列，在任何程式選字按 ★ 就收進來
- **匯出／匯入 JSON**：換電腦或手動同步（圖片、手寫、自製貼紙和群組一起打包，加密的內容仍然加密）
- **桌面右鍵**「新增便利貼」「新增蕃茄鐘」、**開機啟動**，都可以在系統匣關掉
- **線上更新**：安靜地檢查新版（只在系統匣選單顯示）、重要更新自動下載下次啟動套用；新貼紙包和小舖橫幅也能線上送來，都可以關掉（見 [docs/UPDATES.md](docs/UPDATES.md)）
- **多語系**：可以自己加語言，不用重新編譯（見 [locales/README.md](locales/README.md)）

## 使用

- 系統匣圖示：**左鍵**新增便利貼、**右鍵**選單（新增手寫、透明標語、蕃茄鐘也在這裡）
- 手寫：便利貼右上角的筆、或右鍵「手寫」；Esc／Enter 結束，E 切換橡皮擦
- 蕃茄鐘：空白鍵開始／暫停、R 重來；右鍵換樣式、時間、聲音
- 便利貼上：連點兩下編輯、右鍵選單、Delete 刪除（可以從系統匣復原）、Ctrl+L 鎖上
- 資料在 `%APPDATA%\PinPin\`，格式見 [docs/DATA.md](docs/DATA.md)

## 開發者

編譯、開發用參數、專案結構和其他程式怎麼呼叫 PinPin，請看[英文版 README](README.md#building)。歡迎 issue 和 PR，見 [CONTRIBUTING.md](CONTRIBUTING.md)。

想要自己的寵物貼紙或計時器角色？見[官網訂製頁](https://pinpin.fast01.net/#/custom)，或寄信到 ailazytech@gmail.com。

## 授權

[MIT](LICENSE)。內建的圖釘、紙膠帶、貼紙、蕃茄鐘圖片是用 AI 生圖後再處理，一樣以 MIT 釋出。
