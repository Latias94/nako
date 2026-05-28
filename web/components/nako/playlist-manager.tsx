"use client"

import { useState } from "react"
import { 
  ArrowLeft, Play, Pause, SkipForward, SkipBack, Shuffle, Repeat, 
  Plus, MoreVertical, Trash2, GripVertical, Clock, Film, Tv,
  ListMusic, Edit2, Check, X, ChevronRight, Search
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
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
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

interface PlaylistManagerProps {
  onBack: () => void
  onPlayMedia: (id: string, type: "movie" | "series") => void
}

interface QueueItem {
  id: string
  title: string
  type: "movie" | "series"
  poster: string
  duration: string
  season?: number
  episode?: number
  episodeTitle?: string
  isPlaying?: boolean
}

interface Playlist {
  id: string
  name: string
  description?: string
  itemCount: number
  totalDuration: string
  thumbnail: string
  createdAt: string
  items: QueueItem[]
}

// 示例播放队列数据
const queueItems: QueueItem[] = [
  {
    id: "1",
    title: "Dune: Part Two",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg",
    duration: "2h 46m",
    isPlaying: true,
  },
  {
    id: "2",
    title: "True Detective",
    type: "series",
    poster: "https://image.tmdb.org/t/p/w200/aowr4xpLP5sRCL50TkuADomJ98T.jpg",
    duration: "58m",
    season: 1,
    episode: 5,
    episodeTitle: "The Secret Fate of All Life",
  },
  {
    id: "3",
    title: "Oppenheimer",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg",
    duration: "3h 00m",
  },
  {
    id: "4",
    title: "Breaking Bad",
    type: "series",
    poster: "https://image.tmdb.org/t/p/w200/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
    duration: "47m",
    season: 5,
    episode: 16,
    episodeTitle: "Felina",
  },
  {
    id: "5",
    title: "Interstellar",
    type: "movie",
    poster: "https://image.tmdb.org/t/p/w200/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
    duration: "2h 49m",
  },
]

// 示例播放列表数据
const playlists: Playlist[] = [
  {
    id: "1",
    name: "Sci-Fi Marathon",
    description: "The best science fiction movies",
    itemCount: 8,
    totalDuration: "18h 32m",
    thumbnail: "https://image.tmdb.org/t/p/w200/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
    createdAt: "2024-01-10",
    items: [
      { id: "1", title: "Interstellar", type: "movie", poster: "https://image.tmdb.org/t/p/w200/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg", duration: "2h 49m" },
      { id: "2", title: "Dune: Part Two", type: "movie", poster: "https://image.tmdb.org/t/p/w200/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg", duration: "2h 46m" },
      { id: "3", title: "Arrival", type: "movie", poster: "https://image.tmdb.org/t/p/w200/x2FJsf1ElAgr63Y3PNPtJrcmpoe.jpg", duration: "1h 56m" },
    ],
  },
  {
    id: "2",
    name: "Weekend Watch",
    description: "Quick watches for the weekend",
    itemCount: 5,
    totalDuration: "8h 15m",
    thumbnail: "https://image.tmdb.org/t/p/w200/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg",
    createdAt: "2024-01-12",
    items: [
      { id: "1", title: "Oppenheimer", type: "movie", poster: "https://image.tmdb.org/t/p/w200/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg", duration: "3h 00m" },
      { id: "2", title: "The Prestige", type: "movie", poster: "https://image.tmdb.org/t/p/w200/tRNlZbgNCNOpLpbPEz5L8G8A0JN.jpg", duration: "2h 10m" },
    ],
  },
  {
    id: "3",
    name: "TV Catch-up",
    description: "Episodes to catch up on",
    itemCount: 12,
    totalDuration: "10h 45m",
    thumbnail: "https://image.tmdb.org/t/p/w200/aowr4xpLP5sRCL50TkuADomJ98T.jpg",
    createdAt: "2024-01-14",
    items: [
      { id: "1", title: "True Detective", type: "series", poster: "https://image.tmdb.org/t/p/w200/aowr4xpLP5sRCL50TkuADomJ98T.jpg", duration: "58m", season: 1, episode: 5 },
      { id: "2", title: "Breaking Bad", type: "series", poster: "https://image.tmdb.org/t/p/w200/ggFHVNu6YYI5L9pCfOacjizRGt.jpg", duration: "47m", season: 5, episode: 16 },
    ],
  },
]

export function PlaylistManager({ onBack, onPlayMedia }: PlaylistManagerProps) {
  const [activeTab, setActiveTab] = useState("queue")
  const [queue, setQueue] = useState(queueItems)
  const [isShuffled, setIsShuffled] = useState(false)
  const [repeatMode, setRepeatMode] = useState<"off" | "all" | "one">("off")
  const [selectedPlaylist, setSelectedPlaylist] = useState<Playlist | null>(null)
  const [isCreatingPlaylist, setIsCreatingPlaylist] = useState(false)
  const [newPlaylistName, setNewPlaylistName] = useState("")
  const [searchQuery, setSearchQuery] = useState("")

  const currentItem = queue.find((item) => item.isPlaying)
  const currentIndex = queue.findIndex((item) => item.isPlaying)

  const handlePlayItem = (index: number) => {
    setQueue(queue.map((item, i) => ({ ...item, isPlaying: i === index })))
  }

  const handleRemoveFromQueue = (id: string) => {
    setQueue(queue.filter((item) => item.id !== id))
  }

  const handleClearQueue = () => {
    setQueue([])
  }

  const handleNext = () => {
    if (currentIndex < queue.length - 1) {
      handlePlayItem(currentIndex + 1)
    }
  }

  const handlePrevious = () => {
    if (currentIndex > 0) {
      handlePlayItem(currentIndex - 1)
    }
  }

  const handleCreatePlaylist = () => {
    if (newPlaylistName.trim()) {
      // In real app, would save to backend
      setNewPlaylistName("")
      setIsCreatingPlaylist(false)
    }
  }

  const filteredPlaylists = playlists.filter((p) =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center justify-between px-4 lg:px-6">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="icon" onClick={selectedPlaylist ? () => setSelectedPlaylist(null) : onBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <h1 className="text-lg font-semibold">
              {selectedPlaylist ? selectedPlaylist.name : "Playlists & Queue"}
            </h1>
          </div>
        </div>
      </header>

      {/* Playlist Detail View */}
      {selectedPlaylist ? (
        <div className="mx-auto max-w-4xl px-4 py-6 lg:px-6">
          {/* Playlist Header */}
          <div className="mb-6 flex flex-col gap-6 sm:flex-row">
            <div className="relative h-48 w-48 flex-shrink-0 overflow-hidden rounded-lg bg-muted shadow-lg">
              <img
                src={selectedPlaylist.thumbnail}
                alt={selectedPlaylist.name}
                className="h-full w-full object-cover"
              />
              <div className="absolute inset-0 flex items-center justify-center bg-black/40">
                <ListMusic className="h-12 w-12 text-white" />
              </div>
            </div>
            <div className="flex-1">
              <h2 className="text-2xl font-bold">{selectedPlaylist.name}</h2>
              {selectedPlaylist.description && (
                <p className="mt-1 text-muted-foreground">{selectedPlaylist.description}</p>
              )}
              <div className="mt-3 flex items-center gap-4 text-sm text-muted-foreground">
                <span>{selectedPlaylist.itemCount} items</span>
                <span>·</span>
                <span>{selectedPlaylist.totalDuration}</span>
              </div>
              <div className="mt-4 flex gap-2">
                <Button>
                  <Play className="mr-2 h-4 w-4" />
                  Play All
                </Button>
                <Button variant="outline">
                  <Shuffle className="mr-2 h-4 w-4" />
                  Shuffle
                </Button>
                <Button variant="outline" size="icon">
                  <Edit2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>

          {/* Playlist Items */}
          <div className="space-y-2">
            {selectedPlaylist.items.map((item, index) => (
              <PlaylistItem
                key={item.id}
                item={item}
                index={index}
                onPlay={() => onPlayMedia(item.id, item.type)}
                onRemove={() => {}}
              />
            ))}
          </div>
        </div>
      ) : (
        <div className="mx-auto max-w-4xl px-4 py-6 lg:px-6">
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="mb-6">
              <TabsTrigger value="queue">
                Play Queue ({queue.length})
              </TabsTrigger>
              <TabsTrigger value="playlists">
                My Playlists ({playlists.length})
              </TabsTrigger>
            </TabsList>

            {/* Queue Tab */}
            <TabsContent value="queue">
              {queue.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                  <ListMusic className="mb-4 h-12 w-12 text-muted-foreground/50" />
                  <h3 className="text-lg font-medium">Your queue is empty</h3>
                  <p className="mt-1 text-sm text-muted-foreground">
                    Add some movies or shows to start watching
                  </p>
                </div>
              ) : (
                <>
                  {/* Now Playing */}
                  {currentItem && (
                    <Card className="mb-6 border-primary/30 bg-primary/5">
                      <CardHeader className="pb-3">
                        <CardTitle className="text-sm font-medium text-muted-foreground">
                          Now Playing
                        </CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="flex items-center gap-4">
                          <div className="relative h-20 w-14 overflow-hidden rounded bg-muted">
                            <img
                              src={currentItem.poster}
                              alt={currentItem.title}
                              className="h-full w-full object-cover"
                            />
                          </div>
                          <div className="flex-1 min-w-0">
                            <h3 className="font-semibold truncate">{currentItem.title}</h3>
                            {currentItem.season && currentItem.episode && (
                              <p className="text-sm text-muted-foreground">
                                S{currentItem.season.toString().padStart(2, "0")}E{currentItem.episode.toString().padStart(2, "0")}
                                {currentItem.episodeTitle && ` - ${currentItem.episodeTitle}`}
                              </p>
                            )}
                            <div className="mt-1 flex items-center gap-1 text-sm text-muted-foreground">
                              <Clock className="h-3.5 w-3.5" />
                              <span>{currentItem.duration}</span>
                            </div>
                          </div>
                          <div className="flex items-center gap-1">
                            <Button variant="ghost" size="icon" onClick={handlePrevious} disabled={currentIndex === 0}>
                              <SkipBack className="h-5 w-5" />
                            </Button>
                            <Button size="icon" className="h-12 w-12">
                              <Play className="h-6 w-6" />
                            </Button>
                            <Button variant="ghost" size="icon" onClick={handleNext} disabled={currentIndex === queue.length - 1}>
                              <SkipForward className="h-5 w-5" />
                            </Button>
                          </div>
                        </div>
                        
                        {/* Playback Controls */}
                        <div className="mt-4 flex items-center justify-between border-t border-border/50 pt-4">
                          <div className="flex items-center gap-2">
                            <Button
                              variant={isShuffled ? "secondary" : "ghost"}
                              size="sm"
                              onClick={() => setIsShuffled(!isShuffled)}
                            >
                              <Shuffle className="h-4 w-4" />
                            </Button>
                            <Button
                              variant={repeatMode !== "off" ? "secondary" : "ghost"}
                              size="sm"
                              onClick={() => {
                                const modes: ("off" | "all" | "one")[] = ["off", "all", "one"]
                                const nextIndex = (modes.indexOf(repeatMode) + 1) % modes.length
                                setRepeatMode(modes[nextIndex])
                              }}
                            >
                              <Repeat className="h-4 w-4" />
                              {repeatMode === "one" && <span className="ml-1 text-xs">1</span>}
                            </Button>
                          </div>
                          <Button variant="ghost" size="sm" onClick={handleClearQueue}>
                            <Trash2 className="mr-2 h-4 w-4" />
                            Clear Queue
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  )}

                  {/* Up Next */}
                  <div className="mb-4 flex items-center justify-between">
                    <h3 className="font-semibold">Up Next</h3>
                    <span className="text-sm text-muted-foreground">
                      {queue.length - currentIndex - 1} items
                    </span>
                  </div>

                  <div className="space-y-2">
                    {queue.slice(currentIndex + 1).map((item, index) => (
                      <PlaylistItem
                        key={item.id}
                        item={item}
                        index={currentIndex + 1 + index}
                        onPlay={() => handlePlayItem(currentIndex + 1 + index)}
                        onRemove={() => handleRemoveFromQueue(item.id)}
                        isDraggable
                      />
                    ))}
                  </div>
                </>
              )}
            </TabsContent>

            {/* Playlists Tab */}
            <TabsContent value="playlists">
              {/* Search and Create */}
              <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="relative max-w-sm flex-1">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="Search playlists..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9"
                  />
                </div>
                <Dialog open={isCreatingPlaylist} onOpenChange={setIsCreatingPlaylist}>
                  <DialogTrigger asChild>
                    <Button>
                      <Plus className="mr-2 h-4 w-4" />
                      New Playlist
                    </Button>
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Create New Playlist</DialogTitle>
                      <DialogDescription>
                        Give your playlist a name to get started.
                      </DialogDescription>
                    </DialogHeader>
                    <div className="py-4">
                      <Input
                        placeholder="Playlist name"
                        value={newPlaylistName}
                        onChange={(e) => setNewPlaylistName(e.target.value)}
                      />
                    </div>
                    <DialogFooter>
                      <Button variant="outline" onClick={() => setIsCreatingPlaylist(false)}>
                        Cancel
                      </Button>
                      <Button onClick={handleCreatePlaylist} disabled={!newPlaylistName.trim()}>
                        Create
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </div>

              {/* Playlist Grid */}
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {filteredPlaylists.map((playlist) => (
                  <Card
                    key={playlist.id}
                    className="cursor-pointer border-border/50 transition-colors hover:border-primary/50"
                    onClick={() => setSelectedPlaylist(playlist)}
                  >
                    <CardContent className="p-4">
                      <div className="flex gap-4">
                        <div className="relative h-20 w-20 flex-shrink-0 overflow-hidden rounded-lg bg-muted">
                          <img
                            src={playlist.thumbnail}
                            alt={playlist.name}
                            className="h-full w-full object-cover"
                          />
                          <div className="absolute inset-0 flex items-center justify-center bg-black/40">
                            <ListMusic className="h-6 w-6 text-white" />
                          </div>
                        </div>
                        <div className="flex-1 min-w-0">
                          <h3 className="font-semibold truncate">{playlist.name}</h3>
                          {playlist.description && (
                            <p className="mt-0.5 text-sm text-muted-foreground line-clamp-1">
                              {playlist.description}
                            </p>
                          )}
                          <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                            <span>{playlist.itemCount} items</span>
                            <span>·</span>
                            <span>{playlist.totalDuration}</span>
                          </div>
                        </div>
                        <ChevronRight className="h-5 w-5 flex-shrink-0 text-muted-foreground" />
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      )}
    </div>
  )
}

function PlaylistItem({
  item,
  index,
  onPlay,
  onRemove,
  isDraggable = false,
}: {
  item: QueueItem
  index: number
  onPlay: () => void
  onRemove: () => void
  isDraggable?: boolean
}) {
  return (
    <div className={cn(
      "group flex items-center gap-3 rounded-lg border border-border/50 p-2 transition-colors hover:bg-secondary/50",
      item.isPlaying && "border-primary/30 bg-primary/5"
    )}>
      {isDraggable && (
        <button className="cursor-grab text-muted-foreground/50 hover:text-muted-foreground">
          <GripVertical className="h-5 w-5" />
        </button>
      )}
      
      <span className="w-6 text-center text-sm text-muted-foreground">
        {item.isPlaying ? (
          <Play className="mx-auto h-4 w-4 text-primary" />
        ) : (
          index + 1
        )}
      </span>

      <div className="relative h-12 w-9 flex-shrink-0 overflow-hidden rounded bg-muted">
        <img
          src={item.poster}
          alt={item.title}
          className="h-full w-full object-cover"
        />
        {item.type === "movie" ? (
          <Film className="absolute bottom-0.5 left-0.5 h-3 w-3 text-white drop-shadow" />
        ) : (
          <Tv className="absolute bottom-0.5 left-0.5 h-3 w-3 text-white drop-shadow" />
        )}
      </div>

      <div className="flex-1 min-w-0">
        <h4 className={cn("truncate text-sm font-medium", item.isPlaying && "text-primary")}>
          {item.title}
        </h4>
        {item.season && item.episode && (
          <p className="truncate text-xs text-muted-foreground">
            S{item.season.toString().padStart(2, "0")}E{item.episode.toString().padStart(2, "0")}
            {item.episodeTitle && ` - ${item.episodeTitle}`}
          </p>
        )}
      </div>

      <span className="text-xs text-muted-foreground">{item.duration}</span>

      <div className="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onPlay}>
          <Play className="h-4 w-4" />
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={onPlay}>
              <Play className="mr-2 h-4 w-4" />
              Play Now
            </DropdownMenuItem>
            <DropdownMenuItem>
              <Plus className="mr-2 h-4 w-4" />
              Add to Playlist
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem className="text-destructive" onClick={onRemove}>
              <Trash2 className="mr-2 h-4 w-4" />
              Remove
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
