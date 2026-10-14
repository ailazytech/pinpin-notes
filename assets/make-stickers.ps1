# 內建貼紙：把生圖的貼紙表（透明背景、排成格子）切成一張張，再存成 JPEG XR（品質 75%）放進 assets\stickers\。
# 解析度：一律是顯示大小的 4 倍（一般貼紙顯示 75 → 300px），放大也不糊；顯示大小寫在 src\notes\assets.rs。
# 名稱「_」開頭的格子跳過；前綴「_」表示名稱就是完整檔名（同一張表有不同分類時）。
# 主體碰到格子邊時會印出「⚠ 可能被切到」，要重新生圖（生圖要求每格留大邊距）。
# 用法：cargo build --release 之後 powershell -File assets\make-stickers.ps1
param([string]$Exe = "$PSScriptRoot\..\target\release\pinpin.exe", [string]$Quality = "0.75")
$src = "$PSScriptRoot\deco-src"
$png = "$PSScriptRoot\..\target\stickers-png"   # 中間檔（網站也從這裡拿 PNG）
$out = "$PSScriptRoot\stickers"
$env:APPDATA = Join-Path $env:TEMP "pinpin-make-stickers"   # 不碰真實資料
Remove-Item $png -Recurse -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $png, $out | Out-Null
# 表, 前綴, 名稱（閱讀順序）, 欄, 列, 最長邊
$jobs = @(
  # 日常
  @('sheet-cute.png',     'st', 'cat,shiba,boba,strawberry,donut,onigiri,sun,rainbow,apple', 3, 3, 300),
  @('sheet-icons.png',    'st', 'heart,star,thumbsup,check,alert,fire,bolt,sparkles,coffee', 3, 3, 300),
  @('sheet-nature.png',   'st', 'clover,cactus,monstera,sunflower,mushroom,cloud,moon,sakura,rain', 3, 3, 300),
  # 招財祝福（空聚寶盆、金飾）＋韓國福笊籬
  @('sheet-lucky.png',          'lk', 'cat,ingot,goldbars,bag,envelope,koi,bowl,coins,tree', 3, 3, 300),
  @('sheet2-lucky-bowls.png',   '_',  'lk-bowl-gold,lk-bowl-red,lk-bowl-box,ch-bokjori', 2, 2, 400),
  @('sheet2-lucky-gold.png',    'lk-gold', 'beans,bean,ring,bracelet,necklace,ingot,coin,pig,lock', 3, 3, 160),
  # 各國招財（第 7 格舊的木杓不要，改用上面的福笊籬）
  @('sheet-charms.png',   'ch', 'daruma,omamori,horseshoe,pig,nazar,hamsa,_scoop,knot,ladybug', 3, 3, 300),
  @('sheet-plants.png',   'pl', 'monstera,tortoise,succulents,snake,fiddle,pothos,staghorn,pilea,bonsai', 3, 3, 300),
  # 春聯：單字和合體字。舊表只留三張真正融合的合體字（「滿」生圖一直畫成簡體，不用）
  # 單字：生圖的斗方，每個字都放大逐筆檢查過
  @('sheet2-spring-a.png',     'cn', 'fu,chun,an,ji', 2, 2, 440),
  @('sheet2-spring-b.png',     'cn', 'cai,wang,xi,jin', 2, 2, 440),
  @('sheet2-spring-d.png',     'cn', '_yiben,_fulushou,_caiyuan,shuangxi', 2, 2, 440),
  # 合體字斗方和直立春條：商家的真人作品（couplets\src，使用者提供）只用來驗證寫法——生圖時當參考圖、做完並排比對；
  # 毛筆字是 ChatGPT 生的（couplets\ink，白紙黑墨），couplets\compose-all.sh 轉成漸層金字疊到紅紙上 → couplets\final
  @('couplets\final\招財進寶.png', '_', 'cn-zhaocai', 1, 1, 440),
  @('couplets\final\黃金萬兩.png', '_', 'cn-huangjin', 1, 1, 440),
  @('couplets\final\日進斗金.png', '_', 'cn-rijin', 1, 1, 440),
  @('couplets\final\日日見財.png', '_', 'cn-jianzai', 1, 1, 440),
  @('couplets\final\一帆風順.png', '_', 'cn-yifan', 1, 1, 440),
  @('couplets\final\生意興隆.png', '_', 'cn-shengyi', 1, 1, 440),
  @('couplets\final\金玉滿堂.png', '_', 'cn-jinyu', 1, 1, 440),
  @('couplets\final\紫氣東來.png', '_', 'cn-ziqi', 1, 1, 440),
  @('couplets\final\千祥雲集.png', '_', 'cl-qianxiang', 1, 1, 640),
  @('couplets\final\大吉大利.png', '_', 'cl-daji', 1, 1, 640),
  @('couplets\final\五穀豐登.png', '_', 'cl-wugu', 1, 1, 640),
  @('couplets\final\天官賜福.png', '_', 'cl-tianguan', 1, 1, 640),
  @('couplets\final\心想事成.png', '_', 'cl-xinxiang', 1, 1, 640),
  @('couplets\final\平安喜樂.png', '_', 'cl-pingan', 1, 1, 640),
  @('couplets\final\吉星高照.png', '_', 'cl-jixing', 1, 1, 640),
  @('couplets\final\吉祥如意.png', '_', 'cl-jixiang', 1, 1, 640),
  @('couplets\final\年年有餘.png', '_', 'cl-niannian', 1, 1, 640),
  @('couplets\final\身體健康.png', '_', 'cl-shenti', 1, 1, 640),
  @('couplets\final\幸福安康.png', '_', 'cl-xingfu', 1, 1, 640),
  @('couplets\final\新年快樂.png', '_', 'cl-xinnian', 1, 1, 640),
  @('couplets\final\歲歲平安.png', '_', 'cl-suisui', 1, 1, 640),
  @('couplets\final\福星高照.png', '_', 'cl-fuxing', 1, 1, 640),
  @('couplets\final\學業進步.png', '_', 'cl-xueye', 1, 1, 640),
  # 貓狗（每隻眼神不同）
  @('sheet2-pets-a.png',  'cat', 'calico,tuxedo,siamese,british', 2, 2, 300),
  @('sheet2-pets-b.png',  'cat', 'ragdoll,american,black,fold', 2, 2, 300),
  @('sheet2-pets-c.png',  '_',   'cat-munchkin,cat-sphynx,dog-corgi,dog-golden', 2, 2, 300),
  @('sheet2-pets-d.png',  'dog', 'frenchie,dachshund,pom,poodle', 2, 2, 300),
  @('sheet2-pets-e.png',  'dog', 'husky,formosan,bichon,schnauzer', 2, 2, 300),
  @('sheet-scooters.png', 'sc',  'cream,mint,pink,cub,ev,lemon,lavender,cherry,classic', 3, 3, 340),
  # 台灣小物（不畫商標和字）
  @('sheet2-taiwan-a.png', 'tw', 'boba,chicken,greenman,beer,cooker,slippers,bag,pineapplecake,stinkytofu', 3, 3, 300),
  @('sheet2-taiwan-b.png', 'tw', 'guabao,porkrice,mangoice,bear,taipei101,douhua,sausage,oyster,pineapple', 3, 3, 300)
)
# 接 Out-String：pinpin.exe 是視窗程式，不接輸出的話 PowerShell 不會等它做完
foreach ($j in $jobs) {
  & $Exe --split-grid (Join-Path $src $j[0]) $png $j[1] $j[2] $j[3] $j[4] $j[5] $j[5] | Out-String
}
# 春聯不要白邊
& $Exe --unborder $png 'cn-,cl-' | Out-String
Remove-Item (Join-Path $out '*.jxr') -ErrorAction SilentlyContinue
& $Exe --pack-jxr $png $out $Quality | Out-String
