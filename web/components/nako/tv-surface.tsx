"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState, useRef, useEffect, useCallback } from "react"
import { Play, Info, ChevronLeft, ChevronRight, Clock, Star, Search, Settings, User, Home, Film, Tv, Heart, List } from "lucide-react"
import { cn } from "@/lib/utils"

// TV端专用组件 - 10英尺UI设计，支持遥控器/键盘导航

interface MediaItem {
  id: string
  title: string
  year: number
  rating: number
  poster: string
  backdrop?: string
  type: "movie" | "series"
  progress?: number
}

// 模拟数据
const heroItems: MediaItem[] = [
  { id: "1", title: "沙丘2", year: 2024, rating: 8.6, poster: "/posters/dune2.jpg", backdrop: "/backdrops/dune2-backdrop.jpg", type: "movie" },
  { id: "2", title: "奥本海默", year: 2023, rating: 8.4, poster: "/posters/oppenheimer.jpg", backdrop: "/backdrops/dune2-backdrop.jpg", type: "movie" },
  { id: "3", title: "星际穿越", year: 2014, rating: 8.7, poster: "/posters/interstellar.jpg", backdrop: "/backdrops/dune2-backdrop.jpg", type: "movie" },
]

const continueWatching: MediaItem[] = [
  { id: "c1", title: "银翼杀手 2049", year: 2017, rating: 8.0, poster: "/posters/blade-runner.jpg", type: "movie", progress: 65 },
  { id: "c2", title: "真探 第一季", year: 2014, rating: 8.9, poster: "/posters/true-detective.jpg", type: "series", progress: 40 },
  { id: "c3", title: "降临", year: 2016, rating: 7.9, poster: "/backdrops/dune2-backdrop.jpg", type: "movie", progress: 25 },
]

const recommendations: MediaItem[] = [
  { id: "r1", title: "绝命毒师", year: 2008, rating: 9.5, poster: "/posters/breaking-bad.jpg", type: "series" },
  { id: "r2", title: "继承之战", year: 2018, rating: 8.9, poster: "/posters/succession.jpg", type: "series" },
  { id: "r3", title: "怪奇物语", year: 2016, rating: 8.7, poster: "/posters/true-detective.jpg", type: "series" },
  { id: "r4", title: "黑暗骑士", year: 2008, rating: 9.0, poster: "/placeholder.jpg", type: "movie" },
  { id: "r5", title: "盗梦空间", year: 2010, rating: 8.8, poster: "/placeholder.jpg", type: "movie" },
]

// 焦点管理 Hook
function useTVFocus(rows: number, cols: number[]) {
  const [focusRow, setFocusRow] = useState(0)
  const [focusCols, setFocusCols] = useState<number[]>(new Array(rows).fill(0))

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    switch (e.key) {
      case "ArrowUp":
        e.preventDefault()
        setFocusRow(prev => Math.max(0, prev - 1))
        break
      case "ArrowDown":
        e.preventDefault()
        setFocusRow(prev => Math.min(rows - 1, prev + 1))
        break
      case "ArrowLeft":
        e.preventDefault()
        setFocusCols(prev => {
          const newCols = [...prev]
          newCols[focusRow] = Math.max(0, newCols[focusRow] - 1)
          return newCols
        })
        break
      case "ArrowRight":
        e.preventDefault()
        setFocusCols(prev => {
          const newCols = [...prev]
          newCols[focusRow] = Math.min(cols[focusRow] - 1, newCols[focusRow] + 1)
          return newCols
        })
        break
    }
  }, [rows, cols, focusRow])

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [handleKeyDown])

  return { focusRow, focusCol: focusCols[focusRow], setFocusRow, setFocusCols }
}

