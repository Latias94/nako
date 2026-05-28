"use client"

import { useState, useEffect, useCallback } from "react"
import { 
  X, 
  ChevronLeft, 
  ChevronRight, 
  ZoomIn, 
  ZoomOut,
  Download,
  Maximize,
  RotateCw,
  Image as ImageIcon,
  Grid3X3,
  Info
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

interface ImageViewerProps {
  images: {
    id: string
    url: string | null
    title?: string
    type?: "poster" | "backdrop" | "still" | "logo"
    resolution?: string
  }[]
  initialIndex?: number
  onClose: () => void
  mediaTitle?: string
}

export function ImageViewer({
  images,
  initialIndex = 0,
  onClose,
  mediaTitle,
}: ImageViewerProps) {
  const [currentIndex, setCurrentIndex] = useState(initialIndex)
  const [zoom, setZoom] = useState(1)
  const [rotation, setRotation] = useState(0)
  const [showThumbnails, setShowThumbnails] = useState(true)
  const [showInfo, setShowInfo] = useState(false)

  const currentImage = images[currentIndex]

  // 导航
  const goNext = useCallback(() => {
    setCurrentIndex((prev) => (prev + 1) % images.length)
    setZoom(1)
    setRotation(0)
  }, [images.length])

  const goPrevious = useCallback(() => {
    setCurrentIndex((prev) => (prev - 1 + images.length) % images.length)
    setZoom(1)
    setRotation(0)
  }, [images.length])

  // 缩放
  const zoomIn = () => setZoom((prev) => Math.min(3, prev + 0.25))
  const zoomOut = () => setZoom((prev) => Math.max(0.5, prev - 0.25))
  const resetZoom = () => {
    setZoom(1)
    setRotation(0)
  }

  // 旋转
  const rotate = () => setRotation((prev) => (prev + 90) % 360)

  // 键盘控制
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          onClose()
          break
        case "ArrowLeft":
          goPrevious()
          break
        case "ArrowRight":
          goNext()
          break
        case "+":
        case "=":
          zoomIn()
          break
        case "-":
          zoomOut()
          break
        case "r":
          rotate()
          break
        case "0":
          resetZoom()
          break
        case "g":
          setShowThumbnails((prev) => !prev)
          break
        case "i":
          setShowInfo((prev) => !prev)
          break
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [goNext, goPrevious, onClose])

  // 鼠标滚轮缩放
  useEffect(() => {
    const handleWheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault()
        if (e.deltaY < 0) {
          zoomIn()
        } else {
          zoomOut()
        }
      }
    }

    window.addEventListener("wheel", handleWheel, { passive: false })
    return () => window.removeEventListener("wheel", handleWheel)
  }, [])

  const getTypeLabel = (type?: string) => {
    switch (type) {
      case "poster":
        return "海报"
      case "backdrop":
        return "背景图"
      case "still":
        return "剧照"
      case "logo":
        return "Logo"
      default:
        return "图片"
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-black">
      {/* 顶部工具栏 */}
      <div className="flex items-center justify-between bg-black/60 px-4 py-3 backdrop-blur-sm">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-10 w-10 text-white hover:bg-white/10"
          >
            <X className="h-5 w-5" />
          </Button>
          <div>
            <h2 className="text-sm font-medium text-white">
              {mediaTitle || "图片预览"}
            </h2>
            <p className="text-xs text-white/60">
              {currentIndex + 1} / {images.length}
              {currentImage?.type && (
                <span className="ml-2">{getTypeLabel(currentImage.type)}</span>
              )}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1">
          {/* 缩放控制 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={zoomOut}
            disabled={zoom <= 0.5}
            className="h-9 w-9 text-white hover:bg-white/10 disabled:opacity-30"
          >
            <ZoomOut className="h-4 w-4" />
          </Button>
          <span className="w-12 text-center text-xs text-white/80">
            {Math.round(zoom * 100)}%
          </span>
          <Button
            variant="ghost"
            size="icon"
            onClick={zoomIn}
            disabled={zoom >= 3}
            className="h-9 w-9 text-white hover:bg-white/10 disabled:opacity-30"
          >
            <ZoomIn className="h-4 w-4" />
          </Button>

          <div className="mx-2 h-6 w-px bg-white/20" />

          {/* 旋转 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={rotate}
            className="h-9 w-9 text-white hover:bg-white/10"
          >
            <RotateCw className="h-4 w-4" />
          </Button>

          {/* 重置 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={resetZoom}
            className="h-9 w-9 text-white hover:bg-white/10"
            title="重置 (0)"
          >
            <Maximize className="h-4 w-4" />
          </Button>

          <div className="mx-2 h-6 w-px bg-white/20" />

          {/* 缩略图切换 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setShowThumbnails((prev) => !prev)}
            className={cn(
              "h-9 w-9 text-white hover:bg-white/10",
              showThumbnails && "bg-white/10"
            )}
            title="缩略图 (G)"
          >
            <Grid3X3 className="h-4 w-4" />
          </Button>

          {/* 信息 */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setShowInfo((prev) => !prev)}
            className={cn(
              "h-9 w-9 text-white hover:bg-white/10",
              showInfo && "bg-white/10"
            )}
            title="详细信息 (I)"
          >
            <Info className="h-4 w-4" />
          </Button>

          {/* 下载 */}
          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9 text-white hover:bg-white/10"
            title="下载"
          >
            <Download className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* 主图片区域 */}
      <div className="relative flex flex-1 items-center justify-center overflow-hidden">
        {/* 左箭头 */}
        <Button
          variant="ghost"
          size="icon"
          onClick={goPrevious}
          className="absolute left-4 z-10 h-12 w-12 rounded-full bg-black/40 text-white hover:bg-black/60"
        >
          <ChevronLeft className="h-6 w-6" />
        </Button>

        {/* 图片 */}
        <div 
          className="flex h-full w-full items-center justify-center"
          onDoubleClick={() => {
            if (zoom === 1) {
              setZoom(2)
            } else {
              setZoom(1)
            }
          }}
        >
          {currentImage?.url ? (
            <img
              src={currentImage.url}
              alt={currentImage.title || `图片 ${currentIndex + 1}`}
              className="max-h-full max-w-full object-contain transition-transform duration-200"
              style={{
                transform: `scale(${zoom}) rotate(${rotation}deg)`,
              }}
              draggable={false}
            />
          ) : (
            <div 
              className="flex aspect-video w-full max-w-3xl items-center justify-center rounded-lg bg-muted transition-transform duration-200"
              style={{
                transform: `scale(${zoom}) rotate(${rotation}deg)`,
              }}
            >
              <div className="flex flex-col items-center gap-3 text-muted-foreground">
                <ImageIcon className="h-16 w-16" />
                <p className="text-sm">图片占位</p>
                {currentImage?.resolution && (
                  <p className="text-xs">{currentImage.resolution}</p>
                )}
              </div>
            </div>
          )}
        </div>

        {/* 右箭头 */}
        <Button
          variant="ghost"
          size="icon"
          onClick={goNext}
          className="absolute right-4 z-10 h-12 w-12 rounded-full bg-black/40 text-white hover:bg-black/60"
        >
          <ChevronRight className="h-6 w-6" />
        </Button>

        {/* 图片信息面板 */}
        {showInfo && (
          <div className="absolute bottom-4 right-4 w-64 rounded-lg bg-black/80 p-4 backdrop-blur-sm">
            <h3 className="mb-3 text-sm font-medium text-white">图片信息</h3>
            <dl className="space-y-2 text-xs">
              <div className="flex justify-between">
                <dt className="text-white/60">类型</dt>
                <dd className="text-white">{getTypeLabel(currentImage?.type)}</dd>
              </div>
              {currentImage?.resolution && (
                <div className="flex justify-between">
                  <dt className="text-white/60">分辨率</dt>
                  <dd className="text-white">{currentImage.resolution}</dd>
                </div>
              )}
              {currentImage?.title && (
                <div className="flex justify-between">
                  <dt className="text-white/60">标题</dt>
                  <dd className="truncate text-white">{currentImage.title}</dd>
                </div>
              )}
            </dl>
          </div>
        )}
      </div>

      {/* 底部缩略图 */}
      {showThumbnails && images.length > 1 && (
        <div className="bg-black/60 px-4 py-3 backdrop-blur-sm">
          <div className="flex justify-center gap-2 overflow-x-auto scrollbar-none">
            {images.map((image, index) => (
              <button
                key={image.id}
                onClick={() => {
                  setCurrentIndex(index)
                  setZoom(1)
                  setRotation(0)
                }}
                className={cn(
                  "relative h-16 w-24 flex-shrink-0 overflow-hidden rounded border-2 transition-all",
                  index === currentIndex
                    ? "border-primary"
                    : "border-transparent opacity-60 hover:opacity-100"
                )}
              >
                {image.url ? (
                  <img
                    src={image.url}
                    alt={image.title || `缩略图 ${index + 1}`}
                    className="h-full w-full object-cover"
                  />
                ) : (
                  <div className="flex h-full w-full items-center justify-center bg-muted">
                    <ImageIcon className="h-6 w-6 text-muted-foreground" />
                  </div>
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* 键盘快捷键提示 */}
      <div className="hidden bg-black/40 px-4 py-2 text-center text-xs text-white/40 lg:block">
        左右箭头 导航 · +/- 缩放 · R 旋转 · 0 重置 · G 缩略图 · I 详情 · ESC 关闭
      </div>
    </div>
  )
}

// 图片网格组件 - 用于媒体详情页
interface ImageGalleryProps {
  images: {
    id: string
    url: string | null
    title?: string
    type?: "poster" | "backdrop" | "still" | "logo"
    resolution?: string
  }[]
  onViewImage?: (index: number) => void
}

export function ImageGallery({ images, onViewImage }: ImageGalleryProps) {
  const [viewerOpen, setViewerOpen] = useState(false)
  const [viewerIndex, setViewerIndex] = useState(0)

  const handleViewImage = (index: number) => {
    if (onViewImage) {
      onViewImage(index)
    } else {
      setViewerIndex(index)
      setViewerOpen(true)
    }
  }

  const groupedImages = {
    poster: images.filter((img) => img.type === "poster"),
    backdrop: images.filter((img) => img.type === "backdrop"),
    still: images.filter((img) => img.type === "still"),
    other: images.filter((img) => !img.type || !["poster", "backdrop", "still", "logo"].includes(img.type)),
  }

  return (
    <>
      <div className="space-y-6">
        {/* 背景图 */}
        {groupedImages.backdrop.length > 0 && (
          <div>
            <h3 className="mb-3 text-sm font-medium text-muted-foreground">背景图 ({groupedImages.backdrop.length})</h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
              {groupedImages.backdrop.map((image, idx) => (
                <button
                  key={image.id}
                  onClick={() => handleViewImage(images.indexOf(image))}
                  className="group relative aspect-video overflow-hidden rounded-lg bg-muted transition-all hover:ring-2 hover:ring-primary"
                >
                  {image.url ? (
                    <img
                      src={image.url}
                      alt={image.title || `背景图 ${idx + 1}`}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center">
                      <ImageIcon className="h-8 w-8 text-muted-foreground" />
                    </div>
                  )}
                  <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                    <ZoomIn className="h-6 w-6 text-white" />
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* 海报 */}
        {groupedImages.poster.length > 0 && (
          <div>
            <h3 className="mb-3 text-sm font-medium text-muted-foreground">海报 ({groupedImages.poster.length})</h3>
            <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
              {groupedImages.poster.map((image, idx) => (
                <button
                  key={image.id}
                  onClick={() => handleViewImage(images.indexOf(image))}
                  className="group relative aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-all hover:ring-2 hover:ring-primary"
                >
                  {image.url ? (
                    <img
                      src={image.url}
                      alt={image.title || `海报 ${idx + 1}`}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center">
                      <ImageIcon className="h-8 w-8 text-muted-foreground" />
                    </div>
                  )}
                  <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                    <ZoomIn className="h-6 w-6 text-white" />
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* 剧照 */}
        {groupedImages.still.length > 0 && (
          <div>
            <h3 className="mb-3 text-sm font-medium text-muted-foreground">剧照 ({groupedImages.still.length})</h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
              {groupedImages.still.map((image, idx) => (
                <button
                  key={image.id}
                  onClick={() => handleViewImage(images.indexOf(image))}
                  className="group relative aspect-video overflow-hidden rounded-lg bg-muted transition-all hover:ring-2 hover:ring-primary"
                >
                  {image.url ? (
                    <img
                      src={image.url}
                      alt={image.title || `剧照 ${idx + 1}`}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center">
                      <ImageIcon className="h-8 w-8 text-muted-foreground" />
                    </div>
                  )}
                  <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                    <ZoomIn className="h-6 w-6 text-white" />
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* 其他图片 */}
        {groupedImages.other.length > 0 && (
          <div>
            <h3 className="mb-3 text-sm font-medium text-muted-foreground">其他 ({groupedImages.other.length})</h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
              {groupedImages.other.map((image, idx) => (
                <button
                  key={image.id}
                  onClick={() => handleViewImage(images.indexOf(image))}
                  className="group relative aspect-video overflow-hidden rounded-lg bg-muted transition-all hover:ring-2 hover:ring-primary"
                >
                  {image.url ? (
                    <img
                      src={image.url}
                      alt={image.title || `图片 ${idx + 1}`}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center">
                      <ImageIcon className="h-8 w-8 text-muted-foreground" />
                    </div>
                  )}
                  <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                    <ZoomIn className="h-6 w-6 text-white" />
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* 图片查看器 */}
      {viewerOpen && (
        <ImageViewer
          images={images}
          initialIndex={viewerIndex}
          onClose={() => setViewerOpen(false)}
        />
      )}
    </>
  )
}
