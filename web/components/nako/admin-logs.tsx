"use client"

import { useState, useRef, useEffect, useCallback, useMemo } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"
import { 
  Search, Filter, Download, Trash2, RefreshCw, Pause, Play,
  AlertCircle, AlertTriangle, Info, Bug, ChevronDown, X,
  Clock, Server, User, Database, Shield, Wifi, Copy, Check,
  Calendar, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight,
  Loader2
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  DropdownMenuCheckboxItem,
} from "@/components/ui/dropdown-menu"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { cn } from "@/lib/utils"

type LogLevel = "error" | "warn" | "info" | "debug"
type LogSource = "server" | "auth" | "database" | "api" | "playback" | "scanner"

interface LogEntry {
  id: string
  timestamp: string
  level: LogLevel
  source: LogSource
  message: string
  details?: string
  userId?: string
  requestId?: string
}

// 生成大量模拟日志数据
const generateLogs = (count: number = 500): LogEntry[] => {
  const sources: LogSource[] = ["server", "auth", "database", "api", "playback", "scanner"]
  const levels: LogLevel[] = ["error", "warn", "info", "debug"]
  
  const messages = {
    server: [
      "Server started on port 8096",
      "Configuration loaded successfully",
      "Cache cleared",
      "Memory usage: 2.4GB / 8GB",
      "CPU usage spike detected: 85%",
    ],
    auth: [
      "User 'admin' logged in from 192.168.1.100",
      "Failed login attempt for user 'test'",
      "Session expired for user 'john'",
      "Password changed for user 'admin'",
      "New user 'guest' created",
    ],
    database: [
      "Database connection established",
      "Query executed in 45ms",
      "Database backup completed",
      "Index rebuild started",
      "Connection pool: 5/20 active",
    ],
    api: [
      "GET /api/items - 200 OK (125ms)",
      "POST /api/playback - 201 Created",
      "Rate limit exceeded for IP 10.0.0.5",
      "API key validated for external app",
      "Webhook delivery failed: timeout",
    ],
    playback: [
      "Stream started: Movie 'Dune' (1080p)",
      "Transcoding initiated for client iOS",
      "Direct play enabled for client TV",
      "Playback stopped by user",
      "Buffer underrun detected",
    ],
    scanner: [
      "Library scan started: Movies",
      "New file detected: /movies/new_movie.mkv",
      "Metadata fetched for 'Inception'",
      "Scan completed: 847 items processed",
      "Error scanning: Permission denied",
    ],
  }

  const logs: LogEntry[] = []
  const now = new Date()
  
  for (let i = 0; i < count; i++) {
    const source = sources[Math.floor(Math.random() * sources.length)]
    const level = levels[Math.floor(Math.random() * (source === "scanner" && Math.random() > 0.7 ? 1 : 4))]
    const messageList = messages[source]
    const message = messageList[Math.floor(Math.random() * messageList.length)]
    
    // 生成过去24小时内的随机时间
    const timestamp = new Date(now.getTime() - Math.random() * 24 * 60 * 60 * 1000)
    
    logs.push({
      id: `log-${i}`,
      timestamp: timestamp.toISOString(),
      level,
      source,
      message,
      details: Math.random() > 0.7 ? `Stack trace or additional details for log ${i}` : undefined,
      requestId: Math.random() > 0.5 ? `req-${Math.random().toString(36).substr(2, 9)}` : undefined,
    })
  }
  
  // 按时间倒序排列
  return logs.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
}

// 时间范围选项
const timeRanges = [
  { value: "1h", label: "最近1小时" },
  { value: "6h", label: "最近6小时" },
  { value: "24h", label: "最近24小时" },
  { value: "7d", label: "最近7天" },
  { value: "30d", label: "最近30天" },
  { value: "custom", label: "自定义范围" },
]

