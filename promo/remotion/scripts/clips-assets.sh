#!/usr/bin/env bash
# 特點短片（src/Clips.tsx）的素材 → public/clips/。全部是離屏畫圖，不開視窗、不動桌面。
# 用法：bash scripts/clips-assets.sh [pinpin.exe]（預設 ../../target/release/pinpin.exe）
# 原圖在 promo/clips-src/（AI 生圖：寵物照片、原創動漫角色、虛構偶像——都不是真實人物或現有角色）
set -e
cd "$(dirname "$0")/.."
EXE="${1:-../../target/release/pinpin.exe}"
OUT=public/clips
W() { cygpath -w "$1"; }
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
rm -rf "$OUT"
mkdir -p "$OUT"

# 便利貼、拍立得、桌面貼紙（3 倍）→ clips.json
APPDATA="$(W "$TMP/a")" "$EXE" --export-clips "$(W "$OUT")" "$(W ../clips-src)" | cat

# 貼紙小舖（2 倍；影片裡縮到 0.75）
export PINPIN_DPI=2
export APPDATA="$(W "$TMP/b")"
"$EXE" --render-shop "$(W "$OUT/shop-stickers.png")" 0 en | cat
# 春聯分頁往下捲（每 40px 一張）
for i in $(seq 0 45); do
  PINPIN_SHOP_SCROLL=$((i * 40)) "$EXE" --render-shop "$(W "$OUT/shop-couplets-$(printf %02d "$i").png")" 104 en | cat
done
# 自己的群組：貼上第一張、第二張之後
export APPDATA="$(W "$TMP/c")"
PINPIN_SHOP_ADD="$(W ../clips-src/anime.png)" "$EXE" --render-shop "$(W "$OUT/shop-mine-1.png")" 99 en | cat
PINPIN_SHOP_ADD="$(W ../clips-src/idol.png)" "$EXE" --render-shop "$(W "$OUT/shop-mine-2.png")" 99 en | cat
echo "clips: $(ls "$OUT" | wc -l) files"
