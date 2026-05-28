"use client"

import { useState } from "react"
import { ChevronLeft, User, Monitor, Volume2, Subtitles, Globe, Bell, Shield, HardDrive, Wifi, Home, Clock, Eye, Palette, Server, Check, Film, Tv, Music, Image, Sparkles, LayoutGrid, List, Table2, ChevronUp, ChevronDown } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

interface SettingsPageProps {
  onBack: () => void
}

export function SettingsPage({ onBack }: SettingsPageProps) {
  const [activeSection, setActiveSection] = useState("profile")
  
  const sections = [
    { id: "profile", label: "个人资料", icon: User },
    { id: "home", label: "主屏幕", icon: Home },
    { id: "playback", label: "播放", icon: Monitor },
    { id: "audio", label: "音频", icon: Volume2 },
    { id: "subtitles", label: "字幕", icon: Subtitles },
    { id: "language", label: "语言", icon: Globe },
    { id: "display", label: "显示", icon: Palette },
    { id: "server", label: "服务器连接", icon: Server },
  ]

  return (
    <div className="min-h-screen overflow-y-auto bg-background scrollbar-none">
      {/* 顶部导航 */}
      <div className="sticky top-0 z-10 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto flex max-w-5xl items-center gap-3 p-4">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <h1 className="text-xl font-semibold">设置</h1>
        </div>
      </div>

      <div className="mx-auto max-w-5xl">
        <div className="flex flex-col md:flex-row">
          {/* 侧边导航 */}
          <nav className="w-full border-b border-border/50 md:w-56 md:flex-shrink-0 md:border-b-0 md:border-r">
            <div className="flex gap-1 overflow-x-auto p-2 scrollbar-none md:flex-col md:p-4">
              {sections.map((section) => (
                <button
                  key={section.id}
                  onClick={() => setActiveSection(section.id)}
                  className={cn(
                    "flex flex-shrink-0 items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors md:w-full",
                    activeSection === section.id
                      ? "bg-secondary text-foreground"
                      : "text-muted-foreground hover:bg-secondary/50 hover:text-foreground"
                  )}
                >
                  <section.icon className="h-4 w-4" />
                  <span>{section.label}</span>
                </button>
              ))}
            </div>
          </nav>

          {/* 设置内容 */}
          <div className="flex-1 overflow-y-auto p-4 scrollbar-none md:max-h-[calc(100vh-120px)] md:p-6">
            {activeSection === "profile" && <ProfileSettings />}
            {activeSection === "home" && <HomeScreenSettings />}
            {activeSection === "playback" && <PlaybackSettings />}
            {activeSection === "audio" && <AudioSettings />}
            {activeSection === "subtitles" && <SubtitleSettings />}
            {activeSection === "language" && <LanguageSettings />}
            {activeSection === "display" && <DisplaySettings />}
            {activeSection === "server" && <ServerSettings />}
          </div>
        </div>
      </div>
    </div>
  )
}

// 个人资料设置
function ProfileSettings() {
  const [username, setUsername] = useState("Admin")
  const [enablePin, setEnablePin] = useState(false)

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">个人资料</h2>
        <p className="text-sm text-muted-foreground">管理您的账户信息</p>
      </div>

      <div className="space-y-4">
        {/* 头像 */}
        <div className="flex items-center gap-4 rounded-lg border border-border/50 bg-card p-4">
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10 text-2xl font-semibold text-primary">
            {username.charAt(0).toUpperCase()}
          </div>
          <div className="flex-1">
            <p className="font-medium">{username}</p>
            <p className="text-sm text-muted-foreground">用户</p>
          </div>
          <Button variant="outline" size="sm">更换头像</Button>
        </div>

        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <Label>用户名</Label>
          <Input value={username} onChange={(e) => setUsername(e.target.value)} />
        </div>

        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">修改密码</p>
              <p className="text-sm text-muted-foreground">更新您的登录密码</p>
            </div>
            <Button variant="outline" size="sm">修改</Button>
          </div>
        </div>

        <SettingItem
          title="启用 PIN 码"
          description="使用 4 位数字快速解锁"
        >
          <Switch checked={enablePin} onCheckedChange={setEnablePin} />
        </SettingItem>

        {enablePin && (
          <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
            <Label>设置 PIN 码</Label>
            <Input type="password" maxLength={4} placeholder="输入 4 位数字" className="w-32 text-center tracking-widest" />
          </div>
        )}
      </div>
    </div>
  )
}