// Hero 轮播组件
function TVHero({ items, focused, onSelect }: { items: MediaItem[]; focused: boolean; onSelect: (id: string) => void }) {
  const [currentIndex, setCurrentIndex] = useState(0)
  const item = items[currentIndex]

  // 自动轮播
  useEffect(() => {
    if (focused) return // 聚焦时停止自动轮播
    const timer = setInterval(() => {
      setCurrentIndex(prev => (prev + 1) % items.length)
    }, 8000)
    return () => clearInterval(timer)
  }, [items.length, focused])

  // 键盘导航
  useEffect(() => {
    if (!focused) return
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowLeft") {
        setCurrentIndex(prev => (prev - 1 + items.length) % items.length)
      } else if (e.key === "ArrowRight") {
        setCurrentIndex(prev => (prev + 1) % items.length)
      } else if (e.key === "Enter") {
        onSelect(item.id)
      }
    }
    window.addEventListener("keydown", handleKey)
    return () => window.removeEventListener("keydown", handleKey)
  }, [focused, items.length, item.id, onSelect])

  return (
    <div className="relative h-[65vh] w-full overflow-hidden">
      {/* 背景图 */}
      <div className="absolute inset-0">
        <img
          src={resolveArtwork(item.backdrop || item.poster)}
          alt=""
          className="h-full w-full object-cover transition-opacity duration-700"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-background via-background/80 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-t from-background via-transparent to-transparent" />
      </div>

      {/* 内容 */}
      <div className="relative flex h-full items-end pb-16 pl-16">
        <div className="max-w-2xl space-y-6">
          <h1 className="text-6xl font-bold tracking-tight text-foreground drop-shadow-lg">
            {item.title}
          </h1>
          <div className="flex items-center gap-4 text-xl text-muted-foreground">
            <span>{item.year}</span>
            <span className="flex items-center gap-1">
              <Star className="h-5 w-5 fill-yellow-500 text-yellow-500" />
              {item.rating}
            </span>
            <span className="rounded bg-muted px-2 py-0.5 text-sm">
              {item.type === "movie" ? "电影" : "剧集"}
            </span>
          </div>

          {/* 按钮 */}
          <div className="flex gap-4 pt-4">
            <button
              className={cn(
                "flex items-center gap-3 rounded-lg px-8 py-4 text-xl font-semibold transition-all",
                focused
                  ? "scale-105 bg-primary text-primary-foreground ring-4 ring-primary/50"
                  : "bg-white/90 text-black hover:bg-white"
              )}
            >
              <Play className="h-6 w-6" />
              播放
            </button>
            <button
              className="flex items-center gap-3 rounded-lg bg-white/20 px-8 py-4 text-xl font-semibold text-white backdrop-blur transition-all hover:bg-white/30"
            >
              <Info className="h-6 w-6" />
              详情
            </button>
          </div>
        </div>
      </div>

      {/* 轮播指示器 */}
      <div className="absolute bottom-8 right-16 flex gap-2">
        {items.map((_, index) => (
          <div
            key={index}
            className={cn(
              "h-1.5 rounded-full transition-all",
              index === currentIndex ? "w-8 bg-primary" : "w-3 bg-white/40"
            )}
          />
        ))}
      </div>

      {/* 导航箭头 */}
      {focused && (
        <>
          <button className="absolute left-8 top-1/2 -translate-y-1/2 rounded-full bg-black/50 p-3 text-white backdrop-blur transition-transform hover:scale-110">
            <ChevronLeft className="h-8 w-8" />
          </button>
          <button className="absolute right-8 top-1/2 -translate-y-1/2 rounded-full bg-black/50 p-3 text-white backdrop-blur transition-transform hover:scale-110">
            <ChevronRight className="h-8 w-8" />
          </button>
        </>
      )}
    </div>
  )
}

