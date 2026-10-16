#!/usr/bin/env bash
# ink/ 裡生圖的毛筆字 → final/ 春聯貼紙原圖（1254x1254 透明 PNG）。用無頭 Chrome 跑 compose.html，不開視窗。
# src/ 是商家的真人作品，只拿來驗證寫法（生圖時當參考圖、做完並排比對），不會放進程式。
set -e
cd "$(dirname "$0")"
CHROME="/c/Program Files/Google/Chrome/Application/chrome.exe"
PROFILE="$(cygpath -w "$PWD/../../../target/pv/rt-profile")"
mkdir -p final
enc() { node -e "process.stdout.write(encodeURIComponent(process.argv[1]))" "$1"; }
for f in ink/*.png; do
  n=$(basename "$f" .png)
  case "$n" in
    一帆風順|日日見財|日進斗金|生意興隆|招財進寶|金玉滿堂|紫氣東來|黃金萬兩|金榜題名-合體) shape=diamond ;;
    *) shape=strip ;;
  esac
  "$CHROME" --headless=new --hide-scrollbars --allow-file-access-from-files --force-device-scale-factor=1 \
    --default-background-color=00000000 --user-data-dir="$PROFILE" --window-size=1254,1254 \
    --virtual-time-budget=20000 --screenshot="$(cygpath -w "$PWD/final/$n.png")" \
    "file:///$(cygpath -m "$PWD")/compose.html?ink=ink/$(enc "$n.png")&shape=$shape" >/dev/null 2>&1
  echo "$n ($shape)"
done
