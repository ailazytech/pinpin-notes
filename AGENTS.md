# AGENTS.md — working on PinPin Notes

Instructions for AI coding agents (Claude Code, Codex, Cursor, Copilot…) and for humans who want the short version.
The long-form contributor guide is [CONTRIBUTING.md](CONTRIBUTING.md) ([繁體中文](CONTRIBUTING.zh-TW.md)).

## What this is

A Windows 10/11 sticky-notes app written in Rust on raw Win32 (layered windows, per-pixel drawing, no UI framework).
One `.exe`, no installer, **zero idle CPU**. Plus a static website (`site/`, Vite+ / React) and promo video tooling
(`promo/remotion/`, Remotion).

## Commands

| Task | Command (PowerShell or Git Bash, from the repo root) |
|---|---|
| Test the app | `cargo test --release` |
| Build the app | `cargo build --release` (must produce **no warnings**) |
| Format | `cargo fmt` |
| Draw every note style offscreen | `$env:APPDATA="$env:TEMP\pp"; .\target\release\pinpin.exe --render-notes out.png` |
| Draw the Sticker Shop offscreen | `.\target\release\pinpin.exe --render-shop out.png 0 en` |
| Website | `cd site; pnpm install; pnpm run build` (type-checks too) |
| Promo clips | see [promo/remotion/README.md](promo/remotion/README.md) |

If PinPin is running, `target/release/pinpin.exe` is locked — build with `--target-dir target/alt` instead of
closing the user's app. After moving the checkout to another folder, `cargo clean -p pinpin` fixes stale
absolute paths in build-script output (e.g. "failed to load manifest.xml").

## Layout

```
src/app.rs           entry point, single instance, tray, developer flags, WM_COPYDATA
src/notes/           the notes: mod.rs (windows, input), render.rs (drawing), store.rs (notes.json),
                     assets.rs (built-in art), stickers.rs (cut-out), shop.rs, ink.rs, timer.rs,
                     crypto.rs (password notes), backup.rs, promo.rs / sprites.rs (offscreen renders)
src/update.rs        online update check (feed.json, SHA-256 verified downloads)
locales/*.json       UI strings, six languages, bundled at build time
site/                website; strings in site/src/i18n/locales/*.ts (en.ts is the type source)
docs/                FEATURES, DATA (file format + IPC), UPDATES, PRIVACY, RELEASING
```

## Rules that are never broken

1. **No idle work.** No timers or polling while nothing changes. Animations use `SetCoalescableTimer` and stop when done.
2. **Never touch the user's desktop in tests.** Do not simulate keyboard or mouse input. Verify visuals with the
   offscreen renders above, with `APPDATA` pointed at a scratch folder so real notes are untouched.
3. **Backward-compatible data.** Old `notes.json` must still load. Changing the format means bumping
   `store::VERSION` and writing a migration ([docs/DATA.md](docs/DATA.md)).
4. **The `WM_COPYDATA` protocol is frozen** (`PinPin.Main`, `dwData = 0x50424E31`, fields `cmd/text/source/x/y`).
   Vibotype depends on it; only add new `cmd`s or optional fields.
5. **Every visible string goes through `t("key")`** and exists in all six `locales/*.json`
   (`cargo test` checks the keys match). Website strings: all six `site/src/i18n/locales/*.ts`.
6. **Nothing is deleted for real** without going through the trash (restorable).
7. **No new crates** unless Windows already lacks it (WIC, CNG, GDI, WinHTTP come first).
8. **No secrets, local paths or personal names in commits** — not in code, docs, images or test data.

## Branches, commits, PRs

- `main` = released, what the website and the in-app updater ship. Only release PRs from `dev` (or `hotfix/*`) merge here.
- `dev` = integration. Branch from `dev` (`feat/…`, `fix/…`, `docs/…`) and open the PR **into `dev`**.
- Commits and PR titles use [Conventional Commits](https://www.conventionalcommits.org): `feat(timer): …`,
  `fix(shop): …`, `docs: …`. Scopes are module names: `notes render shop stickers ink timer crypto store update i18n site promo ci`.
  CI checks PR titles.
- User-visible changes add one line to `CHANGELOG.md` under `[Unreleased]` (in 繁體中文, like the rest of the file).
- Code comments are in 繁體中文 and explain *why*. Write like the surrounding code.

## Before you say "done"

- [ ] `cargo test --release` passes; `cargo build --release` has no warnings
- [ ] Visual change → offscreen render before/after attached to the PR
- [ ] Website change → `pnpm run build` in `site/` passes
- [ ] New strings in all six languages (app and/or website)
- [ ] `CHANGELOG.md` updated for user-visible changes
- [ ] Security checklist below, if you touched anything on it

## Security review checklist

Run this whenever a change touches input parsing, files, network, crypto or `unsafe`.
Report findings in the PR; real vulnerabilities go to [SECURITY.md](SECURITY.md), not public issues.

- **IPC (`WM_COPYDATA`)** — any window can send it. Treat the JSON as hostile: size limits, no paths or commands
  executed from it, unknown `cmd`s ignored.
- **Files** — `notes.json`, imports and sticker groups are untrusted. No path traversal when writing
  (`..`, absolute paths, ADS like `file:stream`); size and dimension limits before decoding images.
- **Network** (`src/update.rs`) — HTTPS only (`http://localhost` for development); every download is checked against
  the SHA-256 in the feed before use; response sizes are capped. Never add a code path that runs a download unverified.
- **Crypto** (`src/notes/crypto.rs`) — PBKDF2 + AES-256-GCM through Windows CNG. No custom crypto, no lowering the
  iteration count, fresh random salt and nonce every time, never log plaintext or passwords.
- **`unsafe` / Win32** — buffer lengths match the API's units (bytes vs UTF-16 units), handles are closed, no
  use-after-free across window procedures.
- **Registry** (`src/sysreg.rs`) — only the keys documented in `docs/`; startup and shell menu stay opt-in.
- **Website** — no secrets in client code (`VITE_*` variables are public); Pages Functions validate input;
  `STATS_KEY` and other secrets live in Cloudflare, never in the repo.
- **Dependencies** — `cargo audit` and Dependabot run in CI; explain any new crate or npm package in the PR.

Automated checks on every push and PR: CodeQL, `cargo audit`, `pnpm audit` (high and above), gitleaks secret scan
(`.github/workflows/security.yml`, `.github/workflows/codeql.yml`).

## Releasing

Maintainers only; full steps in [docs/RELEASING.md](docs/RELEASING.md). In short: release PR `dev → main`,
bump `Cargo.toml` and `CHANGELOG.md`, tag `vX.Y.Z` on `main` → GitHub builds a draft release → publish it →
Cloudflare rebuilds the site, the download button and `feed.json` follow the new release.
