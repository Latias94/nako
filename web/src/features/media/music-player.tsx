"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState, useRef, useEffect, useCallback } from "react"
import {
  ChevronLeft, Play, Pause, SkipBack, SkipForward, Volume2, VolumeX, Shuffle, Repeat, Repeat1,
  Heart, MoreHorizontal, Search, ListMusic, Disc3, User, Clock, Plus, Download, Share2, X,
  ChevronRight, Music2, Mic2, Radio, Library, Home, Grid3X3, List
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Slider } from "@/components/ui/slider"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"

// Types
interface Track {
  id: string
  title: string
  artist: string
  artistId: string
  album: string
  albumId: string
  duration: number
  cover: string
  url: string
  isFavorite: boolean
  playCount: number
  trackNumber: number
  discNumber: number
  year: number
  genre: string
  bitrate: number
  format: string
}

interface Album {
  id: string
  title: string
  artist: string
  artistId: string
  cover: string
  year: number
  trackCount: number
  duration: number
  genre: string
}

interface Artist {
  id: string
  name: string
  avatar: string
  albumCount: number
  trackCount: number
  bio?: string
}

interface Playlist {
  id: string
  name: string
  cover: string
  trackCount: number
  duration: number
  isPublic: boolean
  createdAt: string
}

interface MusicPlayerProps {
  onBack: () => void
}

// Mock data
const generateMockTracks = (count: number): Track[] => {
  const artists = ["周杰伦", "陈奕迅", "林俊杰", "邓紫棋", "华晨宇", "毛不易", "薛之谦", "李荣浩"]
  const albums = ["范特西", "认了吧", "曹操", "泡沫", "新世界", "平凡的一天", "绅士", "麻雀"]
  const genres = ["流行", "摇滚", "民谣", "电子", "R&B", "嘻哈", "古典", "爵士"]

  return Array.from({ length: count }, (_, i) => ({
    id: `track-${i}`,
    title: `歌曲 ${i + 1}`,
    artist: artists[i % artists.length],
    artistId: `artist-${i % artists.length}`,
    album: albums[i % albums.length],
    albumId: `album-${i % albums.length}`,
    duration: 180 + Math.floor(Math.random() * 180),
    cover: `https://picsum.photos/seed/album${i % 20}/300/300`,
    url: "",
    isFavorite: Math.random() > 0.7,
    playCount: Math.floor(Math.random() * 1000),
    trackNumber: (i % 12) + 1,
    discNumber: 1,
    year: 2015 + (i % 10),
    genre: genres[i % genres.length],
    bitrate: [128, 192, 256, 320][i % 4],
    format: ["MP3", "FLAC", "AAC", "WAV"][i % 4],
  }))
}

const mockAlbums: Album[] = Array.from({ length: 20 }, (_, i) => ({
  id: `album-${i}`,
  title: `专辑 ${i + 1}`,
  artist: ["周杰伦", "陈奕迅", "林俊杰", "邓紫棋"][i % 4],
  artistId: `artist-${i % 4}`,
  cover: `https://picsum.photos/seed/album${i}/300/300`,
  year: 2015 + (i % 10),
  trackCount: 10 + (i % 6),
  duration: 2400 + i * 300,
  genre: ["流行", "摇滚", "民谣", "电子"][i % 4],
}))

const mockArtists: Artist[] = [
  { id: "1", name: "周杰伦", avatar: "https://picsum.photos/seed/jay/200/200", albumCount: 15, trackCount: 180 },
  { id: "2", name: "陈奕迅", avatar: "https://picsum.photos/seed/eason/200/200", albumCount: 20, trackCount: 250 },
  { id: "3", name: "林俊杰", avatar: "https://picsum.photos/seed/jj/200/200", albumCount: 14, trackCount: 160 },
  { id: "4", name: "邓紫棋", avatar: "https://picsum.photos/seed/gem/200/200", albumCount: 8, trackCount: 90 },
  { id: "5", name: "华晨宇", avatar: "https://picsum.photos/seed/hua/200/200", albumCount: 5, trackCount: 60 },
  { id: "6", name: "毛不易", avatar: "https://picsum.photos/seed/mao/200/200", albumCount: 4, trackCount: 45 },
]

