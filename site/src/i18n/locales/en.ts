// English dictionary — the source of truth for the Dictionary shape.
// Every other locale file must mirror these keys exactly.

export const en = {
  meta: {
    title: "PinPin Notes — Cozy sticky notes for your Windows desktop",
    description:
      "Pins, washi tape, stickers, handwriting, transparent slogans and pomodoro timers on your Windows desktop — with almost zero idle CPU. Free and open source.",
  },

  nav: {
    order: "Couplets",
    custom: "Custom Stickers",
    home: "Home",
    features: "Features",
    donate: "Donate",
    qa: "Q&A",
    download: "Download",
    openMenu: "Open menu",
    closeMenu: "Close menu",
    github: "Source code",
    gotoSection: "Go to section",
  },

  theme: {
    toggle: "Toggle theme",
    light: "Light",
    dark: "Dark",
  },

  lang: {
    label: "Language",
    choose: "Choose language",
    current: "Current language",
  },

  curtain: {
    announce: "Changing page…",
    loading: "Warming up the desk…",
    loadingLines: [
      "Sharpening the pins…",
      "Flattening the washi tape…",
      "Pouring the boba…",
    ],
  },

  common: {
    comingSoon: "Coming soon",
    linkPending: "Link coming soon",
    backHome: "Back to home",
    tryIt: "Try it — it's live",
    liveDemo: "Live demo",
    showScreenshot: "Back to screenshot",
    screenshotSlot: "Screenshot slot",
    screenshotPath: "public/img/screens/{file}.webp",
    screenshotHint:
      "This slot is reserved for the real capture — the demo stays until the file lands here.",
    notIncluded: "Not on the roadmap",
    reducedMotion: "Animations are toned down per your system preference.",
  },

  hero: {
    badge: "Free & open source · Windows 10 / 11",
    titleA: "A desk that feels",
    titleB: "like paper.",
    tagline: "Little notes that stick with you.",
    sub: "Cozy sticky notes for your Windows desktop — pins, washi tape, stickers, handwriting, transparent slogans and pomodoro timers, with almost zero idle CPU.",
    download: "Download for Windows",
    downloadNote: "Single .exe · about 11 MB · no installer",
    explore: "Explore features",
    downloadPending:
      "GitHub Releases link is being finalized — star the repo to get notified on release.",
    stats: [
      { value: "0%", label: "idle CPU" },
      { value: "≈ 15 MB", label: "memory" },
      { value: "17", label: "paper colors" },
      { value: "6", label: "languages" },
    ],
    sceneCaption: "A corner of a PinPin desktop",
    scroll: "Scroll",
    note1: "buy boba\nafter work ♪",
    note2: "focus\n25:00",
    note3: "call mom\non Sunday",
    slogan: "begin again,\ngently.",
  },

  marquee: {
    label: "Some of the 146 built-in stickers",
    hint: "Drag one out and stick it anywhere · click it, then scroll to rotate, Shift + scroll to resize",
    reset: "Reset",
  },

  video: {
    eyebrow: "Film",
    title: "See it in motion",
    sub: "A short film about notes, tapes and pins — pinned to this very spot.",
    play: "Play the promo film",
    makingTitle: "Film in the making",
    makingDesc:
      "The promo video is being remade. This slot is already wired to public/promo/pinpin-promo.mp4 — drop the file in and it plays, no code needed.",
    poster: "Promo film poster",
  },

  overview: {
    eyebrow: "Features",
    title: "Everything within arm's reach",
    sub: "Sixteen little joys for your desktop. Tap any card to visit its chapter.",
    cards: [
      { id: "notes", title: "Paper notes", blurb: "17 colors, 7 textures" },
      { id: "todo", title: "Red-pen todos", blurb: "Strike it done, sweetly" },
      { id: "shop", title: "Sticker Shop", blurb: "Pins, clips, tape & 146 friends" },
      { id: "custom", title: "Custom stickers", blurb: "Any picture, die-cut" },
      { id: "handwriting", title: "Handwriting", blurb: "Four pens, real texture" },
      { id: "transparent", title: "Transparent paper", blurb: "Words on the wallpaper" },
      { id: "text", title: "Text styling", blurb: "Fonts, colors, 6× sizes" },
      { id: "memories", title: "Polaroid & secrets", blurb: "Photos and passwords" },
      { id: "pomodoro", title: "Pomodoro timers", blurb: "Six little timekeepers" },
      { id: "detail-password", title: "Password notes", blurb: "Secrets in disguise" },
      { id: "detail-starred", title: "Starred notebook", blurb: "Collect ★ from anywhere" },
      { id: "snap", title: "Snapping", blurb: "Magnetic, tidy placement" },
      { id: "detail-context", title: "Tray & menus", blurb: "Always at hand" },
      { id: "detail-export", title: "Export / Import", blurb: "One JSON carries all" },
      { id: "langs", title: "Six languages", blurb: "RTL included" },
      { id: "power", title: "Power sipping", blurb: "0% idle CPU, ~15 MB memory" },
    ],
  },

  chapters: [
    {
      id: "notes",
      eyebrow: "01",
      title: "Notes that feel like paper",
      desc: "Seventeen paper colors and seven textures, from classic sticky pads to kraft fiber. Double-click to type right on the paper, drag the corner to resize — and when you lift a note, its shadow lifts with it, like paper caught in the light.",
      bullets: [
        "17 colors — classic 8, Morandi 8, plus fully transparent",
        "7 textures: sticky, smooth, grid, lined, dotted, washi fiber, kraft fiber",
        "New notes remember your last color, texture and pin",
        "Blank notes are never auto-deleted — a deliberate choice",
      ],
      media: "notes",
    },
    {
      id: "todo",
      eyebrow: "02",
      title: "Todos, struck through in red",
      desc: "Every line starts with a little circle. Click it and a red pen crosses the line out — a wobbly, handwritten stroke that draws itself. Click again to undo.",
      bullets: [
        "Handwritten strike-through animation, always in red",
        "Right-click: strike all, unstrike all, or clear the struck lines",
        "Right-to-left lines (Arabic, Hebrew) strike from the right",
        "The demo beside is clickable — go on",
      ],
      media: "todo",
    },
    {
      id: "shop",
      eyebrow: "03",
      title: "The Sticker Shop",
      desc: "A little shop where everything is free: pins, clips, washi tape and 146 stickers in eight sets — everyday cuties, good fortune (with empty treasure bowls to fill with gold), lucky charms from around the world, houseplants, gold-calligraphy spring couplets, cat & dog breeds, little scooters and Taiwan favorites. New packs arrive online. Drag them onto notes, or straight onto the desktop.",
      bullets: [
        "8 pins · 7 clips · 8 washi tapes · 146 stickers, all free",
        "Stickers can live on the desktop without any paper at all",
        "Drop animation lands with a squash and a bounce",
        "Rotate and scale handles; Shift snaps rotation to 15°",
      ],
      media: "shop",
    },
    {
      id: "custom",
      eyebrow: "04",
      title: "Make your own stickers",
      desc: "Paste any picture and PinPin cuts it out for you — entirely offline, no AI, just careful pixel work — then adds a smooth die-cut white edge, the kind that follows every curve.",
      bullets: [
        "Drop a file in, press Ctrl+V, or pick from a folder",
        "Background removed from edge colors; busy photos become rounded photo-stickers",
        "The white edge fills concave curves like a real cut sticker",
        "Original resolution kept up to 4096 px — groups travel with export",
      ],
      media: "custom",
    },
    {
      id: "handwriting",
      eyebrow: "05",
      title: "Handwrite on everything",
      desc: "Four pens, four tempers: a fountain pen with tapered ends, a fat translucent highlighter, dusty chalk and waxy crayon — each rendered with its own texture, right on the note.",
      bullets: [
        "8 ink colors × 3 widths, plus eraser and undo",
        "Ink sits above the text and below the pins — no flicker while dragging",
        "Esc / Enter to finish, Ctrl+Z to undo, E toggles the eraser",
        "Strokes are simplified on save, so files stay tiny",
      ],
      media: "handwriting",
    },
    {
      id: "transparent",
      eyebrow: "06",
      title: "Invisible paper for big words",
      desc: "Choose transparent paper and the note disappears — leaving only your words and ink, as if written on the wallpaper itself. Hover, and a faint dashed frame appears so you can still move things.",
      bullets: [
        "No paper, no shadow — only text and handwriting",
        "Motto-sized typography in bright inks",
        "Pin it beneath all windows to live on the desktop",
        "A faint dashed frame appears only on hover",
      ],
      media: "transparent",
    },
    {
      id: "text",
      eyebrow: "07",
      title: "Type in your own voice",
      desc: "Ten ink colors plus a custom picker, every font your PC has, bold, and seven sizes up to a six-times giant. Set a global default from the tray, then let individual notes stray.",
      bullets: [
        "A− / A+ and Ctrl+wheel resize in 12% steps; paper grows to fit",
        "Global defaults from the tray, per-note overrides via right-click",
        "Familiar fonts listed first — JhengHei, Yu Gothic, Meiryo, Malgun…",
        "“Set this note's style as default” in one click",
      ],
      media: "text",
    },
    {
      id: "memories",
      eyebrow: "08",
      title: "Polaroids & well-kept secrets",
      desc: "Drop an image onto a note and it becomes a polaroid with a handwritten caption. Give a note its own password and it disguises itself as an ordinary grocery list until you unlock it.",
      bullets: [
        "Polaroid frame with a caption in a handwriting font",
        "PBKDF2-SHA256 (200k rounds) + AES-256-GCM via Windows crypto",
        "Locked notes show a decoy list — no hint that a password exists",
        "Unlocking scrambles into the real text; wrong passwords shake",
      ],
      media: "memories",
    },
    {
      id: "pomodoro",
      eyebrow: "09",
      title: "Six little timekeepers",
      desc: "A tomato, an hourglass, an LED watch, a twin-bell alarm clock, a cat with a clock and a shiba with a clock. Pick whichever matches the mood of your desk.",
      bullets: [
        "Focus and break alternate automatically; finished tomatoes glow as dots",
        "Set minutes with the wheel — Shift for fives, 1 to 180 minutes",
        "Sounds: built-in chimes, Windows alarms, your own .wav — or silence",
        "Remembers only the finish time: survives reboot, never over-rings",
      ],
      media: "pomodoro",
    },
    {
      id: "snap",
      eyebrow: "10",
      title: "Placement that tidies itself",
      desc: "Notes gently magnetize — within 7 px of a neighbor or the screen edge they click into alignment, spacing themselves 10 px apart. Pin them to the desktop, beneath every window, or keep one floating above it all.",
      bullets: [
        "Magnetic snap to notes and screen edges; 10 px even spacing",
        "Hold Alt to float free; arrow keys nudge by 1 px (Shift for 10)",
        "Pin to the desktop — beneath every window",
        "Notes lost off-screen drift back to the main monitor",
      ],
      media: "snap",
    },
  ],

  demos: {
    todo: {
      lines: ["reply to Mia", "water the cactus", "print the tickets", "book the dentist"],
      strikeHint: "click a line",
    },
    password: {
      title: "groceries",
      decoy: ["milk", "eggs", "bread"],
      secret: "the cake is for Mia's surprise party!",
      hint: "any password works in this demo",
      unlock: "Unlock",
      lock: "Lock again",
      inputLabel: "Password",
    },
    pomodoro: {
      start: "Start",
      pause: "Pause",
      reset: "Reset",
      focus: "Focus",
      break: "Break",
      minutes: "min",
      styles: ["Tomato", "Hourglass", "LED", "Alarm", "Cat", "Shiba"],
      done: "tomatoes done",
    },
    text: {
      sample: "Begin again, gently.",
      smaller: "Smaller",
      larger: "Larger",
      sizeLabel: "Size",
      fontLabel: "Font",
      fonts: ["Segoe Print", "Comic Sans MS", "Georgia", "Impact"],
      note: "up to 6× — the paper grows to fit",
    },
    handwriting: {
      pens: ["Fountain", "Highlighter", "Chalk", "Crayon"],
      word: "hello!",
      caption: "one stroke, four tempers",
    },
    custom: {
      paste: "Paste from clipboard",
      drop: "Drop an image",
      or: "or",
      arrow: "becomes",
      result: "die-cut sticker",
      keepRes: "original resolution, up to 4096 px",
    },
    shop: {
      pins: "pins",
      clips: "clips",
      tapes: "washi tapes",
      stickers: "stickers",
      free: "all free",
      shelf: "the shelf",
    },
    snap: {
      hint: "Drag the butter note — it magnetizes.",
      alt: "hold Alt to ignore snapping",
      gap: "10 px",
    },
    transparent: {
      hint: "hover to reveal the dashed frame",
      motto: "begin again, gently.",
    },
    notes: {
      colorsLabel: "17 colors",
      texturesLabel: "7 textures",
      transparent: "transparent",
    },
  },

  details: {
    eyebrow: "Small details",
    title: "The little things, polished",
    sub: "The features that rarely make headlines but make every day smoother.",
    cards: [
      {
        id: "detail-snap",
        title: "Magnetic placement",
        desc: "Notes gently snap within 7 px of neighbors and screen edges, spacing themselves 10 px apart. Hold Alt to float free; arrow keys nudge 1 px (Shift for 10).",
      },
      {
        id: "detail-starred",
        title: "Starred notebook",
        desc: "Paired with Vibotype: select text in any app, press ★, and it lands in one notebook with a timestamp, the source program and its window title.",
      },
      {
        id: "detail-export",
        title: "One-file everything",
        desc: "Export all notes, images, handwriting, custom stickers and groups into a single JSON — encrypted things stay encrypted. Import merges gently.",
      },
      {
        id: "detail-undo",
        title: "Gentle undo",
        desc: "Deleted notes rest in a recycle bin that keeps the last 20 — restore them any time from the tray.",
      },
      {
        id: "detail-context",
        title: "Right where you are",
        desc: "Turn on the desktop right-click menu in the tray to create notes and timers from the desktop; the tray menu does everything else. Autostart is one click too (both are off by default), and only one PinPin ever runs.",
      },
      {
        id: "detail-langs",
        title: "Six languages, mirrored",
        desc: "English, 繁體中文， 简体中文， 日本語， 한국어 and العربية — with the whole interface mirrored for right-to-left readers.",
      },
    ],
  },

  power: {
    eyebrow: "Power",
    title: "Just like real paper, it sits there quietly",
    desc: "No polling, no idle spinning. PinPin paints only when something changes — animations stop the moment they finish, bitmaps are released as soon as they're drawn, and stickers only decode when first used.",
    stats: [
      { value: "0%", label: "idle CPU" },
      { value: "≈ 15 MB", label: "memory" },
      { value: "11 MB", label: "single .exe" },
      { value: "≤ 1", label: "update check a day (optional)" },
    ],
    footnote: "Measured on a quiet desktop with a few notes pinned.",
    points: [
      "Nothing polls or spins while idle",
      "Animations end — and stop drawing",
      "Repaints only while the mouse actually moves",
      "Layered windows hand the pixels to the system",
    ],
  },

  langs: {
    eyebrow: "Languages",
    title: "At home in six languages",
    desc: "English, 繁體中文， 简体中文， 日本語， 한국어 and العربية ship built-in. Arabic mirrors the entire interface — menus, the Sticker Shop, everything — while note content detects its direction line by line, even when mixed.",
    customTitle: "Bring your own language",
    customSub: "The language file is a flat, friendly JSON. Copy the template, translate the values, and it appears in the menu — no recompiling.",
    customSteps: [
      "Open the custom-language folder from the tray menu",
      "Copy the English template and translate the values",
      "Save — it shows up in the language menu instantly",
      "Missing keys quietly fall back to English",
    ],
    jsonTitle: "a-little-türkçe.json",
    jsonLines: [
      '{',
      '  "menu.new_note": "Yeni not",',
      '  "menu.pomodoro": "Yeni pomodoro sayacı",',
      '  "menu.sticker_shop": "Çıkartma dükkânı"',
      '}',
    ],
    rtlTitle: "Right-to-left, properly",
    rtlSample: "Everything reads from the right — menus, tapes and all.",
    lineDirection: "Each line finds its own direction, even mixed in one note.",
  },

  download: {
    eyebrow: "Download",
    title: "Take it to your desktop",
    sub: "One small file. No installer, no account — it just sits quietly in a little corner, keeping you warm.",
    button: "Download pinpin.exe",
    size: "about 11 MB",
    pending: "The GitHub Releases link is being finalized — the button wakes up the moment it lands.",
    reqTitle: "Requirements & privacy",
    reqs: [
      "Windows 10 or 11, 64-bit",
      "A single pinpin.exe — no installer, no .NET, no runtimes",
      "No account, no cloud — your notes never leave your PC",
      "At most one update check a day; turn it off from the tray",
      "Your data lives in %APPDATA%\\PinPin\\",
      "MIT License — free, forever",
    ],
    license: "Released under the MIT License.",
    sourcePending: "Source code link coming soon.",
  },

  footer: {
    tagline: "Made of paper, pins and a little red pen.",
    siteTitle: "Site",
    projectTitle: "Project",
    languageTitle: "Language",
    links: {
      order: "Couplet orders",
      custom: "Custom stickers",
      home: "Home",
      qa: "Q&A",
      donate: "Donate",
      download: "Download",
      source: "Source code (soon)",
      license: "MIT License",
    },
    copyright: "PinPin · free and open source under MIT",
    crafted: "Crafted with care",
  },

  donate: {
    eyebrow: "Donate",
    title: "Buy PinPin a boba",
    sub: "PinPin is free, open source, and stays that way — no third-party ads, no cloud fees, no accounts. If it makes your desk feel cozier, a small treat keeps the pins polished.",
    tiersTitle: "Pick a treat",
    tiers: [
      { name: "A little sticker", amount: "$1", desc: "A tiny sparkle — thank you!" },
      { name: "A cup of boba", amount: "$3", desc: "The classic. Extra pearls, extra happy." },
      { name: "Coffee & sunshine", amount: "$5", desc: "One warm afternoon of coding." },
      { name: "Afternoon-tea set", amount: "$10", desc: "A whole tray of new stickers." },
    ],
    customLabel: "Or name your own treat",
    customPlaceholder: "Enter an amount",
    customUnit: "USD",
    methodsTitle: "Pick a way to say thanks",
    methodsPending: "Payment links are being set up — they'll appear here soon.",
    methods: [
      { name: "PayPal", note: "link coming soon" },
      { name: "Ko-fi", note: "link coming soon" },
      { name: "GitHub Sponsors", note: "link coming soon" },
    ],
    otherTitle: "Other ways to help",
    others: [
      { title: "Star it on GitHub", desc: "A star costs nothing and brightens the repo." },
      { title: "Tell a friend", desc: "Cozy desks are better shared — digitally, of course." },
      { title: "Help translate", desc: "Custom language files are just JSON; send yours in." },
      { title: "Report a wrinkle", desc: "Found a bug? A crisp report helps more than a frown." },
    ],
    thanks: "Thank you — truly. Every treat goes into stickers, pixels and bubble tea.",
    hearts: "with love",
  },

  qa: {
    eyebrow: "Q&A",
    title: "Questions, answered gently",
    sub: "Everything people usually ask before pinning their first note.",
    searchPlaceholder: "Search a question…",
    noResults: "Nothing matches that yet — try another word?",
    categories: {
      all: "All",
      general: "General",
      notes: "Notes & stickers",
      data: "Data & privacy",
      timer: "Timers",
      langs: "Languages",
    },
    items: [
      {
        cat: "general",
        q: "Is PinPin really free?",
        a: "Yes — MIT open source. Free to use, share and modify. No third-party ads, no accounts, no upsells.",
      },
      {
        cat: "general",
        q: "Which systems does it run on?",
        a: "Windows 10 and Windows 11, 64-bit. It's a single pinpin.exe of about 11 MB — no installer, no .NET or other runtimes. Mac, Linux and phone versions aren't planned.",
      },
      {
        cat: "general",
        q: "Where do I download it?",
        a: "The GitHub Releases link is being finalized. The download button on this site will wake up the moment it's published.",
      },
      {
        cat: "general",
        q: "Does it use much CPU or memory?",
        a: "It's built to sip: about 15 MB of memory with a few notes pinned, and 0% CPU while idle — nothing repaints unless something actually changes.",
      },
      {
        cat: "data",
        q: "Is there cloud sync?",
        a: "No. Your notes stay on your PC and are never uploaded. PinPin checks for updates at most once a day (with anonymous usage stats); both can be turned off from the tray. To move to another PC, export a JSON file on one and import it on the other; encrypted notes stay encrypted.",
      },
      {
        cat: "data",
        q: "Where is my data stored?",
        a: "Everything lives in %APPDATA%\\PinPin\\ — notes.json, images, custom stickers, settings and custom language files. Delete that folder to erase all traces.",
      },
      {
        cat: "data",
        q: "I deleted a note by mistake!",
        a: "Deleted notes go to a recycle bin that keeps the last 20. Restore them from the tray menu — “Undo deleted notes”.",
      },
      {
        cat: "data",
        q: "How do password notes work?",
        a: "Each note can have its own password. Content is encrypted with PBKDF2-SHA256 (200,000 rounds) + AES-256-GCM using Windows' built-in crypto. Locked notes display a decoy shopping or chores list; the key lives only in memory and re-locks on exit.",
      },
      {
        cat: "notes",
        q: "Can I make my own stickers?",
        a: "Yes — drag an image into the Sticker Shop, press Ctrl+V, or pick files. PinPin removes the background from edge colors (offline, no AI), adds a die-cut white edge, and keeps resolution up to 4096 px.",
      },
      {
        cat: "notes",
        q: "What can handwriting do?",
        a: "Four pens — fountain, highlighter, chalk, crayon — with 8 colors and 3 widths, plus an eraser and undo. Strokes sit above text and below pins.",
      },
      {
        cat: "notes",
        q: "How do transparent notes work?",
        a: "Choose transparent paper and the note has no paper, no shadow — only text and ink. Hover to reveal a faint dashed frame with buttons. They can be pinned beneath all windows.",
      },
      {
        cat: "timer",
        q: "How many pomodoro styles are there?",
        a: "Six: tomato, hourglass, LED watch, twin-bell alarm clock, a cat with a clock and a shiba with a clock. Switch anytime from the right-click menu — the timer never jiggles; only the countdown changes.",
      },
      {
        cat: "timer",
        q: "What sounds can it make?",
        a: "Built-in “ding-dong” and “ring-ring”, whichever Windows alarm sounds your system offers (up to ten), a custom .wav of your choice — or blessed silence.",
      },
      {
        cat: "timer",
        q: "Does the timer survive a reboot?",
        a: "Yes. While running it only remembers the finish time — close the app or reboot and it picks up where it left off. If the moment passed while shut, it quietly moves on without ringing.",
      },
      {
        cat: "langs",
        q: "Which languages are built in?",
        a: "English, 繁體中文， 简体中文， 日本語， 한국어 and العربية. Arabic mirrors the whole interface right-to-left, and note content detects direction line by line — even when mixed in one note.",
      },
      {
        cat: "langs",
        q: "Can I add my own language?",
        a: "Yes. Language files are flat JSON. Copy the English template from the tray, translate the values, drop it back in — no recompiling. Missing keys fall back to English.",
      },
      {
        cat: "general",
        q: "Can it start with Windows?",
        a: "Yes, autostart toggles from the tray menu (off by default). Only one PinPin runs at a time — launching it again simply creates a new note.",
      },
      {
        cat: "general",
        q: "How do I uninstall?",
        a: "If you turned on autostart or the desktop right-click menu, turn them off in the tray first. Then delete pinpin.exe and, if you like, the %APPDATA%\\PinPin folder. That's the entire footprint.",
      },
      {
        cat: "notes",
        q: "What is Vibotype?",
        a: "A companion tool by the same author. Select text in any app, press ★, and it's collected into PinPin's starred notebook with a timestamp, the source program and its window title.",
      },
    ],
  },

  order: {
    eyebrow: "Handwritten couplets · shipped worldwide",
    title: "Your own gold-calligraphy spring couplets",
    sub: "We write code, and we write calligraphy — bringing the warmth of paper into the physical world. Pick a traditional greeting or tell us your custom words. We write in bold gold calligraphy on red paper and ship it to your door — for your home, your shop, or as a gift.",
    shipping: "Worldwide tracked shipping",
    stylesTitle: "Styles",
    styles: [
      {
        name: "Combined-character diamond",
        desc: "Four-character blessings fused into one glyph, like 招財進寶 and 黃金萬兩, on a diamond doufang.",
      },
      {
        name: "Vertical strip",
        desc: "A tall red strip with four characters, e.g. 吉祥如意, 歲歲平安, 學業進步.",
      },
      {
        name: "Single-character diamond",
        desc: "One big character — 福, 春, 財, 喜… The classic for doors and windows.",
      },
      {
        name: "Your own words",
        desc: "Your shop name, your family's wish, a couplet pair — tell us and we'll quote.",
      },
    ],
    formTitle: "Request a quote",
    fields: {
      name: "Name",
      email: "Email",
      style: "Style",
      quantity: "Quantity",
      text: "Characters",
      country: "Country / region",
      deadline: "Needed by",
      address: "Shipping address",
      notes: "Notes",
    },
    textHint: "e.g. 招財進寶 — or describe what you'd like.",
    privacy: "We only use these details to quote and ship your order.",
    submit: "Send request",
    sending: "Sending…",
    success: "Thank you! We'll reply with a quote within 1–2 working days.",
    error: "Couldn't send. Please email us instead.",
    notesTitle: "Good to know",
    notes: [
      "We reply with a price and delivery time within 1–2 working days; nothing is charged before you confirm.",
      "Every character is checked against traditional forms before writing.",
      "Every parcel is tracked: about 2–3 working days within Taiwan, about 7–14 overseas (depends on destination and carrier — we confirm it in the quote).",
      "Lunar New Year is our busy season: order 3 weeks ahead within Taiwan, 6 weeks for overseas. We close orders early once we're full.",
    ],
    mailSubject: "PinPin couplet order",
  },

  commission: {
    "eyebrow": "Custom stickers & assets",
    "title": "Bring your pets and art to your desktop",
    "sub": "Send us a photo, a drawing or just an idea — we turn it into die-cut stickers, washi tape or a pomodoro timer character made just for you, ready to import into PinPin.",
    "packagesTitle": "Pick a package",
    "popular": "Most loved",
    "packages": [
      {
        "name": "Mini sticker pack",
        "price": "$5",
        "desc": "5 die-cut stickers from your photos or drawings.",
        "items": [
          "5 stickers with a white border",
          "Cleaned-up cut-outs",
          "Ready to import"
        ]
      },
      {
        "name": "Sticker pack",
        "price": "$12",
        "desc": "A proper sheet for decorating every note.",
        "items": [
          "12 stickers",
          "2 matching washi tapes",
          "Ready to import"
        ]
      },
      {
        "name": "Pet timer",
        "price": "$15",
        "desc": "Your pet hugging a clock — a pomodoro timer style of its own.",
        "items": [
          "1 timer character",
          "3 matching stickers",
          "Works like the cat & shiba timers"
        ]
      },
      {
        "name": "Theme set",
        "price": "$29",
        "desc": "A whole look for your desktop.",
        "items": [
          "20 stickers",
          "3 washi tapes",
          "1 timer character",
          "Paper colours to match"
        ]
      },
      {
        "name": "Team edition",
        "price": "Let's talk",
        "desc": "Your brand on every note — for studios and teams.",
        "items": [
          "Brand colours & logo",
          "Custom sticker set",
          "A build for your team"
        ]
      }
    ],
    "stepsTitle": "How it works",
    "steps": [
      {
        "title": "Tell us",
        "desc": "Email your idea and a few photos or sketches."
      },
      {
        "title": "Quote",
        "desc": "We confirm what you get and the price, then send a payment link."
      },
      {
        "title": "Preview",
        "desc": "A first look in about 3 days, with two rounds of changes included."
      },
      {
        "title": "Delivery",
        "desc": "PNG files plus an import pack — drop them into the Sticker Shop."
      }
    ],
    "notesTitle": "Good to know",
    "notes": [
      "For personal use. Ask us if you need commercial use.",
      "Your photos stay yours. We only show your order as an example if you say yes.",
      "Most orders are done in 5–7 days.",
      "Payment is via PayPal or Ko-fi after we agree on the quote."
    ],
    "ctaTitle": "Have an idea?",
    "ctaSub": "Write to us — a photo and one sentence is enough to start.",
    "ctaButton": "Email us",
    "copy": "Copy address",
    "copied": "Email address copied",
    "mailSubject": "PinPin custom order",
    "mailBody": "Hi PinPin!\n\nPackage: \nWhat I would like: \n\n(Attach your photos or sketches.)\n"
  },
};

export type Dictionary = typeof en;
