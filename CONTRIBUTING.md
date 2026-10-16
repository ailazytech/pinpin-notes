# Contributing to PinPin

**English** · [繁體中文](CONTRIBUTING.zh-TW.md)

Thanks for helping! This explains how to report problems, how to change the code, and what a pull request needs.
Working with an AI agent? Point it at [AGENTS.md](AGENTS.md) first (the short version plus a security checklist).

By taking part you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting problems and ideas

- Bugs: use the "Bug report" template with your Windows version, display scale and steps to reproduce; attach the end of `%APPDATA%\PinPin\pinpin.log` if you can
- Features: open a "Feature request" first and agree on the direction before writing code
- Translations: use the "Translation" template — which language, which strings
- Security issues (password notes, update downloads…): **don't open a public issue** — follow [SECURITY.md](SECURITY.md)

## Branches

| Branch | Purpose |
|---|---|
| `main` | Released code. The website and the in-app updater follow it. Only release PRs from `dev` (or `hotfix/*`) |
| `dev` | Integration branch. **Open every PR against `dev`** |
| `feat/…`, `fix/…`, `docs/…` | Work branches cut from `dev`, one change each |

Urgent fix: branch `hotfix/…` from `main`, PR into `main`; after the release `main` is merged back into `dev`.

## Setup

| Part | Needs | Commands |
|---|---|---|
| App | Windows 10/11, stable Rust (MSVC) | `cargo test`, `cargo build --release` |
| Website `site/` | Node 22+, pnpm | `pnpm install`, `pnpm exec vp dev`, `pnpm run build` |
| Promo video `promo/remotion/` | Node 22+ | `npm install`, `npm run render` |

The code layout is in [AGENTS.md](AGENTS.md#layout) and, in more detail, in the [繁體中文 guide](CONTRIBUTING.zh-TW.md#程式結構app).

## Rules that keep PinPin PinPin

A PR that breaks one of these won't be merged:

1. **No idle work** — no timers or polling while nothing changes; redraw only when something changes; animations use `SetCoalescableTimer` and stop when done
2. **Low memory** — decode images at display size and cache that, never keep originals around
3. **Never lose the user's things** — empty notes aren't auto-deleted; deleting always goes through the restorable trash
4. **Strike-throughs are always red**
5. **The `WM_COPYDATA` protocol is frozen** (`PinPin.Main`, `dwData = 0x50424E31`, JSON `cmd/text/source/x/y`); only add new `cmd`s or optional fields. New window classes start with `PinPin.`
6. **Backward-compatible data** — old `notes.json` must load; format changes bump `store::VERSION` with a migration ([docs/DATA.md](docs/DATA.md))
7. **Tests never touch the real desktop** — no simulated keyboard/mouse; verify visuals with `--render-notes` / `--render-shop` and `APPDATA` set to a scratch folder
8. **No private data in commits** — no secrets, local paths or personal names in code, docs, images or test data

## UI text and translations

- Every visible string goes through `t("key")`
- New keys go into **all six** `locales/*.json`; `cargo test` (`bundled_files_parse_and_have_same_keys`) checks they match
- Don't speak a language? Put English in and say so in the PR
- Website strings live in `site/src/i18n/locales/*.ts`; `en.ts` is the type source, a missing key fails the build

## Commit messages and PR titles

[Conventional Commits](https://www.conventionalcommits.org). **PR titles must follow it too** — CI checks:

```
<type>(<scope>): <summary>

feat(timer): add the LED style
fix(shop): rotate handle vanished on hover
docs: explain the data format
```

| Type | For |
|---|---|
| `feat` | a new feature |
| `fix` | a bug fix |
| `perf` | performance |
| `refactor` | restructuring, same behavior |
| `docs` | documentation |
| `test` | tests |
| `build`, `ci` | build system, CI |
| `chore` | everything else, dependency bumps |
| `revert` | reverting a commit |

Scopes (optional) are module names: `notes`, `render`, `shop`, `stickers`, `ink`, `timer`, `crypto`, `store`, `update`, `i18n`, `site`, `promo`, `ci`.
Summaries may be in English or Chinese. Breaking changes get a `!` (`feat(store)!: …`) and a `BREAKING CHANGE:` line in the body.

## Before opening a PR

- [ ] The PR targets `dev` and its title follows Conventional Commits
- [ ] `cargo test` passes and `cargo build --release` has no warnings
- [ ] Visual change → offscreen render before/after attached
- [ ] Website change → `pnpm run build` passes, screenshot attached
- [ ] One line in `CHANGELOG.md` under `[Unreleased]` (written in 繁體中文 like the rest; skip for refactors, docs, CI)
- [ ] Data format changed → `docs/DATA.md` updated and `store::VERSION` bumped
- [ ] Touched input parsing, files, network, crypto or `unsafe` → went through the [security checklist](AGENTS.md#security-review-checklist)

## Automated checks

Every PR runs:

- **CI**: `cargo test`, `cargo build`, website type check and build
- **PR title**: Conventional Commits; PRs into `main` only from `dev` or `hotfix/*`
- **Security**: CodeQL, `cargo audit` (RustSec advisories), `pnpm audit`, gitleaks (secret scan)
- **Dependabot**: weekly dependency updates, opened against `dev`

## Style

- `cargo fmt`; comments in 繁體中文, explaining *why* rather than *what*
- Avoid new crates; prefer what Windows already ships (WIC, CNG, GDI, WinHTTP)
- One PR, one change
