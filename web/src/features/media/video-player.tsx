"use client"

import { useState, useRef, useEffect, useCallback } from "react"
import { 
  Play, 
  Pause, 
  Volume2, 
  VolumeX,
  Maximize,
  Minimize,
  Settings,
  SkipBack,
  SkipForward,
  ChevronLeft,
  Subtitles,
  Monitor,
  Rewind,
  FastForward,
  PictureInPicture,
  Cast,
  Loader2,
  MessageSquare
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Slider } from "@/components/ui/slider"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"

interface VideoPlayerProps {
  onBack: () => void
  onNext?: () => void
  onPrevious?: () => void
  mediaTitle: string
  episodeTitle?: string
  episodeInfo?: string // 如 "S01E04"
  sources?: {
    quality: string
    url: string
    contentType?: string
  }[]
  audioTracks?: {
    id: string
    language: string
    codec?: string
  }[]
  subtitles?: {
    id: string
    language: string
    url?: string
    srcLang?: string
    default?: boolean
    forced?: boolean
    contentType?: string
  }[]
  startTime?: number // 续播位置（秒）
  hasPrevious?: boolean
  hasNext?: boolean
}

export function VideoPlayer({
  onBack,
  onNext,
  onPrevious,
  mediaTitle,
  episodeTitle,
  episodeInfo,
  sources = [
    { quality: "4K", url: "" },
    { quality: "1080p", url: "" },
    { quality: "720p", url: "" },
  ],
  audioTracks = [
    { id: "en", language: "英语", codec: "TrueHD Atmos" },
    { id: "zh", language: "普通话", codec: "AAC 5.1" },
  ],
  subtitles = [
    { id: "none", language: "关闭" },
    { id: "chs", language: "简体中文" },
    { id: "cht", language: "繁體中文" },
    { id: "en", language: "English" },
  ],
  startTime = 0,
  hasPrevious = true,
  hasNext = true,
}: VideoPlayerProps) {
  // 播放状态
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(startTime)
  const [duration, setDuration] = useState(166 * 60) // 模拟 166 分钟
  const [volume, setVolume] = useState(80)
  const [isMuted, setIsMuted] = useState(false)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [isBuffering, setIsBuffering] = useState(false)
  
  // UI 状态
  const [showControls, setShowControls] = useState(true)
  const [showSettings, setShowSettings] = useState(false)
  
  // 设置
  const [selectedQuality, setSelectedQuality] = useState(sources[0]?.quality || "4K")
  const [selectedAudio, setSelectedAudio] = useState(audioTracks[0]?.id || "en")
  const [selectedSubtitle, setSelectedSubtitle] = useState(
    subtitles.find((sub) => sub.default)?.id ?? "none",
  )
  const [playbackSpeed, setPlaybackSpeed] = useState("1")
  
  const containerRef = useRef<HTMLDivElement>(null)
  const videoRef = useRef<HTMLVideoElement>(null)
  const hideControlsTimeout = useRef<NodeJS.Timeout | null>(null)
  const progressInterval = useRef<NodeJS.Timeout | null>(null)
  const selectedSource = sources.find((source) => source.quality === selectedQuality) ?? sources[0]
  const selectedSourceUrl = selectedSource?.url?.trim() ?? ""
  const hasPlayableSource = selectedSourceUrl.length > 0
  const subtitleOptions = [{ id: "none", language: "关闭" }, ...subtitles]

  useEffect(() => {
    if (!sources.some((source) => source.quality === selectedQuality)) {
      setSelectedQuality(sources[0]?.quality || "Auto")
    }
  }, [selectedQuality, sources])

  useEffect(() => {
    const defaultSubtitleId = subtitles.find((sub) => sub.default)?.id ?? "none"
    setSelectedSubtitle((current) =>
      current === "none" || subtitles.some((sub) => sub.id === current)
        ? current
        : defaultSubtitleId,
    )
  }, [subtitles])

  // 模拟播放进度
  useEffect(() => {
    if (hasPlayableSource) {
      return
    }

    if (isPlaying && !isBuffering) {
      progressInterval.current = setInterval(() => {
        setCurrentTime(prev => {
          if (prev >= duration) {
            setIsPlaying(false)
            return duration
          }
          return prev + 1
        })
      }, 1000 / parseFloat(playbackSpeed))
    }

    return () => {
      if (progressInterval.current) {
        clearInterval(progressInterval.current)
      }
    }
  }, [hasPlayableSource, isPlaying, isBuffering, duration, playbackSpeed])

  useEffect(() => {
    const video = videoRef.current
    if (!video || !hasPlayableSource) {
      return
    }

    if (isPlaying) {
      video.play().catch(() => setIsPlaying(false))
    } else {
      video.pause()
    }
  }, [hasPlayableSource, isPlaying, selectedSourceUrl])

  useEffect(() => {
    if (videoRef.current) {
      videoRef.current.playbackRate = Number.parseFloat(playbackSpeed)
    }
  }, [playbackSpeed])

  useEffect(() => {
    const video = videoRef.current
    if (!video || !hasPlayableSource) {
      return
    }

    const textTracks = Array.from(video.textTracks)
    subtitles.forEach((sub, index) => {
      const track = textTracks[index]
      if (track) {
        track.mode = sub.id === selectedSubtitle ? "showing" : "disabled"
      }
    })
  }, [hasPlayableSource, selectedSubtitle, subtitles])

  // 自动隐藏控制栏
  const resetHideTimeout = useCallback(() => {
    if (hideControlsTimeout.current) {
      clearTimeout(hideControlsTimeout.current)
    }
    setShowControls(true)
    
    if (isPlaying) {
      hideControlsTimeout.current = setTimeout(() => {
        setShowControls(false)
      }, 3000)
    }
  }, [isPlaying])

  useEffect(() => {
    resetHideTimeout()
    return () => {
      if (hideControlsTimeout.current) {
        clearTimeout(hideControlsTimeout.current)
      }
    }
  }, [isPlaying, resetHideTimeout])

  // 键盘控制
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case " ":
        case "k":
          e.preventDefault()
          setIsPlaying(prev => !prev)
          resetHideTimeout()
          break
        case "ArrowLeft":
          e.preventDefault()
          setCurrentTime(prev => Math.max(0, prev - 10))
          resetHideTimeout()
          break
        case "ArrowRight":
          e.preventDefault()
          setCurrentTime(prev => Math.min(duration, prev + 10))
          resetHideTimeout()
          break
        case "ArrowUp":
          e.preventDefault()
          setVolume(prev => Math.min(100, prev + 5))
          setIsMuted(false)
          resetHideTimeout()
          break
        case "ArrowDown":
          e.preventDefault()
          setVolume(prev => Math.max(0, prev - 5))
          resetHideTimeout()
          break
        case "m":
          e.preventDefault()
          setIsMuted(prev => !prev)
          resetHideTimeout()
          break
        case "f":
          e.preventDefault()
          toggleFullscreen()
          break
        case "Escape":
          if (isFullscreen) {
            toggleFullscreen()
          }
          break
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [duration, isFullscreen, resetHideTimeout])

  // 全屏切换
  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      containerRef.current?.requestFullscreen()
      setIsFullscreen(true)
    } else {
      document.exitFullscreen()
      setIsFullscreen(false)
    }
  }

  // 格式化时间
  const formatTime = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = Math.floor(seconds % 60)
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`
    }
    return `${m}:${s.toString().padStart(2, "0")}`
  }

  // 快进/快退
  const skip = (seconds: number) => {
    setCurrentTime(prev => Math.max(0, Math.min(duration, prev + seconds)))
    resetHideTimeout()
  }

  return (
    <div 
      ref={containerRef}
      className={cn(
        "relative flex h-screen w-full items-center justify-center overflow-hidden bg-black",
        !showControls && "cursor-none"
      )}
      onMouseMove={resetHideTimeout}
      onClick={() => {
        if (!showSettings) {
          setIsPlaying(prev => !prev)
          resetHideTimeout()
        }
      }}
    >
      {/* 视频区域 */}
      <div className="absolute inset-0 flex items-center justify-center bg-black">
        {hasPlayableSource ? (
          <video
            ref={videoRef}
            className="h-full w-full bg-black object-contain"
            playsInline
            preload="metadata"
            onLoadedMetadata={(event) => {
              const nextDuration = event.currentTarget.duration
              if (Number.isFinite(nextDuration) && nextDuration > 0) {
                setDuration(nextDuration)
              }
            }}
            onTimeUpdate={(event) => setCurrentTime(event.currentTarget.currentTime)}
            onWaiting={() => setIsBuffering(true)}
            onCanPlay={() => setIsBuffering(false)}
            onPlaying={() => {
              setIsBuffering(false)
              setIsPlaying(true)
            }}
            onPause={() => setIsPlaying(false)}
            data-testid="nako-video-player"
          >
            <source
              src={selectedSourceUrl}
              type={selectedSource?.contentType}
              data-testid="nako-video-source"
            />
            {subtitles
              .filter((sub) => sub.url?.trim())
              .map((sub) => (
                <track
                  key={sub.id}
                  kind="subtitles"
                  src={sub.url}
                  srcLang={sub.srcLang ?? "und"}
                  label={sub.language}
                  default={sub.default}
                  data-subtitle-id={sub.id}
                  data-testid="nako-video-subtitle-track"
                />
              ))}
          </video>
        ) : (
          <div className="text-muted-foreground">
          {isBuffering ? (
            <Loader2 className="h-16 w-16 animate-spin" />
          ) : (
            <div className="flex flex-col items-center gap-4">
              <div className="flex h-24 w-24 items-center justify-center rounded-full bg-muted/20">
                {isPlaying ? (
                  <Pause className="h-12 w-12" />
                ) : (
                  <Play className="h-12 w-12" />
                )}
              </div>
              <p className="text-sm">视频播放区域</p>
            </div>
          )}
          </div>
        )}
      </div>

      {/* 顶部渐变 & 标题栏 */}
      <div 
        className={cn(
          "absolute inset-x-0 top-0 z-10 bg-gradient-to-b from-black/80 via-black/40 to-transparent p-4 transition-opacity duration-300 lg:p-6",
          showControls ? "opacity-100" : "opacity-0"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            <Button 
              variant="ghost" 
              size="icon" 
              onClick={onBack}
              className="h-10 w-10 text-white hover:bg-white/10"
            >
              <ChevronLeft className="h-6 w-6" />
            </Button>
            <div>
              <h1 className="text-lg font-medium text-white lg:text-xl">
                {mediaTitle}
              </h1>
              {(episodeTitle || episodeInfo) && (
                <p className="text-sm text-white/70">
                  {episodeInfo && <span className="mr-2">{episodeInfo}</span>}
                  {episodeTitle}
                </p>
              )}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button 
              variant="ghost" 
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10"
            >
              <Cast className="h-5 w-5" />
            </Button>
            <Button 
              variant="ghost" 
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10"
            >
              <PictureInPicture className="h-5 w-5" />
            </Button>
          </div>
        </div>
      </div>

      {/* 中间快进/快退指示器 */}
      <div className="pointer-events-none absolute left-4 top-1/2 -translate-y-1/2 lg:left-8">
        <Button
          variant="ghost"
          size="icon"
          className="pointer-events-auto h-16 w-16 rounded-full text-white opacity-0 transition-opacity hover:bg-white/10 hover:opacity-100"
          onClick={(e) => {
            e.stopPropagation()
            skip(-10)
          }}
        >
          <Rewind className="h-8 w-8" />
        </Button>
      </div>
      <div className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 lg:right-8">
        <Button
          variant="ghost"
          size="icon"
          className="pointer-events-auto h-16 w-16 rounded-full text-white opacity-0 transition-opacity hover:bg-white/10 hover:opacity-100"
          onClick={(e) => {
            e.stopPropagation()
            skip(10)
          }}
        >
          <FastForward className="h-8 w-8" />
        </Button>
      </div>

      {/* 底部控制栏 */}
      <div 
        className={cn(
          "absolute inset-x-0 bottom-0 z-10 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-4 transition-opacity duration-300 lg:p-6",
          showControls ? "opacity-100" : "opacity-0"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {/* 进度条 */}
        <div className="group mb-4">
          <div className="flex items-center gap-2">
            <span className="w-14 text-right text-xs text-white/80 lg:w-16 lg:text-sm">
              {formatTime(currentTime)}
            </span>
            <div className="relative flex-1">
              <Slider
                value={[currentTime]}
                max={duration}
                step={1}
                onValueChange={([value]) => setCurrentTime(value)}
                className="[&_[role=slider]]:h-4 [&_[role=slider]]:w-4 [&_[role=slider]]:border-2 [&_[role=slider]]:border-white [&_[role=slider]]:bg-primary [&_[role=slider]]:opacity-0 [&_[role=slider]]:transition-opacity group-hover:[&_[role=slider]]:opacity-100"
              />
              {/* 缓冲进度模拟 */}
              <div 
                className="pointer-events-none absolute left-0 top-1/2 h-1 -translate-y-1/2 rounded-full bg-white/30"
                style={{ width: `${Math.min(100, (currentTime / duration) * 100 + 10)}%` }}
              />
            </div>
            <span className="w-14 text-xs text-white/80 lg:w-16 lg:text-sm">
              {formatTime(duration)}
            </span>
          </div>
        </div>

        {/* 控制按钮 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1 lg:gap-2">
            {/* 播放/暂停 */}
            <Button
              variant="ghost"
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10 lg:h-12 lg:w-12"
              onClick={() => setIsPlaying(prev => !prev)}
            >
              {isPlaying ? (
                <Pause className="h-6 w-6 lg:h-7 lg:w-7" />
              ) : (
                <Play className="h-6 w-6 lg:h-7 lg:w-7" />
              )}
            </Button>

            {/* 上一集/下一集 */}
            <Button
              variant="ghost"
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10 disabled:opacity-30"
              disabled={!hasPrevious}
              onClick={onPrevious}
            >
              <SkipBack className="h-5 w-5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10 disabled:opacity-30"
              disabled={!hasNext}
              onClick={onNext}
            >
              <SkipForward className="h-5 w-5" />
            </Button>

            {/* 音量 */}
            <div className="group/volume flex items-center">
              <Button
                variant="ghost"
                size="icon"
                className="h-10 w-10 text-white hover:bg-white/10"
                onClick={() => setIsMuted(prev => !prev)}
              >
                {isMuted || volume === 0 ? (
                  <VolumeX className="h-5 w-5" />
                ) : (
                  <Volume2 className="h-5 w-5" />
                )}
              </Button>
              <div className="hidden w-0 overflow-hidden transition-all group-hover/volume:w-24 lg:block">
                <Slider
                  value={[isMuted ? 0 : volume]}
                  max={100}
                  step={1}
                  onValueChange={([value]) => {
                    setVolume(value)
                    if (value > 0) setIsMuted(false)
                  }}
                  className="mx-2"
                />
              </div>
            </div>

            {/* 时间显示 (移动端) */}
            <span className="ml-2 hidden text-sm text-white/80 sm:block">
              {formatTime(duration - currentTime)} 剩余
            </span>
          </div>

          <div className="flex items-center gap-1 lg:gap-2">
            {/* 字幕 */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-10 w-10 text-white hover:bg-white/10"
                >
                  <Subtitles className="h-5 w-5" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48">
                <DropdownMenuLabel>字幕</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuRadioGroup value={selectedSubtitle} onValueChange={setSelectedSubtitle}>
                  {subtitleOptions.map((sub) => (
                    <DropdownMenuRadioItem key={sub.id} value={sub.id}>
                      {sub.language}
                    </DropdownMenuRadioItem>
                  ))}
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* 设置 */}
            <DropdownMenu open={showSettings} onOpenChange={setShowSettings}>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-10 w-10 text-white hover:bg-white/10"
                >
                  <Settings className="h-5 w-5" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-56">
                <DropdownMenuLabel>播放设置</DropdownMenuLabel>
                <DropdownMenuSeparator />
                
                {/* 画质 */}
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger>
                    <Monitor className="mr-2 h-4 w-4" />
                    画质
                    <span className="ml-auto text-xs text-muted-foreground">{selectedQuality}</span>
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent>
                    <DropdownMenuRadioGroup value={selectedQuality} onValueChange={setSelectedQuality}>
                      {sources.map((source) => (
                        <DropdownMenuRadioItem key={source.quality} value={source.quality}>
                          {source.quality}
                        </DropdownMenuRadioItem>
                      ))}
                    </DropdownMenuRadioGroup>
                  </DropdownMenuSubContent>
                </DropdownMenuSub>

                {/* 音轨 */}
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger>
                    <Volume2 className="mr-2 h-4 w-4" />
                    音轨
                    <span className="ml-auto text-xs text-muted-foreground">
                      {audioTracks.find(a => a.id === selectedAudio)?.language}
                    </span>
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent>
                    <DropdownMenuRadioGroup value={selectedAudio} onValueChange={setSelectedAudio}>
                      {audioTracks.map((track) => (
                        <DropdownMenuRadioItem key={track.id} value={track.id}>
                          {track.language}
                          {track.codec && <span className="ml-2 text-xs text-muted-foreground">{track.codec}</span>}
                        </DropdownMenuRadioItem>
                      ))}
                    </DropdownMenuRadioGroup>
                  </DropdownMenuSubContent>
                </DropdownMenuSub>

                {/* 播放速度 */}
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger>
                    <Play className="mr-2 h-4 w-4" />
                    播放速度
                    <span className="ml-auto text-xs text-muted-foreground">{playbackSpeed}x</span>
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent>
                    <DropdownMenuRadioGroup value={playbackSpeed} onValueChange={setPlaybackSpeed}>
                      <DropdownMenuRadioItem value="0.5">0.5x</DropdownMenuRadioItem>
                      <DropdownMenuRadioItem value="0.75">0.75x</DropdownMenuRadioItem>
                      <DropdownMenuRadioItem value="1">1x (正常)</DropdownMenuRadioItem>
                      <DropdownMenuRadioItem value="1.25">1.25x</DropdownMenuRadioItem>
                      <DropdownMenuRadioItem value="1.5">1.5x</DropdownMenuRadioItem>
                      <DropdownMenuRadioItem value="2">2x</DropdownMenuRadioItem>
                    </DropdownMenuRadioGroup>
                  </DropdownMenuSubContent>
                </DropdownMenuSub>

                <DropdownMenuSeparator />
                
                <DropdownMenuItem>
                  <MessageSquare className="mr-2 h-4 w-4" />
                  弹幕设置
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* 全屏 */}
            <Button
              variant="ghost"
              size="icon"
              className="h-10 w-10 text-white hover:bg-white/10"
              onClick={toggleFullscreen}
            >
              {isFullscreen ? (
                <Minimize className="h-5 w-5" />
              ) : (
                <Maximize className="h-5 w-5" />
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* 快捷键提示 (仅桌面) */}
      {showControls && (
        <div className="absolute bottom-24 left-1/2 hidden -translate-x-1/2 text-xs text-white/50 lg:block">
          空格/K 播放/暂停 · 左右箭头 快进快退 · M 静音 · F 全屏
        </div>
      )}
    </div>
  )
}
