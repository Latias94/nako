"use client"

import { useState } from "react"
import { 
  Bell, Check, CheckCheck, Trash2, Settings, Film, Tv, Download, 
  AlertCircle, Info, CheckCircle, RefreshCw, HardDrive, Users, 
  ChevronLeft, Filter, MoreHorizontal, Clock, X
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"

interface Notification {
  id: string
  type: "media" | "system" | "download" | "user"
  level: "info" | "success" | "warning" | "error"
  title: string
  message: string
  time: string
  read: boolean
  mediaId?: string
  mediaPoster?: string
  actionLabel?: string
  actionUrl?: string
}

// 模拟通知数据
const mockNotifications: Notification[] = [
  {
    id: "1",
    type: "media",
    level: "info",
    title: "新剧集可用",
    message: "《真探》第四季第3集已添加到媒体库",
    time: "5分钟前",
    read: false,
    mediaId: "td-s4e3",
    mediaPoster: "https://image.tmdb.org/t/p/w200/aowr4xpLP5sRCL50TkuADomJ98T.jpg",
    actionLabel: "立即观看",
  },
  {
    id: "2",
    type: "download",
    level: "success",
    title: "下载完成",
    message: "《沙丘2》(4K HDR) 下载完成，文件大小 45.2GB",
    time: "15分钟前",
    read: false,
    mediaId: "dune2",
    mediaPoster: "https://image.tmdb.org/t/p/w200/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg",
  },
  {
    id: "3",
    type: "system",
    level: "warning",
    title: "存储空间不足",
    message: "媒体存储空间已使用 92%，建议清理或扩容",
    time: "1小时前",
    read: false,
    actionLabel: "管理存储",
  },
  {
    id: "4",
    type: "media",
    level: "info",
    title: "媒体库扫描完成",
    message: "电影库扫描完成，新增 5 部电影，更新 12 部元数据",
    time: "2小时前",
    read: true,
  },
  {
    id: "5",
    type: "system",
    level: "success",
    title: "转码任务完成",
    message: "《奥本海默》转码完成 (H.265 4K → H.264 1080p)",
    time: "3小时前",
    read: true,
    mediaId: "opp",
  },
  {
    id: "6",
    type: "user",
    level: "info",
    title: "新用户登录",
    message: "用户「家人」从新设备登录 (iPhone 15 Pro)",
    time: "5小时前",
    read: true,
  },
  {
    id: "7",
    type: "system",
    level: "info",
    title: "系统更新可用",
    message: "Nako v1.3.0 版本可用，包含性能优化和新功能",
    time: "昨天",
    read: true,
    actionLabel: "查看更新",
  },
  {
    id: "8",
    type: "media",
    level: "info",
    title: "新电影添加",
    message: "《星际穿越》已添加到媒体库",
    time: "昨天",
    read: true,
    mediaId: "interstellar",
    mediaPoster: "https://image.tmdb.org/t/p/w200/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
  },
  {
    id: "9",
    type: "download",
    level: "error",
    title: "下载失败",
    message: "《银翼杀手 2049》下载失败：连接超时",
    time: "2天前",
    read: true,
    actionLabel: "重试",
  },
  {
    id: "10",
    type: "system",
    level: "success",
    title: "备份完成",
    message: "数据库自动备份完成 (大小: 128MB)",
    time: "3天前",
    read: true,
  },
]

interface NotificationCenterProps {
  onBack?: () => void
}

export function NotificationCenter({ onBack }: NotificationCenterProps) {
  const [notifications, setNotifications] = useState(mockNotifications)
  const [activeTab, setActiveTab] = useState("all")
  const [showSettings, setShowSettings] = useState(false)
  
  const unreadCount = notifications.filter(n => !n.read).length
  
  const filteredNotifications = notifications.filter(n => {
    if (activeTab === "all") return true
    if (activeTab === "unread") return !n.read
    return n.type === activeTab
  })
  
  const markAsRead = (id: string) => {
    setNotifications(prev => 
      prev.map(n => n.id === id ? { ...n, read: true } : n)
    )
  }
  
  const markAllAsRead = () => {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })))
  }
  
  const deleteNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id))
  }
  
  const clearAll = () => {
    setNotifications([])
  }
  
  const getTypeIcon = (type: string) => {
    switch (type) {
      case "media": return Film
      case "download": return Download
      case "user": return Users
      default: return Bell
    }
  }
  
  const getLevelColor = (level: string) => {
    switch (level) {
      case "success": return "text-green-500"
      case "warning": return "text-yellow-500"
      case "error": return "text-red-500"
      default: return "text-blue-500"
    }
  }
  
  const getLevelIcon = (level: string) => {
    switch (level) {
      case "success": return CheckCircle
      case "warning": return AlertCircle
      case "error": return AlertCircle
      default: return Info
    }
  }
  
  if (showSettings) {
    return (
      <div className="flex h-full flex-col bg-background">
        {/* Header */}
        <div className="flex items-center gap-4 border-b border-border px-4 py-4 lg:px-6">
          <Button variant="ghost" size="icon" onClick={() => setShowSettings(false)}>
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <h1 className="text-xl font-semibold">通知设置</h1>
        </div>
        
        <ScrollArea className="flex-1">
          <div className="space-y-6 p-4 lg:p-6">
            {/* 通知类型设置 */}
            <section className="space-y-4">
              <h2 className="text-lg font-medium">通知类型</h2>
              <div className="space-y-3 rounded-lg border border-border bg-card p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Film className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="font-medium">媒体更新</p>
                      <p className="text-sm text-muted-foreground">新内容添加、剧集更新</p>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Download className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="font-medium">下载通知</p>
                      <p className="text-sm text-muted-foreground">下载完成、失败提醒</p>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <HardDrive className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="font-medium">系统通知</p>
                      <p className="text-sm text-muted-foreground">存储、更新、任务完成</p>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Users className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="font-medium">用户活动</p>
                      <p className="text-sm text-muted-foreground">登录、权限变更</p>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
              </div>
            </section>
            
            {/* 通知行为 */}
            <section className="space-y-4">
              <h2 className="text-lg font-medium">通知行为</h2>
              <div className="space-y-3 rounded-lg border border-border bg-card p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">桌面通知</p>
                    <p className="text-sm text-muted-foreground">在系统通知中心显示</p>
                  </div>
                  <Switch defaultChecked />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">声音提醒</p>
                    <p className="text-sm text-muted-foreground">收到通知时播放声音</p>
                  </div>
                  <Switch />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">邮件通知</p>
                    <p className="text-sm text-muted-foreground">重要通知发送到邮箱</p>
                  </div>
                  <Switch />
                </div>
              </div>
            </section>
            
            {/* 自动清理 */}
            <section className="space-y-4">
              <h2 className="text-lg font-medium">自动清理</h2>
              <div className="space-y-3 rounded-lg border border-border bg-card p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">自动删除已读通知</p>
                    <p className="text-sm text-muted-foreground">30 天后自动删除</p>
                  </div>
                  <Switch defaultChecked />
                </div>
              </div>
            </section>
          </div>
        </ScrollArea>
      </div>
    )
  }
  
  return (
    <div className="flex h-full flex-col bg-background">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-4 lg:px-6">
        <div className="flex items-center gap-4">
          {onBack && (
            <Button variant="ghost" size="icon" onClick={onBack}>
              <ChevronLeft className="h-5 w-5" />
            </Button>
          )}
          <div>
            <h1 className="text-xl font-semibold">通知中心</h1>
            {unreadCount > 0 && (
              <p className="text-sm text-muted-foreground">{unreadCount} 条未读消息</p>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          {unreadCount > 0 && (
            <Button variant="ghost" size="sm" onClick={markAllAsRead}>
              <CheckCheck className="mr-2 h-4 w-4" />
              全部已读
            </Button>
          )}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon">
                <MoreHorizontal className="h-5 w-5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setShowSettings(true)}>
                <Settings className="mr-2 h-4 w-4" />
                通知设置
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={clearAll} className="text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                清空所有通知
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      
      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col">
        <div className="border-b border-border px-4 lg:px-6">
          <TabsList className="h-12 w-full justify-start gap-1 bg-transparent p-0">
            <TabsTrigger 
              value="all" 
              className="relative h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              全部
              <Badge variant="secondary" className="ml-2">{notifications.length}</Badge>
            </TabsTrigger>
            <TabsTrigger 
              value="unread"
              className="relative h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              未读
              {unreadCount > 0 && <Badge className="ml-2">{unreadCount}</Badge>}
            </TabsTrigger>
            <TabsTrigger 
              value="media"
              className="relative h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              <Film className="mr-2 h-4 w-4" />
              媒体
            </TabsTrigger>
            <TabsTrigger 
              value="download"
              className="relative h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              <Download className="mr-2 h-4 w-4" />
              下载
            </TabsTrigger>
            <TabsTrigger 
              value="system"
              className="relative h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
            >
              <HardDrive className="mr-2 h-4 w-4" />
              系统
            </TabsTrigger>
          </TabsList>
        </div>
        
        <TabsContent value={activeTab} className="flex-1 mt-0">
          <ScrollArea className="h-full">
            {filteredNotifications.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-center">
                <Bell className="mb-4 h-12 w-12 text-muted-foreground/50" />
                <p className="text-lg font-medium text-muted-foreground">暂无通知</p>
                <p className="mt-1 text-sm text-muted-foreground/70">
                  {activeTab === "unread" ? "所有通知都已读" : "这里还没有任何通知"}
                </p>
              </div>
            ) : (
              <div className="divide-y divide-border">
                {filteredNotifications.map(notification => {
                  const TypeIcon = getTypeIcon(notification.type)
                  const LevelIcon = getLevelIcon(notification.level)
                  
                  return (
                    <div
                      key={notification.id}
                      className={cn(
                        "flex gap-4 p-4 transition-colors hover:bg-muted/50 lg:px-6",
                        !notification.read && "bg-primary/5"
                      )}
                    >
                      {/* 媒体海报或图标 */}
                      {notification.mediaPoster ? (
                        <div className="relative h-16 w-12 flex-shrink-0 overflow-hidden rounded-md">
                          <img 
                            src={notification.mediaPoster} 
                            alt="" 
                            className="h-full w-full object-cover"
                          />
                        </div>
                      ) : (
                        <div className={cn(
                          "flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full",
                          notification.level === "error" ? "bg-red-500/10" :
                          notification.level === "warning" ? "bg-yellow-500/10" :
                          notification.level === "success" ? "bg-green-500/10" :
                          "bg-blue-500/10"
                        )}>
                          <LevelIcon className={cn("h-5 w-5", getLevelColor(notification.level))} />
                        </div>
                      )}
                      
                      {/* 内容 */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <h3 className={cn(
                                "font-medium",
                                !notification.read && "text-foreground"
                              )}>
                                {notification.title}
                              </h3>
                              {!notification.read && (
                                <span className="h-2 w-2 rounded-full bg-primary" />
                              )}
                            </div>
                            <p className="mt-0.5 text-sm text-muted-foreground line-clamp-2">
                              {notification.message}
                            </p>
                            <div className="mt-2 flex items-center gap-3">
                              <span className="flex items-center gap-1 text-xs text-muted-foreground">
                                <Clock className="h-3 w-3" />
                                {notification.time}
                              </span>
                              <Badge variant="outline" className="text-xs">
                                <TypeIcon className="mr-1 h-3 w-3" />
                                {notification.type === "media" ? "媒体" :
                                 notification.type === "download" ? "下载" :
                                 notification.type === "user" ? "用户" : "系统"}
                              </Badge>
                            </div>
                          </div>
                          
                          {/* 操作 */}
                          <div className="flex items-center gap-1">
                            {!notification.read && (
                              <Button 
                                variant="ghost" 
                                size="icon" 
                                className="h-8 w-8"
                                onClick={() => markAsRead(notification.id)}
                              >
                                <Check className="h-4 w-4" />
                              </Button>
                            )}
                            <Button 
                              variant="ghost" 
                              size="icon" 
                              className="h-8 w-8 text-muted-foreground hover:text-destructive"
                              onClick={() => deleteNotification(notification.id)}
                            >
                              <X className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                        
                        {/* 操作按钮 */}
                        {notification.actionLabel && (
                          <Button variant="outline" size="sm" className="mt-3">
                            {notification.actionLabel}
                          </Button>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </ScrollArea>
        </TabsContent>
      </Tabs>
    </div>
  )
}
