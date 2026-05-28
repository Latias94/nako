"use client"

import { useState, useRef, useEffect, useCallback } from "react"
import { 
  ChevronLeft, Grid3X3, Calendar, MapPin, Heart, Share2, Download, Trash2,
  ZoomIn, ZoomOut, RotateCw, Info, FolderOpen, Plus, Search, SlidersHorizontal,
  Image as ImageIcon, Video, Check, X, MoreHorizontal, Upload, Cloud, Star
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Skeleton } from "@/components/ui/skeleton"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Slider } from "@/components/ui/slider"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

// Types
interface PhotoItem {
  id: string
  url: string
  thumbnail: string
  title?: string
  date: string
  location?: string
  width: number
  height: number
  size: number
  type: "photo" | "video"
  duration?: number
  isFavorite: boolean
  albumIds: string[]
  tags: string[]
  exif?: {
    camera?: string
    lens?: string
    aperture?: string
    shutter?: string
    iso?: number
    focalLength?: string
  }
}

interface Album {
  id: string
  name: string
  cover: string
  count: number
  createdAt: string
  isShared: boolean
}

interface PhotoLibraryProps {
  onBack: () => void
  onSelectPhoto?: (photo: PhotoItem) => void
}

// Mock data generator
const generateMockPhotos = (count: number): PhotoItem[] => {
  const locations = ["东京", "巴黎", "纽约", "上海", "伦敦", "悉尼", "北京", "首尔"]
  const tags = ["风景", "人像", "建筑", "美食", "动物", "夜景", "街拍", "旅行"]
  
  return Array.from({ length: count }, (_, i) => ({
    id: `photo-${i}`,
    url: `https://picsum.photos/seed/${i}/1920/1080`,
    thumbnail: `https://picsum.photos/seed/${i}/400/300`,
    title: i % 5 === 0 ? `照片 ${i + 1}` : undefined,
    date: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString(),
    location: Math.random() > 0.3 ? locations[i % locations.length] : undefined,
    width: 1920,
    height: 1080,
    size: Math.floor(2 + Math.random() * 8) * 1024 * 1024,
    type: Math.random() > 0.9 ? "video" : "photo",
    duration: Math.random() > 0.9 ? Math.floor(10 + Math.random() * 300) : undefined,
    isFavorite: Math.random() > 0.8,
    albumIds: [],
    tags: [tags[i % tags.length], tags[(i + 3) % tags.length]],
    exif: {
      camera: ["Sony A7IV", "Canon R5", "Nikon Z8", "Fuji X-T5"][i % 4],
      lens: "24-70mm f/2.8",
      aperture: "f/2.8",
      shutter: "1/250s",
      iso: [100, 200, 400, 800, 1600][i % 5],
      focalLength: "50mm",
    },
  }))
}

const mockAlbums: Album[] = [
  { id: "1", name: "2024 日本旅行", cover: "https://picsum.photos/seed/japan/400/300", count: 156, createdAt: "2024-03-15", isShared: true },
  { id: "2", name: "家庭聚会", cover: "https://picsum.photos/seed/family/400/300", count: 89, createdAt: "2024-02-20", isShared: false },
  { id: "3", name: "风景收藏", cover: "https://picsum.photos/seed/landscape/400/300", count: 234, createdAt: "2023-12-01", isShared: false },
  { id: "4", name: "美食记录", cover: "https://picsum.photos/seed/food/400/300", count: 67, createdAt: "2024-01-10", isShared: true },
  { id: "5", name: "城市建筑", cover: "https://picsum.photos/seed/city/400/300", count: 112, createdAt: "2023-11-05", isShared: false },
]

type ViewMode = "timeline" | "albums" | "places" | "favorites"