// 主屏幕设置
function HomeScreenSettings() {
  const [hideWatchedFromLatest, setHideWatchedFromLatest] = useState(false)
  const [posterSize, setPosterSize] = useState("medium")
  
  // 主屏幕区块配置 - 类似 Jellyfin 的设计
  const sectionTypes = [
    { value: "none", label: "无" },
    { value: "libraries", label: "我的媒体库" },
    { value: "libraries_small", label: "我的媒体库 (紧凑)" },
    { value: "continue_watching", label: "继续观看" },
    { value: "continue_listening", label: "继续收听" },
    { value: "next_up", label: "接下来观看" },
    { value: "latest_media", label: "最新媒体" },
    { value: "favorites", label: "我的收藏" },
    { value: "recommendations", label: "为你推荐" },
  ]
  
  const [homeSections, setHomeSections] = useState([
    { position: 1, type: "continue_watching" },
    { position: 2, type: "next_up" },
    { position: 3, type: "latest_media" },
    { position: 4, type: "recommendations" },
    { position: 5, type: "favorites" },
    { position: 6, type: "none" },
  ])
  
  // 媒体库排序
  const [libraryOrder, setLibraryOrder] = useState([
    { id: "movies", name: "电影", icon: Film, visible: true },
    { id: "shows", name: "剧集", icon: Tv, visible: true },
    { id: "anime", name: "动画", icon: Sparkles, visible: true },
    { id: "music", name: "音乐", icon: Music, visible: true },
    { id: "photos", name: "照片", icon: Image, visible: false },
  ])
  
  // 移动媒体库顺序
  const moveLibrary = (index: number, direction: "up" | "down") => {
    const newOrder = [...libraryOrder]
    const targetIndex = direction === "up" ? index - 1 : index + 1
    if (targetIndex < 0 || targetIndex >= newOrder.length) return
    ;[newOrder[index], newOrder[targetIndex]] = [newOrder[targetIndex], newOrder[index]]
    setLibraryOrder(newOrder)
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">主屏幕</h2>
        <p className="text-sm text-muted-foreground">自定义首页的显示内容和布局</p>
      </div>

      {/* 基础设置 */}
      <div className="space-y-4">
        <SettingItem
          title="从最新媒体中隐藏已观看内容"
          description="不在最新媒体区块中显示已观看的内容"
        >
          <Switch checked={hideWatchedFromLatest} onCheckedChange={setHideWatchedFromLatest} />
        </SettingItem>
        
        <SettingItem
          title="海报尺寸"
          description="调整媒体封面的显示大小"
        >
          <Select value={posterSize} onValueChange={setPosterSize}>
            <SelectTrigger className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="small">小</SelectItem>
              <SelectItem value="medium">中</SelectItem>
              <SelectItem value="large">大</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>
      </div>

      {/* 主屏幕区块配置 */}
      <div className="space-y-3">
        <div>
          <Label className="text-base">首页区块</Label>
          <p className="text-sm text-muted-foreground">选择每个位置显示的内容类型</p>
        </div>
        <div className="space-y-2 rounded-lg border border-border/50 bg-card p-4">
          {homeSections.map((section) => (
            <div
              key={section.position}
              className="flex items-center gap-3"
            >
              <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-muted text-xs font-medium">
                {section.position}
              </span>
              <Select 
                value={section.type} 
                onValueChange={(value) => {
                  setHomeSections(homeSections.map(s =>
                    s.position === section.position ? { ...s, type: value } : s
                  ))
                }}
              >
                <SelectTrigger className="flex-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {sectionTypes.map(type => (
                    <SelectItem key={type.value} value={type.value}>{type.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ))}
        </div>
      </div>

      {/* 媒体库显示顺序 */}
      <div className="space-y-3">
        <div>
          <Label className="text-base">媒体库顺序</Label>
          <p className="text-sm text-muted-foreground">调整侧边栏和首页的媒体库显示顺序</p>
        </div>
        <div className="space-y-2 rounded-lg border border-border/50 bg-card p-4 max-h-[280px] overflow-y-auto scrollbar-none">
          {libraryOrder.map((lib, index) => (
            <div
              key={lib.id}
              className="flex items-center gap-2 rounded-md border border-border/50 bg-background p-2.5"
            >
              <div className="flex flex-col -my-1">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground hover:text-foreground disabled:opacity-30"
                  disabled={index === 0}
                  onClick={() => moveLibrary(index, "up")}
                >
                  <ChevronUp className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground hover:text-foreground disabled:opacity-30"
                  disabled={index === libraryOrder.length - 1}
                  onClick={() => moveLibrary(index, "down")}
                >
                  <ChevronDown className="h-4 w-4" />
                </Button>
              </div>
              <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded bg-muted text-[10px] font-medium">
                {index + 1}
              </span>
              <lib.icon className="h-4 w-4 shrink-0 text-muted-foreground" />
              <span className="flex-1 text-sm">{lib.name}</span>
              <Switch
                checked={lib.visible}
                onCheckedChange={(checked) => {
                  setLibraryOrder(libraryOrder.map(l =>
                    l.id === lib.id ? { ...l, visible: checked } : l
                  ))
                }}
              />
            </div>
          ))}
        </div>
      </div>

      {/* 每个媒体库的显示偏好 */}
      <div className="space-y-3">
        <div>
          <Label className="text-base">媒体库显示偏好</Label>
          <p className="text-sm text-muted-foreground">为每个媒体库设置默认视图和排序方式</p>
        </div>
        <div className="space-y-3">
          {libraryOrder.filter(lib => lib.visible).map((lib) => (
            <div key={lib.id} className="rounded-lg border border-border/50 bg-card p-4">
              <div className="mb-3 flex items-center gap-2">
                <lib.icon className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">{lib.name}</span>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label className="text-xs text-muted-foreground">默认视图</Label>
                  <Select defaultValue="grid">
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="grid">
                        <span className="flex items-center gap-2">
                          <LayoutGrid className="h-3 w-3" /> 网格
                        </span>
                      </SelectItem>
                      <SelectItem value="list">
                        <span className="flex items-center gap-2">
                          <List className="h-3 w-3" /> 列表
                        </span>
                      </SelectItem>
                      <SelectItem value="table">
                        <span className="flex items-center gap-2">
                          <Table2 className="h-3 w-3" /> 表格
                        </span>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1.5">
                  <Label className="text-xs text-muted-foreground">默认排序</Label>
                  <Select defaultValue="date_added">
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="date_added">添加日期</SelectItem>
                      <SelectItem value="name">名称</SelectItem>
                      <SelectItem value="year">年份</SelectItem>
                      <SelectItem value="rating">评分</SelectItem>
                      <SelectItem value="runtime">时长</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

// 播放设置
function PlaybackSettings() {
  const [preferredQuality, setPreferredQuality] = useState("auto")
  const [enableTranscoding, setEnableTranscoding] = useState(true)
  const [maxBitrate, setMaxBitrate] = useState("auto")
  const [resumePlayback, setResumePlayback] = useState(true)
  const [autoPlayNext, setAutoPlayNext] = useState(true)
  const [cinemaMode, setCinemaMode] = useState(false)

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">播放设置</h2>
        <p className="text-sm text-muted-foreground">配置视频播放选项</p>
      </div>

      <div className="space-y-4">
        <SettingItem
          title="首选质量"
          description="选择优先播放的视频质量"
        >
          <Select value={preferredQuality} onValueChange={setPreferredQuality}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">自动</SelectItem>
              <SelectItem value="original">原始质量</SelectItem>
              <SelectItem value="4k">4K (2160p)</SelectItem>
              <SelectItem value="1080p">1080p</SelectItem>
              <SelectItem value="720p">720p</SelectItem>
              <SelectItem value="480p">480p</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        <SettingItem
          title="允许转码"
          description="当设备不支持原始格式时进行转码"
        >
          <Switch checked={enableTranscoding} onCheckedChange={setEnableTranscoding} />
        </SettingItem>

        {enableTranscoding && (
          <SettingItem
            title="最大比特率"
            description="限制转码视频的最大比特率"
          >
            <Select value={maxBitrate} onValueChange={setMaxBitrate}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">自动</SelectItem>
                <SelectItem value="120">120 Mbps</SelectItem>
                <SelectItem value="80">80 Mbps</SelectItem>
                <SelectItem value="60">60 Mbps</SelectItem>
                <SelectItem value="40">40 Mbps</SelectItem>
                <SelectItem value="20">20 Mbps</SelectItem>
                <SelectItem value="10">10 Mbps</SelectItem>
              </SelectContent>
            </Select>
          </SettingItem>
        )}

        <SettingItem
          title="记住播放位置"
          description="下次播放时从上次位置继续"
        >
          <Switch checked={resumePlayback} onCheckedChange={setResumePlayback} />
        </SettingItem>

        <SettingItem
          title="自动播放下一集"
          description="当前集结束后自动播放下一集"
        >
          <Switch checked={autoPlayNext} onCheckedChange={setAutoPlayNext} />
        </SettingItem>

        <SettingItem
          title="影院模式"
          description="播放时自动调暗界面背景"
        >
          <Switch checked={cinemaMode} onCheckedChange={setCinemaMode} />
        </SettingItem>
      </div>
    </div>
  )
}

// 音频设置
function AudioSettings() {
  const [preferredAudioLanguage, setPreferredAudioLanguage] = useState("original")
  const [enableDts, setEnableDts] = useState(true)
  const [enableTrueHd, setEnableTrueHd] = useState(true)
  const [maxAudioChannels, setMaxAudioChannels] = useState("auto")
  const [boostDialogue, setBoostDialogue] = useState(false)

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">音频设置</h2>
        <p className="text-sm text-muted-foreground">配置音频播放偏好</p>
      </div>

      <div className="space-y-4">
        <SettingItem
          title="首选音轨语言"
          description="优先选择的音频语言"
        >
          <Select value={preferredAudioLanguage} onValueChange={setPreferredAudioLanguage}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="original">原始语言</SelectItem>
              <SelectItem value="zh">中文</SelectItem>
              <SelectItem value="en">英语</SelectItem>
              <SelectItem value="ja">日语</SelectItem>
              <SelectItem value="ko">韩语</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        <SettingItem
          title="最大音频声道"
          description="限制音频输出声道数"
        >
          <Select value={maxAudioChannels} onValueChange={setMaxAudioChannels}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">自动</SelectItem>
              <SelectItem value="7.1">7.1 声道</SelectItem>
              <SelectItem value="5.1">5.1 声道</SelectItem>
              <SelectItem value="stereo">立体声</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        {/* 音频编解码器支持 */}
        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <Label>音频直通 (Passthrough)</Label>
          <p className="text-xs text-muted-foreground">允许设备直接解码以下音频格式</p>
          <div className="mt-3 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm">DTS / DTS-HD</span>
              <Switch checked={enableDts} onCheckedChange={setEnableDts} />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Dolby TrueHD / Atmos</span>
              <Switch checked={enableTrueHd} onCheckedChange={setEnableTrueHd} />
            </div>
          </div>
        </div>

        <SettingItem
          title="对白增强"
          description="提升人声对白音量"
        >
          <Switch checked={boostDialogue} onCheckedChange={setBoostDialogue} />
        </SettingItem>
      </div>
    </div>
  )
}

// ��幕设置
function SubtitleSettings() {
  const [subtitlesEnabled, setSubtitlesEnabled] = useState(true)
  const [preferredLanguage, setPreferredLanguage] = useState("zh")
  const [subtitleMode, setSubtitleMode] = useState("default")
  const [fontSize, setFontSize] = useState("medium")
  const [burnInSubtitles, setBurnInSubtitles] = useState(false)

  const fontSizes = [
    { value: "small", label: "小" },
    { value: "medium", label: "中" },
    { value: "large", label: "大" },
    { value: "extra-large", label: "特大" },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">字幕设置</h2>
        <p className="text-sm text-muted-foreground">配置字幕显示偏好</p>
      </div>

      <div className="space-y-4">
        <SettingItem
          title="默认显示字幕"
          description="播放时自动加载字幕"
        >
          <Switch checked={subtitlesEnabled} onCheckedChange={setSubtitlesEnabled} />
        </SettingItem>

        <SettingItem
          title="首选字幕语言"
          description="优先选择的字幕语言"
        >
          <Select value={preferredLanguage} onValueChange={setPreferredLanguage}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zh">简��中文</SelectItem>
              <SelectItem value="zh-TW">繁體中文</SelectItem>
              <SelectItem value="en">English</SelectItem>
              <SelectItem value="ja">日本語</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        <SettingItem
          title="字幕模式"
          description="选择何时显示字幕"
        >
          <Select value={subtitleMode} onValueChange={setSubtitleMode}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="default">默认</SelectItem>
              <SelectItem value="always">始终显示</SelectItem>
              <SelectItem value="foreign">仅外语内容</SelectItem>
              <SelectItem value="forced">仅强制字幕</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <Label>字幕大小</Label>
          <div className="flex gap-2">
            {fontSizes.map((size) => (
              <button
                key={size.value}
                onClick={() => setFontSize(size.value)}
                className={cn(
                  "flex-1 rounded-md px-3 py-2 text-sm transition-colors",
                  fontSize === size.value
                    ? "bg-primary text-primary-foreground"
                    : "bg-secondary hover:bg-secondary/80"
                )}
              >
                {size.label}
              </button>
            ))}
          </div>
        </div>

        <SettingItem
          title="烧录字幕"
          description="将字幕烧录到视频中（用于不支持外挂字幕的设备）"
        >
          <Switch checked={burnInSubtitles} onCheckedChange={setBurnInSubtitles} />
        </SettingItem>

        {/* 字幕预览 */}
        <div className="space-y-2">
          <Label>预览</Label>
          <div className="relative overflow-hidden rounded-lg bg-muted/50">
            <div className="aspect-video bg-gradient-to-br from-secondary to-muted" />
            <div className="absolute inset-x-0 bottom-4 text-center">
              <span 
                className={cn(
                  "inline-block rounded bg-black/70 px-3 py-1 text-white",
                  fontSize === "small" && "text-sm",
                  fontSize === "medium" && "text-base",
                  fontSize === "large" && "text-lg",
                  fontSize === "extra-large" && "text-xl",
                )}
              >
                这是字幕预览效果
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

// 语言设置
function LanguageSettings() {
  const [uiLanguage, setUiLanguage] = useState("zh-CN")

  const languages = [
    { value: "zh-CN", label: "简体中文", native: "简体中文" },
    { value: "zh-TW", label: "繁體中文", native: "繁體中文" },
    { value: "en", label: "English", native: "English" },
    { value: "ja", label: "日本語", native: "日本語" },
    { value: "ko", label: "한국어", native: "한국어" },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">语言设置</h2>
        <p className="text-sm text-muted-foreground">选择界面显示语言</p>
      </div>

      <div className="space-y-2">
        {languages.map((lang) => (
          <button
            key={lang.value}
            onClick={() => setUiLanguage(lang.value)}
            className={cn(
              "flex w-full items-center justify-between rounded-lg border border-border/50 p-4 transition-colors",
              uiLanguage === lang.value
                ? "border-primary bg-primary/5"
                : "hover:bg-secondary/50"
            )}
          >
            <div>
              <p className="font-medium">{lang.native}</p>
              <p className="text-sm text-muted-foreground">{lang.label}</p>
            </div>
            {uiLanguage === lang.value && (
              <Check className="h-5 w-5 text-primary" />
            )}
          </button>
        ))}
      </div>
    </div>
  )
}

// 显示设置
function DisplaySettings() {
  const [theme, setTheme] = useState("dark")
  const [primaryColor, setPrimaryColor] = useState("teal")
  const [showBackdrop, setShowBackdrop] = useState(true)
  const [showLogo, setShowLogo] = useState(true)
  const [cardStyle, setCardStyle] = useState("poster")

  const themes = [
    { value: "light", label: "浅色" },
    { value: "dark", label: "深色" },
    { value: "system", label: "跟随系统" },
  ]

  const colors = [
    { value: "teal", label: "青色", color: "#14b8a6" },
    { value: "blue", label: "蓝色", color: "#3b82f6" },
    { value: "purple", label: "紫色", color: "#8b5cf6" },
    { value: "pink", label: "粉色", color: "#ec4899" },
    { value: "orange", label: "橙色", color: "#f97316" },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">显示设置</h2>
        <p className="text-sm text-muted-foreground">自定义界面外观</p>
      </div>

      <div className="space-y-4">
        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <Label>主题</Label>
          <div className="flex gap-2">
            {themes.map((t) => (
              <button
                key={t.value}
                onClick={() => setTheme(t.value)}
                className={cn(
                  "flex-1 rounded-md px-3 py-2 text-sm transition-colors",
                  theme === t.value
                    ? "bg-primary text-primary-foreground"
                    : "bg-secondary hover:bg-secondary/80"
                )}
              >
                {t.label}
              </button>
            ))}
          </div>
        </div>

        <div className="space-y-3 rounded-lg border border-border/50 bg-card p-4">
          <Label>主题色</Label>
          <div className="flex gap-2">
            {colors.map((c) => (
              <button
                key={c.value}
                onClick={() => setPrimaryColor(c.value)}
                className={cn(
                  "relative h-10 w-10 rounded-full transition-transform hover:scale-110",
                  primaryColor === c.value && "ring-2 ring-offset-2 ring-offset-background"
                )}
                style={{ backgroundColor: c.color }}
              >
                {primaryColor === c.value && (
                  <Check className="absolute inset-0 m-auto h-5 w-5 text-white" />
                )}
              </button>
            ))}
          </div>
        </div>

        <SettingItem
          title="卡片样式"
          description="选择媒体卡片的显示方式"
        >
          <Select value={cardStyle} onValueChange={setCardStyle}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="poster">海报</SelectItem>
              <SelectItem value="thumb">缩略图</SelectItem>
              <SelectItem value="banner">横幅</SelectItem>
            </SelectContent>
          </Select>
        </SettingItem>

        <SettingItem
          title="显示背景图"
          description="在详情页显示背景图片"
        >
          <Switch checked={showBackdrop} onCheckedChange={setShowBackdrop} />
        </SettingItem>

        <SettingItem
          title="显示媒体 Logo"
          description="在详情页显示电影/剧集 Logo"
        >
          <Switch checked={showLogo} onCheckedChange={setShowLogo} />
        </SettingItem>
      </div>
    </div>
  )
}

// 服务器连接设置
function ServerSettings() {
  const [serverAddress, setServerAddress] = useState("192.168.1.100")
  const [serverPort, setServerPort] = useState("8096")
  const [useHttps, setUseHttps] = useState(false)
  const [quickConnect, setQuickConnect] = useState(true)

  const servers = [
    { name: "家庭服务���", address: "192.168.1.100:8096", status: "connected", current: true },
    { name: "远程服务器", address: "media.example.com", status: "saved", current: false },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-medium">服务器连接</h2>
        <p className="text-sm text-muted-foreground">管理媒体服务器连接</p>
      </div>

      <div className="space-y-4">
        {/* 已保存的服务器 */}
        <div className="space-y-3">
          <Label>已保存的服务器</Label>
          {servers.map((server) => (
            <div
              key={server.address}
              className={cn(
                "flex items-center justify-between rounded-lg border p-4",
                server.current
                  ? "border-primary bg-primary/5"
                  : "border-border/50 bg-card"
              )}
            >
              <div className="flex items-center gap-3">
                <Server className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="font-medium">{server.name}</p>
                  <p className="font-mono text-sm text-muted-foreground">{server.address}</p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {server.status === "connected" ? (
                  <Badge variant="secondary" className="bg-green-500/10 text-green-500">
                    已连接
                  </Badge>
                ) : (
                  <Button variant="outline" size="sm">连接</Button>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* 添加新服务器 */}
        <div className="space-y-4 rounded-lg border border-border/50 bg-card p-4">
          <Label>添加新服务器</Label>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">服务器地址</Label>
              <Input
                value={serverAddress}
                onChange={(e) => setServerAddress(e.target.value)}
                placeholder="192.168.1.100 或 example.com"
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">端口</Label>
              <Input
                value={serverPort}
                onChange={(e) => setServerPort(e.target.value)}
                placeholder="8096"
                className="w-24"
              />
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <Switch checked={useHttps} onCheckedChange={setUseHttps} />
              <Label className="text-sm">使用 HTTPS</Label>
            </div>
          </div>
          <Button className="w-full sm:w-auto">连接服务器</Button>
        </div>

        <SettingItem
          title="快速连接"
          description="使用快速连接码连接到服务器"
        >
          <Switch checked={quickConnect} onCheckedChange={setQuickConnect} />
        </SettingItem>

        {/* 连接信息 */}
        <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
          <h4 className="text-sm font-medium">当前连接信息</h4>
          <div className="mt-2 space-y-1 text-sm text-muted-foreground">
            <p>服务器版本: Nako 1.0.0</p>
            <p>连接延迟: 12ms</p>
            <p>最后同步: 2 分钟前</p>
          </div>
        </div>
      </div>
    </div>
  )
}

// 设置项组件
function SettingItem({ 
  title, 
  description, 
  children 
}: { 
  title: string
  description: string
  children: React.ReactNode 
}) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-border/50 bg-card p-4">
      <div className="flex-1 pr-4">
        <p className="font-medium">{title}</p>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      {children}
    </div>
  )
}
