"use client"

import { createContext, useContext, useState, useCallback, type ReactNode } from "react"

// 支持的语言
export const SUPPORTED_LOCALES = ["zh-CN", "zh-TW", "en", "ja"] as const
export type Locale = typeof SUPPORTED_LOCALES[number]

export const LOCALE_NAMES: Record<Locale, string> = {
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
  "en": "English",
  "ja": "日本語",
}

// 翻译字典类型
type TranslationDict = {
  [key: string]: string | TranslationDict
}

// 翻译数据
const translations: Record<Locale, TranslationDict> = {
  "zh-CN": {
    common: {
      search: "搜索",
      settings: "设置",
      back: "返回",
      cancel: "取消",
      confirm: "确认",
      save: "保存",
      delete: "删除",
      edit: "编辑",
      add: "添加",
      close: "关闭",
      loading: "加载中...",
      noData: "暂无数据",
      viewAll: "查看全部",
      play: "播放",
      pause: "暂停",
      resume: "继续播放",
      playFromStart: "从头播放",
    },
    nav: {
      home: "首页",
      library: "媒体库",
      favorites: "收藏",
      downloads: "下载",
      history: "历史",
      settings: "设置",
      admin: "管理面板",
    },
    media: {
      movies: "电影",
      series: "剧集",
      anime: "动画",
      documentary: "纪录片",
      music: "音乐",
      photos: "照片",
      continueWatching: "继续观看",
      recentlyAdded: "最近添加",
      recommended: "为你推荐",
      trending: "热门",
      genres: "类型",
      year: "年份",
      rating: "评分",
      duration: "时长",
      director: "导演",
      cast: "演员",
      overview: "简介",
      episodes: "集数",
      seasons: "季",
      episode: "第{n}集",
      season: "第{n}季",
    },
    player: {
      subtitles: "字幕",
      audio: "音轨",
      quality: "画质",
      speed: "播放速度",
      cast: "投屏",
      fullscreen: "全屏",
      exitFullscreen: "退出全屏",
      volume: "音量",
      mute: "静音",
      unmute: "取消静音",
      skipIntro: "跳过片头",
      nextEpisode: "下一集",
      previousEpisode: "上一集",
    },
    library: {
      all: "全部",
      unwatched: "未观看",
      watching: "观看中",
      watched: "已观看",
      favorites: "收藏",
      collections: "合集",
      categories: "分类",
      sortBy: "排序",
      filterBy: "筛选",
      gridView: "网格视图",
      detailView: "详细视图",
      tableView: "表格视图",
      scanLibrary: "扫描媒体库",
      refreshMetadata: "刷新元数据",
    },
    settings: {
      general: "通用",
      playback: "播放",
      display: "显示",
      language: "语言",
      theme: "主题",
      dark: "深色",
      light: "浅色",
      system: "跟随系统",
      notifications: "通知",
      about: "关于",
      version: "版本",
    },
    admin: {
      dashboard: "仪表盘",
      libraries: "媒体库",
      users: "用户",
      metadata: "元数据",
      plugins: "插件",
      logs: "日志",
      tasks: "计划任务",
      transcoding: "转码",
      network: "网络",
    },
    time: {
      now: "刚刚",
      minutesAgo: "{n}分钟前",
      hoursAgo: "{n}小时前",
      daysAgo: "{n}天前",
      yesterday: "昨天",
    },
  },
  "zh-TW": {
    common: {
      search: "搜尋",
      settings: "設定",
      back: "返回",
      cancel: "取消",
      confirm: "確認",
      save: "儲存",
      delete: "刪除",
      edit: "編輯",
      add: "新增",
      close: "關閉",
      loading: "載入中...",
      noData: "暫無資料",
      viewAll: "查看全部",
      play: "播放",
      pause: "暫停",
      resume: "繼續播放",
      playFromStart: "從頭播放",
    },
    nav: {
      home: "首頁",
      library: "媒體庫",
      favorites: "收藏",
      downloads: "下載",
      history: "歷史",
      settings: "設定",
      admin: "管理面板",
    },
    media: {
      movies: "電影",
      series: "劇集",
      anime: "動畫",
      documentary: "紀錄片",
      music: "音樂",
      photos: "照片",
      continueWatching: "繼續觀看",
      recentlyAdded: "最近新增",
      recommended: "為你推薦",
      trending: "熱門",
      genres: "類型",
      year: "年份",
      rating: "評分",
      duration: "時長",
      director: "導演",
      cast: "演員",
      overview: "簡介",
      episodes: "集數",
      seasons: "季",
      episode: "第{n}集",
      season: "第{n}季",
    },
    player: {
      subtitles: "字幕",
      audio: "音軌",
      quality: "畫質",
      speed: "播放速度",
      cast: "投放",
      fullscreen: "全螢幕",
      exitFullscreen: "退出全螢幕",
      volume: "音量",
      mute: "靜音",
      unmute: "取消靜音",
      skipIntro: "跳過片頭",
      nextEpisode: "下一集",
      previousEpisode: "上一集",
    },
    library: {
      all: "全部",
      unwatched: "未觀看",
      watching: "觀看中",
      watched: "已觀看",
      favorites: "收藏",
      collections: "合集",
      categories: "分類",
      sortBy: "排序",
      filterBy: "篩選",
      gridView: "網格視圖",
      detailView: "詳細視圖",
      tableView: "表格視圖",
      scanLibrary: "掃描媒體庫",
      refreshMetadata: "重新整理元資料",
    },
    settings: {
      general: "一般",
      playback: "播放",
      display: "顯示",
      language: "語言",
      theme: "主題",
      dark: "深色",
      light: "淺色",
      system: "跟隨系統",
      notifications: "通知",
      about: "關於",
      version: "版本",
    },
    admin: {
      dashboard: "儀表板",
      libraries: "媒體庫",
      users: "使用者",
      metadata: "元資料",
      plugins: "外掛程式",
      logs: "日誌",
      tasks: "排程任務",
      transcoding: "轉碼",
      network: "網路",
    },
    time: {
      now: "剛剛",
      minutesAgo: "{n}分鐘前",
      hoursAgo: "{n}小時前",
      daysAgo: "{n}天前",
      yesterday: "昨天",
    },
  },
  en: {
    common: {
      search: "Search",
      settings: "Settings",
      back: "Back",
      cancel: "Cancel",
      confirm: "Confirm",
      save: "Save",
      delete: "Delete",
      edit: "Edit",
      add: "Add",
      close: "Close",
      loading: "Loading...",
      noData: "No data",
      viewAll: "View All",
      play: "Play",
      pause: "Pause",
      resume: "Resume",
      playFromStart: "Play from Start",
    },
    nav: {
      home: "Home",
      library: "Library",
      favorites: "Favorites",
      downloads: "Downloads",
      history: "History",
      settings: "Settings",
      admin: "Admin Panel",
    },
    media: {
      movies: "Movies",
      series: "TV Shows",
      anime: "Anime",
      documentary: "Documentary",
      music: "Music",
      photos: "Photos",
      continueWatching: "Continue Watching",
      recentlyAdded: "Recently Added",
      recommended: "Recommended for You",
      trending: "Trending",
      genres: "Genres",
      year: "Year",
      rating: "Rating",
      duration: "Duration",
      director: "Director",
      cast: "Cast",
      overview: "Overview",
      episodes: "Episodes",
      seasons: "Seasons",
      episode: "Episode {n}",
      season: "Season {n}",
    },
    player: {
      subtitles: "Subtitles",
      audio: "Audio",
      quality: "Quality",
      speed: "Speed",
      cast: "Cast",
      fullscreen: "Fullscreen",
      exitFullscreen: "Exit Fullscreen",
      volume: "Volume",
      mute: "Mute",
      unmute: "Unmute",
      skipIntro: "Skip Intro",
      nextEpisode: "Next Episode",
      previousEpisode: "Previous Episode",
    },
    library: {
      all: "All",
      unwatched: "Unwatched",
      watching: "Watching",
      watched: "Watched",
      favorites: "Favorites",
      collections: "Collections",
      categories: "Categories",
      sortBy: "Sort by",
      filterBy: "Filter by",
      gridView: "Grid View",
      detailView: "Detail View",
      tableView: "Table View",
      scanLibrary: "Scan Library",
      refreshMetadata: "Refresh Metadata",
    },
    settings: {
      general: "General",
      playback: "Playback",
      display: "Display",
      language: "Language",
      theme: "Theme",
      dark: "Dark",
      light: "Light",
      system: "System",
      notifications: "Notifications",
      about: "About",
      version: "Version",
    },
    admin: {
      dashboard: "Dashboard",
      libraries: "Libraries",
      users: "Users",
      metadata: "Metadata",
      plugins: "Plugins",
      logs: "Logs",
      tasks: "Scheduled Tasks",
      transcoding: "Transcoding",
      network: "Network",
    },
    time: {
      now: "Just now",
      minutesAgo: "{n} minutes ago",
      hoursAgo: "{n} hours ago",
      daysAgo: "{n} days ago",
      yesterday: "Yesterday",
    },
  },
  ja: {
    common: {
      search: "検索",
      settings: "設定",
      back: "戻る",
      cancel: "キャンセル",
      confirm: "確認",
      save: "保存",
      delete: "削除",
      edit: "編集",
      add: "追加",
      close: "閉じる",
      loading: "読み込み中...",
      noData: "データなし",
      viewAll: "すべて表示",
      play: "再生",
      pause: "一時停止",
      resume: "再開",
      playFromStart: "最初から再生",
    },
    nav: {
      home: "ホーム",
      library: "ライブラリ",
      favorites: "お気に入り",
      downloads: "ダウンロード",
      history: "履歴",
      settings: "設定",
      admin: "管理パネル",
    },
    media: {
      movies: "映画",
      series: "ドラマ",
      anime: "アニメ",
      documentary: "ドキュメンタリー",
      music: "音楽",
      photos: "写真",
      continueWatching: "視聴を続ける",
      recentlyAdded: "最近追加",
      recommended: "おすすめ",
      trending: "人気",
      genres: "ジャンル",
      year: "年",
      rating: "評価",
      duration: "時間",
      director: "監督",
      cast: "出演者",
      overview: "あらすじ",
      episodes: "話数",
      seasons: "シーズン",
      episode: "第{n}話",
      season: "シーズン{n}",
    },
    player: {
      subtitles: "字幕",
      audio: "音声",
      quality: "画質",
      speed: "再生速度",
      cast: "キャスト",
      fullscreen: "フルスクリーン",
      exitFullscreen: "フルスクリーン解除",
      volume: "音量",
      mute: "ミュート",
      unmute: "ミュート解除",
      skipIntro: "イントロをスキップ",
      nextEpisode: "次のエピソード",
      previousEpisode: "前のエピソード",
    },
    library: {
      all: "すべて",
      unwatched: "未視聴",
      watching: "視聴中",
      watched: "視聴済み",
      favorites: "お気に入り",
      collections: "コレクション",
      categories: "カテゴリ",
      sortBy: "並び替え",
      filterBy: "フィルター",
      gridView: "グリッド表示",
      detailView: "詳細表示",
      tableView: "テーブル表示",
      scanLibrary: "ライブラリをスキャン",
      refreshMetadata: "メタデータを更新",
    },
    settings: {
      general: "一般",
      playback: "再生",
      display: "表示",
      language: "言語",
      theme: "テーマ",
      dark: "ダーク",
      light: "ライト",
      system: "システム",
      notifications: "通知",
      about: "情報",
      version: "バージョン",
    },
    admin: {
      dashboard: "ダッシュボード",
      libraries: "ライブラリ",
      users: "ユーザー",
      metadata: "メタデータ",
      plugins: "プラグイン",
      logs: "ログ",
      tasks: "スケジュールタスク",
      transcoding: "トランスコード",
      network: "ネットワーク",
    },
    time: {
      now: "たった今",
      minutesAgo: "{n}分前",
      hoursAgo: "{n}時間前",
      daysAgo: "{n}日前",
      yesterday: "昨日",
    },
  },
}

