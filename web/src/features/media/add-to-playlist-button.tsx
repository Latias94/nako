"use client"

import { useState } from "react"
import { Check, ListPlus, Loader2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"
import { useAddUserPlaylistItemMutation, useUserPlaylists } from "@/lib/use-media"

type AddToPlaylistButtonProps = {
  itemId: string
  itemTitle: string
  variant?: "hero" | "icon"
  className?: string
  triggerClassName?: string
}

export function AddToPlaylistButton({
  itemId,
  itemTitle,
  variant = "hero",
  className,
  triggerClassName,
}: AddToPlaylistButtonProps) {
  const playlists = useUserPlaylists()
  const addItemMutation = useAddUserPlaylistItemMutation()
  const [pendingPlaylistId, setPendingPlaylistId] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const isIcon = variant === "icon"
  const isAdding = addItemMutation.isPending

  async function addToPlaylist(playlistId: string, playlistName: string) {
    setPendingPlaylistId(playlistId)
    setMessage(null)

    try {
      const payload = await addItemMutation.mutateAsync({ playlistId, itemId })

      if (payload.persisted) {
        setMessage(`已添加到 ${playlistName}`)
      } else {
        setMessage(payload.error ?? "播放列表条目未保存。")
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "播放列表条目未保存。")
    } finally {
      setPendingPlaylistId(null)
    }
  }

  return (
    <div className={cn("inline-flex flex-col items-start gap-2", className)}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant={isIcon ? "secondary" : "outline"}
            size={isIcon ? "icon-sm" : "lg"}
            aria-label={isIcon ? `添加 ${itemTitle} 到播放列表` : "添加到播放列表"}
            className={cn(
              isIcon
                ? "h-8 w-8 rounded-full bg-black/70 text-white shadow-sm backdrop-blur-sm hover:bg-black/85 hover:text-white"
                : "h-12 gap-2 border-white/20 bg-white/10 text-white backdrop-blur-sm hover:bg-white/20 hover:text-white",
              triggerClassName,
            )}
            disabled={!itemId || isAdding}
          >
            {isAdding ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <ListPlus className={cn(isIcon ? "h-4 w-4" : "h-5 w-5")} />
            )}
            {!isIcon && <span>添加到播放列表</span>}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="min-w-52">
          {playlists.isLoading && <DropdownMenuItem disabled>加载播放列表</DropdownMenuItem>}
          {playlists.isError && <DropdownMenuItem disabled>播放列表不可用</DropdownMenuItem>}
          {!playlists.isLoading && !playlists.isError && playlists.data?.playlists.length === 0 && (
            <DropdownMenuItem disabled>没有可用播放列表</DropdownMenuItem>
          )}
          {playlists.data?.playlists.map((playlist) => (
            <DropdownMenuItem
              key={playlist.id}
              disabled={isAdding}
              onSelect={() => {
                void addToPlaylist(playlist.id, playlist.name)
              }}
            >
              {pendingPlaylistId === playlist.id ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Check className="h-4 w-4" />
              )}
              添加到 {playlist.name}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
      {message && (
        <p
          role="status"
          className={cn(
            "max-w-64 text-xs leading-5",
            isIcon
              ? "max-w-[10rem] rounded-md bg-black/80 px-2 py-1 text-[10px] text-white shadow-sm backdrop-blur-sm"
              : "text-white/70",
            message.startsWith("已添加") ? "text-emerald-300" : "text-warning",
          )}
        >
          {message}
        </p>
      )}
    </div>
  )
}
