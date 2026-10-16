# Security Policy · 安全性政策

## Supported versions · 支援的版本

Only the latest release gets security fixes. PinPin updates itself (or tells you about the update in the tray menu),
so please stay on the newest version.

只有最新版會收到安全性修正。PinPin 會自動檢查更新（或在系統匣選單提示），請保持最新版。

## Reporting a vulnerability · 回報漏洞

**Please don't open a public issue.** Report privately in one of these ways:

1. GitHub: **[Report a vulnerability](https://github.com/ailazytech/pinpin-notes/security/advisories/new)**
   (Security tab → "Report a vulnerability") — preferred
2. Email **ailazytech@gmail.com** with the subject `[PinPin security]`

**請不要開公開 issue。** 用下面任一方式私下回報：

1. GitHub：**[回報漏洞](https://github.com/ailazytech/pinpin-notes/security/advisories/new)**（Security 分頁 →「Report a vulnerability」），優先用這個
2. 寄信到 **ailazytech@gmail.com**，主旨寫 `[PinPin security]`

Please include · 請附上：

- what is affected and how bad it is · 影響範圍和嚴重程度
- steps or a proof of concept · 重現步驟或概念驗證
- the PinPin version and Windows version · PinPin 和 Windows 版本

## What to expect · 處理流程

| | |
|---|---|
| Acknowledgement · 確認收到 | within 2 weeks · 2 週內 |
| First assessment · 初步評估 | within 3 weeks · 3 週內 |
| Fix released · 發佈修正 | as soon as possible; critical issues are pushed as an automatic update · 越快越好，嚴重的問題會用自動更新推送 |

We'll credit you in the release notes and the advisory unless you'd rather stay anonymous.

修好後會在更新說明和安全公告中致謝，不想具名也可以。

## Scope · 範圍

In scope · 在範圍內：

- the PinPin app (`src/`) — password notes (`crypto.rs`), the online updater (`update.rs`), file import / export,
  the `WM_COPYDATA` interface, registry changes · App 本身：密碼便利貼、線上更新、匯入匯出、`WM_COPYDATA` 介面、登錄檔
- the website and its Pages Functions (`site/`) · 官網和 Pages Functions
- the release pipeline (`.github/workflows/`) · 發版流程

Out of scope · 不在範圍內：

- attacks that need an attacker already running code as the same Windows user (they can read your files anyway)
  · 攻擊者已經能以同一個 Windows 使用者身分執行程式的情況（本來就能讀你的檔案）
- the decoy text of password notes being "guessable" — it's a disguise, the real text is encrypted
  · 密碼便利貼的偽裝內容被看穿——那只是偽裝，真正的內容是加密的
- denial of service by sending huge amounts of local IPC messages · 用大量本機 IPC 訊息造成的阻斷服務

## How we keep it safe · 我們怎麼把關

- Every push and pull request runs CodeQL, `cargo audit` (RustSec), `pnpm audit` and a gitleaks secret scan
- Dependabot watches Rust crates, npm packages and GitHub Actions
- Updates are downloaded over HTTPS only and checked against the SHA-256 in the update feed before use
- Contributors and AI agents follow the checklist in [AGENTS.md](AGENTS.md#security-review-checklist)

- 每次 push 和 PR 都會跑 CodeQL、`cargo audit`（RustSec）、`pnpm audit` 和 gitleaks 金鑰掃描
- Dependabot 監看 Rust crate、npm 套件和 GitHub Actions
- 更新只走 HTTPS，下載後先核對 feed 裡的 SHA-256 才使用
- 貢獻者和 AI agent 照 [AGENTS.md 的安全檢查清單](AGENTS.md#security-review-checklist)檢查
