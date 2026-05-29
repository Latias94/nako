"use client"

import { useState } from "react"
import { AlertCircle, ChevronLeft, Grid3X3, List, ListMusic, Loader2, Play, Star } from "lucide-react"
import { resolveArtwork } from "@/lib/artwork"
import { useUserPlaylistItems, useUserPlaylists } from "@/lib/use-media"
import type { PublicUserPlaylistItem, PublicUserPlaylistSummary } from "@/src/api/public/media-data-source"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { cn } from "@/lib/utils"

export type MyListViewMode = "grid" | "list"

export type MyListRouteState = {
  playlistId?: string
  viewMode?: MyListViewMode
}

interface MyListPageProps {
  onBack: () => void
  onSelectMedia: (id: string, type: "movie" | "series") => void
  playlistId?: string
  viewMode?: MyListViewMode
  onRouteStateChange?: (state: MyListRouteState) => void
}

export function MyListPage({
  onBack,
  onSelectMedia,
  playlistId,
  viewMode,
  onRouteStateChange,
}: MyListPageProps) {
  const playlistsQuery = useUserPlaylists()
  const playlists = playlistsQuery.data?.playlists ?? []
  const [localPlaylistId, setLocalPlaylistId] = useState<string | undefined>()
  const [localViewMode, setLocalViewMode] = useState<MyListViewMode>("grid")
  const requestedPlaylistId = playlistId ?? localPlaylistId
  const matchedPlaylist = playlists.find((playlist) => playlist.id === requestedPlaylistId) ?? null
  const defaultPlaylist = playlists[0] ?? null
  const activePlaylistId =
    matchedPlaylist?.id ?? (requestedPlaylistId && playlists.length === 0 ? requestedPlaylistId : defaultPlaylist?.id)
  const activeViewMode = viewMode ?? localViewMode
  const playlistItemsQuery = useUserPlaylistItems(activePlaylistId)
  const activePlaylist = matchedPlaylist ?? playlistItemsQuery.data?.playlist ?? defaultPlaylist
  const playlistItems = playlistItemsQuery.data?.items ?? []
  const source = playlistItemsQuery.data?.source ?? playlistsQuery.data?.source ?? "fixture"
  const fallback = Boolean(playlistsQuery.data?.fallback || playlistItemsQuery.data?.fallback)
  const errorMessage =
    playlistsQuery.error instanceof Error
      ? playlistsQuery.error.message
      : playlistItemsQuery.error instanceof Error
        ? playlistItemsQuery.error.message
        : playlistsQuery.data?.error ?? playlistItemsQuery.data?.error
  const isLoadingPlaylists = playlistsQuery.isLoading
  const isLoadingItems = playlistItemsQuery.isLoading || (Boolean(activePlaylistId) && playlistItemsQuery.isFetching)

  const commitPlaylist = (nextPlaylistId: string) => {
    setLocalPlaylistId(nextPlaylistId)
    onRouteStateChange?.({
      playlistId: nextPlaylistId,
      viewMode: activeViewMode,
    })
  }

  const commitViewMode = (nextViewMode: MyListViewMode) => {
    setLocalViewMode(nextViewMode)
    onRouteStateChange?.({
      playlistId: activePlaylistId,
      viewMode: nextViewMode,
    })
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="sticky top-0 z-10 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/70">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-3 p-4">
          <div className="flex min-w-0 items-center gap-3">
            <Button variant="ghost" size="icon" onClick={onBack} aria-label="返回媒体库">
              <ChevronLeft className="h-5 w-5" />
            </Button>
            <div className="min-w-0">
              <h1 className="truncate text-xl font-semibold">我的列表</h1>
              <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                <Badge variant={source === "live" ? "default" : "secondary"}>
                  {source === "live" ? "Live Public Client" : "Fixture fallback"}
                </Badge>
                {fallback && errorMessage ? <span className="truncate">{errorMessage}</span> : null}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-1 rounded-md border border-border/60 bg-muted/40 p-1">
            <Button
              variant={activeViewMode === "grid" ? "secondary" : "ghost"}
              size="icon"
              className="h-8 w-8"
              onClick={() => commitViewMode("grid")}
              aria-label="网格视图"
              title="网格视图"
            >
              <Grid3X3 className="h-4 w-4" />
            </Button>
            <Button
              variant={activeViewMode === "list" ? "secondary" : "ghost"}
              size="icon"
              className="h-8 w-8"
              onClick={() => commitViewMode("list")}
              aria-label="列表视图"
              title="列表视图"
            >
              <List className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      <main className="mx-auto flex max-w-6xl flex-col gap-5 p-4">
        {errorMessage && fallback ? (
          <div className="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/10 p-3 text-sm text-warning-foreground">
            <AlertCircle className="mt-0.5 h-4 w-4 flex-shrink-0" />
            <span className="min-w-0">{errorMessage}</span>
          </div>
        ) : null}

        {isLoadingPlaylists ? (
          <PlaylistTabsSkeleton />
        ) : playlists.length > 0 ? (
          <PlaylistTabs
            playlists={playlists}
            activePlaylistId={activePlaylistId}
            onSelectPlaylist={commitPlaylist}
          />
        ) : (
          <EmptyState title="暂无播放列表" description="当前账号还没有可访问的播放列表。" />
        )}

        {activePlaylist ? (
          <section className="space-y-4">
            <div className="flex flex-wrap items-end justify-between gap-3">
              <div className="min-w-0">
                <h2 className="truncate text-lg font-semibold">{activePlaylist.name}</h2>
                <p className="text-sm text-muted-foreground">
                  {activePlaylist.itemCount} 个条目
                </p>
              </div>
              {isLoadingItems ? (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  加载中
                </div>
              ) : null}
            </div>

            {isLoadingItems && playlistItems.length === 0 ? (
              <PlaylistItemsSkeleton viewMode={activeViewMode} />
            ) : playlistItems.length > 0 ? (
              <PlaylistItems
                items={playlistItems}
                viewMode={activeViewMode}
                onSelectMedia={onSelectMedia}
              />
            ) : (
              <EmptyState title="列表为空" description="这个播放列表当前没有可访问的媒体条目。" />
            )}
          </section>
        ) : null}
      </main>
    </div>
  )
}

