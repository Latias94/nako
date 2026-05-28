"use client"

import { useState } from "react"
import { 
  ArrowLeft, Download, Pause, Play, Trash2, MoreVertical, 
  CheckCircle2, XCircle, Clock, HardDrive, Wifi, Film, Tv,
  FolderOpen, Settings, Search, Filter, ChevronDown
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Card, CardContent } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"

interface DownloadManagerProps {
  onBack: () => void
}

type DownloadStatus = "downloading" | "paused" | "completed" | "failed" | "queued"

interface DownloadItem {
  id: string
  title: string
  originalTitle?: string
  type: "movie" | "series"
  poster: string
  season?: number
  episode?: number
  quality: string
  size: string
  downloaded: string
  progress: number
  speed?: string
  eta?: string
  status: DownloadStatus
  addedAt: string
}

const downloads: DownloadItem[] = [
  {
    id: "1",
    title: "Dune: Part Two",
    originalTitle: "Dune: Part Two",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg",
    quality: "4K HDR",
    size: "45.2 GB",
    downloaded: "32.1 GB",
    progress: 71,
    speed: "15.2 MB/s",
    eta: "14 min",
    status: "downloading",
    addedAt: "2024-01-15 14:30",
  },
  {
    id: "2",
    title: "True Detective",
    type: "series",
    poster: "https://image.tmdb.org/t/p/w200/aowr4xpLP5sRCL50TkuADomJ98T.jpg",
    season: 1,
    episode: 5,
    quality: "1080p",
    size: "2.8 GB",
    downloaded: "1.4 GB",
    progress: 50,
    speed: "8.5 MB/s",
    eta: "3 min",
    status: "downloading",
    addedAt: "2024-01-15 14:25",
  },
  {
    id: "3",
    title: "Oppenheimer",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg",
    quality: "4K Dolby Vision",
    size: "52.8 GB",
    downloaded: "12.3 GB",
    progress: 23,
    status: "paused",
    addedAt: "2024-01-15 12:00",
  },
  {
    id: "4",
    title: "Breaking Bad",
    type: "series",
    poster: "https://image.tmdb.org/t/p/w200/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
    season: 5,
    episode: 16,
    quality: "1080p",
    size: "3.2 GB",
    downloaded: "0 GB",
    progress: 0,
    status: "queued",
    addedAt: "2024-01-15 10:30",
  },
  {
    id: "5",
    title: "Interstellar",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
    quality: "4K HDR",
    size: "38.5 GB",
    downloaded: "38.5 GB",
    progress: 100,
    status: "completed",
    addedAt: "2024-01-14 20:00",
  },
  {
    id: "6",
    title: "The Prestige",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/tRNlZbgNCNOpLpbPEz5L8G8A0JN.jpg",
    quality: "1080p",
    size: "8.2 GB",
    downloaded: "8.2 GB",
    progress: 100,
    status: "completed",
    addedAt: "2024-01-14 18:30",
  },
  {
    id: "7",
    title: "Succession",
    type: "series",
    poster: "https://image.tmdb.org/t/p/w200/7HW47XbkNQ5fiwQFYGWdw9gs144.jpg",
    season: 4,
    episode: 10,
    quality: "1080p",
    size: "2.5 GB",
    downloaded: "0.8 GB",
    progress: 32,
    status: "failed",
    addedAt: "2024-01-14 16:00",
  },
]

const storageInfo = {
  total: "2 TB",
  used: "1.2 TB",
  available: "800 GB",
  percentage: 60,
}

