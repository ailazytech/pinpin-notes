import type { Dictionary } from "./en";

export const zhCN: Dictionary = {
  meta: {
    title: "贴贴 PinPin Notes — 温暖可爱的 Windows 桌面便签",
    description:
      "在 Windows 桌面上钉图钉、贴和纸胶带、玩贴纸、手写、挂透明标语、跑番茄钟——平时几乎不耗电。免费开源。",
  },

  nav: {
    order: "春联定制",
    custom: "定制",
    home: "首页",
    features: "功能",
    donate: "赞助",
    qa: "问答",
    download: "下载",
    openMenu: "打开菜单",
    closeMenu: "关闭菜单",
    github: "源代码",
    gotoSection: "前往章节",
  },

  theme: {
    toggle: "切换主题",
    light: "浅色",
    dark: "深色",
  },

  lang: {
    label: "语言",
    choose: "选择语言",
    current: "当前语言",
  },

  curtain: {
    announce: "换页中…",
    loading: "正在给桌面热身…",
    loadingLines: ["削尖图钉中…", "抚平和纸胶带…", "倒珍珠奶茶中…"],
  },

  common: {
    comingSoon: "即将推出",
    linkPending: "链接即将公布",
    backHome: "回到首页",
    tryIt: "试试看——这是真的",
    liveDemo: "互动演示",
    showScreenshot: "回到截图",
    screenshotSlot: "截图预留位",
    screenshotPath: "public/img/screens/{file}.webp",
    screenshotHint:
      "这个位置留给真实截图——在文件放进来之前，演示会一直住在这里。",
    notIncluded: "不在规划中",
    reducedMotion: "已根据系统偏好调淡动画。",
  },

  hero: {
    badge: "免费开源 · Windows 10 / 11",
    titleA: "让桌面，",
    titleB: "有纸的温度。",
    tagline: "桌面上的小纸条，一直陪着你。",
    sub: "温暖可爱的 Windows 桌面便签——图钉、和纸胶带、贴纸、手写、透明标语、番茄钟，平时几乎不耗电。",
    download: "下载 Windows 版",
    downloadNote: "单个 .exe · 约 11 MB · 免安装",
    explore: "探索功能",
    downloadPending:
      "GitHub Releases 链接整理中——链接一上线，按钮就会醒来。",
    stats: [
      { value: "0%", label: "空闲 CPU" },
      { value: "≈ 15 MB", label: "内存" },
      { value: "17", label: "种纸色" },
      { value: "6", label: "种语言" },
    ],
    sceneCaption: "贴贴的桌面一角",
    scroll: "向下滚动",
    note1: "下班后\n去喝珍珠奶茶 ♪",
    note2: "专注\n25:00",
    note3: "周日\n给妈妈打电话",
    slogan: "温柔地，\n再开始一次。",
  },

  marquee: {
    label: "146 张内置贴纸里的几张",
    hint: "把贴纸拖下来贴在任何地方 · 点一下后，滚轮转角度、Shift＋滚轮调大小",
    reset: "全部归位",
  },

  video: {
    eyebrow: "视频",
    title: "看看它动起来的样子",
    sub: "一部关于便签、和纸胶带与图钉的小短片——就钉在这里等你。",
    play: "播放宣传视频",
    makingTitle: "宣传视频重制中",
    makingDesc:
      "宣传视频正在重新制作。这个位置已接好 public/promo/pinpin-promo.mp4——把文件放进去就会自动播放，无需改动代码。",
    poster: "宣传视频海报",
  },

  overview: {
    eyebrow: "功能",
    title: "伸手可及的全部",
    sub: "十六个为桌面而生的小确幸。点击任意卡片，前往对应章节。",
    cards: [
      { id: "notes", title: "便签本体", blurb: "17 色、7 种纸质" },
      { id: "todo", title: "红笔待办", blurb: "温柔地划掉它" },
      { id: "shop", title: "贴纸商店", blurb: "图钉、夹子、和纸胶带与 146 位朋友" },
      { id: "custom", title: "自制贴纸", blurb: "任何图片，裁好白边" },
      { id: "handwriting", title: "手写", blurb: "四种笔，真实笔触" },
      { id: "transparent", title: "透明纸", blurb: "文字写在壁纸上" },
      { id: "text", title: "文字样式", blurb: "字体、颜色、最大 6 倍" },
      { id: "memories", title: "拍立得与秘密", blurb: "照片与密码" },
      { id: "pomodoro", title: "番茄钟", blurb: "六个小计时员" },
      { id: "detail-password", title: "密码便签", blurb: "伪装中的秘密" },
      { id: "detail-starred", title: "星号笔记本", blurb: "从任何地方收集 ★" },
      { id: "snap", title: "磁吸对齐", blurb: "整齐得像自己排的" },
      { id: "detail-context", title: "托盘与菜单", blurb: "随手可得" },
      { id: "detail-export", title: "导出导入", blurb: "一个 JSON 带走全部" },
      { id: "langs", title: "六种语言", blurb: "含从右到左" },
      { id: "power", title: "省电设计", blurb: "空闲 CPU 0%" },
    ],
  },

  chapters: [
    {
      id: "notes",
      eyebrow: "01",
      title: "像纸一样的便签",
      desc: "十七种纸色、七种纸质，从经典便签到牛皮纸纤维。双击直接在纸上打字，拖动右下角调整大小——把便签拿起来时，阴影也会跟着浮起，像一张被光接住的纸。",
      bullets: [
        "17 种纸色——经典 8 色、莫兰迪 8 色，外加全透明",
        "7 种纸质：便签、平滑、方格、横线、点阵、和纸纤维、牛皮纸纤维",
        "新便签会记住上一次选的颜色、纸质与钉法",
        "空白便签永远不会被自动删除——这是刻意的设计",
      ],
      media: "notes",
    },
    {
      id: "todo",
      eyebrow: "02",
      title: "用红笔划掉的待办",
      desc: "每一行左边都有个小圆圈。点一下，红笔就会把那行划掉——一笔歪歪扭扭的手写笔迹，自己画完。再点一下就取消。",
      bullets: [
        "手写感的划线动画，永远是红色",
        "右键：全部划掉、全部取消、清除已划掉的行",
        "从右到左的文字（阿拉伯文、希伯来文）从右边划过去",
        "旁边的演示可以点击——来吧",
      ],
      media: "todo",
    },
    {
      id: "shop",
      eyebrow: "03",
      title: "贴纸商店",
      desc: "一间永远免费的小店：图钉、夹子、和纸胶带，还有 146 张贴纸分成八区——日常小可爱、招财祝福（附空聚宝盆，金豆金饰一个个放进去）、各国招财小物、盆栽、金字毛笔春联、猫狗品种、文艺小摩托和台湾小物，新贴纸包还会在线送来。拖到便签上，或直接贴在桌面。",
      bullets: [
        "8 款图钉 · 7 款夹子 · 8 卷和纸胶带 · 146 张贴纸，全部免费",
        "贴纸可以不依附任何纸张，直接摆在桌面上",
        "贴上时有“落下、压一下、弹回”的动画",
        "旋转与缩放控制点；按住 Shift 让旋转一次 15°",
      ],
      media: "shop",
    },
    {
      id: "custom",
      eyebrow: "04",
      title: "自制贴纸",
      desc: "贴上任何图片，贴贴就帮你裁好——完全离线、不用 AI，只靠细致的像素功夫——再补上一圈平滑的白色裁边，连凹进去的弧线都照顾到。",
      bullets: [
        "把文件拖进来、按 Ctrl+V，或从文件夹挑选",
        "按图片边缘颜色抠图；背景太杂的照片会做成圆角照片贴纸",
        "白边会填平凹陷处，像真的裁切好的贴纸",
        "保留原图分辨率至 4096 px——分组随导出导入一起带走",
      ],
      media: "custom",
    },
    {
      id: "handwriting",
      eyebrow: "05",
      title: "在任何纸上手写",
      desc: "四种笔、四种脾气：笔尖收分的钢笔、半透明的粗荧光笔、带颗粒感的粉笔，和透出纸纹的蜡笔——每一种都以自己的笔触画在纸上。",
      bullets: [
        "8 色墨水 × 3 种粗细，还有橡皮擦与撤销",
        "笔迹在文字上层、图钉下层——拖动便签时不会闪烁",
        "Esc / Enter 结束、Ctrl+Z 撤销、E 切换橡皮擦",
        "保存前自动简化笔迹，文件很小",
      ],
      media: "handwriting",
    },
    {
      id: "transparent",
      eyebrow: "06",
      title: "写在桌面上的大字",
      desc: "把纸色设为“透明”，纸就消失了——只剩你的文字和笔迹，像直接写在壁纸上。鼠标移过去，才会浮现淡淡的虚线框，方便移动和编辑。",
      bullets: [
        "没有纸、没有阴影——只有文字和笔迹",
        "座右铭级别的大字与亮色墨水",
        "可以钉在桌面、沉到所有窗口下面",
        "鼠标移过去才出现淡淡的虚线框",
      ],
      media: "transparent",
    },
    {
      id: "text",
      eyebrow: "07",
      title: "打出你的语气",
      desc: "十种墨色加自定义取色、电脑里的每一种字体、粗体，以及七种字号（最大 6 倍）。从托盘设好全局默认，再让单张便签自由发挥。",
      bullets: [
        "A− / A+ 与 Ctrl+滚轮，每档 ±12%；纸张会自动长高",
        "托盘里的全局默认 + 每张便签的单独覆盖",
        "常用字体排在前面——微软雅黑、游ゴシック、Meiryo、맑은 고딕…",
        "一键“将这张便签的样式设为默认”",
      ],
      media: "text",
    },
    {
      id: "memories",
      eyebrow: "08",
      title: "拍立得与藏好的秘密",
      desc: "把图片拖到便签上，它会变成一张拍立得，图注用手写风格字体。给便签设置专属密码后，它会伪装成一张普通的购物清单，直到你解锁为止。",
      bullets: [
        "拍立得白框，图注用手写风格字体",
        "PBKDF2-SHA256（20 万轮迭代）+ AES-256-GCM，调用 Windows 加密 API",
        "上锁时显示伪装内容——完全看不出设有密码",
        "解锁时文字先变成乱码再复原；密码输错会摇头",
      ],
      media: "memories",
    },
    {
      id: "pomodoro",
      eyebrow: "09",
      title: "六个小计时员",
      desc: "番茄、沙漏、LED 电子表、双铃闹钟、抱着时钟的猫、抱着时钟的柴犬。挑一个最像你桌面气质的。",
      bullets: [
        "专注与休息自动轮替；完成的番茄用小红点亮起来",
        "滚轮调分钟——Shift 一次 5 分钟，1 到 180 分钟",
        "声音：内置叮咚与铃铃、Windows 闹钟音效、自己的 .wav——也可以静音",
        "只记“几点结束”：重启也接得上，过了也不补响",
      ],
      media: "pomodoro",
    },
    {
      id: "snap",
      eyebrow: "10",
      title: "自己排整齐的摆放",
      desc: "便签会轻轻磁吸——靠近其他便签或屏幕边缘 7 px 内就“咔哒”一声对齐，间距固定 10 px。把它们钉在桌面（所有窗口下面），或让某一张永远浮在最上层。",
      bullets: [
        "便签与屏幕边缘的磁吸对齐；间距固定 10 px",
        "按住 Alt 自由移动；方向键微调 1 px（Shift 一次 10 px）",
        "钉在桌面——放在所有窗口下面",
        "跑到不存在的屏幕上，下次启动自动搬回主显示器",
      ],
      media: "snap",
    },
  ],

  demos: {
    todo: {
      lines: ["回复 Mia", "给仙人掌浇水", "打印门票", "预约牙医"],
      strikeHint: "点一下任意一行",
    },
    password: {
      title: "超市采购",
      decoy: ["牛奶", "鸡蛋", "吐司"],
      secret: "蛋糕是为了 Mia 的惊喜派对！",
      hint: "演示中，任何密码都可以",
      unlock: "解锁",
      lock: "重新上锁",
      inputLabel: "密码",
    },
    pomodoro: {
      start: "开始",
      pause: "暂停",
      reset: "重置",
      focus: "专注",
      break: "休息",
      minutes: "分",
      styles: ["番茄", "沙漏", "LED", "闹钟", "猫咪", "柴犬"],
      done: "颗番茄完成",
    },
    text: {
      sample: "温柔地，再开始一次。",
      smaller: "缩小",
      larger: "放大",
      sizeLabel: "大小",
      fontLabel: "字体",
      fonts: ["Segoe Print", "Comic Sans MS", "Georgia", "Impact"],
      note: "最大可到 6×——纸张会跟着长高",
    },
    handwriting: {
      pens: ["钢笔", "荧光笔", "粉笔", "蜡笔"],
      word: "你好！",
      caption: "同一笔，四种脾气",
    },
    custom: {
      paste: "从剪贴板粘贴",
      drop: "拖一张图片进来",
      or: "或",
      arrow: "变成",
      result: "裁形贴纸",
      keepRes: "保留原始分辨率，最长边 4096 px",
    },
    shop: {
      pins: "图钉",
      clips: "夹子",
      tapes: "和纸胶带",
      stickers: "贴纸",
      free: "全部免费",
      shelf: "货架",
    },
    snap: {
      hint: "拖拖看黄油色的那张——它会磁吸。",
      alt: "按住 Alt 可暂时不吸",
      gap: "10 px",
    },
    transparent: {
      hint: "鼠标移过会出现虚线框",
      motto: "温柔地，再开始一次。",
    },
    notes: {
      colorsLabel: "17 种纸色",
      texturesLabel: "7 种纸质",
      transparent: "透明",
    },
  },

  details: {
    eyebrow: "小细节",
    title: "小地方，打磨到发亮",
    sub: "不太上得了头条、却让每一天更顺手的功能。",
    cards: [
      {
        id: "detail-snap",
        title: "磁吸摆放",
        desc: "靠近其他便签或屏幕边缘 7 px 内会轻轻吸住对齐，自动留出 10 px 间距。按住 Alt 不吸；方向键微调 1 px（Shift 10 px）。",
      },
      {
        id: "detail-starred",
        title: "星号笔记本",
        desc: "搭配 Vibotype：在任何程序里选中文字、按下 ★，就会连同时间与来源程序一起收进同一本。",
      },
      {
        id: "detail-export",
        title: "一个文件带走全部",
        desc: "把所有便签、图片、手写、自制贴纸与分组导出成一个 JSON——加密的保持加密。导入时温柔合并。",
      },
      {
        id: "detail-undo",
        title: "温柔的撤销",
        desc: "删掉的便签会在回收站休息（保留最近 20 张）——随时可以从托盘救回来。",
      },
      {
        id: "detail-context",
        title: "就在你手边",
        desc: "桌面右键就能新建便签与番茄钟；其余交给托盘菜单。开机自启一键开关，而且永远只有一个贴贴在运行。",
      },
      {
        id: "detail-langs",
        title: "六种语言，完整镜像",
        desc: "English、繁体中文、简体中文、日本語、한국어 与 العربية——为从右到左的读者提供整面镜像的界面。",
      },
    ],
  },

  power: {
    eyebrow: "省电",
    title: "像一张纸一样安静",
    desc: "永远没有后台计时器。只有内容改变时才会绘制——动画播完就停、位图画完立即释放、贴纸首次使用时才解码。",
    stats: [
      { value: "0%", label: "空闲 CPU" },
      { value: "≈ 15 MB", label: "内存" },
      { value: "11 MB", label: "单个 .exe" },
      { value: "0", label: "次网络请求" },
    ],
    footnote: "在安静的桌面、常驻几张便签的环境下实测。",
    points: [
      "空闲时没有任何计时器在运行",
      "动画结束——就停止绘制",
      "只有鼠标真的在动时才重绘",
      "分层窗口把画好的像素交给系统合成",
    ],
  },

  langs: {
    eyebrow: "语言",
    title: "六种语言，宾至如归",
    desc: "内置 English、繁体中文、简体中文、日本語、한국어 与 العربية。阿拉伯文会把整个界面左右镜像——菜单、贴纸商店，全部；便签内容则逐行自动判断方向，混着写也正确。",
    customTitle: "带上你自己的语言",
    customSub: "语言文件是一份平易近人的 JSON。复制模板、翻译右边的值，保存后就会出现在菜单里——无需重新编译。",
    customSteps: [
      "从托盘打开自定义语言文件夹",
      "复制英文模板、翻译右边的值",
      "保存——立刻出现在语言菜单里",
      "没翻到的词条自动回退到英文",
    ],
    jsonTitle: "a-little-türkçe.json",
    jsonLines: [
      "{",
      '  "menu.new_note": "Yeni not",',
      '  "menu.pomodoro": "Yeni pomodoro sayacı",',
      '  "menu.sticker_shop": "Çıkartma dükkânı"',
      "}",
    ],
    rtlTitle: "从右到左，同样到位",
    rtlSample: "每一行都从右边读起——菜单、和纸胶带，全部。",
    lineDirection: "每一行各自找到方向，混在一张便签里也没问题。",
  },

  download: {
    eyebrow: "下载",
    title: "带它回你的桌面",
    sub: "一个小文件。免安装、免账号、不联网——只是一口袋的纸。",
    button: "下载 pinpin.exe",
    size: "约 11 MB",
    pending: "GitHub Releases 链接整理中——链接一上线，按钮就会醒来。",
    reqTitle: "系统需求与须知",
    reqs: [
      "Windows 10 或 11（64 位）",
      "单个 pinpin.exe——免安装、免 .NET、免运行时",
      "没有账号、没有云端、不使用网络",
      "数据存放在 %APPDATA%\\PinPin\\",
      "MIT 许可——永远免费",
    ],
    license: "以 MIT 许可证发布。",
    sourcePending: "源代码链接即将公布。",
  },

  footer: {
    tagline: "由纸张、图钉和一支小红笔做成。",
    siteTitle: "网站",
    projectTitle: "项目",
    languageTitle: "语言",
    links: {
      order: "春联定制",
      custom: "定制服务",
      home: "首页",
      qa: "问答",
      donate: "赞助",
      download: "下载",
      source: "源代码（即将）",
      license: "MIT 许可证",
    },
    copyright: "贴贴 · MIT 许可，自由免费",
    crafted: "用心制作",
  },

  donate: {
    eyebrow: "赞助",
    title: "请贴贴喝杯珍珠奶茶",
    sub: "贴贴免费、开源，并将一直如此——没有广告、没有云端费用、没有账号。如果它让你的桌面更舒服，一份小点心能让图钉一直发亮。",
    tiersTitle: "挑一份点心",
    tiers: [
      { name: "一张小贴纸", amount: "$1", desc: "一点小闪光——谢谢你！" },
      { name: "一杯珍珠奶茶", amount: "$3", desc: "经典款。珍珠多一点，开心多一点。" },
      { name: "咖啡与阳光", amount: "$5", desc: "一个暖洋洋的写代码午后。" },
      { name: "下午茶套餐", amount: "$10", desc: "一整盘新贴纸。" },
    ],
    customLabel: "或自定义你的心意",
    customPlaceholder: "输入金额",
    customUnit: "美元",
    methodsTitle: "挑一种说谢谢的方式",
    methodsPending: "支付渠道筹备中——很快就会出现在这里。",
    methods: [
      { name: "PayPal", note: "链接即将公布" },
      { name: "Ko-fi", note: "链接即将公布" },
      { name: "GitHub Sponsors", note: "链接即将公布" },
    ],
    otherTitle: "其他帮忙的方式",
    others: [
      { title: "去 GitHub 加一颗星", desc: "一颗星不花一分钱，却能让项目发亮。" },
      { title: "告诉朋友", desc: "舒服的桌面值得分享——当然是数字方式的那种。" },
      { title: "帮忙翻译", desc: "自定义语言文件就是 JSON；欢迎把你翻译的版本寄来。" },
      { title: "反馈小褶皱", desc: "发现 bug？一份清晰的反馈比皱眉有用多了。" },
    ],
    thanks: "谢谢你——真的。每一份心意都会变成贴纸、像素和珍珠奶茶。",
    hearts: "满满的爱",
  },

  qa: {
    eyebrow: "问答",
    title: "问答，温柔解答",
    sub: "钉下第一张便签之前，大家最常问的事。",
    searchPlaceholder: "搜索问题…",
    noResults: "暂时没有匹配的结果——换个关键词试试？",
    categories: {
      all: "全部",
      general: "一般",
      notes: "便签与贴纸",
      data: "数据与隐私",
      timer: "计时器",
      langs: "语言",
    },
    items: [
      {
        cat: "general",
        q: "贴贴真的免费吗？",
        a: "是的——MIT 开源许可。使用、分享、修改都免费。没有广告、没有账号、没有付费升级。",
      },
      {
        cat: "general",
        q: "支持哪些系统？",
        a: "Windows 10 与 Windows 11（64 位）。它是单个 pinpin.exe（约 11 MB）——免安装、免 .NET 或其他运行时。Mac、Linux 和手机版不在规划中。",
      },
      {
        cat: "general",
        q: "去哪里下载？",
        a: "GitHub Releases 链接正在整理。链接一公布，网站上的下载按钮就会醒来。",
      },
      {
        cat: "general",
        q: "会很耗 CPU 或内存吗？",
        a: "它天生省电：常驻几张便签约 15 MB 内存，空闲时 CPU 0%——没有内容改变，就不重绘。",
      },
      {
        cat: "data",
        q: "有云端同步吗？",
        a: "没有——贴贴完全不碰网络。要搬到另一台电脑，在一台导出 JSON、到另一台导入即可；加密的便签保持加密。",
      },
      {
        cat: "data",
        q: "我的数据存在哪里？",
        a: "全部存放在 %APPDATA%\\PinPin\\——notes.json、图片、自制贴纸、设置与自定义语言文件。删掉这个文件夹就能清理得干干净净。",
      },
      {
        cat: "data",
        q: "我不小心删掉了一张便签！",
        a: "删掉的便签会进回收站，保留最近 20 张。从托盘菜单选“撤销删除的便签”即可找回。",
      },
      {
        cat: "data",
        q: "密码便签是怎么工作的？",
        a: "每张便签都可以有自己的密码。内容用 PBKDF2-SHA256（20 万轮迭代）+ AES-256-GCM 加密，调用 Windows 内置加密 API。上锁时显示伪装的待办清单；密钥只存在内存中，退出时自动重新上锁。",
      },
      {
        cat: "notes",
        q: "可以自制贴纸吗？",
        a: "可以——把图片拖进贴纸商店、按 Ctrl+V，或挑选文件。贴贴按边缘颜色抠图（离线、不用 AI）、加上白色裁边，分辨率保留到 4096 px。",
      },
      {
        cat: "notes",
        q: "手写能做到什么？",
        a: "四种笔——钢笔、荧光笔、粉笔、蜡笔——8 色 3 种粗细，附橡皮擦与撤销。笔迹在文字上、图钉下；保存时自动简化，文件很小。",
      },
      {
        cat: "notes",
        q: "透明便签是什么？",
        a: "选择透明纸色后，便签没有纸也没有阴影——只剩文字和笔迹。鼠标移过会出现淡淡的虚线框与按钮。也可以钉在所有窗口下面。",
      },
      {
        cat: "timer",
        q: "番茄钟有几种样式？",
        a: "六种：番茄、沙漏、LED 电子表、双铃闹钟、抱时钟的猫与抱时钟的柴犬。右键随时切换——计时器本体不晃动，只有倒计时的部分会变。",
      },
      {
        cat: "timer",
        q: "能发出什么声音？",
        a: "内置“叮咚”与“铃铃铃”、系统提供的 Windows 闹钟音效（最多十种）、自定义 .wav——或者，安静。",
      },
      {
        cat: "timer",
        q: "重启后计时还在吗？",
        a: "在。运行期间只记“几点结束”——关掉程序或重启电脑都能接上。如果关机期间时间早就过了，它会安静地翻篇，不补响。",
      },
      {
        cat: "langs",
        q: "内置哪些语言？",
        a: "English、繁体中文、简体中文、日本語、한국어 与 العربية。阿拉伯文把整个界面从右到左镜像；便签内容逐行判断方向——混在一张里也正确。",
      },
      {
        cat: "langs",
        q: "可以添加自己的语言吗？",
        a: "可以。语言文件是扁平的 JSON：复制英文模板、翻好值、放回去——不用重新编译。没翻到的词条自动回退到英文。",
      },
      {
        cat: "general",
        q: "可以随 Windows 开机自启吗？",
        a: "可以，托盘菜单一键开关。同一时间只会运行一个贴贴——再次打开，等同于让它新建一张便签。",
      },
      {
        cat: "general",
        q: "怎么卸载？",
        a: "删掉 pinpin.exe，再（愿意的话）删掉 %APPDATA%\\PinPin 文件夹。占用就这么多。",
      },
      {
        cat: "notes",
        q: "Vibotype 是什么？",
        a: "同一位作者的搭档工具。在任何程序里选中文字、按下 ★，就会连同时间与来源程序一起收进贴贴的星号笔记本。",
      },
    ],
  },

  order: {
    eyebrow: "春联定制・承接订单",
    title: "专属你的金字毛笔春联",
    sub: "挑一句经典吉祥话，或告诉我们你想写的字。我们用大气的金色毛笔写在红纸上，寄到你家门口——自用、开店、送礼都合适。",
    shipping: "支持全球寄送",
    stylesTitle: "款式",
    styles: [
      {
        name: "合体字斗方",
        desc: "四字吉祥话融成一个字，例如招财进宝、黄金万两，写在菱形斗方上。",
      },
      {
        name: "竖版春条",
        desc: "长条红纸写四个字，例如吉祥如意、岁岁平安、学业进步。",
      },
      {
        name: "单字斗方",
        desc: "一个大字——福、春、财、喜……门窗最经典的款式。",
      },
      {
        name: "自定内容",
        desc: "店名、家训、整副对联都可以，告诉我们再报价。",
      },
    ],
    formTitle: "填写需求、获取报价",
    fields: {
      name: "姓名",
      email: "Email",
      style: "款式",
      quantity: "数量",
      text: "想写的字",
      country: "国家／地区",
      deadline: "希望到货日",
      address: "寄送地址",
      notes: "备注",
    },
    textHint: "例如：招财进宝；也可以直接描述你的想法。",
    privacy: "这些资料只用来报价和寄送你的订单。",
    submit: "提交需求",
    sending: "提交中…",
    success: "谢谢！我们会在 1–2 个工作日内回信报价。",
    error: "没有提交成功，请直接写信给我们。",
    notesTitle: "注意事项",
    notes: [
      "1–2 个工作日内回复价格与交期，确认后才收费。",
      "每个字下笔前都会核对正确写法。",
      "寄送有追踪单号；海外通常 7–14 天送达。",
      "春节前要用，请至少提前三周下单。",
    ],
    mailSubject: "贴贴春联定制",
  },

  commission: {
    "eyebrow": "定制服务",
    "title": "把你的毛孩子，贴上桌面",
    "sub": "发给我们一张照片、一幅涂鸦或者只是一个想法，我们帮你做成专属的异形贴纸、和纸胶带或番茄钟角色，直接导入贴贴就能用。",
    "packagesTitle": "选一个方案",
    "popular": "最受欢迎",
    "packages": [
      {
        "name": "迷你贴纸包",
        "price": "$5",
        "desc": "用你的照片或画做 5 张异形贴纸。",
        "items": [
          "5 张白边贴纸",
          "边缘修得干干净净",
          "可以直接导入"
        ]
      },
      {
        "name": "贴纸包",
        "price": "$12",
        "desc": "一整张贴纸，每张便签都能装饰。",
        "items": [
          "12 张贴纸",
          "2 款搭配的和纸胶带",
          "可以直接导入"
        ]
      },
      {
        "name": "萌宠番茄钟",
        "price": "$15",
        "desc": "你的宠物抱着时钟，变成专属的番茄钟样式。",
        "items": [
          "1 个计时器角色",
          "3 张搭配贴纸",
          "和猫咪、柴犬计时器一样好用"
        ]
      },
      {
        "name": "主题套装",
        "price": "$29",
        "desc": "整个桌面换上一套新风格。",
        "items": [
          "20 张贴纸",
          "3 款和纸胶带",
          "1 个计时器角色",
          "搭配的纸色"
        ]
      },
      {
        "name": "团队版",
        "price": "来信洽谈",
        "desc": "每张便签都有你的品牌——给工作室和团队。",
        "items": [
          "品牌色与 Logo",
          "专属贴纸组",
          "给团队用的版本"
        ]
      }
    ],
    "stepsTitle": "流程",
    "steps": [
      {
        "title": "告诉我们",
        "desc": "来信说说你的想法，附上几张照片或草图。"
      },
      {
        "title": "报价",
        "desc": "确认内容和价格后，发付款链接给你。"
      },
      {
        "title": "初稿",
        "desc": "大约 3 天给你看初稿，含两次修改。"
      },
      {
        "title": "交付",
        "desc": "PNG 文件加上导入包，放进贴纸商店就能用。"
      }
    ],
    "notesTitle": "小提醒",
    "notes": [
      "仅限个人使用；需要商业使用请先来信询问。",
      "照片的权利仍然是你的；要拿你的订单当示例，一定先征得你同意。",
      "大部分订单 5–7 天完成。",
      "报价确认后通过 PayPal 或 Ko-fi 付款。"
    ],
    "ctaTitle": "有想法了吗？",
    "ctaSub": "给我们写信——一张照片加一句话就能开始。",
    "ctaButton": "给我们写信",
    "copy": "复制邮箱",
    "copied": "已复制邮箱",
    "mailSubject": "贴贴定制",
    "mailBody": "你好贴贴！\n\n方案：\n想做的内容：\n\n（请附上照片或草图）\n"
  },
};