// 媒体行组件
function TVMediaRow({
  title,
  items,
  focused,
  focusIndex,
  showProgress,
  onSelect
}: {
  title: string
  items: MediaItem[]
  focused: boolean
  focusIndex: number
  showProgress?: boolean
  onSelect: (id: string) => void
}) {
  const scrollRef = useRef<HTMLDivElement>(null)

  // 聚焦时滚动到可见区域
  useEffect(() => {
    if (focused && scrollRef.current) {
      const focusedEl = scrollRef.current.children[focusIndex] as HTMLElement
      if (focusedEl) {
        focusedEl.scrollIntoView({ behavior: "smooth", inline: "center", block: "nearest" })
      }
    }
  }, [focused, focusIndex])

  // Enter 键选择
  useEffect(() => {
    if (!focused) return
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Enter") {
        onSelect(items[focusIndex].id)
      }
    }
    window.addEventListener("keydown", handleKey)
    return () => window.removeEventListener("keydown", handleKey)
  }, [focused, focusIndex, items, onSelect])

  return (
    <section className="py-4">
      <h2 className={cn(
        "mb-4 pl-16 text-2xl font-semibold transition-colors",
        focused ? "text-primary" : "text-foreground"
      )}>
        {title}
      </h2>
      <div
        ref={scrollRef}
        className="flex gap-4 overflow-x-auto px-16 pb-4 scrollbar-none"
      >
        {items.map((item, index) => (
          <div
            key={item.id}
            className={cn(
              "group relative flex-shrink-0 cursor-pointer overflow-hidden rounded-lg transition-all duration-300",
              focused && index === focusIndex
                ? "scale-110 ring-4 ring-primary shadow-2xl shadow-primary/30 z-10"
                : "hover:scale-105"
            )}
            style={{ width: "200px" }}
          >
            <div className="aspect-[2/3] overflow-hidden">
              <img
                src={resolveArtwork(item.poster)}
                alt={item.title}
                className="h-full w-full object-cover"
              />
            </div>

            {/* 进度条 */}
            {showProgress && item.progress && (
              <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
                <div
                  className="h-full bg-primary transition-all"
                  style={{ width: `${item.progress}%` }}
                />
              </div>
            )}

            {/* 聚焦时显示信息 */}
            {focused && index === focusIndex && (
              <div className="absolute inset-0 flex flex-col justify-end bg-gradient-to-t from-black/90 via-black/50 to-transparent p-4">
                <h3 className="text-lg font-semibold text-white">{item.title}</h3>
                <div className="flex items-center gap-2 text-sm text-white/80">
                  <span>{item.year}</span>
                  <Star className="h-3 w-3 fill-yellow-500 text-yellow-500" />
                  <span>{item.rating}</span>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </section>
  )
}

// 侧边导航栏
function TVSidebar({ focused, selectedIndex }: { focused: boolean; selectedIndex: number }) {
  const navItems = [
    { icon: Home, label: "首页" },
    { icon: Search, label: "搜索" },
    { icon: Film, label: "电影" },
    { icon: Tv, label: "剧集" },
    { icon: Heart, label: "收藏" },
    { icon: List, label: "播放列表" },
    { icon: Settings, label: "设置" },
  ]

  return (
    <aside className={cn(
      "fixed left-0 top-0 z-50 flex h-full flex-col items-center gap-2 bg-background/95 py-8 backdrop-blur transition-all",
      focused ? "w-56 px-4" : "w-20"
    )}>
      {/* Logo */}
      <div className="mb-8 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/20">
        <Film className="h-6 w-6 text-primary" />
      </div>

      {/* 导航项 */}
      {navItems.map((item, index) => (
        <button
          key={item.label}
          className={cn(
            "flex w-full items-center gap-4 rounded-xl px-4 py-3 transition-all",
            focused && index === selectedIndex
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-muted hover:text-foreground"
          )}
        >
          <item.icon className="h-6 w-6 flex-shrink-0" />
          {focused && <span className="text-lg font-medium">{item.label}</span>}
        </button>
      ))}

      {/* 用户 */}
      <div className="mt-auto">
        <button className={cn(
          "flex items-center gap-4 rounded-xl px-4 py-3 transition-all",
          "text-muted-foreground hover:bg-muted hover:text-foreground"
        )}>
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary text-sm font-medium">
            管理
          </div>
          {focused && <span className="text-lg font-medium">管理员</span>}
        </button>
      </div>
    </aside>
  )
}

// 主组件
export function TVSurface() {
  const [sidebarFocused, setSidebarFocused] = useState(false)
  const [sidebarIndex, setSidebarIndex] = useState(0)
  const [contentRow, setContentRow] = useState(0)
  const [contentCols, setContentCols] = useState([0, 0, 0]) // 每行的列索引

  const rows = [
    { title: "继续观看", items: continueWatching, showProgress: true },
    { title: "为你推荐", items: recommendations },
  ]

  // 全局键盘导航
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Tab 切换侧边栏焦点
      if (e.key === "Tab") {
        e.preventDefault()
        setSidebarFocused(prev => !prev)
        return
      }

      if (sidebarFocused) {
        // 侧边栏导航
        if (e.key === "ArrowUp") {
          setSidebarIndex(prev => Math.max(0, prev - 1))
        } else if (e.key === "ArrowDown") {
          setSidebarIndex(prev => Math.min(6, prev + 1))
        }
      } else {
        // 内容区导航
        if (e.key === "ArrowUp") {
          setContentRow(prev => Math.max(-1, prev - 1)) // -1 表示 Hero
        } else if (e.key === "ArrowDown") {
          setContentRow(prev => Math.min(rows.length - 1, prev + 1))
        } else if (e.key === "ArrowLeft" && contentRow >= 0) {
          setContentCols(prev => {
            const newCols = [...prev]
            newCols[contentRow] = Math.max(0, newCols[contentRow] - 1)
            return newCols
          })
        } else if (e.key === "ArrowRight" && contentRow >= 0) {
          setContentCols(prev => {
            const newCols = [...prev]
            newCols[contentRow] = Math.min(rows[contentRow].items.length - 1, newCols[contentRow] + 1)
            return newCols
          })
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [sidebarFocused, contentRow, rows])

  const handleSelect = (id: string) => {
    console.log("Selected:", id)
    // 这里可以导航到详情页
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* 侧边栏 */}
      <TVSidebar focused={sidebarFocused} selectedIndex={sidebarIndex} />

      {/* 主内容 */}
      <main className={cn(
        "transition-all",
        sidebarFocused ? "ml-56" : "ml-20"
      )}>
        {/* Hero 轮播 */}
        <TVHero
          items={heroItems}
          focused={!sidebarFocused && contentRow === -1}
          onSelect={handleSelect}
        />

        {/* 媒体行 */}
        {rows.map((row, index) => (
          <TVMediaRow
            key={row.title}
            title={row.title}
            items={row.items}
            focused={!sidebarFocused && contentRow === index}
            focusIndex={contentCols[index]}
            showProgress={row.showProgress}
            onSelect={handleSelect}
          />
        ))}
      </main>

      {/* 导航提示 */}
      <div className="fixed bottom-8 right-8 flex items-center gap-6 rounded-xl bg-black/70 px-6 py-3 text-sm text-white/80 backdrop-blur">
        <span className="flex items-center gap-2">
          <kbd className="rounded bg-white/20 px-2 py-1">Tab</kbd>
          切换侧边栏
        </span>
        <span className="flex items-center gap-2">
          <kbd className="rounded bg-white/20 px-2 py-1">方向键</kbd>
          导航
        </span>
        <span className="flex items-center gap-2">
          <kbd className="rounded bg-white/20 px-2 py-1">Enter</kbd>
          选择
        </span>
      </div>
    </div>
  )
}
