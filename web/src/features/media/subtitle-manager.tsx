"use client"

import { useState, useCallback } from "react"
import { 
  Subtitles, Upload, Download, Search, Trash2, Check, Plus,
  ChevronDown, Globe, Clock, AlertCircle, RefreshCw, ExternalLink,
  FileText, Settings, Loader2, X, GripVertical
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

// 字幕类型
export interface SubtitleTrack {
  id: string
  language: string
  languageCode: string
  label: string
  format: "srt" | "ass" | "vtt" | "sub"
  source: "embedded" | "external" | "online"
  isDefault: boolean
  isForced: boolean
  isHearingImpaired: boolean
  syncOffset: number // 毫秒
  path?: string
}

// 在线字幕搜索结果
interface OnlineSubtitle {
  id: string
  fileName: string
  language: string
  languageCode: string
  format: string
  source: string // OpenSubtitles, Subscene, etc.
  downloads: number
  rating: number
  uploadedBy?: string
  uploadDate?: string
  hearingImpaired: boolean
}

// 语言选项
const LANGUAGES = [
  { code: "zh-CN", name: "简体中文", flag: "🇨🇳" },
  { code: "zh-TW", name: "繁体中文", flag: "🇹🇼" },
  { code: "en", name: "English", flag: "🇺🇸" },
  { code: "ja", name: "日本語", flag: "🇯🇵" },
  { code: "ko", name: "한국어", flag: "🇰🇷" },
  { code: "fr", name: "Français", flag: "🇫🇷" },
  { code: "de", name: "Deutsch", flag: "🇩🇪" },
  { code: "es", name: "Español", flag: "🇪🇸" },
  { code: "pt", name: "Português", flag: "🇵🇹" },
  { code: "ru", name: "Русский", flag: "🇷🇺" },
]

// Mock 本地字幕
const mockLocalSubtitles: SubtitleTrack[] = [
  {
    id: "sub-1",
    language: "简体中文",
    languageCode: "zh-CN",
    label: "简体中文 (内嵌)",
    format: "ass",
    source: "embedded",
    isDefault: true,
    isForced: false,
    isHearingImpaired: false,
    syncOffset: 0,
  },
  {
    id: "sub-2",
    language: "English",
    languageCode: "en",
    label: "English (Embedded)",
    format: "srt",
    source: "embedded",
    isDefault: false,
    isForced: false,
    isHearingImpaired: false,
    syncOffset: 0,
  },
  {
    id: "sub-3",
    language: "日本語",
    languageCode: "ja",
    label: "日本語 (外部)",
    format: "srt",
    source: "external",
    isDefault: false,
    isForced: false,
    isHearingImpaired: false,
    syncOffset: -500,
    path: "/subtitles/movie.ja.srt",
  },
]

// Mock 在线字幕
const mockOnlineSubtitles: OnlineSubtitle[] = [
  {
    id: "online-1",
    fileName: "Dune.Part.Two.2024.简体中文.srt",
    language: "简体中文",
    languageCode: "zh-CN",
    format: "srt",
    source: "OpenSubtitles",
    downloads: 15420,
    rating: 9.2,
    uploadedBy: "subtitle_master",
    uploadDate: "2024-03-15",
    hearingImpaired: false,
  },
  {
    id: "online-2",
    fileName: "Dune.Part.Two.2024.繁体中文.ass",
    language: "繁体中文",
    languageCode: "zh-TW",
    format: "ass",
    source: "Subscene",
    downloads: 8932,
    rating: 8.8,
    uploadedBy: "tw_subs",
    uploadDate: "2024-03-14",
    hearingImpaired: false,
  },
  {
    id: "online-3",
    fileName: "Dune.Part.Two.2024.English.SDH.srt",
    language: "English",
    languageCode: "en",
    format: "srt",
    source: "OpenSubtitles",
    downloads: 32156,
    rating: 9.5,
    uploadedBy: "eng_subs_pro",
    uploadDate: "2024-03-10",
    hearingImpaired: true,
  },
  {
    id: "online-4",
    fileName: "Dune.Part.Two.2024.日本語.srt",
    language: "日本語",
    languageCode: "ja",
    format: "srt",
    source: "OpenSubtitles",
    downloads: 5621,
    rating: 8.5,
    uploadedBy: "jp_subs",
    uploadDate: "2024-03-16",
    hearingImpaired: false,
  },
]

interface SubtitleManagerProps {
  mediaTitle: string
  mediaYear?: number
  subtitles: SubtitleTrack[]
  currentSubtitleId?: string
  onSelectSubtitle: (subtitleId: string | null) => void
  onAddSubtitle?: (subtitle: SubtitleTrack) => void
  onRemoveSubtitle?: (subtitleId: string) => void
  onUpdateSubtitle?: (subtitleId: string, updates: Partial<SubtitleTrack>) => void
  onDownloadSubtitle?: (onlineSubtitle: OnlineSubtitle) => void
  className?: string
}

export function SubtitleManager({
  mediaTitle,
  mediaYear,
  subtitles = mockLocalSubtitles,
  currentSubtitleId,
  onSelectSubtitle,
  onAddSubtitle,
  onRemoveSubtitle,
  onUpdateSubtitle,
  onDownloadSubtitle,
  className,
}: SubtitleManagerProps) {
  const [activeTab, setActiveTab] = useState("local")
  const [searchQuery, setSearchQuery] = useState("")
  const [searchLanguage, setSearchLanguage] = useState("all")
  const [isSearching, setIsSearching] = useState(false)
  const [onlineResults, setOnlineResults] = useState<OnlineSubtitle[]>([])
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [editingSubtitle, setEditingSubtitle] = useState<SubtitleTrack | null>(null)
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false)

  // 字幕样式设置
  const [subtitleSettings, setSubtitleSettings] = useState({
    fontSize: 24,
    fontFamily: "system",
    textColor: "#FFFFFF",
    backgroundColor: "#000000",
    backgroundOpacity: 50,
    position: "bottom", // bottom, top
    marginBottom: 50,
  })

  // 搜索在线字幕
  const handleSearch = useCallback(async () => {
    setIsSearching(true)
    // 模拟搜索延迟
    await new Promise((resolve) => setTimeout(resolve, 1500))
    
    let results = mockOnlineSubtitles
    if (searchLanguage !== "all") {
      results = results.filter((s) => s.languageCode === searchLanguage)
    }
    if (searchQuery) {
      results = results.filter((s) => 
        s.fileName.toLowerCase().includes(searchQuery.toLowerCase())
      )
    }
    
    setOnlineResults(results)
    setIsSearching(false)
  }, [searchQuery, searchLanguage])

  // 下载字幕
  const handleDownload = async (subtitle: OnlineSubtitle) => {
    // 模拟下载
    await new Promise((resolve) => setTimeout(resolve, 1000))
    
    const newSubtitle: SubtitleTrack = {
      id: `downloaded-${subtitle.id}`,
      language: subtitle.language,
      languageCode: subtitle.languageCode,
      label: `${subtitle.language} (${subtitle.source})`,
      format: subtitle.format as SubtitleTrack["format"],
      source: "external",
      isDefault: false,
      isForced: false,
      isHearingImpaired: subtitle.hearingImpaired,
      syncOffset: 0,
      path: `/subtitles/${subtitle.fileName}`,
    }
    
    onAddSubtitle?.(newSubtitle)
    onDownloadSubtitle?.(subtitle)
  }

  return (
    <div className={cn("flex flex-col rounded-xl border border-border bg-card", className)}>
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <Subtitles className="h-5 w-5 text-muted-foreground" />
          <h3 className="font-medium">字幕管理</h3>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon" onClick={() => setUploadDialogOpen(true)}>
            <Upload className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" onClick={() => setSettingsOpen(true)}>
            <Settings className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
        <div className="border-b border-border px-4">
          <TabsList className="h-10">
            <TabsTrigger value="local" className="gap-2">
              <FileText className="h-4 w-4" />
              本地字幕 ({subtitles.length})
            </TabsTrigger>
            <TabsTrigger value="search" className="gap-2">
              <Search className="h-4 w-4" />
              在线搜索
            </TabsTrigger>
          </TabsList>
        </div>

        {/* Local Subtitles */}
        <TabsContent value="local" className="mt-0 flex-1">
          <ScrollArea className="h-[400px]">
            {subtitles.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12">
                <Subtitles className="mb-3 h-10 w-10 text-muted-foreground/30" />
                <p className="text-sm text-muted-foreground">暂无字幕</p>
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-3"
                  onClick={() => setUploadDialogOpen(true)}
                >
                  <Plus className="mr-2 h-4 w-4" />
                  添加字幕
                </Button>
              </div>
            ) : (
              <div className="divide-y divide-border">
                {/* 关闭字幕选项 */}
                <button
                  className={cn(
                    "flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-muted/50",
                    !currentSubtitleId && "bg-primary/5"
                  )}
                  onClick={() => onSelectSubtitle(null)}
                >
                  <div className={cn(
                    "flex h-5 w-5 items-center justify-center rounded-full border-2",
                    !currentSubtitleId ? "border-primary bg-primary text-primary-foreground" : "border-muted-foreground"
                  )}>
                    {!currentSubtitleId && <Check className="h-3 w-3" />}
                  </div>
                  <span className="text-sm">关闭字幕</span>
                </button>

                {/* 字幕列表 */}
                {subtitles.map((subtitle) => (
                  <SubtitleTrackItem
                    key={subtitle.id}
                    subtitle={subtitle}
                    isSelected={currentSubtitleId === subtitle.id}
                    onSelect={() => onSelectSubtitle(subtitle.id)}
                    onEdit={() => setEditingSubtitle(subtitle)}
                    onRemove={() => onRemoveSubtitle?.(subtitle.id)}
                    onUpdateOffset={(offset) => onUpdateSubtitle?.(subtitle.id, { syncOffset: offset })}
                  />
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>

        {/* Online Search */}
        <TabsContent value="search" className="mt-0 flex-1">
          <div className="border-b border-border p-4">
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder={`搜索 "${mediaTitle}" 的字幕...`}
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                  onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                />
              </div>
              <Select value={searchLanguage} onValueChange={setSearchLanguage}>
                <SelectTrigger className="w-[140px]">
                  <Globe className="mr-2 h-4 w-4" />
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">所有语言</SelectItem>
                  {LANGUAGES.map((lang) => (
                    <SelectItem key={lang.code} value={lang.code}>
                      {lang.flag} {lang.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button onClick={handleSearch} disabled={isSearching}>
                {isSearching ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Search className="h-4 w-4" />
                )}
              </Button>
            </div>
          </div>

          <ScrollArea className="h-[350px]">
            {isSearching ? (
              <div className="flex flex-col items-center justify-center py-12">
                <Loader2 className="mb-3 h-8 w-8 animate-spin text-primary" />
                <p className="text-sm text-muted-foreground">搜索中...</p>
              </div>
            ) : onlineResults.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12">
                <Search className="mb-3 h-10 w-10 text-muted-foreground/30" />
                <p className="text-sm text-muted-foreground">
                  {activeTab === "search" && onlineResults.length === 0 
                    ? "点击搜索按钮查找字幕" 
                    : "未找到匹配的字幕"}
                </p>
              </div>
            ) : (
              <div className="divide-y divide-border">
                {onlineResults.map((subtitle) => (
                  <OnlineSubtitleItem
                    key={subtitle.id}
                    subtitle={subtitle}
                    onDownload={() => handleDownload(subtitle)}
                  />
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>
      </Tabs>

      {/* Subtitle Settings Dialog */}
      <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>字幕设置</DialogTitle>
            <DialogDescription>调整字幕的显示样式</DialogDescription>
          </DialogHeader>
          
          <div className="space-y-6 py-4">
            {/* Font Size */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>字体大小</Label>
                <span className="text-sm text-muted-foreground">{subtitleSettings.fontSize}px</span>
              </div>
              <Slider
                value={[subtitleSettings.fontSize]}
                onValueChange={([v]) => setSubtitleSettings((s) => ({ ...s, fontSize: v }))}
                min={12}
                max={48}
                step={2}
              />
            </div>

            {/* Font Family */}
            <div className="space-y-2">
              <Label>字体</Label>
              <Select
                value={subtitleSettings.fontFamily}
                onValueChange={(v) => setSubtitleSettings((s) => ({ ...s, fontFamily: v }))}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="system">系统默认</SelectItem>
                  <SelectItem value="sans">无衬线</SelectItem>
                  <SelectItem value="serif">衬线</SelectItem>
                  <SelectItem value="mono">等宽</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Background Opacity */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>背景透明度</Label>
                <span className="text-sm text-muted-foreground">{subtitleSettings.backgroundOpacity}%</span>
              </div>
              <Slider
                value={[subtitleSettings.backgroundOpacity]}
                onValueChange={([v]) => setSubtitleSettings((s) => ({ ...s, backgroundOpacity: v }))}
                min={0}
                max={100}
                step={10}
              />
            </div>

            {/* Margin Bottom */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>底部边距</Label>
                <span className="text-sm text-muted-foreground">{subtitleSettings.marginBottom}px</span>
              </div>
              <Slider
                value={[subtitleSettings.marginBottom]}
                onValueChange={([v]) => setSubtitleSettings((s) => ({ ...s, marginBottom: v }))}
                min={0}
                max={150}
                step={10}
              />
            </div>

            {/* Preview */}
            <div className="relative overflow-hidden rounded-lg bg-black">
              <div className="aspect-video bg-gradient-to-b from-gray-800 to-gray-900" />
              <div
                className="absolute inset-x-0 flex justify-center"
                style={{ bottom: `${subtitleSettings.marginBottom / 3}px` }}
              >
                <span
                  className="rounded px-3 py-1 text-white"
                  style={{
                    fontSize: `${subtitleSettings.fontSize / 2}px`,
                    backgroundColor: `rgba(0,0,0,${subtitleSettings.backgroundOpacity / 100})`,
                    fontFamily: subtitleSettings.fontFamily === "system" ? "inherit" : subtitleSettings.fontFamily,
                  }}
                >
                  字幕预览效果
                </span>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setSettingsOpen(false)}>
              取消
            </Button>
            <Button onClick={() => setSettingsOpen(false)}>
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Upload Dialog */}
      <Dialog open={uploadDialogOpen} onOpenChange={setUploadDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加字幕文件</DialogTitle>
            <DialogDescription>支持 SRT、ASS、VTT、SUB 格式</DialogDescription>
          </DialogHeader>
          
          <div className="py-4">
            <div className="flex flex-col items-center justify-center rounded-lg border-2 border-dashed border-muted-foreground/25 p-8">
              <Upload className="mb-3 h-10 w-10 text-muted-foreground/50" />
              <p className="mb-2 text-sm font-medium">拖放字幕文件到这里</p>
              <p className="mb-4 text-xs text-muted-foreground">或者点击下方按钮选择文件</p>
              <Button variant="outline" size="sm">
                选择文件
              </Button>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setUploadDialogOpen(false)}>
              取消
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Subtitle Dialog */}
      {editingSubtitle && (
        <Dialog open={!!editingSubtitle} onOpenChange={() => setEditingSubtitle(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>编辑字幕</DialogTitle>
            </DialogHeader>
            
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label>标签</Label>
                <Input
                  value={editingSubtitle.label}
                  onChange={(e) => setEditingSubtitle({ ...editingSubtitle, label: e.target.value })}
                />
              </div>
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label>同步偏移</Label>
                  <span className="text-sm text-muted-foreground">
                    {editingSubtitle.syncOffset > 0 ? "+" : ""}{editingSubtitle.syncOffset}ms
                  </span>
                </div>
                <Slider
                  value={[editingSubtitle.syncOffset]}
                  onValueChange={([v]) => setEditingSubtitle({ ...editingSubtitle, syncOffset: v })}
                  min={-5000}
                  max={5000}
                  step={100}
                />
                <p className="text-xs text-muted-foreground">
                  正值表示字幕延迟显示，负值表示字幕提前显示
                </p>
              </div>

              <div className="flex items-center justify-between">
                <Label>设为默认</Label>
                <Switch
                  checked={editingSubtitle.isDefault}
                  onCheckedChange={(v) => setEditingSubtitle({ ...editingSubtitle, isDefault: v })}
                />
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setEditingSubtitle(null)}>
                取消
              </Button>
              <Button onClick={() => {
                onUpdateSubtitle?.(editingSubtitle.id, editingSubtitle)
                setEditingSubtitle(null)
              }}>
                保存
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </div>
  )
}

// 本地字幕项
function SubtitleTrackItem({
  subtitle,
  isSelected,
  onSelect,
  onEdit,
  onRemove,
  onUpdateOffset,
}: {
  subtitle: SubtitleTrack
  isSelected: boolean
  onSelect: () => void
  onEdit: () => void
  onRemove: () => void
  onUpdateOffset: (offset: number) => void
}) {
  const lang = LANGUAGES.find((l) => l.code === subtitle.languageCode)

  return (
    <div
      className={cn(
        "group flex items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/50",
        isSelected && "bg-primary/5"
      )}
    >
      <button
        className={cn(
          "flex h-5 w-5 items-center justify-center rounded-full border-2",
          isSelected ? "border-primary bg-primary text-primary-foreground" : "border-muted-foreground"
        )}
        onClick={onSelect}
      >
        {isSelected && <Check className="h-3 w-3" />}
      </button>

      <div className="flex-1">
        <div className="flex items-center gap-2">
          {lang && <span>{lang.flag}</span>}
          <span className="text-sm font-medium">{subtitle.label}</span>
          {subtitle.isDefault && (
            <Badge variant="secondary" className="text-xs">默认</Badge>
          )}
          {subtitle.isHearingImpaired && (
            <Badge variant="outline" className="text-xs">CC</Badge>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="uppercase">{subtitle.format}</span>
          <span>·</span>
          <span>{subtitle.source === "embedded" ? "内嵌" : "外部"}</span>
          {subtitle.syncOffset !== 0 && (
            <>
              <span>·</span>
              <span className={subtitle.syncOffset > 0 ? "text-orange-500" : "text-blue-500"}>
                {subtitle.syncOffset > 0 ? "+" : ""}{subtitle.syncOffset}ms
              </span>
            </>
          )}
        </div>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 opacity-0 group-hover:opacity-100"
          >
            <ChevronDown className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={onEdit}>
            <Settings className="mr-2 h-4 w-4" />
            编辑
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onUpdateOffset(subtitle.syncOffset - 500)}>
            <Clock className="mr-2 h-4 w-4" />
            字幕提前 0.5s
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onUpdateOffset(subtitle.syncOffset + 500)}>
            <Clock className="mr-2 h-4 w-4" />
            字幕延迟 0.5s
          </DropdownMenuItem>
          {subtitle.source === "external" && (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={onRemove} className="text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                移除
              </DropdownMenuItem>
            </>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}

// 在线字幕项
function OnlineSubtitleItem({
  subtitle,
  onDownload,
}: {
  subtitle: OnlineSubtitle
  onDownload: () => void
}) {
  const [isDownloading, setIsDownloading] = useState(false)
  const lang = LANGUAGES.find((l) => l.code === subtitle.languageCode)

  const handleDownload = async () => {
    setIsDownloading(true)
    await onDownload()
    setIsDownloading(false)
  }

  return (
    <div className="flex items-center gap-3 px-4 py-3 hover:bg-muted/50">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          {lang && <span>{lang.flag}</span>}
          <span className="text-sm font-medium">{subtitle.language}</span>
          {subtitle.hearingImpaired && (
            <Badge variant="outline" className="text-xs">CC</Badge>
          )}
        </div>
        <p className="truncate text-xs text-muted-foreground">{subtitle.fileName}</p>
        <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
          <span>{subtitle.source}</span>
          <span>·</span>
          <span className="uppercase">{subtitle.format}</span>
          <span>·</span>
          <span>{subtitle.downloads.toLocaleString()} 下载</span>
          <span>·</span>
          <span className="flex items-center gap-0.5">
            ★ {subtitle.rating.toFixed(1)}
          </span>
        </div>
      </div>

      <Button
        variant="outline"
        size="sm"
        onClick={handleDownload}
        disabled={isDownloading}
      >
        {isDownloading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <Download className="h-4 w-4" />
        )}
      </Button>
    </div>
  )
}