// Context
interface I18nContextType {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: string, params?: Record<string, string | number>) => string
}

const I18nContext = createContext<I18nContextType | null>(null)

// Provider
export function I18nProvider({ children, defaultLocale = "zh-CN" }: { children: ReactNode; defaultLocale?: Locale }) {
  const [locale, setLocaleState] = useState<Locale>(defaultLocale)

  const setLocale = useCallback((newLocale: Locale) => {
    setLocaleState(newLocale)
    // 可以在这里保存到 localStorage
    if (typeof window !== "undefined") {
      localStorage.setItem("nako-locale", newLocale)
    }
  }, [])

  // 翻译函数
  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    const keys = key.split(".")
    let value: string | TranslationDict | undefined = translations[locale]
    
    for (const k of keys) {
      if (value && typeof value === "object" && k in value) {
        value = value[k]
      } else {
        // 回退到英文
        value = translations.en
        for (const fallbackKey of keys) {
          if (value && typeof value === "object" && fallbackKey in value) {
            value = value[fallbackKey]
          } else {
            return key // 如果都找不到，返回原始 key
          }
        }
        break
      }
    }

    if (typeof value !== "string") {
      return key
    }

    // 替换参数
    if (params) {
      return value.replace(/\{(\w+)\}/g, (_, paramKey) => {
        return params[paramKey]?.toString() ?? `{${paramKey}}`
      })
    }

    return value
  }, [locale])

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  )
}

// Hook
export function useI18n() {
  const context = useContext(I18nContext)
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider")
  }
  return context
}

// 语言选择器组件
export function LanguageSelector({ className }: { className?: string }) {
  const { locale, setLocale } = useI18n()
  
  return (
    <select
      value={locale}
      onChange={(e) => setLocale(e.target.value as Locale)}
      className={className}
    >
      {SUPPORTED_LOCALES.map((loc) => (
        <option key={loc} value={loc}>
          {LOCALE_NAMES[loc]}
        </option>
      ))}
    </select>
  )
}
