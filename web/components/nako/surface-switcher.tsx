"use client"

import { Film, Settings, Search, Bell, User, LogOut, Users, Check } from "lucide-react"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { ScrollArea } from "@/components/ui/scroll-area"

interface SurfaceSwitcherProps {
  currentSurface: "media" | "admin"
  onSurfaceChange: (surface: "media" | "admin") => void
  onSearchClick?: () => void
  onSettingsClick?: () => void
  onSwitchUserClick?: () => void
  onNotificationsClick?: () => void
}

// Mock notifications data
const notifications = [
  { id: 1, title: "媒体库扫描完成", message: "电影库已更新，新增 3 部电影", time: "5分钟前", read: false },
  { id: 2, title: "新剧集可用", message: "《真探》第二季已添加到媒体库", time: "1小时前", read: false },
  { id: 3, title: "转码完成", message: "奥本海默 (4K) 转码完成", time: "2小时前", read: true },
  { id: 4, title: "系统更新", message: "Nako 1.2.0 版本可用", time: "昨天", read: true },
]

export function SurfaceSwitcher({ currentSurface, onSurfaceChange, onSearchClick, onSettingsClick, onSwitchUserClick, onNotificationsClick }: SurfaceSwitcherProps) {
  const unreadCount = notifications.filter(n => !n.read).length

  return (
    <header className="sticky top-0 z-50 border-b border-border/50 bg-background/95 backdrop-blur-sm">
      <div className="flex h-14 items-center justify-between px-4 lg:px-6">
        {/* Logo and Brand */}
        <div className="flex items-center gap-3">
          <img src="/nako-icon.png" alt="Nako" className="h-8 w-8 rounded-lg" />
          <span className="text-lg font-semibold tracking-tight text-foreground">Nako</span>
          <span className="hidden text-xs text-muted-foreground sm:inline-block">私人媒体库</span>
        </div>

        {/* Surface Tabs */}
        <div className="flex items-center gap-1 rounded-lg bg-muted/50 p-1">
          <button
            onClick={() => onSurfaceChange("media")}
            className={cn(
              "flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium transition-all",
              currentSurface === "media"
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Film className="h-3.5 w-3.5" />
            <span>媒体库</span>
          </button>
          <button
            onClick={() => onSurfaceChange("admin")}
            className={cn(
              "flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium transition-all",
              currentSurface === "admin"
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Settings className="h-3.5 w-3.5" />
            <span>管理面板</span>
          </button>
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-2">
          <Button 
            variant="ghost" 
            size="icon" 
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
            onClick={onSearchClick}
          >
            <Search className="h-4 w-4" />
            <span className="sr-only">搜索</span>
          </Button>
          
          {/* Notifications Popover */}
          <Popover>
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" className="relative h-8 w-8 text-muted-foreground hover:text-foreground">
                <Bell className="h-4 w-4" />
                {unreadCount > 0 && (
                  <span className="absolute -right-0.5 -top-0.5 flex h-3.5 min-w-[14px] items-center justify-center rounded-full bg-red-500 px-1 text-[9px] font-medium text-white">
                    {unreadCount > 99 ? "99+" : unreadCount}
                  </span>
                )}
                <span className="sr-only">通知</span>
              </Button>
            </PopoverTrigger>
            <PopoverContent align="end" className="w-80 p-0">
              <div className="flex items-center justify-between border-b border-border px-4 py-3">
                <h4 className="text-sm font-semibold">通知</h4>
                <Button variant="ghost" size="sm" className="h-auto p-0 text-xs text-muted-foreground hover:text-foreground">
                  全部标为已读
                </Button>
              </div>
              <ScrollArea className="h-80">
                <div className="divide-y divide-border">
                  {notifications.map((notification) => (
                    <div
                      key={notification.id}
                      className={cn(
                        "flex gap-3 p-4 transition-colors hover:bg-muted/50",
                        !notification.read && "bg-primary/5"
                      )}
                    >
                      <div className={cn(
                        "mt-1 h-2 w-2 flex-shrink-0 rounded-full",
                        notification.read ? "bg-transparent" : "bg-primary"
                      )} />
                      <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">{notification.title}</p>
                        <p className="text-xs text-muted-foreground">{notification.message}</p>
                        <p className="text-xs text-muted-foreground/70">{notification.time}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </ScrollArea>
              <div className="border-t border-border p-2">
                <Button variant="ghost" size="sm" className="w-full text-xs" onClick={onNotificationsClick}>
                  查看全部通知
                </Button>
              </div>
            </PopoverContent>
          </Popover>
          
          {/* User Avatar Dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className="ml-2 flex h-8 w-8 items-center justify-center rounded-full bg-secondary text-xs font-medium text-secondary-foreground transition-colors hover:bg-secondary/80">
                管理
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <DropdownMenuLabel>
                <div className="flex flex-col space-y-1">
                  <p className="text-sm font-medium">管理员</p>
                  <p className="text-xs text-muted-foreground">admin@localhost</p>
                </div>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={onSettingsClick}>
                <User className="mr-2 h-4 w-4" />
                个人设置
              </DropdownMenuItem>
              <DropdownMenuItem onClick={onSwitchUserClick}>
                <Users className="mr-2 h-4 w-4" />
                切换用户
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem className="text-destructive focus:text-destructive">
                <LogOut className="mr-2 h-4 w-4" />
                退出登录
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </header>
  )
}