export function DownloadManager({ onBack }: DownloadManagerProps) {
  const [activeTab, setActiveTab] = useState("all")
  const [searchQuery, setSearchQuery] = useState("")

  const filteredDownloads = downloads.filter((item) => {
    const matchesSearch = item.title.toLowerCase().includes(searchQuery.toLowerCase())
    const matchesTab = 
      activeTab === "all" ||
      (activeTab === "active" && (item.status === "downloading" || item.status === "paused")) ||
      (activeTab === "completed" && item.status === "completed") ||
      (activeTab === "queued" && item.status === "queued")
    return matchesSearch && matchesTab
  })

  const activeDownloads = downloads.filter((d) => d.status === "downloading")
  const totalSpeed = activeDownloads.reduce((acc, d) => {
    const speed = parseFloat(d.speed || "0")
    return acc + speed
  }, 0)

  const getStatusIcon = (status: DownloadStatus) => {
    switch (status) {
      case "downloading":
        return <Download className="h-4 w-4 text-primary animate-pulse" />
      case "paused":
        return <Pause className="h-4 w-4 text-yellow-500" />
      case "completed":
        return <CheckCircle2 className="h-4 w-4 text-green-500" />
      case "failed":
        return <XCircle className="h-4 w-4 text-destructive" />
      case "queued":
        return <Clock className="h-4 w-4 text-muted-foreground" />
    }
  }

  const getStatusBadge = (status: DownloadStatus) => {
    const variants: Record<DownloadStatus, { variant: "default" | "secondary" | "destructive" | "outline"; label: string }> = {
      downloading: { variant: "default", label: "Downloading" },
      paused: { variant: "secondary", label: "Paused" },
      completed: { variant: "outline", label: "Completed" },
      failed: { variant: "destructive", label: "Failed" },
      queued: { variant: "secondary", label: "Queued" },
    }
    const { variant, label } = variants[status]
    return <Badge variant={variant}>{label}</Badge>
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center justify-between px-4 lg:px-6">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="icon" onClick={onBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <h1 className="text-lg font-semibold">Download Manager</h1>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm">
              <Settings className="mr-2 h-4 w-4" />
              Settings
            </Button>
          </div>
        </div>
      </header>

      <div className="mx-auto max-w-6xl px-4 py-6 lg:px-6">
        {/* Stats Cards */}
        <div className="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <Card className="border-border/50 bg-card/50">
            <CardContent className="flex items-center gap-4 p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                <Download className="h-6 w-6 text-primary" />
              </div>
              <div>
                <p className="text-2xl font-bold">{activeDownloads.length}</p>
                <p className="text-sm text-muted-foreground">Active Downloads</p>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border/50 bg-card/50">
            <CardContent className="flex items-center gap-4 p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-green-500/10">
                <Wifi className="h-6 w-6 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{totalSpeed.toFixed(1)} MB/s</p>
                <p className="text-sm text-muted-foreground">Total Speed</p>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border/50 bg-card/50">
            <CardContent className="flex items-center gap-4 p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-accent/10">
                <CheckCircle2 className="h-6 w-6 text-accent" />
              </div>
              <div>
                <p className="text-2xl font-bold">{downloads.filter((d) => d.status === "completed").length}</p>
                <p className="text-sm text-muted-foreground">Completed</p>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border/50 bg-card/50">
            <CardContent className="p-4">
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <HardDrive className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm text-muted-foreground">Storage</span>
                </div>
                <span className="text-sm font-medium">{storageInfo.used} / {storageInfo.total}</span>
              </div>
              <Progress value={storageInfo.percentage} className="h-2" />
              <p className="mt-1 text-xs text-muted-foreground">{storageInfo.available} available</p>
            </CardContent>
          </Card>
        </div>

        {/* Search and Filter */}
        <div className="mb-4 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="relative max-w-sm flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search downloads..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm">
              <FolderOpen className="mr-2 h-4 w-4" />
              Open Folder
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm">
                  <Filter className="mr-2 h-4 w-4" />
                  Actions
                  <ChevronDown className="ml-2 h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem>
                  <Play className="mr-2 h-4 w-4" />
                  Resume All
                </DropdownMenuItem>
                <DropdownMenuItem>
                  <Pause className="mr-2 h-4 w-4" />
                  Pause All
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem className="text-destructive">
                  <Trash2 className="mr-2 h-4 w-4" />
                  Clear Completed
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>

        {/* Tabs */}
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="mb-4">
            <TabsTrigger value="all">
              All ({downloads.length})
            </TabsTrigger>
            <TabsTrigger value="active">
              Active ({downloads.filter((d) => d.status === "downloading" || d.status === "paused").length})
            </TabsTrigger>
            <TabsTrigger value="queued">
              Queued ({downloads.filter((d) => d.status === "queued").length})
            </TabsTrigger>
            <TabsTrigger value="completed">
              Completed ({downloads.filter((d) => d.status === "completed").length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value={activeTab} className="space-y-3">
            {filteredDownloads.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <Download className="mb-4 h-12 w-12 text-muted-foreground/50" />
                <h3 className="text-lg font-medium">No downloads</h3>
                <p className="text-sm text-muted-foreground">
                  {activeTab === "all" ? "Your download queue is empty" : `No ${activeTab} downloads`}
                </p>
              </div>
            ) : (
              filteredDownloads.map((item) => (
                <DownloadCard key={item.id} item={item} />
              ))
            )}
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}

function DownloadCard({ item }: { item: DownloadItem }) {
  const isActive = item.status === "downloading" || item.status === "paused"
  
  return (
    <Card className={cn(
      "overflow-hidden border-border/50 transition-colors",
      item.status === "failed" && "border-destructive/30"
    )}>
      <CardContent className="p-0">
        <div className="flex">
          {/* Poster */}
          <div className="relative h-28 w-20 flex-shrink-0 overflow-hidden bg-muted sm:h-32 sm:w-24">
            <img
              src={item.poster}
              alt={item.title}
              className="h-full w-full object-cover"
            />
            <div className="absolute bottom-1 left-1">
              {item.type === "movie" ? (
                <Film className="h-4 w-4 text-white drop-shadow-md" />
              ) : (
                <Tv className="h-4 w-4 text-white drop-shadow-md" />
              )}
            </div>
          </div>

          {/* Content */}
          <div className="flex flex-1 flex-col justify-between p-3 sm:p-4">
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <h3 className="truncate font-medium">{item.title}</h3>
                  {item.season && item.episode && (
                    <Badge variant="outline" className="text-xs">
                      S{item.season.toString().padStart(2, "0")}E{item.episode.toString().padStart(2, "0")}
                    </Badge>
                  )}
                </div>
                <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <Badge variant="secondary" className="text-xs">{item.quality}</Badge>
                  <span>{item.downloaded} / {item.size}</span>
                  {item.speed && item.status === "downloading" && (
                    <>
                      <span>·</span>
                      <span className="text-primary">{item.speed}</span>
                    </>
                  )}
                  {item.eta && item.status === "downloading" && (
                    <>
                      <span>·</span>
                      <span>{item.eta} left</span>
                    </>
                  )}
                </div>
              </div>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8">
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {item.status === "downloading" && (
                    <DropdownMenuItem>
                      <Pause className="mr-2 h-4 w-4" />
                      Pause
                    </DropdownMenuItem>
                  )}
                  {item.status === "paused" && (
                    <DropdownMenuItem>
                      <Play className="mr-2 h-4 w-4" />
                      Resume
                    </DropdownMenuItem>
                  )}
                  {item.status === "failed" && (
                    <DropdownMenuItem>
                      <Play className="mr-2 h-4 w-4" />
                      Retry
                    </DropdownMenuItem>
                  )}
                  {item.status === "completed" && (
                    <DropdownMenuItem>
                      <FolderOpen className="mr-2 h-4 w-4" />
                      Open Folder
                    </DropdownMenuItem>
                  )}
                  <DropdownMenuSeparator />
                  <DropdownMenuItem className="text-destructive">
                    <Trash2 className="mr-2 h-4 w-4" />
                    Remove
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>

            {/* Progress Bar */}
            {isActive && (
              <div className="mt-3">
                <div className="mb-1 flex items-center justify-between text-xs">
                  <span className="text-muted-foreground">{item.progress}%</span>
                </div>
                <Progress 
                  value={item.progress} 
                  className={cn(
                    "h-1.5",
                    item.status === "paused" && "[&>div]:bg-yellow-500"
                  )}
                />
              </div>
            )}

            {/* Status Badge for non-active */}
            {!isActive && (
              <div className="mt-2 flex items-center gap-2">
                {item.status === "completed" && (
                  <Badge variant="outline" className="gap-1 border-green-500/30 text-green-500">
                    <CheckCircle2 className="h-3 w-3" />
                    Completed
                  </Badge>
                )}
                {item.status === "failed" && (
                  <Badge variant="destructive" className="gap-1">
                    <XCircle className="h-3 w-3" />
                    Failed
                  </Badge>
                )}
                {item.status === "queued" && (
                  <Badge variant="secondary" className="gap-1">
                    <Clock className="h-3 w-3" />
                    Queued
                  </Badge>
                )}
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