export function PhotoLibrary({ onBack, onSelectPhoto }: PhotoLibraryProps) {
  const [viewMode, setViewMode] = useState<ViewMode>("timeline")
  const [photos] = useState(() => generateMockPhotos(200))
  const [selectedPhotos, setSelectedPhotos] = useState<Set<string>>(new Set())
  const [isSelectionMode, setIsSelectionMode] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")
  const [gridSize, setGridSize] = useState(180)
  const [selectedPhoto, setSelectedPhoto] = useState<PhotoItem | null>(null)
  const [showInfo, setShowInfo] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [zoom, setZoom] = useState(1)
  const [rotation, setRotation] = useState(0)

  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 500)
    return () => clearTimeout(timer)
  }, [])

  // Group photos by date
  const photosByDate = photos.reduce((acc, photo) => {
    const date = new Date(photo.date).toLocaleDateString("zh-CN", { year: "numeric", month: "long", day: "numeric" })
    if (!acc[date]) acc[date] = []
    acc[date].push(photo)
    return acc
  }, {} as Record<string, PhotoItem[]>)

  // Group photos by location
  const photosByLocation = photos.reduce((acc, photo) => {
    const loc = photo.location || "未知位置"
    if (!acc[loc]) acc[loc] = []
    acc[loc].push(photo)
    return acc
  }, {} as Record<string, PhotoItem[]>)

  const favoritePhotos = photos.filter(p => p.isFavorite)

  const toggleSelect = (id: string) => {
    const newSelected = new Set(selectedPhotos)
    if (newSelected.has(id)) {
      newSelected.delete(id)
    } else {
      newSelected.add(id)
    }
    setSelectedPhotos(newSelected)
  }

  const handlePhotoClick = (photo: PhotoItem) => {
    if (isSelectionMode) {
      toggleSelect(photo.id)
    } else {
      setSelectedPhoto(photo)
      onSelectPhoto?.(photo)
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
    return `${(bytes / 1024).toFixed(0)} KB`
  }

  const formatDuration = (seconds: number) => {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${s.toString().padStart(2, "0")}`
  }

  // Photo Grid Item
  const PhotoGridItem = ({ photo }: { photo: PhotoItem }) => (
    <div
      className={cn(
        "group relative cursor-pointer overflow-hidden rounded-lg bg-muted",
        isSelectionMode && selectedPhotos.has(photo.id) && "ring-2 ring-primary"
      )}
      style={{ aspectRatio: "1" }}
      onClick={() => handlePhotoClick(photo)}
    >
      <img
        src={photo.thumbnail}
        alt={photo.title || ""}
        className="h-full w-full object-cover transition-transform group-hover:scale-105"
        loading="lazy"
      />
      
      {/* Selection checkbox */}
      {isSelectionMode && (
        <div className={cn(
          "absolute left-2 top-2 flex h-6 w-6 items-center justify-center rounded-full border-2 transition-colors",
          selectedPhotos.has(photo.id) ? "border-primary bg-primary text-primary-foreground" : "border-white bg-black/30"
        )}>
          {selectedPhotos.has(photo.id) && <Check className="h-4 w-4" />}
        </div>
      )}

      {/* Video indicator */}
      {photo.type === "video" && (
        <div className="absolute bottom-2 right-2 flex items-center gap-1 rounded bg-black/70 px-1.5 py-0.5 text-xs text-white">
          <Video className="h-3 w-3" />
          {photo.duration && formatDuration(photo.duration)}
        </div>
      )}

      {/* Favorite indicator */}
      {photo.isFavorite && !isSelectionMode && (
        <Heart className="absolute right-2 top-2 h-4 w-4 fill-red-500 text-red-500" />
      )}

      {/* Hover overlay */}
      {!isSelectionMode && (
        <div className="absolute inset-0 flex items-end bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 transition-opacity group-hover:opacity-100">
          <div className="w-full p-2">
            {photo.title && <p className="truncate text-sm font-medium text-white">{photo.title}</p>}
            <p className="text-xs text-white/80">
              {new Date(photo.date).toLocaleDateString("zh-CN")}
              {photo.location && ` · ${photo.location}`}
            </p>
          </div>
        </div>
      )}
    </div>
  )

  // Album Card
  const AlbumCard = ({ album }: { album: Album }) => (
    <div className="group cursor-pointer">
      <div className="relative mb-2 overflow-hidden rounded-xl">
        <img
          src={album.cover}
          alt={album.name}
          className="aspect-square w-full object-cover transition-transform group-hover:scale-105"
        />
        {album.isShared && (
          <Badge className="absolute right-2 top-2 bg-black/70">
            <Share2 className="mr-1 h-3 w-3" />
            已共享
          </Badge>
        )}
      </div>
      <h3 className="font-medium">{album.name}</h3>
      <p className="text-sm text-muted-foreground">{album.count} 张</p>
    </div>
  )

  // Location Card
  const LocationCard = ({ location, photos }: { location: string; photos: PhotoItem[] }) => (
    <div className="group cursor-pointer">
      <div className="relative mb-2 overflow-hidden rounded-xl">
        <img
          src={photos[0]?.thumbnail}
          alt={location}
          className="aspect-video w-full object-cover transition-transform group-hover:scale-105"
        />
        <div className="absolute inset-0 flex items-end bg-gradient-to-t from-black/70 via-transparent to-transparent p-3">
          <div className="flex items-center gap-2">
            <MapPin className="h-4 w-4 text-white" />
            <span className="font-medium text-white">{location}</span>
          </div>
        </div>
      </div>
      <p className="text-sm text-muted-foreground">{photos.length} 张照片</p>
    </div>
  )

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-center gap-2">
            <img src="/nako-icon.png" alt="Nako" className="h-8 w-8 rounded-lg" />
            <h1 className="text-lg font-semibold">图片库</h1>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜索照片..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-64 pl-9"
            />
          </div>
          
          <Button variant="outline" size="icon">
            <Upload className="h-4 w-4" />
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="icon">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setIsSelectionMode(!isSelectionMode)}>
                {isSelectionMode ? "取消选择" : "选择照片"}
              </DropdownMenuItem>
              <DropdownMenuItem>
                <Cloud className="mr-2 h-4 w-4" />
                同步设置
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem>
                <Trash2 className="mr-2 h-4 w-4" />
                回收站
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      {/* View tabs */}
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as ViewMode)}>
          <TabsList>
            <TabsTrigger value="timeline" className="gap-2">
              <Calendar className="h-4 w-4" />
              时间线
            </TabsTrigger>
            <TabsTrigger value="albums" className="gap-2">
              <FolderOpen className="h-4 w-4" />
              相册
            </TabsTrigger>
            <TabsTrigger value="places" className="gap-2">
              <MapPin className="h-4 w-4" />
              地点
            </TabsTrigger>
            <TabsTrigger value="favorites" className="gap-2">
              <Heart className="h-4 w-4" />
              收藏
            </TabsTrigger>
          </TabsList>
        </Tabs>

        <div className="flex items-center gap-3">
          <span className="text-sm text-muted-foreground">网格大小</span>
          <Slider
            value={[gridSize]}
            onValueChange={([v]) => setGridSize(v)}
            min={120}
            max={280}
            step={20}
            className="w-24"
          />
        </div>
      </div>

      {/* Selection bar */}
      {isSelectionMode && selectedPhotos.size > 0 && (
        <div className="flex items-center justify-between border-b border-border bg-muted/50 px-4 py-2">
          <span className="text-sm">已选择 {selectedPhotos.size} 张</span>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={() => setSelectedPhotos(new Set())}>
              取消全选
            </Button>
            <Button variant="ghost" size="sm">
              <Heart className="mr-2 h-4 w-4" />
              收藏
            </Button>
            <Button variant="ghost" size="sm">
              <FolderOpen className="mr-2 h-4 w-4" />
              添加到相册
            </Button>
            <Button variant="ghost" size="sm">
              <Download className="mr-2 h-4 w-4" />
              下载
            </Button>
            <Button variant="ghost" size="sm" className="text-destructive">
              <Trash2 className="mr-2 h-4 w-4" />
              删除
            </Button>
          </div>
        </div>
      )}

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="p-4">
          {/* Timeline View */}
          {viewMode === "timeline" && (
            <div className="space-y-8">
              {isLoading ? (
                <div className="grid gap-2" style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${gridSize}px, 1fr))` }}>
                  {Array.from({ length: 20 }).map((_, i) => (
                    <Skeleton key={i} className="aspect-square rounded-lg" />
                  ))}
                </div>
              ) : (
                Object.entries(photosByDate)
                  .sort(([a], [b]) => new Date(b).getTime() - new Date(a).getTime())
                  .map(([date, datePhotos]) => (
                    <section key={date}>
                      <h2 className="mb-3 text-lg font-semibold">{date}</h2>
                      <div 
                        className="grid gap-2"
                        style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${gridSize}px, 1fr))` }}
                      >
                        {datePhotos.map(photo => (
                          <PhotoGridItem key={photo.id} photo={photo} />
                        ))}
                      </div>
                    </section>
                  ))
              )}
            </div>
          )}

          {/* Albums View */}
          {viewMode === "albums" && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">我的相册</h2>
                <Button variant="outline" size="sm">
                  <Plus className="mr-2 h-4 w-4" />
                  新建相册
                </Button>
              </div>
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                {mockAlbums.map(album => (
                  <AlbumCard key={album.id} album={album} />
                ))}
              </div>
            </div>
          )}

          {/* Places View */}
          {viewMode === "places" && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold">按地点浏览</h2>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                {Object.entries(photosByLocation)
                  .filter(([loc]) => loc !== "未知位置")
                  .map(([location, locPhotos]) => (
                    <LocationCard key={location} location={location} photos={locPhotos} />
                  ))}
              </div>
            </div>
          )}

          {/* Favorites View */}
          {viewMode === "favorites" && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">收藏的照片</h2>
                <span className="text-sm text-muted-foreground">{favoritePhotos.length} 张</span>
              </div>
              <div 
                className="grid gap-2"
                style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${gridSize}px, 1fr))` }}
              >
                {favoritePhotos.map(photo => (
                  <PhotoGridItem key={photo.id} photo={photo} />
                ))}
              </div>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Photo Viewer Dialog */}
      <Dialog open={!!selectedPhoto} onOpenChange={() => { setSelectedPhoto(null); setZoom(1); setRotation(0) }}>
        <DialogContent className="max-w-6xl p-0">
          <div className="relative flex h-[80vh] flex-col">
            {/* Viewer header */}
            <div className="flex items-center justify-between border-b border-border p-3">
              <div>
                {selectedPhoto?.title && <h3 className="font-medium">{selectedPhoto.title}</h3>}
                <p className="text-sm text-muted-foreground">
                  {selectedPhoto && new Date(selectedPhoto.date).toLocaleDateString("zh-CN", { 
                    year: "numeric", month: "long", day: "numeric", hour: "2-digit", minute: "2-digit" 
                  })}
                </p>
              </div>
              <div className="flex items-center gap-1">
                <Button variant="ghost" size="icon" onClick={() => setZoom(z => Math.max(0.5, z - 0.25))}>
                  <ZoomOut className="h-4 w-4" />
                </Button>
                <span className="w-12 text-center text-sm">{Math.round(zoom * 100)}%</span>
                <Button variant="ghost" size="icon" onClick={() => setZoom(z => Math.min(3, z + 0.25))}>
                  <ZoomIn className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" onClick={() => setRotation(r => (r + 90) % 360)}>
                  <RotateCw className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" onClick={() => setShowInfo(!showInfo)}>
                  <Info className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {/* Image display */}
            <div className="relative flex flex-1 items-center justify-center overflow-hidden bg-black">
              {selectedPhoto && (
                <img
                  src={selectedPhoto.url}
                  alt={selectedPhoto.title || ""}
                  className="max-h-full max-w-full object-contain transition-transform"
                  style={{ transform: `scale(${zoom}) rotate(${rotation}deg)` }}
                />
              )}

              {/* Info panel */}
              {showInfo && selectedPhoto && (
                <div className="absolute right-0 top-0 h-full w-80 overflow-y-auto scrollbar-none border-l border-border bg-background p-4">
                  <h4 className="mb-4 font-semibold">照片信息</h4>
                  
                  <div className="space-y-4 text-sm">
                    <div>
                      <p className="text-muted-foreground">尺寸</p>
                      <p>{selectedPhoto.width} x {selectedPhoto.height}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">文件大小</p>
                      <p>{formatFileSize(selectedPhoto.size)}</p>
                    </div>
                    {selectedPhoto.location && (
                      <div>
                        <p className="text-muted-foreground">位置</p>
                        <p className="flex items-center gap-1">
                          <MapPin className="h-3 w-3" />
                          {selectedPhoto.location}
                        </p>
                      </div>
                    )}
                    {selectedPhoto.exif && (
                      <>
                        <div className="border-t border-border pt-4">
                          <p className="mb-2 font-medium">EXIF 信息</p>
                        </div>
                        <div>
                          <p className="text-muted-foreground">相机</p>
                          <p>{selectedPhoto.exif.camera}</p>
                        </div>
                        <div>
                          <p className="text-muted-foreground">镜头</p>
                          <p>{selectedPhoto.exif.lens}</p>
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                          <div>
                            <p className="text-muted-foreground">光圈</p>
                            <p>{selectedPhoto.exif.aperture}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">快门</p>
                            <p>{selectedPhoto.exif.shutter}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">ISO</p>
                            <p>{selectedPhoto.exif.iso}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">焦距</p>
                            <p>{selectedPhoto.exif.focalLength}</p>
                          </div>
                        </div>
                      </>
                    )}
                    {selectedPhoto.tags.length > 0 && (
                      <div className="border-t border-border pt-4">
                        <p className="mb-2 text-muted-foreground">标签</p>
                        <div className="flex flex-wrap gap-1">
                          {selectedPhoto.tags.map(tag => (
                            <Badge key={tag} variant="secondary">{tag}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>

            {/* Action bar */}
            <div className="flex items-center justify-center gap-2 border-t border-border p-3">
              <Button variant="ghost" size="sm">
                <Heart className={cn("mr-2 h-4 w-4", selectedPhoto?.isFavorite && "fill-red-500 text-red-500")} />
                收藏
              </Button>
              <Button variant="ghost" size="sm">
                <Share2 className="mr-2 h-4 w-4" />
                分享
              </Button>
              <Button variant="ghost" size="sm">
                <Download className="mr-2 h-4 w-4" />
                下载
              </Button>
              <Button variant="ghost" size="sm">
                <FolderOpen className="mr-2 h-4 w-4" />
                添加到相册
              </Button>
              <Button variant="ghost" size="sm" className="text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                删除
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