export function AdminLogs() {
  const [logs] = useState<LogEntry[]>(() => generateLogs(500))
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedLevels, setSelectedLevels] = useState<LogLevel[]>(["error", "warn", "info", "debug"])
  const [selectedSources, setSelectedSources] = useState<LogSource[]>(["server", "auth", "database", "api", "playback", "scanner"])
  const [activeTab, setActiveTab] = useState<"all" | "errors" | "warnings">("all")
  const [isLive, setIsLive] = useState(true)
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set())
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const [timeRange, setTimeRange] = useState("24h")
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  
  const parentRef = useRef<HTMLDivElement>(null)

  // 根据时间范围过滤
  const getTimeRangeFilter = useCallback((range: string) => {
    const now = new Date()
    switch (range) {
      case "1h": return new Date(now.getTime() - 1 * 60 * 60 * 1000)
      case "6h": return new Date(now.getTime() - 6 * 60 * 60 * 1000)
      case "24h": return new Date(now.getTime() - 24 * 60 * 60 * 1000)
      case "7d": return new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
      case "30d": return new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000)
      default: return new Date(0)
    }
  }, [])

  // 过滤日志
  const filteredLogs = useMemo(() => {
    const timeFilter = getTimeRangeFilter(timeRange)
    
    return logs.filter(log => {
      // 时间范围过滤
      if (new Date(log.timestamp) < timeFilter) return false
      
      // Tab 过滤
      if (activeTab === "errors" && log.level !== "error") return false
      if (activeTab === "warnings" && log.level !== "warn") return false
      
      // 级别过滤
      if (!selectedLevels.includes(log.level)) return false
      
      // 来源过滤
      if (!selectedSources.includes(log.source)) return false
      
      // 搜索过滤
      if (searchQuery) {
        const query = searchQuery.toLowerCase()
        return (
          log.message.toLowerCase().includes(query) ||
          log.source.toLowerCase().includes(query) ||
          log.level.toLowerCase().includes(query) ||
          (log.details?.toLowerCase().includes(query) ?? false)
        )
      }
      
      return true
    })
  }, [logs, activeTab, selectedLevels, selectedSources, searchQuery, timeRange, getTimeRangeFilter])

  // 虚拟化列表
  const rowVirtualizer = useVirtualizer({
    count: filteredLogs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: useCallback((index: number) => {
      const log = filteredLogs[index]
      // 根据是否展开和是否有详情调整高度
      if (expandedLogs.has(log?.id)) return 120
      return 56
    }, [filteredLogs, expandedLogs]),
    overscan: 10,
  })

  // 统计
  const errorCount = logs.filter(l => l.level === "error").length
  const warnCount = logs.filter(l => l.level === "warn").length

  const getLevelIcon = (level: LogLevel) => {
    switch (level) {
      case "error": return <AlertCircle className="h-4 w-4 text-destructive" />
      case "warn": return <AlertTriangle className="h-4 w-4 text-yellow-500" />
      case "info": return <Info className="h-4 w-4 text-blue-500" />
      case "debug": return <Bug className="h-4 w-4 text-muted-foreground" />
    }
  }

  const getSourceIcon = (source: LogSource) => {
    switch (source) {
      case "server": return <Server className="h-4 w-4" />
      case "auth": return <Shield className="h-4 w-4" />
      case "database": return <Database className="h-4 w-4" />
      case "api": return <Wifi className="h-4 w-4" />
      case "playback": return <Play className="h-4 w-4" />
      case "scanner": return <Search className="h-4 w-4" />
    }
  }

  const toggleLevel = (level: LogLevel) => {
    setSelectedLevels(prev => 
      prev.includes(level) 
        ? prev.filter(l => l !== level)
        : [...prev, level]
    )
  }

  const toggleSource = (source: LogSource) => {
    setSelectedSources(prev =>
      prev.includes(source)
        ? prev.filter(s => s !== source)
        : [...prev, source]
    )
  }

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    })
  }

  const copyToClipboard = (log: LogEntry) => {
    const text = `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.source}] ${log.message}${log.details ? `\n${log.details}` : ""}`
    navigator.clipboard.writeText(text)
    setCopiedId(log.id)
    setTimeout(() => setCopiedId(null), 2000)
  }

  // 模拟加载更多历史日志
  const loadMoreHistory = () => {
    setIsLoadingMore(true)
    setTimeout(() => {
      setIsLoadingMore(false)
    }, 1000)
  }

  return (
    <div className="space-y-4 p-1">
      {/* Header */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">系统日志</h1>
          <p className="text-sm text-muted-foreground">监控服务器活动和排查问题</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={isLive ? "default" : "outline"}
            size="sm"
            onClick={() => setIsLive(!isLive)}
          >
            {isLive ? (
              <>
                <Pause className="mr-2 h-4 w-4" />
                暂停
              </>
            ) : (
              <>
                <Play className="mr-2 h-4 w-4" />
                继续
              </>
            )}
          </Button>
          <Button variant="outline" size="sm">
            <Download className="mr-2 h-4 w-4" />
            导出
          </Button>
          <Button variant="outline" size="sm">
            <Trash2 className="mr-2 h-4 w-4" />
            清空
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid gap-3 sm:grid-cols-4">
        <Card className="border-border/50 bg-card/50">
          <CardContent className="flex items-center gap-3 p-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-muted">
              <Clock className="h-4 w-4 text-muted-foreground" />
            </div>
            <div>
              <p className="text-xl font-bold">{filteredLogs.length}</p>
              <p className="text-[10px] text-muted-foreground">筛选后 / 共{logs.length}</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-destructive/30 bg-destructive/5">
          <CardContent className="flex items-center gap-3 p-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-destructive/10">
              <AlertCircle className="h-4 w-4 text-destructive" />
            </div>
            <div>
              <p className="text-xl font-bold text-destructive">{errorCount}</p>
              <p className="text-[10px] text-muted-foreground">错误</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-yellow-500/30 bg-yellow-500/5">
          <CardContent className="flex items-center gap-3 p-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-yellow-500/10">
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
            </div>
            <div>
              <p className="text-xl font-bold text-yellow-500">{warnCount}</p>
              <p className="text-[10px] text-muted-foreground">警告</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-green-500/30 bg-green-500/5">
          <CardContent className="flex items-center gap-3 p-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-green-500/10">
              {isLive ? (
                <RefreshCw className="h-4 w-4 animate-spin text-green-500" />
              ) : (
                <Pause className="h-4 w-4 text-muted-foreground" />
              )}
            </div>
            <div>
              <p className="text-sm font-medium">{isLive ? "实时" : "已暂停"}</p>
              <p className="text-[10px] text-muted-foreground">流状态</p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜索日志内容..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
        
        {/* 时间范围选择 */}
        <Select value={timeRange} onValueChange={setTimeRange}>
          <SelectTrigger className="w-[140px]">
            <Calendar className="mr-2 h-4 w-4" />
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {timeRanges.map(range => (
              <SelectItem key={range.value} value={range.value}>
                {range.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Filter className="mr-2 h-4 w-4" />
              级别
              <ChevronDown className="ml-2 h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            {(["error", "warn", "info", "debug"] as LogLevel[]).map((level) => (
              <DropdownMenuCheckboxItem
                key={level}
                checked={selectedLevels.includes(level)}
                onCheckedChange={() => toggleLevel(level)}
              >
                <span className="flex items-center gap-2">
                  {getLevelIcon(level)}
                  {level.charAt(0).toUpperCase() + level.slice(1)}
                </span>
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Server className="mr-2 h-4 w-4" />
              来源
              <ChevronDown className="ml-2 h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            {(["server", "auth", "database", "api", "playback", "scanner"] as LogSource[]).map((source) => (
              <DropdownMenuCheckboxItem
                key={source}
                checked={selectedSources.includes(source)}
                onCheckedChange={() => toggleSource(source)}
              >
                <span className="flex items-center gap-2">
                  {getSourceIcon(source)}
                  {source.charAt(0).toUpperCase() + source.slice(1)}
                </span>
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)}>
        <TabsList>
          <TabsTrigger value="all">全部日志</TabsTrigger>
          <TabsTrigger value="errors" className="gap-2">
            错误
            {errorCount > 0 && (
              <Badge variant="destructive" className="h-5 px-1.5 text-[10px]">
                {errorCount}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="warnings" className="gap-2">
            警告
            {warnCount > 0 && (
              <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                {warnCount}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <div className="mt-4">
          <Card className="border-border/50 bg-card/30">
            {/* 虚拟化日志列表 */}
            <div
              ref={parentRef}
              className="h-[calc(100vh-420px)] min-h-[400px] overflow-auto"
              style={{ scrollbarWidth: "thin" }}
            >
              {filteredLogs.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                  <Info className="mb-4 h-12 w-12 text-muted-foreground/50" />
                  <p className="text-muted-foreground">没有匹配的日志</p>
                </div>
              ) : (
                <div
                  style={{
                    height: `${rowVirtualizer.getTotalSize()}px`,
                    width: "100%",
                    position: "relative",
                  }}
                >
                  {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                    const log = filteredLogs[virtualRow.index]
                    const isExpanded = expandedLogs.has(log.id)
                    
                    return (
                      <div
                        key={virtualRow.key}
                        style={{
                          position: "absolute",
                          top: 0,
                          left: 0,
                          width: "100%",
                          transform: `translateY(${virtualRow.start}px)`,
                        }}
                      >
                        <div
                          className={cn(
                            "flex flex-col border-b border-border/30 px-4 py-2 hover:bg-muted/30 cursor-pointer transition-colors",
                            log.level === "error" && "bg-destructive/5",
                            log.level === "warn" && "bg-yellow-500/5"
                          )}
                          onClick={() => {
                            if (log.details) {
                              setExpandedLogs(prev => {
                                const next = new Set(prev)
                                if (next.has(log.id)) next.delete(log.id)
                                else next.add(log.id)
                                return next
                              })
                            }
                          }}
                        >
                          <div className="flex items-center gap-3">
                            {getLevelIcon(log.level)}
                            <span className="w-[120px] shrink-0 text-xs text-muted-foreground font-mono">
                              {formatTimestamp(log.timestamp)}
                            </span>
                            <Badge variant="outline" className="shrink-0 text-[10px] px-1.5 py-0 gap-1">
                              {getSourceIcon(log.source)}
                              {log.source}
                            </Badge>
                            <span className="flex-1 truncate text-sm">{log.message}</span>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-6 w-6 shrink-0"
                              onClick={(e) => {
                                e.stopPropagation()
                                copyToClipboard(log)
                              }}
                            >
                              {copiedId === log.id ? (
                                <Check className="h-3 w-3 text-green-500" />
                              ) : (
                                <Copy className="h-3 w-3" />
                              )}
                            </Button>
                          </div>
                          
                          {/* 展开的详情 */}
                          {isExpanded && log.details && (
                            <div className="mt-2 ml-7 p-2 rounded bg-muted/50 text-xs font-mono text-muted-foreground">
                              {log.details}
                            </div>
                          )}
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
            
            {/* 底部状态栏 */}
            <div className="flex items-center justify-between border-t border-border/30 px-4 py-2 text-xs text-muted-foreground">
              <div className="flex items-center gap-4">
                <span>显示 {filteredLogs.length} 条日志</span>
                {isLive && (
                  <span className="flex items-center gap-1 text-green-500">
                    <span className="h-1.5 w-1.5 rounded-full bg-green-500 animate-pulse" />
                    实时更新中
                  </span>
                )}
              </div>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 text-xs"
                onClick={loadMoreHistory}
                disabled={isLoadingMore}
              >
                {isLoadingMore ? (
                  <>
                    <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                    加载中...
                  </>
                ) : (
                  "加载更多历史"
                )}
              </Button>
            </div>
          </Card>
        </div>
      </Tabs>
    </div>
  )
}