function PlaylistTabs({
  playlists,
  activePlaylistId,
  onSelectPlaylist,
}: {
  playlists: PublicUserPlaylistSummary[]
  activePlaylistId?: string
  onSelectPlaylist: (playlistId: string) => void
}) {
  return (
    <Tabs value={activePlaylistId ?? ""} onValueChange={onSelectPlaylist} className="w-full">
      <TabsList className="flex h-auto w-full justify-start gap-1 overflow-x-auto rounded-lg bg-muted/40 p-1">
        {playlists.map((playlist) => (
          <TabsTrigger
            key={playlist.id}
            value={playlist.id}
            className="min-w-0 flex-shrink-0 gap-2 rounded-md px-3 py-2"
          >
            <ListMusic className="h-4 w-4" />
            <span className="max-w-[11rem] truncate">{playlist.name}</span>
            <Badge variant="secondary" className="ml-1">
              {playlist.itemCount}
            </Badge>
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  )
}

function PlaylistItems({
  items,
  viewMode,
  onSelectMedia,
}: {
  items: PublicUserPlaylistItem[]
  viewMode: MyListViewMode
  onSelectMedia: (id: string, type: "movie" | "series") => void
}) {
  if (viewMode === "list") {
    return (
      <div className="space-y-2">
        {items.map((entry) => (
          <PlaylistListRow key={`${entry.playlistId}:${entry.itemId}`} entry={entry} onSelectMedia={onSelectMedia} />
        ))}
      </div>
    )
  }

  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
      {items.map((entry) => (
        <PlaylistPosterCard key={`${entry.playlistId}:${entry.itemId}`} entry={entry} onSelectMedia={onSelectMedia} />
      ))}
    </div>
  )
}

function PlaylistListRow({
  entry,
  onSelectMedia,
}: {
  entry: PublicUserPlaylistItem
  onSelectMedia: (id: string, type: "movie" | "series") => void
}) {
  const item = entry.item

  return (
    <button
      type="button"
      onClick={() => onSelectMedia(item.id, item.type)}
      className="group flex w-full items-center gap-4 rounded-lg border border-border/50 bg-card p-3 text-left transition-colors hover:bg-secondary/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <div className="relative h-20 w-14 flex-shrink-0 overflow-hidden rounded-md bg-muted">
        <img src={resolveArtwork(item.poster)} alt="" className="h-full w-full object-cover" />
        <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100">
          <Play className="h-6 w-6 text-white" />
        </div>
      </div>

      <div className="min-w-0 flex-1">
        <h3 className="truncate font-medium text-foreground group-hover:text-primary">{item.title}</h3>
        <div className="mt-1 flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
          <span>{item.year || "未知年份"}</span>
          <span aria-hidden="true">·</span>
          <span className="flex items-center gap-1">
            <Star className="h-3 w-3 fill-accent text-accent" />
            {item.rating || "N/A"}
          </span>
          <span aria-hidden="true">·</span>
          <Badge variant="outline" className="text-[10px]">
            {item.type === "series" ? "剧集" : "电影"}
          </Badge>
        </div>
        <p className="mt-1 text-xs text-muted-foreground">添加于 {formatDate(entry.addedAt)}</p>
      </div>
    </button>
  )
}

function PlaylistPosterCard({
  entry,
  onSelectMedia,
}: {
  entry: PublicUserPlaylistItem
  onSelectMedia: (id: string, type: "movie" | "series") => void
}) {
  const item = entry.item

  return (
    <button
      type="button"
      onClick={() => onSelectMedia(item.id, item.type)}
      className="group min-w-0 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted">
        <img
          src={resolveArtwork(item.poster)}
          alt=""
          className="h-full w-full object-cover transition-transform duration-200 group-hover:scale-[1.02]"
        />
        <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100">
          <span className="flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-sm">
            <Play className="h-4 w-4" />
          </span>
        </div>
      </div>
      <h3 className="mt-2 truncate text-sm font-medium text-foreground group-hover:text-primary">{item.title}</h3>
      <div className="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
        <span>{item.year || "未知年份"}</span>
        <span aria-hidden="true">·</span>
        <Star className="h-3 w-3 fill-accent text-accent" />
        <span>{item.rating || "N/A"}</span>
      </div>
    </button>
  )
}

function EmptyState({ title, description }: { title: string; description: string }) {
  return (
    <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border/70 bg-card/40 px-6 py-16 text-center">
      <ListMusic className="mb-4 h-12 w-12 text-muted-foreground/50" />
      <h3 className="text-lg font-medium">{title}</h3>
      <p className="mt-2 max-w-sm text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

function PlaylistTabsSkeleton() {
  return (
    <div className="flex gap-2 overflow-hidden rounded-lg bg-muted/40 p-1">
      {[0, 1, 2].map((index) => (
        <Skeleton key={index} className="h-10 w-32 flex-shrink-0" />
      ))}
    </div>
  )
}

function PlaylistItemsSkeleton({ viewMode }: { viewMode: MyListViewMode }) {
  const count = viewMode === "list" ? 4 : 6

  return (
    <div
      className={cn(
        viewMode === "list"
          ? "space-y-2"
          : "grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6",
      )}
    >
      {Array.from({ length: count }).map((_, index) => (
        <Skeleton key={index} className={viewMode === "list" ? "h-24 w-full" : "aspect-[2/3] w-full"} />
      ))}
    </div>
  )
}

function formatDate(value: string) {
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) {
    return value
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date(timestamp))
}
