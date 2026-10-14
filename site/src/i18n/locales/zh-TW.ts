import type { Dictionary } from "./en";

export const zhTW: Dictionary = {
  meta: {
    title: "貼貼 PinPin Notes — 溫暖可愛的 Windows 桌面便利貼",
    description:
      "在 Windows 桌面上釘圖釘、貼紙膠帶、玩貼紙、手寫、掛透明標語、跑蕃茄鐘——平常幾乎不耗電。免費開源。",
  },

  nav: {
    order: "春聯訂製",
    custom: "訂製",
    home: "首頁",
    features: "功能",
    donate: "贊助",
    qa: "問答",
    download: "下載",
    openMenu: "開啟選單",
    closeMenu: "關閉選單",
    github: "原始碼",
    gotoSection: "前往章節",
  },

  theme: {
    toggle: "切換主題",
    light: "淺色",
    dark: "深色",
  },

  lang: {
    label: "語言",
    choose: "選擇語言",
    current: "目前語言",
  },

  curtain: {
    announce: "換頁中…",
    loading: "正在暖桌…",
    loadingLines: ["削尖圖釘中…", "撫平紙膠帶…", "倒珍奶中…"],
  },

  common: {
    comingSoon: "即將推出",
    linkPending: "連結即將公布",
    backHome: "回到首頁",
    tryIt: "試試看——這是真的",
    liveDemo: "互動示範",
    showScreenshot: "回到截圖",
    screenshotSlot: "截圖預留位",
    screenshotPath: "public/img/screens/{file}.webp",
    screenshotHint: "這個位置留給真實截圖——檔案放進來之前，示範會一直住在這裡。",
    notIncluded: "不在規劃中",
    reducedMotion: "依系統偏好，動畫已調淡。",
  },

  hero: {
    badge: "免費開源 · Windows 10 / 11",
    titleA: "讓桌面，",
    titleB: "有紙的溫度。",
    tagline: "桌面上的小紙條，一直陪著你。",
    sub: "溫暖可愛的 Windows 桌面便利貼——圖釘、紙膠帶、貼紙、手寫、透明標語、蕃茄鐘，平常幾乎不耗電。",
    download: "下載 Windows 版",
    downloadNote: "單一 .exe · 約 11 MB · 免安裝",
    explore: "探索功能",
    downloadPending: "GitHub Releases 連結整理中——連結一上線，按鈕就會醒來。",
    stats: [
      { value: "0%", label: "閒置 CPU" },
      { value: "≈ 15 MB", label: "記憶體" },
      { value: "17", label: "種紙色" },
      { value: "6", label: "種語言" },
    ],
    sceneCaption: "貼貼的桌面一角",
    scroll: "向下捲動",
    note1: "下班後\n去喝珍奶 ♪",
    note2: "專注\n25:00",
    note3: "週日\n打給媽媽",
    slogan: "溫柔地，\n再開始一次。",
  },

  marquee: {
    label: "146 張內建貼紙的其中幾張",
    hint: "把貼紙拖下來貼在任何地方 · 點一下後，滾輪轉角度、Shift＋滾輪調大小",
    reset: "全部歸位",
  },

  video: {
    eyebrow: "影片",
    title: "看看它動起來的樣子",
    sub: "一部關於便利貼、紙膠帶與圖釘的小片子——釘在這裡等你。",
    play: "播放宣傳影片",
    makingTitle: "影片重新製作中",
    makingDesc:
      "宣傳影片正在重製。這個位置已經接上 public/promo/pinpin-promo.mp4——把檔案放進去就會播放，不用改程式。",
    poster: "宣傳影片海報",
  },

  overview: {
    eyebrow: "功能",
    title: "伸手可及的全部",
    sub: "十六個為桌面而生的小確幸。點任何卡片，前往它的章節。",
    cards: [
      { id: "notes", title: "便利貼本體", blurb: "17 色、7 種紙質" },
      { id: "todo", title: "紅筆待辦", blurb: "溫柔地劃掉它" },
      { id: "shop", title: "貼紙小舖", blurb: "圖釘、夾子、紙膠帶與 146 位朋友" },
      { id: "custom", title: "自製貼紙", blurb: "任何圖片，割好白邊" },
      { id: "handwriting", title: "手寫", blurb: "四種筆，真實筆觸" },
      { id: "transparent", title: "透明紙", blurb: "字寫在桌布上" },
      { id: "text", title: "文字樣式", blurb: "字型、顏色、最大 6 倍" },
      { id: "memories", title: "拍立得與秘密", blurb: "照片與密碼" },
      { id: "pomodoro", title: "蕃茄鐘", blurb: "六個小計時員" },
      { id: "detail-password", title: "密碼便利貼", blurb: "偽裝中的秘密" },
      { id: "detail-starred", title: "星號記事本", blurb: "從任何地方收集 ★" },
      { id: "snap", title: "磁吸對齊", blurb: "整齊得像自己排的" },
      { id: "detail-context", title: "系統匣與選單", blurb: "隨手可得" },
      { id: "detail-export", title: "匯出匯入", blurb: "一個 JSON 帶走全部" },
      { id: "langs", title: "六種語言", blurb: "含由右到左" },
      { id: "power", title: "省電設計", blurb: "閒置 CPU 0%" },
    ],
  },

  chapters: [
    {
      id: "notes",
      eyebrow: "01",
      title: "像紙一樣的便利貼",
      desc: "十七種紙色、七種紙質，從經典便利貼到牛皮紙纖維。點兩下直接在紙上打字，拖右下角改大小——把便利貼拿起來時，陰影也會跟著浮起，像一張被光接住的紙。",
      bullets: [
        "17 種紙色——經典 8 色、莫蘭迪 8 色，加上全透明",
        "7 種紙質：便利貼、平滑、方格、橫線、點陣、和紙纖維、牛皮紙纖維",
        "新便利貼記得上一次選的顏色、紙質與釘法",
        "空白便利貼永遠不會自動刪除——這是刻意的",
      ],
      media: "notes",
    },
    {
      id: "todo",
      eyebrow: "02",
      title: "用紅筆劃掉的待辦",
      desc: "每一行左邊都有個小圓圈。點一下，紅筆就會把那行劃掉——一筆歪歪的手寫筆跡，自己畫完。再點一下就取消。",
      bullets: [
        "手寫感的劃線動畫，永遠是紅色",
        "右鍵：全部劃掉、全部取消、清掉已劃掉的行",
        "由右到左的行（阿拉伯文、希伯來文）從右邊劃過去",
        "旁邊的示範可以點——來吧",
      ],
      media: "todo",
    },
    {
      id: "shop",
      eyebrow: "03",
      title: "貼紙小舖",
      desc: "一間永遠免費的小舖：圖釘、夾子、紙膠帶，和 146 張貼紙分成八區——日常小可愛、招財祝福（附空聚寶盆，金豆金飾一個個放進去）、各國招財小物、盆栽、金字毛筆春聯、貓狗品種、文青小機車和台灣小物，新貼紙包還會線上送來。拖到便利貼上，或直接貼在桌面。",
      bullets: [
        "8 款圖釘 · 7 款夾子 · 8 款紙膠帶 · 146 張貼紙，全部免費",
        "貼紙可以不依附紙張、直接住在桌面上",
        "貼上時有「落下、壓一下、彈回」的動畫",
        "旋轉與縮放控制點；按住 Shift 讓旋轉一次 15°",
      ],
      media: "shop",
    },
    {
      id: "custom",
      eyebrow: "04",
      title: "自製貼紙",
      desc: "貼上任何圖片，貼貼就幫你割好——完全離線、不用 AI，只靠細心的像素功夫——再補上一圈平滑的白色割邊，連凹進去的弧線都照顧到。",
      bullets: [
        "拖檔案進來、按 Ctrl+V，或從資料夾挑選",
        "依圖片邊緣顏色去背；背景太花的照片會做成圓角照片貼紙",
        "白邊會填平凹陷處，像真的割好的貼紙",
        "保留原圖解析度到 4096 px——群組跟著匯出匯入走",
      ],
      media: "custom",
    },
    {
      id: "handwriting",
      eyebrow: "05",
      title: "在任何紙上手寫",
      desc: "四種筆、四種脾氣：鋼筆（頭尾稍細）、半透明的胖螢光筆、帶顆粒的粉筆，和露紙紋的蠟筆——每一種都用屬於自己的筆觸畫在紙上。",
      bullets: [
        "8 色墨水 × 3 種粗細，還有橡皮擦與復原",
        "筆跡在文字上面、圖釘下面——拖動便利貼不會閃",
        "Esc / Enter 結束、Ctrl+Z 復原、E 切換橡皮擦",
        "存檔前自動簡化筆跡，檔案很小",
      ],
      media: "handwriting",
    },
    {
      id: "transparent",
      eyebrow: "06",
      title: "寫在桌面上的大字",
      desc: "把紙色選成「透明」，紙就消失了——只剩你的字和筆跡，像直接寫在桌布上。滑鼠移過去，才會出現淡淡的虛線框讓你搬動、編輯。",
      bullets: [
        "沒有紙、沒有陰影——只有字和筆跡",
        "座右銘等級的大字與亮色墨水",
        "可以釘在桌面、放在所有視窗下面",
        "滑鼠移過去才出現淡淡的虛線框",
      ],
      media: "transparent",
    },
    {
      id: "text",
      eyebrow: "07",
      title: "打出你的語氣",
      desc: "十種墨色加自訂色、電腦裡的每一個字型、粗體，和七種大小（最大 6 倍）。從系統匣設好預設，再讓單張便利貼自由發揮。",
      bullets: [
        "A− / A+ 與 Ctrl+滾輪，每次 ±12%；紙會自動長高",
        "系統匣的通用預設 + 每張便利貼的專屬覆寫",
        "常用字型排前面——正黑體、游ゴシック、Meiryo、맑은 고딕…",
        "一鍵「把這張的樣式設成預設」",
      ],
      media: "text",
    },
    {
      id: "memories",
      eyebrow: "08",
      title: "拍立得與藏好的秘密",
      desc: "把圖片拖到便利貼上，它會變成拍立得，圖說用手寫風字型。給便利貼設專屬密碼，它就會偽裝成一張普通的採買清單，直到你解鎖為止。",
      bullets: [
        "拍立得白框，圖說是手寫風字型",
        "PBKDF2-SHA256（20 萬次）+ AES-256-GCM，走 Windows 加密 API",
        "鎖著時顯示偽裝清單——完全看不出來有密碼",
        "解鎖時文字先變亂碼再回來；密碼錯會搖頭",
      ],
      media: "memories",
    },
    {
      id: "pomodoro",
      eyebrow: "09",
      title: "六個小計時員",
      desc: "蕃茄、沙漏、LED 數字錶、雙鈴鬧鐘、抱時鐘的貓、抱時鐘的柴犬。挑一個最像你桌面個性的。",
      bullets: [
        "專注與休息自動輪替；完成的蕃茄用小紅點亮起來",
        "滾輪調分鐘——Shift 一次 5 分，1 到 180 分",
        "聲音：內建叮咚與鈴鈴、Windows 鬧鐘音效、自己的 .wav——也可以靜音",
        "只記「幾點結束」：重開機接得上，過了也不補響",
      ],
      media: "pomodoro",
    },
    {
      id: "snap",
      eyebrow: "10",
      title: "自己排整齊的擺放",
      desc: "便利貼會輕輕磁吸——靠近別張或螢幕邊緣 7 px 內就喀一聲對齊，間距整整 10 px。把它們釘在桌面（所有視窗下面），或讓某一張永遠浮在最上層。",
      bullets: [
        "對齊便利貼與螢幕邊緣的磁吸；間距固定 10 px",
        "按住 Alt 自由漂；方向鍵微調 1 px（Shift 一次 10 px）",
        "釘在桌面——放在所有視窗下面",
        "跑到不存在的螢幕上，下次開啟自動搬回主螢幕",
      ],
      media: "snap",
    },
  ],

  demos: {
    todo: {
      lines: ["回覆 Mia", "替仙人掌澆水", "列印票券", "掛號牙醫"],
      strikeHint: "點一下任何一行",
    },
    password: {
      title: "超市採買",
      decoy: ["牛奶", "雞蛋", "吐司"],
      secret: "蛋糕是為了 Mia 的驚喜派對！",
      hint: "示範中，任何密碼都可以",
      unlock: "解鎖",
      lock: "重新上鎖",
      inputLabel: "密碼",
    },
    pomodoro: {
      start: "開始",
      pause: "暫停",
      reset: "重來",
      focus: "專注",
      break: "休息",
      minutes: "分",
      styles: ["蕃茄", "沙漏", "LED", "鬧鐘", "貓咪", "柴犬"],
      done: "顆番茄完成",
    },
    text: {
      sample: "溫柔地，再開始一次。",
      smaller: "縮小",
      larger: "放大",
      sizeLabel: "大小",
      fontLabel: "字型",
      fonts: ["Segoe Print", "Comic Sans MS", "Georgia", "Impact"],
      note: "最大可到 6×——紙會跟著長高",
    },
    handwriting: {
      pens: ["鋼筆", "螢光筆", "粉筆", "蠟筆"],
      word: "你好！",
      caption: "同一筆，四種脾氣",
    },
    custom: {
      paste: "從剪貼簿貼上",
      drop: "拖一張圖片進來",
      or: "或",
      arrow: "變成",
      result: "割形貼紙",
      keepRes: "保留原解析度，最長邊 4096 px",
    },
    shop: {
      pins: "圖釘",
      clips: "夾子",
      tapes: "紙膠帶",
      stickers: "貼紙",
      free: "全部免費",
      shelf: "貨架",
    },
    snap: {
      hint: "拖拖看奶油黃那張——它會磁吸。",
      alt: "按住 Alt 可暫時不吸",
      gap: "10 px",
    },
    transparent: {
      hint: "滑過去會出現虛線框",
      motto: "溫柔地，再開始一次。",
    },
    notes: {
      colorsLabel: "17 種紙色",
      texturesLabel: "7 種紙質",
      transparent: "透明",
    },
  },

  details: {
    eyebrow: "小細節",
    title: "小地方，磨到會發亮",
    sub: "不太上得了版面、卻讓每一天更順手的功能。",
    cards: [
      {
        id: "detail-snap",
        title: "磁吸擺放",
        desc: "靠近其他便利貼或螢幕邊緣 7 px 內會輕輕吸住對齊，自動留 10 px 間距。按住 Alt 不吸；方向鍵微調 1 px（Shift 10 px）。",
      },
      {
        id: "detail-starred",
        title: "星號記事本",
        desc: "搭配 Vibotype：在任何程式選取文字、按 ★，就連時間與來源程式一起收進同一本。",
      },
      {
        id: "detail-export",
        title: "一個檔案帶走全部",
        desc: "把所有便利貼、圖片、手寫、自製貼紙與群組匯出成一個 JSON——加密的維持加密。匯入時溫柔合併。",
      },
      {
        id: "detail-undo",
        title: "溫柔的復原",
        desc: "刪掉的便利貼會在回收區休息（保留最近 20 張）——隨時從系統匣救回來。",
      },
      {
        id: "detail-context",
        title: "就在你手邊",
        desc: "桌面右鍵就能新增便利貼與蕃茄鐘；系統匣選單包辦其他。開機啟動一鍵開關，而且永遠只有一個貼貼在跑。",
      },
      {
        id: "detail-langs",
        title: "六種語系，完整鏡像",
        desc: "English、繁體中文、简体中文、日本語、한국어 與 العربية——由右到左的讀者有整面鏡像的介面。",
      },
    ],
  },

  power: {
    eyebrow: "省電",
    title: "像一張紙一樣安靜",
    desc: "永遠沒有背景計時器。只在有東西改變時才畫一次——動畫播完就停、點陣圖畫完立刻釋放、貼紙第一次用到才解碼。",
    stats: [
      { value: "0%", label: "閒置 CPU" },
      { value: "≈ 15 MB", label: "記憶體" },
      { value: "11 MB", label: "單一 .exe" },
      { value: "0", label: "網路連線" },
    ],
    footnote: "在安靜桌面、幾張便利貼常駐的環境下實測。",
    points: [
      "閒置時沒有任何計時器在跑",
      "動畫結束——就停止繪製",
      "只有滑鼠真的在動時才重畫",
      "分層視窗把畫好的像素交給系統合成",
    ],
  },

  langs: {
    eyebrow: "語言",
    title: "六種語言，賓至如歸",
    desc: "內建 English、繁體中文、简体中文、日本語、한국어 與 العربية。阿拉伯文會把整個介面左右鏡像——選單、貼紙小舖，全部；便利貼內容則一行一行自動判斷方向，混著寫也正確。",
    customTitle: "帶上你自己的語言",
    customSub: "語言檔是平易近人的 JSON。複製範本、翻好右邊的值，存檔就出現在選單——不用重新編譯。",
    customSteps: [
      "從系統匣開啟自訂語言檔資料夾",
      "複製英文範本、翻好右邊的值",
      "存檔——立刻出現在語言選單",
      "沒翻到的字自動退回英文",
    ],
    jsonTitle: "a-little-türkçe.json",
    jsonLines: [
      "{",
      '  "menu.new_note": "Yeni not",',
      '  "menu.pomodoro": "Yeni pomodoro sayacı",',
      '  "menu.sticker_shop": "Çıkartma dükkânı"',
      "}",
    ],
    rtlTitle: "由右到左，做得完整",
    rtlSample: "每一行都從右邊讀起——選單、紙膠帶，全部。",
    lineDirection: "每一行自己找方向，混在一張便利貼裡也行。",
  },

  download: {
    eyebrow: "下載",
    title: "帶它回你的桌面",
    sub: "一個小檔案。免安裝、免帳號、不連網——只是一口袋的紙。",
    button: "下載 pinpin.exe",
    size: "約 11 MB",
    pending: "GitHub Releases 連結整理中——連結一上線，按鈕就會醒來。",
    reqTitle: "系統需求與事實",
    reqs: [
      "Windows 10 或 11（64 位元）",
      "單一 pinpin.exe——免安裝、免 .NET、免執行環境",
      "沒有帳號、沒有雲端、不使用網路",
      "資料住在 %APPDATA%\\PinPin\\",
      "MIT 授權——永遠免費",
    ],
    license: "以 MIT 授權釋出。",
    sourcePending: "原始碼連結即將公布。",
  },

  footer: {
    tagline: "由紙張、圖釘和一枝小紅筆做成。",
    siteTitle: "網站",
    projectTitle: "專案",
    languageTitle: "語言",
    links: {
      order: "春聯訂製",
      custom: "訂製服務",
      home: "首頁",
      qa: "問答",
      donate: "贊助",
      download: "下載",
      source: "原始碼（即將）",
      license: "MIT 授權",
    },
    copyright: "貼貼 · MIT 授權，自由免費",
    crafted: "用心製作",
  },

  donate: {
    eyebrow: "贊助",
    title: "請貼貼喝杯珍奶",
    sub: "貼貼免費、開源，也會一直如此——沒有廣告、沒有雲端費用、沒有帳號。如果它讓你的桌面更舒服，一份小點心能讓圖釘持續發亮。",
    tiersTitle: "挑一份點心",
    tiers: [
      { name: "一張小貼紙", amount: "$1", desc: "一顆小亮光——謝謝你！" },
      { name: "一杯珍奶", amount: "$3", desc: "經典款。珍珠多一點，開心多一點。" },
      { name: "咖啡與陽光", amount: "$5", desc: "一個暖洋洋的寫程式午後。" },
      { name: "下午茶套餐", amount: "$10", desc: "一整盤的新貼紙。" },
    ],
    customLabel: "或自訂你的心意",
    customPlaceholder: "輸入金額",
    customUnit: "美元",
    methodsTitle: "挑一種說謝謝的方式",
    methodsPending: "付款管道建置中——很快就會出現在這裡。",
    methods: [
      { name: "PayPal", note: "連結即將公布" },
      { name: "Ko-fi", note: "連結即將公布" },
      { name: "GitHub Sponsors", note: "連結即將公布" },
    ],
    otherTitle: "其他幫上忙的方式",
    others: [
      { title: "到 GitHub 給它一顆星", desc: "一顆星不花錢，卻能讓專案發亮。" },
      { title: "告訴朋友", desc: "舒服的桌面值得分享——數位的那種。" },
      { title: "幫忙翻譯", desc: "自訂語言檔就是 JSON；歡迎寄你翻的來。" },
      { title: "回報小皺褶", desc: "發現 bug？一份清楚的回報比皺眉頭有用多了。" },
    ],
    thanks: "謝謝你——真的。每一份心意都會變成貼紙、像素和珍珠奶茶。",
    hearts: "滿滿的愛",
  },

  qa: {
    eyebrow: "問答",
    title: "問答，溫柔解答",
    sub: "釘下第一張便利貼之前，大家最常問的事。",
    searchPlaceholder: "搜尋問題…",
    noResults: "目前沒有符合的——換個關鍵字試試？",
    categories: {
      all: "全部",
      general: "一般",
      notes: "便利貼與貼紙",
      data: "資料與隱私",
      timer: "計時器",
      langs: "語言",
    },
    items: [
      {
        cat: "general",
        q: "貼貼真的免費嗎？",
        a: "是的——MIT 開源授權。使用、分享、修改都免費。沒有廣告、沒有帳號、沒有付費升級。",
      },
      {
        cat: "general",
        q: "支援哪些系統？",
        a: "Windows 10 與 Windows 11（64 位元）。它是單一的 pinpin.exe（約 11 MB）——免安裝、免 .NET 或其他執行環境。Mac、Linux 與手機版不在規劃中。",
      },
      {
        cat: "general",
        q: "要去哪裡下載？",
        a: "GitHub Releases 的連結正在整理。連結一公布，網站上的下載按鈕就會醒過來。",
      },
      {
        cat: "general",
        q: "會很耗 CPU 或記憶體嗎？",
        a: "它天生省電：幾張便利貼常駐約 15 MB 記憶體，閒置時 CPU 0%——沒有東西改變，就不重畫。",
      },
      {
        cat: "data",
        q: "有雲端同步嗎？",
        a: "沒有——貼貼完全不碰網路。要搬到另一台電腦，在一台匯出 JSON、到另一台匯入即可；加密的便利貼維持加密。",
      },
      {
        cat: "data",
        q: "我的資料存在哪裡？",
        a: "全部住在 %APPDATA%\\PinPin\\——notes.json、圖片、自製貼紙、設定與自訂語言檔。刪掉資料夾就清得乾乾淨淨。",
      },
      {
        cat: "data",
        q: "我不小心刪掉一張便利貼！",
        a: "刪掉的便利貼會進回收區，保留最近 20 張。從系統匣選「復原刪掉的便利貼」就能救回來。",
      },
      {
        cat: "data",
        q: "密碼便利貼是怎麼運作的？",
        a: "每張便利貼可以有自己的密碼。內容以 PBKDF2-SHA256（20 萬次迭代）+ AES-256-GCM 加密，使用 Windows 內建加密 API。鎖著時顯示偽裝的待辦清單；金鑰只存在記憶體，離開時自動鎖回。",
      },
      {
        cat: "notes",
        q: "可以自製貼紙嗎？",
        a: "可以——把圖片拖進貼紙小舖、按 Ctrl+V，或挑選檔案。貼貼依邊緣顏色去背（離線、不用 AI）、加上白色割邊，解析度保留到 4096 px。",
      },
      {
        cat: "notes",
        q: "手寫可以做到什麼？",
        a: "四種筆——鋼筆、螢光筆、粉筆、蠟筆——8 色 3 種粗細，附橡皮擦與復原。筆跡在文字上、圖釘下；存檔時自動簡化，檔案很小。",
      },
      {
        cat: "notes",
        q: "透明便利貼是什麼？",
        a: "選透明紙色後，便利貼沒有紙也沒有陰影——只剩文字和筆跡。滑過去會出現淡淡的虛線框與按鈕。也可以釘在所有視窗下面。",
      },
      {
        cat: "timer",
        q: "蕃茄鐘有幾種樣式？",
        a: "六種：蕃茄、沙漏、LED 數字錶、雙鈴鬧鐘、抱時鐘的貓與抱時鐘的柴犬。右鍵隨時切換——計時器本體不晃動，只有倒數的地方會變。",
      },
      {
        cat: "timer",
        q: "可以發出什麼聲音？",
        a: "內建「叮咚」與「鈴鈴鈴」、系統提供的 Windows 鬧鐘音效（最多十種）、自訂 .wav——或者，安靜。",
      },
      {
        cat: "timer",
        q: "重開機後計時還在嗎？",
        a: "在。運行時只記「幾點結束」——關掉程式或重開機都接得上。如果關著的時候早就到了，它會安靜地切到下一段，不補響。",
      },
      {
        cat: "langs",
        q: "內建哪些語言？",
        a: "English、繁體中文、简体中文、日本語、한국어 與 العربية。阿拉伯文把整個介面由右到左鏡像；便利貼內容逐行判斷方向——混在一張裡也正確。",
      },
      {
        cat: "langs",
        q: "可以加自己的語言嗎？",
        a: "可以。語言檔是平的 JSON：複製英文範本、翻好值、放回去——不用重新編譯。沒翻到的字自動退回英文。",
      },
      {
        cat: "general",
        q: "可以跟 Windows 一起開機啟動嗎？",
        a: "可以，系統匣選單一鍵開關。同一時間只會有一個貼貼在跑——再開一次，等同叫它新增一張便利貼。",
      },
      {
        cat: "general",
        q: "怎麼解除安裝？",
        a: "刪掉 pinpin.exe，再（願意的話）刪掉 %APPDATA%\\PinPin 資料夾。佔用就這麼多。",
      },
      {
        cat: "notes",
        q: "Vibotype 是什麼？",
        a: "同一位作者的搭配工具。在任何程式選取文字、按 ★，就會連時間與來源程式一起收進貼貼的星號記事本。",
      },
    ],
  },

  order: {
    eyebrow: "春聯訂製・承接訂單",
    title: "專屬你的金字毛筆春聯",
    sub: "挑一句經典吉祥話，或告訴我們你想寫的字。我們用大氣的金色毛筆寫在紅紙上，寄到你家門口——自用、開店、送禮都適合。",
    shipping: "支持全球寄送",
    stylesTitle: "款式",
    styles: [
      {
        name: "合體字斗方",
        desc: "四字吉祥話融成一個字，例如招財進寶、黃金萬兩，寫在菱形斗方上。",
      },
      {
        name: "直立春條",
        desc: "長條紅紙寫四個字，例如吉祥如意、歲歲平安、學業進步。",
      },
      {
        name: "單字斗方",
        desc: "一個大字——福、春、財、喜……門窗最經典的款式。",
      },
      {
        name: "自訂內容",
        desc: "店名、家訓、整副對聯都可以，告訴我們再報價。",
      },
    ],
    formTitle: "填寫需求、取得報價",
    fields: {
      name: "姓名",
      email: "Email",
      style: "款式",
      quantity: "數量",
      text: "想寫的字",
      country: "國家／地區",
      deadline: "希望到貨日",
      address: "寄送地址",
      notes: "備註",
    },
    textHint: "例如：招財進寶；也可以直接描述你的想法。",
    privacy: "這些資料只用來報價和寄送你的訂單。",
    submit: "送出需求",
    sending: "送出中…",
    success: "謝謝！我們會在 1–2 個工作天內回信報價。",
    error: "沒有送出成功，請直接寫信給我們。",
    notesTitle: "注意事項",
    notes: [
      "1–2 個工作天內回覆價格與交期，確認後才收費。",
      "每個字下筆前都會核對正確的繁體寫法。",
      "寄送有追蹤編號；海外通常 7–14 天送達。",
      "過年前要用，請至少提前三週下單。",
    ],
    mailSubject: "貼貼春聯訂製",
  },

  commission: {
    "eyebrow": "訂製服務",
    "title": "把你的毛孩，貼上桌面",
    "sub": "寄給我們一張照片、一張塗鴉或只是一個想法，我們幫你做成專屬的割形貼紙、紙膠帶或蕃茄鐘角色，直接匯入貼貼就能用。",
    "packagesTitle": "選一個方案",
    "popular": "最受歡迎",
    "packages": [
      {
        "name": "迷你貼紙包",
        "price": "$5",
        "desc": "用你的照片或畫做 5 張割形貼紙。",
        "items": [
          "5 張白邊貼紙",
          "邊緣修得乾乾淨淨",
          "可以直接匯入"
        ]
      },
      {
        "name": "貼紙包",
        "price": "$12",
        "desc": "一整張貼紙，每張便利貼都能裝飾。",
        "items": [
          "12 張貼紙",
          "2 款搭配的紙膠帶",
          "可以直接匯入"
        ]
      },
      {
        "name": "毛孩蕃茄鐘",
        "price": "$15",
        "desc": "你的毛孩抱著時鐘，變成專屬的蕃茄鐘樣式。",
        "items": [
          "1 個計時器角色",
          "3 張搭配貼紙",
          "和貓咪、柴犬計時器一樣好用"
        ]
      },
      {
        "name": "主題套組",
        "price": "$29",
        "desc": "整個桌面換上一套新風格。",
        "items": [
          "20 張貼紙",
          "3 款紙膠帶",
          "1 個計時器角色",
          "搭配的紙色"
        ]
      },
      {
        "name": "團隊版",
        "price": "來信洽談",
        "desc": "每張便利貼都有你的品牌——給工作室和團隊。",
        "items": [
          "品牌色與 Logo",
          "專屬貼紙組",
          "給團隊用的版本"
        ]
      }
    ],
    "stepsTitle": "怎麼進行",
    "steps": [
      {
        "title": "告訴我們",
        "desc": "來信說說你的想法，附上幾張照片或草圖。"
      },
      {
        "title": "報價",
        "desc": "確認內容和價格後，寄付款連結給你。"
      },
      {
        "title": "初稿",
        "desc": "大約 3 天給你看初稿，含兩次修改。"
      },
      {
        "title": "交件",
        "desc": "PNG 檔加上匯入包，丟進貼紙小舖就能用。"
      }
    ],
    "notesTitle": "小提醒",
    "notes": [
      "僅限個人使用；需要商業使用請先來信詢問。",
      "照片的權利仍然是你的；要拿你的訂單當範例，一定先問過你。",
      "大部分訂單 5–7 天完成。",
      "報價確認後透過 PayPal 或 Ko-fi 付款。"
    ],
    "ctaTitle": "有想法了嗎？",
    "ctaSub": "寫信給我們——一張照片加一句話就能開始。",
    "ctaButton": "寄信給我們",
    "copy": "複製信箱",
    "copied": "已複製信箱",
    "mailSubject": "貼貼訂製",
    "mailBody": "嗨貼貼！\n\n方案：\n想做的內容：\n\n（請附上照片或草圖）\n"
  },
};