const mockPlaylists: Playlist[] = [
  { id: "1", name: "我喜欢的音乐", cover: "https://picsum.photos/seed/fav/300/300", trackCount: 128, duration: 28800, isPublic: false, createdAt: "2024-01-01" },
  { id: "2", name: "工作学习", cover: "https://picsum.photos/seed/work/300/300", trackCount: 45, duration: 10800, isPublic: false, createdAt: "2024-02-15" },
  { id: "3", name: "运动健身", cover: "https://picsum.photos/seed/sport/300/300", trackCount: 32, duration: 7200, isPublic: true, createdAt: "2024-03-10" },
  { id: "4", name: "睡前放松", cover: "https://picsum.photos/seed/sleep/300/300", trackCount: 20, duration: 4800, isPublic: false, createdAt: "2024-04-05" },
]

type ViewMode = "home" | "albums" | "artists" | "playlists" | "songs" | "album-detail" | "artist-detail"

export function MusicPlayer({ onBack }: MusicPlayerProps) {
  const [viewMode, setViewMode] = useState<ViewMode>("home")
  const [tracks] = useState(() => generateMockTracks(100))
  const [currentTrack, setCurrentTrack] = useState<Track | null>(null)
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [volume, setVolume] = useState(80)
  const [isMuted, setIsMuted] = useState(false)
  const [isShuffled, setIsShuffled] = useState(false)
  const [repeatMode, setRepeatMode] = useState<"off" | "all" | "one">("off")
  const [queue, setQueue] = useState<Track[]>([])
  const [showQueue, setShowQueue] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedAlbum, setSelectedAlbum] = useState<Album | null>(null)
  const [selectedArtist, setSelectedArtist] = useState<Artist | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 500)
    return () => clearTimeout(timer)
  }, [])

  // Simulate playback progress
  useEffect(() => {
    if (isPlaying && currentTrack) {
      const interval = setInterval(() => {
        setCurrentTime(t => {
          if (t >= currentTrack.duration) {
            // Play next track
            return 0
          }
          return t + 1
        })
      }, 1000)
      return () => clearInterval(interval)
    }
  }, [isPlaying, currentTrack])

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m}:${s.toString().padStart(2, "0")}`
  }

  const formatDuration = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    if (h > 0) return `${h} 小时 ${m} 分钟`
    return `${m} 分钟`
  }

  const playTrack = (track: Track) => {
    setCurrentTrack(track)
    setCurrentTime(0)
    setIsPlaying(true)
  }

  const togglePlay = () => setIsPlaying(!isPlaying)

  const cycleRepeat = () => {
    if (repeatMode === "off") setRepeatMode("all")
    else if (repeatMode === "all") setRepeatMode("one")
    else setRepeatMode("off")
  }

  // Track list item
  const TrackItem = ({ track, index, showAlbum = true }: { track: Track; index?: number; showAlbum?: boolean }) => (
    <div
      className={cn(
        "group flex items-center gap-3 rounded-md px-2 py-2 hover:bg-muted/50",
        currentTrack?.id === track.id && "bg-muted"
      )}
      onClick={() => playTrack(track)}
    >
      <div className="w-8 text-center text-sm text-muted-foreground group-hover:hidden">
        {index !== undefined ? index + 1 : <Music2 className="mx-auto h-4 w-4" />}
      </div>
      <Button
        variant="ghost"
        size="icon"
        className="hidden h-8 w-8 group-hover:flex"
        onClick={(e) => { e.stopPropagation(); playTrack(track) }}
      >
        <Play className="h-4 w-4" />
      </Button>

      {showAlbum && (
        <img src={resolveArtwork(track.cover)} alt={track.album} className="h-10 w-10 rounded object-cover" />
      )}

      <div className="min-w-0 flex-1">
        <p className={cn("truncate font-medium", currentTrack?.id === track.id && "text-primary")}>
          {track.title}
        </p>
        <p className="truncate text-sm text-muted-foreground">{track.artist}</p>
      </div>

      {showAlbum && (
        <p className="hidden truncate text-sm text-muted-foreground md:block md:w-40">{track.album}</p>
      )}

      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 opacity-0 group-hover:opacity-100"
          onClick={(e) => { e.stopPropagation() }}
        >
          <Heart className={cn("h-4 w-4", track.isFavorite && "fill-red-500 text-red-500")} />
        </Button>
        <span className="w-12 text-right text-sm text-muted-foreground">{formatTime(track.duration)}</span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 opacity-0 group-hover:opacity-100"
              onClick={(e) => e.stopPropagation()}
            >
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>添加到播放队列</DropdownMenuItem>
            <DropdownMenuItem>添加到播放列表</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>查看专辑</DropdownMenuItem>
            <DropdownMenuItem>查看艺术家</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>下载</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )

  // Album card
  const AlbumCard = ({ album }: { album: Album }) => (
    <div
      className="group cursor-pointer"
      onClick={() => { setSelectedAlbum(album); setViewMode("album-detail") }}
    >
      <div className="relative mb-2 overflow-hidden rounded-lg">
        <img
          src={resolveArtwork(album.cover)}
          alt={album.title}
          className="aspect-square w-full object-cover transition-transform group-hover:scale-105"
        />
        <Button
          size="icon"
          className="absolute bottom-2 right-2 h-10 w-10 rounded-full opacity-0 shadow-lg transition-opacity group-hover:opacity-100"
          onClick={(e) => { e.stopPropagation(); /* play album */ }}
        >
          <Play className="h-5 w-5" />
        </Button>
      </div>
      <h3 className="truncate font-medium">{album.title}</h3>
      <p className="truncate text-sm text-muted-foreground">{album.artist} · {album.year}</p>
    </div>
  )

  // Artist card
  const ArtistCard = ({ artist }: { artist: Artist }) => (
    <div
      className="group cursor-pointer text-center"
      onClick={() => { setSelectedArtist(artist); setViewMode("artist-detail") }}
    >
      <div className="relative mx-auto mb-3 overflow-hidden rounded-full">
        <img
          src={resolveArtwork(artist.avatar)}
          alt={artist.name}
          className="aspect-square w-full object-cover transition-transform group-hover:scale-105"
        />
      </div>
      <h3 className="font-medium">{artist.name}</h3>
      <p className="text-sm text-muted-foreground">{artist.albumCount} 张专辑</p>
    </div>
  )

  // Playlist card
  const PlaylistCard = ({ playlist }: { playlist: Playlist }) => (
    <div className="group cursor-pointer">
      <div className="relative mb-2 overflow-hidden rounded-lg">
        <img
          src={resolveArtwork(playlist.cover)}
          alt={playlist.name}
          className="aspect-square w-full object-cover transition-transform group-hover:scale-105"
        />
        <Button
          size="icon"
          className="absolute bottom-2 right-2 h-10 w-10 rounded-full opacity-0 shadow-lg transition-opacity group-hover:opacity-100"
        >
          <Play className="h-5 w-5" />
        </Button>
      </div>
      <h3 className="truncate font-medium">{playlist.name}</h3>
      <p className="text-sm text-muted-foreground">{playlist.trackCount} 首歌曲</p>
    </div>
  )

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Main content area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="hidden w-56 flex-shrink-0 border-r border-border md:block">
          <div className="flex h-full flex-col">
            <div className="flex items-center gap-2 border-b border-border p-4">
              <Button variant="ghost" size="icon" onClick={onBack}>
                <ChevronLeft className="h-5 w-5" />
              </Button>
              <div className="flex items-center gap-2">
                <img src={resolveArtwork("/nako-icon.png")} alt="Nako" className="h-8 w-8 rounded-lg" />
                <span className="font-semibold">音乐</span>
              </div>
            </div>

            <ScrollArea className="flex-1 p-2">
              <nav className="space-y-1">
                <Button
                  variant={viewMode === "home" ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setViewMode("home")}
                >
                  <Home className="mr-2 h-4 w-4" />
                  主页
                </Button>
                <Button
                  variant={viewMode === "songs" ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setViewMode("songs")}
                >
                  <Music2 className="mr-2 h-4 w-4" />
                  所有歌曲
                </Button>
                <Button
                  variant={viewMode === "albums" ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setViewMode("albums")}
                >
                  <Disc3 className="mr-2 h-4 w-4" />
                  专辑
                </Button>
                <Button
                  variant={viewMode === "artists" ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setViewMode("artists")}
                >
                  <Mic2 className="mr-2 h-4 w-4" />
                  艺术家
                </Button>
                <Button
                  variant={viewMode === "playlists" ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setViewMode("playlists")}
                >
                  <ListMusic className="mr-2 h-4 w-4" />
                  播放列表
                </Button>
              </nav>

              <div className="mt-6">
                <div className="mb-2 flex items-center justify-between px-2">
                  <span className="text-xs font-medium uppercase text-muted-foreground">播放列表</span>
                  <Button variant="ghost" size="icon" className="h-6 w-6">
                    <Plus className="h-4 w-4" />
                  </Button>
                </div>
                <nav className="space-y-1">
                  {mockPlaylists.map(playlist => (
                    <Button key={playlist.id} variant="ghost" className="w-full justify-start truncate text-sm">
                      {playlist.name}
                    </Button>
                  ))}
                </nav>
              </div>
            </ScrollArea>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex flex-1 flex-col overflow-hidden">
          {/* Search bar */}
          <div className="flex items-center gap-3 border-b border-border px-4 py-3">
            <div className="relative flex-1 md:max-w-md">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜索歌曲、专辑、艺术家..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>
          </div>

          {/* Content */}
          <ScrollArea className="flex-1">
            <div className="p-4 pb-32">
              {/* Home View */}
              {viewMode === "home" && (
                <div className="space-y-8">
                  {/* Recently played */}
                  <section>
                    <div className="mb-4 flex items-center justify-between">
                      <h2 className="text-xl font-semibold">最近播放</h2>
                      <Button variant="ghost" size="sm" className="text-muted-foreground hover:bg-transparent hover:text-foreground">
                        查看全部 <ChevronRight className="ml-1 h-4 w-4" />
                      </Button>
                    </div>
                    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                      {mockAlbums.slice(0, 6).map(album => (
                        <AlbumCard key={album.id} album={album} />
                      ))}
                    </div>
                  </section>

                  {/* Favorite artists */}
                  <section>
                    <div className="mb-4 flex items-center justify-between">
                      <h2 className="text-xl font-semibold">喜爱的艺术家</h2>
                      <Button variant="ghost" size="sm" className="text-muted-foreground hover:bg-transparent hover:text-foreground">
                        查看全部 <ChevronRight className="ml-1 h-4 w-4" />
                      </Button>
                    </div>
                    <div className="grid grid-cols-3 gap-4 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
                      {mockArtists.map(artist => (
                        <ArtistCard key={artist.id} artist={artist} />
                      ))}
                    </div>
                  </section>

                  {/* Playlists */}
                  <section>
                    <div className="mb-4 flex items-center justify-between">
                      <h2 className="text-xl font-semibold">我的播放列表</h2>
                      <Button variant="outline" size="sm">
                        <Plus className="mr-2 h-4 w-4" />
                        新建
                      </Button>
                    </div>
                    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                      {mockPlaylists.map(playlist => (
                        <PlaylistCard key={playlist.id} playlist={playlist} />
                      ))}
                    </div>
                  </section>
                </div>
              )}

              {/* Songs View */}
              {viewMode === "songs" && (
                <div>
                  <div className="mb-4 flex items-center justify-between">
                    <h2 className="text-xl font-semibold">所有歌曲 ({tracks.length})</h2>
                    <div className="flex items-center gap-2">
                      <Button variant="outline" size="sm">
                        <Shuffle className="mr-2 h-4 w-4" />
                        随机播放
                      </Button>
                    </div>
                  </div>
                  <div className="space-y-1">
                    {isLoading ? (
                      Array.from({ length: 10 }).map((_, i) => (
                        <div key={i} className="flex items-center gap-3 py-2">
                          <Skeleton className="h-10 w-10 rounded" />
                          <div className="flex-1 space-y-1">
                            <Skeleton className="h-4 w-1/3" />
                            <Skeleton className="h-3 w-1/4" />
                          </div>
                        </div>
                      ))
                    ) : (
                      tracks.map((track, i) => (
                        <TrackItem key={track.id} track={track} index={i} />
                      ))
                    )}
                  </div>
                </div>
              )}

              {/* Albums View */}
              {viewMode === "albums" && (
                <div>
                  <div className="mb-4 flex items-center justify-between">
                    <h2 className="text-xl font-semibold">专辑 ({mockAlbums.length})</h2>
                  </div>
                  <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                    {mockAlbums.map(album => (
                      <AlbumCard key={album.id} album={album} />
                    ))}
                  </div>
                </div>
              )}

              {/* Artists View */}
              {viewMode === "artists" && (
                <div>
                  <div className="mb-4">
                    <h2 className="text-xl font-semibold">艺术家 ({mockArtists.length})</h2>
                  </div>
                  <div className="grid grid-cols-3 gap-6 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
                    {mockArtists.map(artist => (
                      <ArtistCard key={artist.id} artist={artist} />
                    ))}
                  </div>
                </div>
              )}

              {/* Playlists View */}
              {viewMode === "playlists" && (
                <div>
                  <div className="mb-4 flex items-center justify-between">
                    <h2 className="text-xl font-semibold">播放列表</h2>
                    <Button variant="outline" size="sm">
                      <Plus className="mr-2 h-4 w-4" />
                      新建播放列表
                    </Button>
                  </div>
                  <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                    {mockPlaylists.map(playlist => (
                      <PlaylistCard key={playlist.id} playlist={playlist} />
                    ))}
                  </div>
                </div>
              )}

              {/* Album Detail View */}
              {viewMode === "album-detail" && selectedAlbum && (
                <div>
                  <Button variant="ghost" className="mb-4" onClick={() => setViewMode("albums")}>
                    <ChevronLeft className="mr-2 h-4 w-4" />
                    返回
                  </Button>

                  <div className="mb-6 flex gap-6">
                    <img
                      src={resolveArtwork(selectedAlbum.cover)}
                      alt={selectedAlbum.title}
                      className="h-48 w-48 rounded-lg object-cover shadow-lg"
                    />
                    <div className="flex flex-col justify-end">
                      <p className="text-sm text-muted-foreground">专辑</p>
                      <h1 className="mb-2 text-3xl font-bold">{selectedAlbum.title}</h1>
                      <p className="mb-4 text-muted-foreground">
                        {selectedAlbum.artist} · {selectedAlbum.year} · {selectedAlbum.trackCount} 首歌曲 · {formatDuration(selectedAlbum.duration)}
                      </p>
                      <div className="flex items-center gap-2">
                        <Button>
                          <Play className="mr-2 h-4 w-4" />
                          播放
                        </Button>
                        <Button variant="outline">
                          <Shuffle className="mr-2 h-4 w-4" />
                          随机播放
                        </Button>
                        <Button variant="ghost" size="icon">
                          <Heart className="h-5 w-5" />
                        </Button>
                        <Button variant="ghost" size="icon">
                          <MoreHorizontal className="h-5 w-5" />
                        </Button>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-1">
                    {tracks.slice(0, selectedAlbum.trackCount).map((track, i) => (
                      <TrackItem key={track.id} track={track} index={i} showAlbum={false} />
                    ))}
                  </div>
                </div>
              )}

              {/* Artist Detail View */}
              {viewMode === "artist-detail" && selectedArtist && (
                <div>
                  <Button variant="ghost" className="mb-4" onClick={() => setViewMode("artists")}>
                    <ChevronLeft className="mr-2 h-4 w-4" />
                    返回
                  </Button>

                  <div className="mb-6 flex items-end gap-6">
                    <img
                      src={resolveArtwork(selectedArtist.avatar)}
                      alt={selectedArtist.name}
                      className="h-48 w-48 rounded-full object-cover shadow-lg"
                    />
                    <div>
                      <p className="text-sm text-muted-foreground">艺术家</p>
                      <h1 className="mb-2 text-3xl font-bold">{selectedArtist.name}</h1>
                      <p className="mb-4 text-muted-foreground">
                        {selectedArtist.albumCount} 张专辑 · {selectedArtist.trackCount} 首歌曲
                      </p>
                      <div className="flex items-center gap-2">
                        <Button>
                          <Play className="mr-2 h-4 w-4" />
                          播放
                        </Button>
                        <Button variant="outline">
                          <Shuffle className="mr-2 h-4 w-4" />
                          随机播放
                        </Button>
                      </div>
                    </div>
                  </div>

                  <section className="mb-8">
                    <h2 className="mb-4 text-xl font-semibold">热门歌曲</h2>
                    <div className="space-y-1">
                      {tracks.slice(0, 5).map((track, i) => (
                        <TrackItem key={track.id} track={track} index={i} />
                      ))}
                    </div>
                  </section>

                  <section>
                    <h2 className="mb-4 text-xl font-semibold">专辑</h2>
                    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                      {mockAlbums.filter(a => a.artist === selectedArtist.name).map(album => (
                        <AlbumCard key={album.id} album={album} />
                      ))}
                    </div>
                  </section>
                </div>
              )}
            </div>
          </ScrollArea>
        </main>

        {/* Queue panel */}
        {showQueue && (
          <aside className="w-80 border-l border-border">
            <div className="flex items-center justify-between border-b border-border p-4">
              <h3 className="font-semibold">播放队列</h3>
              <Button variant="ghost" size="icon" onClick={() => setShowQueue(false)}>
                <X className="h-4 w-4" />
              </Button>
            </div>
            <ScrollArea className="h-[calc(100%-57px)]">
              <div className="p-2">
                {queue.length === 0 ? (
                  <p className="py-8 text-center text-sm text-muted-foreground">队列为空</p>
                ) : (
                  queue.map((track, i) => (
                    <TrackItem key={track.id} track={track} index={i} showAlbum={false} />
                  ))
                )}
              </div>
            </ScrollArea>
          </aside>
        )}
      </div>

      {/* Now playing bar */}
      {currentTrack && (
        <div className="flex h-20 items-center justify-between border-t border-border bg-card px-4">
          {/* Track info */}
          <div className="flex items-center gap-3 w-1/4 min-w-0">
            <img src={resolveArtwork(currentTrack.cover)} alt={currentTrack.album} className="h-14 w-14 rounded object-cover" />
            <div className="min-w-0">
              <p className="truncate font-medium">{currentTrack.title}</p>
              <p className="truncate text-sm text-muted-foreground">{currentTrack.artist}</p>
            </div>
            <Button variant="ghost" size="icon">
              <Heart className={cn("h-5 w-5", currentTrack.isFavorite && "fill-red-500 text-red-500")} />
            </Button>
          </div>

          {/* Playback controls */}
          <div className="flex flex-col items-center gap-1 w-2/4">
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="icon"
                className={cn(isShuffled && "text-primary")}
                onClick={() => setIsShuffled(!isShuffled)}
              >
                <Shuffle className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="icon">
                <SkipBack className="h-5 w-5" />
              </Button>
              <Button size="icon" className="h-10 w-10 rounded-full" onClick={togglePlay}>
                {isPlaying ? <Pause className="h-5 w-5" /> : <Play className="h-5 w-5 ml-0.5" />}
              </Button>
              <Button variant="ghost" size="icon">
                <SkipForward className="h-5 w-5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className={cn(repeatMode !== "off" && "text-primary")}
                onClick={cycleRepeat}
              >
                {repeatMode === "one" ? <Repeat1 className="h-4 w-4" /> : <Repeat className="h-4 w-4" />}
              </Button>
            </div>

            <div className="flex w-full max-w-md items-center gap-2">
              <span className="w-10 text-right text-xs text-muted-foreground">{formatTime(currentTime)}</span>
              <Slider
                value={[currentTime]}
                max={currentTrack.duration}
                step={1}
                onValueChange={([v]) => setCurrentTime(v)}
                className="flex-1"
              />
              <span className="w-10 text-xs text-muted-foreground">{formatTime(currentTrack.duration)}</span>
            </div>
          </div>

          {/* Volume & extras */}
          <div className="flex items-center justify-end gap-2 w-1/4">
            <Button
              variant="ghost"
              size="icon"
              className={cn(showQueue && "text-primary")}
              onClick={() => setShowQueue(!showQueue)}
            >
              <ListMusic className="h-5 w-5" />
            </Button>
            <Button variant="ghost" size="icon" onClick={() => setIsMuted(!isMuted)}>
              {isMuted || volume === 0 ? <VolumeX className="h-5 w-5" /> : <Volume2 className="h-5 w-5" />}
            </Button>
            <Slider
              value={[isMuted ? 0 : volume]}
              max={100}
              step={1}
              onValueChange={([v]) => { setVolume(v); setIsMuted(false) }}
              className="w-24"
            />
          </div>
        </div>
      )}
    </div>
  )
}
