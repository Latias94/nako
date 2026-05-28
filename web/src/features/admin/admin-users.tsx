"use client"

import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { 
  Users, 
  Plus, 
  Search, 
  MoreHorizontal,
  Shield,
  ShieldCheck,
  UserCog,
  Key,
  Trash2,
  Clock,
  Film,
  CheckCircle2,
  XCircle,
  Eye,
  Pencil,
  Monitor,
  Smartphone,
  Tablet,
  Globe,
  LogOut,
  History,
  ChevronLeft,
  ChevronRight,
  Activity,
  Play,
  Download,
  Settings,
  AlertTriangle,
  Ban
} from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import {
  ADMIN_USERS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminUserReadModel,
} from "@/src/api/admin/read-models-data-source"

// 模拟用户数据
const users = [
  {
    id: "1",
    name: "管理员",
    username: "admin",
    email: "admin@localhost",
    role: "admin",
    avatar: null,
    status: "online",
    lastActive: "在线",
    createdAt: "2024-01-01",
    libraryAccess: ["all"],
    settings: {
      canDownload: true,
      canDelete: true,
      canManageUsers: true,
      maxBitrate: null,
      transcodePolicy: "auto",
      maxStreams: null,
      remoteAccess: true
    },
    stats: {
      totalPlays: 1234,
      totalWatchTime: "156 小时",
      lastLogin: "2024-03-15 14:30"
    }
  },
  {
    id: "2",
    name: "小明",
    username: "xiaoming",
    email: "xiaoming@example.com",
    role: "user",
    avatar: null,
    status: "online",
    lastActive: "3 分钟前",
    createdAt: "2024-02-15",
    libraryAccess: ["1", "2", "3"],
    settings: {
      canDownload: true,
      canDelete: false,
      canManageUsers: false,
      maxBitrate: 20000,
      transcodePolicy: "auto",
      maxStreams: 3,
      remoteAccess: true
    },
    stats: {
      totalPlays: 567,
      totalWatchTime: "89 小时",
      lastLogin: "2024-03-15 12:20"
    }
  },
  {
    id: "3",
    name: "小红",
    username: "xiaohong",
    email: "xiaohong@example.com",
    role: "user",
    avatar: null,
    status: "offline",
    lastActive: "昨天 22:15",
    createdAt: "2024-02-20",
    libraryAccess: ["1", "3"],
    settings: {
      canDownload: false,
      canDelete: false,
      canManageUsers: false,
      maxBitrate: 8000,
      transcodePolicy: "always",
      maxStreams: 2,
      remoteAccess: true
    },
    stats: {
      totalPlays: 234,
      totalWatchTime: "45 小时",
      lastLogin: "2024-03-14 22:15"
    }
  },
  {
    id: "4",
    name: "访客",
    username: "guest",
    email: null,
    role: "guest",
    avatar: null,
    status: "offline",
    lastActive: "从未登录",
    createdAt: "2024-03-01",
    libraryAccess: ["1"],
    settings: {
      canDownload: false,
      canDelete: false,
      canManageUsers: false,
      maxBitrate: 4000,
      transcodePolicy: "always",
      maxStreams: 1,
      remoteAccess: false
    },
    stats: {
      totalPlays: 0,
      totalWatchTime: "0 小时",
      lastLogin: "从未"
    }
  },
  {
    id: "5",
    name: "家人",
    username: "family",
    email: "family@example.com",
    role: "user",
    avatar: null,
    status: "disabled",
    lastActive: "账户已禁用",
    createdAt: "2024-02-10",
    libraryAccess: ["1", "2"],
    settings: {
      canDownload: true,
      canDelete: false,
      canManageUsers: false,
      maxBitrate: 10000,
      transcodePolicy: "auto",
      maxStreams: 2,
      remoteAccess: false
    },
    stats: {
      totalPlays: 89,
      totalWatchTime: "12 小时",
      lastLogin: "2024-03-01 18:30"
    }
  },
]

// 活跃会话
const activeSessions = [
  { 
    id: "s1", 
    userId: "1", 
    userName: "管理员",
    device: "Chrome - Windows", 
    deviceType: "desktop",
    ip: "192.168.1.100", 
    location: "本地网络",
    lastActivity: "正在播放: 沙丘2",
    startTime: "2024-03-15 14:30",
    current: true
  },
  { 
    id: "s2", 
    userId: "2", 
    userName: "小明",
    device: "Nako iOS App", 
    deviceType: "mobile",
    ip: "192.168.1.105", 
    location: "本地网络",
    lastActivity: "浏览媒体库",
    startTime: "2024-03-15 12:20",
    current: false
  },
  { 
    id: "s3", 
    userId: "2", 
    userName: "小明",
    device: "Samsung TV", 
    deviceType: "tv",
    ip: "192.168.1.110", 
    location: "本地网络",
    lastActivity: "正在播放: 奥本海默",
    startTime: "2024-03-15 14:00",
    current: false
  },
]

// 登录历史
const generateLoginHistory = (count: number = 50) => {
  const history = []
  const users = ["admin", "xiaoming", "xiaohong", "guest", "family"]
  const devices = ["Chrome - Windows", "Firefox - macOS", "Nako iOS App", "Nako Android App", "Samsung TV", "Apple TV"]
  const locations = ["本地网络", "北京", "上海", "深圳", "广州"]
  const now = new Date()
  
  for (let i = 0; i < count; i++) {
    const success = Math.random() > 0.1
    history.push({
      id: `login-${i}`,
      username: users[Math.floor(Math.random() * users.length)],
      device: devices[Math.floor(Math.random() * devices.length)],
      ip: `192.168.${Math.floor(Math.random() * 5)}.${Math.floor(Math.random() * 255)}`,
      location: locations[Math.floor(Math.random() * locations.length)],
      timestamp: new Date(now.getTime() - i * 2 * 60 * 60 * 1000).toISOString(),
      success,
      failReason: success ? null : ["密码错误", "账户已禁用", "验证码错误"][Math.floor(Math.random() * 3)]
    })
  }
  return history
}

// 用户活动日志
const generateActivityLog = (count: number = 30) => {
  const activities = []
  const actions = [
    { type: "play", text: "开始播放", icon: Play },
    { type: "download", text: "下载文件", icon: Download },
    { type: "settings", text: "修改设置", icon: Settings },
    { type: "login", text: "登录", icon: LogOut },
  ]
  const items = ["沙丘2", "奥本海默", "星际穿越", "盗梦空间", "怪物猎人"]
  const now = new Date()
  
  for (let i = 0; i < count; i++) {
    const action = actions[Math.floor(Math.random() * actions.length)]
    activities.push({
      id: `activity-${i}`,
      action: action.type,
      actionText: action.text,
      icon: action.icon,
      item: action.type === "play" || action.type === "download" ? items[Math.floor(Math.random() * items.length)] : null,
      timestamp: new Date(now.getTime() - i * 30 * 60 * 1000).toISOString(),
    })
  }
  return activities
}

const allLibraries = [
  { id: "1", name: "电影", type: "movie" },
  { id: "2", name: "剧集", type: "tv" },
  { id: "3", name: "动画", type: "anime" },
  { id: "4", name: "纪录片", type: "documentary" },
  { id: "5", name: "个人收藏", type: "personal" },
]

export function AdminUsers() {
  const { data: usersData = ADMIN_USERS_READ_MODEL_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "users"],
    queryFn: () => createAdminReadModelsDataSource().loadUsers(),
    staleTime: 30 * 1000,
    retry: 0,
  })
  const users = usersData.users
  const activeSessions = usersData.activeSessions
  const allLibraries = usersData.libraries
  const [searchQuery, setSearchQuery] = useState("")
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false)
  const [selectedUser, setSelectedUser] = useState<AdminUserReadModel | null>(null)
  const [activeTab, setActiveTab] = useState("users")
  
  // 登录历史分页
  const [loginHistory] = useState(() => generateLoginHistory(50))
  const [loginHistoryPage, setLoginHistoryPage] = useState(1)
  const loginHistoryPerPage = 10

  const filteredUsers = users.filter(user => 
    user.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    user.username.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "online":
        return (
          <span className="flex items-center gap-1.5 text-xs">
            <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
            <span className="text-green-500">在线</span>
          </span>
        )
      case "offline":
        return (
          <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span className="h-2 w-2 rounded-full bg-muted-foreground/50" />
            离线
          </span>
        )
      case "disabled":
        return (
          <span className="flex items-center gap-1.5 text-xs text-destructive">
            <Ban className="h-3 w-3" />
            已禁用
          </span>
        )
      default:
        return null
    }
  }

  const getRoleBadge = (role: string) => {
    switch (role) {
      case "admin":
        return (
          <Badge variant="secondary" className="bg-warning/10 text-warning gap-1">
            <ShieldCheck className="h-3 w-3" />
            管理员
          </Badge>
        )
      case "user":
        return (
          <Badge variant="secondary" className="gap-1">
            <Shield className="h-3 w-3" />
            用户
          </Badge>
        )
      case "guest":
        return (
          <Badge variant="outline" className="gap-1">
            <Eye className="h-3 w-3" />
            访客
          </Badge>
        )
      default:
        return <Badge variant="outline">{role}</Badge>
    }
  }

  const getDeviceIcon = (type: string) => {
    switch (type) {
      case "desktop": return <Monitor className="h-4 w-4" />
      case "mobile": return <Smartphone className="h-4 w-4" />
      case "tablet": return <Tablet className="h-4 w-4" />
      case "tv": return <Monitor className="h-4 w-4" />
      default: return <Globe className="h-4 w-4" />
    }
  }

  const getLibraryAccessText = (access: string[]) => {
    if (access.includes("all")) return "所有媒体库"
    const count = access.length
    if (count === 0) return "无访问权限"
    return `${count} 个媒体库`
  }

  // 计算统计
  const stats = {
    total: users.length,
    admins: users.filter(u => u.role === "admin").length,
    online: users.filter(u => u.status === "online").length,
    disabled: users.filter(u => u.status === "disabled").length,
  }

  return (
    <div className="space-y-6 p-1">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">用户管理</h1>
          <p className="text-sm text-muted-foreground">
            管理用户账户、会话和访问权限
            <span className="ml-2 text-xs">
              {usersData.source === "live" ? "Live Admin API" : "Fixture fallback"}
              {usersData.error ? ` · ${usersData.error}` : ""}
            </span>
          </p>
        </div>
        <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
          <DialogTrigger asChild>
            <Button className="gap-2">
              <Plus className="h-4 w-4" />
              添加用户
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-lg">
            <DialogHeader>
              <DialogTitle>添加用户</DialogTitle>
              <DialogDescription>
                创建新用户账户并配置访问权限
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>用户名</Label>
                  <Input placeholder="username" />
                </div>
                <div className="space-y-2">
                  <Label>显示名称</Label>
                  <Input placeholder="显示名称" />
                </div>
              </div>
              <div className="space-y-2">
                <Label>邮箱（可选）</Label>
                <Input type="email" placeholder="user@example.com" />
              </div>
              <div className="space-y-2">
                <Label>密码</Label>
                <Input type="password" placeholder="设置密码" />
              </div>
              <div className="space-y-2">
                <Label>角色</Label>
                <Select defaultValue="user">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="admin">管理员</SelectItem>
                    <SelectItem value="user">普通用户</SelectItem>
                    <SelectItem value="guest">访客</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>媒体库访问</Label>
                <div className="space-y-2 mt-2 rounded-lg border border-border/50 bg-secondary/20 p-3">
                  <div className="flex items-center space-x-3">
                    <Checkbox id="all-libraries" />
                    <label htmlFor="all-libraries" className="text-sm font-medium cursor-pointer">
                      所有媒体库
                    </label>
                  </div>
                  <div className="pl-6 space-y-2 border-l-2 border-border/30 ml-2">
                    {allLibraries.map((lib) => (
                      <div key={lib.id} className="flex items-center space-x-3">
                        <Checkbox id={`lib-${lib.id}`} />
                        <label htmlFor={`lib-${lib.id}`} className="text-sm cursor-pointer text-muted-foreground">
                          {lib.name}
                        </label>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
                取消
              </Button>
              <Button onClick={() => setIsAddDialogOpen(false)}>
                创建用户
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Card className="border-border/50">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-primary/10">
                <Users className="h-5 w-5 text-primary" />
              </div>
              <div>
                <p className="text-2xl font-bold">{stats.total}</p>
                <p className="text-xs text-muted-foreground">总用户数</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-warning/10">
                <ShieldCheck className="h-5 w-5 text-warning" />
              </div>
              <div>
                <p className="text-2xl font-bold">{stats.admins}</p>
                <p className="text-xs text-muted-foreground">管理员</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-green-500/10">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{stats.online}</p>
                <p className="text-xs text-muted-foreground">当前在线</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-destructive/10">
                <Ban className="h-5 w-5 text-destructive" />
              </div>
              <div>
                <p className="text-2xl font-bold">{stats.disabled}</p>
                <p className="text-xs text-muted-foreground">已禁用</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 主要内容区 Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="users" className="gap-2">
            <Users className="h-4 w-4" />
            用户列表
          </TabsTrigger>
          <TabsTrigger value="sessions" className="gap-2">
            <Monitor className="h-4 w-4" />
            活跃会话
            <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-[10px]">
              {activeSessions.length}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="history" className="gap-2">
            <History className="h-4 w-4" />
            登录历史
          </TabsTrigger>
        </TabsList>

        {/* 用户列表 Tab */}
        <TabsContent value="users" className="mt-4 space-y-4">
          {/* 搜索 */}
          <div className="flex items-center gap-4">
            <div className="relative flex-1 max-w-sm">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input 
                placeholder="搜索用户..." 
                className="pl-9"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
            <Select defaultValue="all">
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="状态筛选" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="online">在线</SelectItem>
                <SelectItem value="offline">离线</SelectItem>
                <SelectItem value="disabled">已禁用</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* 用户列表 */}
          <Card className="border-border/50">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>用户</TableHead>
                  <TableHead>角色</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>媒体库</TableHead>
                  <TableHead>播放统计</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredUsers.map((user) => (
                  <TableRow key={user.id} className={cn(user.status === "disabled" && "opacity-60")}>
                    <TableCell>
                      <div className="flex items-center gap-3">
                        <Avatar className="h-9 w-9">
                          <AvatarFallback className="bg-secondary text-secondary-foreground">
                            {user.name.charAt(0)}
                          </AvatarFallback>
                        </Avatar>
                        <div>
                          <p className="font-medium">{user.name}</p>
                          <p className="text-xs text-muted-foreground">@{user.username}</p>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>{getRoleBadge(user.role)}</TableCell>
                    <TableCell>{getStatusBadge(user.status)}</TableCell>
                    <TableCell>
                      <span className="text-sm">{getLibraryAccessText(user.libraryAccess)}</span>
                    </TableCell>
                    <TableCell>
                      <div className="text-sm">
                        <p>{user.stats.totalPlays} 次播放</p>
                        <p className="text-xs text-muted-foreground">{user.stats.totalWatchTime}</p>
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="sm">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => setSelectedUser(user)}>
                            <UserCog className="h-4 w-4 mr-2" />
                            编辑用户
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <Key className="h-4 w-4 mr-2" />
                            重置密码
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <Activity className="h-4 w-4 mr-2" />
                            查看活动
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          {user.status !== "disabled" ? (
                            <DropdownMenuItem className="text-destructive">
                              <Ban className="h-4 w-4 mr-2" />
                              禁用账户
                            </DropdownMenuItem>
                          ) : (
                            <DropdownMenuItem className="text-green-500">
                              <CheckCircle2 className="h-4 w-4 mr-2" />
                              启用账户
                            </DropdownMenuItem>
                          )}
                          {user.role !== "admin" && (
                            <DropdownMenuItem className="text-destructive">
                              <Trash2 className="h-4 w-4 mr-2" />
                              删除用户
                            </DropdownMenuItem>
                          )}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Card>
        </TabsContent>

        {/* 活跃会话 Tab */}
        <TabsContent value="sessions" className="mt-4 space-y-4">
          <div className="flex items-center justify-between">
            <p className="text-sm text-muted-foreground">
              当前有 {activeSessions.length} 个活跃会话
            </p>
            <Button variant="outline" size="sm" className="text-destructive">
              <LogOut className="h-4 w-4 mr-2" />
              终止所有其他会话
            </Button>
          </div>

          <div className="grid gap-3">
            {activeSessions.map((session) => (
              <Card key={session.id} className={cn(
                "border-border/50",
                session.current && "border-primary/50 bg-primary/5"
              )}>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <div className={cn(
                        "p-2.5 rounded-lg",
                        session.current ? "bg-primary/10" : "bg-muted"
                      )}>
                        {getDeviceIcon(session.deviceType)}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <p className="font-medium">{session.userName}</p>
                          {session.current && (
                            <Badge variant="secondary" className="text-[10px]">当前会话</Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{session.device}</p>
                        <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <Globe className="h-3 w-3" />
                            {session.ip}
                          </span>
                          <span>{session.location}</span>
                          <span className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {session.startTime.split(" ")[1]}
                          </span>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      {session.lastActivity.startsWith("正在播放") && (
                        <Badge variant="secondary" className="bg-green-500/10 text-green-500 gap-1">
                          <Play className="h-3 w-3" />
                          {session.lastActivity.replace("正在播放: ", "")}
                        </Badge>
                      )}
                      {!session.current && (
                        <Button variant="ghost" size="sm" className="text-destructive">
                          <LogOut className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* 登录历史 Tab */}
        <TabsContent value="history" className="mt-4 space-y-4">
          <div className="flex items-center justify-between">
            <Select defaultValue="all">
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="筛选" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部</SelectItem>
                <SelectItem value="success">成功</SelectItem>
                <SelectItem value="failed">失败</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Card className="border-border/50">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>用户</TableHead>
                  <TableHead>设备</TableHead>
                  <TableHead>IP 地址</TableHead>
                  <TableHead>位置</TableHead>
                  <TableHead>时间</TableHead>
                  <TableHead>状态</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loginHistory
                  .slice((loginHistoryPage - 1) * loginHistoryPerPage, loginHistoryPage * loginHistoryPerPage)
                  .map((log) => (
                    <TableRow key={log.id}>
                      <TableCell className="font-medium">@{log.username}</TableCell>
                      <TableCell className="text-sm">{log.device}</TableCell>
                      <TableCell className="font-mono text-xs">{log.ip}</TableCell>
                      <TableCell className="text-sm">{log.location}</TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {new Date(log.timestamp).toLocaleString("zh-CN", {
                          month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit"
                        })}
                      </TableCell>
                      <TableCell>
                        {log.success ? (
                          <Badge variant="secondary" className="bg-green-500/10 text-green-500 gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            成功
                          </Badge>
                        ) : (
                          <Badge variant="secondary" className="bg-destructive/10 text-destructive gap-1">
                            <XCircle className="h-3 w-3" />
                            {log.failReason}
                          </Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
              </TableBody>
            </Table>
            
            {/* 分页 */}
            <div className="flex items-center justify-between border-t border-border/30 px-4 py-3">
              <p className="text-xs text-muted-foreground">
                共 {loginHistory.length} 条记录
              </p>
              <div className="flex items-center gap-1">
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7"
                  disabled={loginHistoryPage === 1}
                  onClick={() => setLoginHistoryPage(1)}
                >
                  <ChevronLeft className="h-3 w-3" />
                  <ChevronLeft className="h-3 w-3 -ml-2" />
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7"
                  disabled={loginHistoryPage === 1}
                  onClick={() => setLoginHistoryPage(p => Math.max(1, p - 1))}
                >
                  <ChevronLeft className="h-3 w-3" />
                </Button>
                <span className="px-2 text-xs text-muted-foreground min-w-[60px] text-center">
                  {loginHistoryPage} / {Math.ceil(loginHistory.length / loginHistoryPerPage)}
                </span>
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7"
                  disabled={loginHistoryPage === Math.ceil(loginHistory.length / loginHistoryPerPage)}
                  onClick={() => setLoginHistoryPage(p => p + 1)}
                >
                  <ChevronRight className="h-3 w-3" />
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7"
                  disabled={loginHistoryPage === Math.ceil(loginHistory.length / loginHistoryPerPage)}
                  onClick={() => setLoginHistoryPage(Math.ceil(loginHistory.length / loginHistoryPerPage))}
                >
                  <ChevronRight className="h-3 w-3" />
                  <ChevronRight className="h-3 w-3 -ml-2" />
                </Button>
              </div>
            </div>
          </Card>
        </TabsContent>
      </Tabs>

      {/* 用户详情/编辑对话框 */}
      <Dialog open={!!selectedUser} onOpenChange={() => setSelectedUser(null)}>
        <DialogContent className="sm:max-w-2xl max-h-[85vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>编辑用户</DialogTitle>
            <DialogDescription>
              修改用户信息和权限设置
            </DialogDescription>
          </DialogHeader>
          
          {selectedUser && (
            <Tabs defaultValue="info" className="flex-1 overflow-hidden flex flex-col">
              <TabsList className="w-full justify-start">
                <TabsTrigger value="info">基本信息</TabsTrigger>
                <TabsTrigger value="permissions">权限设置</TabsTrigger>
                <TabsTrigger value="playback">播放限制</TabsTrigger>
                <TabsTrigger value="activity">活动记录</TabsTrigger>
              </TabsList>
              
              <ScrollArea className="flex-1 mt-4">
                <TabsContent value="info" className="mt-0 space-y-4 pr-4">
                  {/* 用户头像和基本信息 */}
                  <div className="flex items-center gap-4">
                    <Avatar className="h-16 w-16">
                      <AvatarFallback className="bg-secondary text-secondary-foreground text-xl">
                        {selectedUser.name.charAt(0)}
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <p className="font-medium text-lg">{selectedUser.name}</p>
                        {getRoleBadge(selectedUser.role)}
                        {getStatusBadge(selectedUser.status)}
                      </div>
                      <p className="text-muted-foreground">@{selectedUser.username}</p>
                    </div>
                  </div>

                  <Separator />

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>显示名称</Label>
                      <Input defaultValue={selectedUser.name} />
                    </div>
                    <div className="space-y-2">
                      <Label>用户名</Label>
                      <Input defaultValue={selectedUser.username} />
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <Label>邮箱</Label>
                    <Input type="email" defaultValue={selectedUser.email || ""} placeholder="user@example.com" />
                  </div>

                  <div className="space-y-2">
                    <Label>角色</Label>
                    <Select defaultValue={selectedUser.role}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="admin">管理员</SelectItem>
                        <SelectItem value="user">普通用户</SelectItem>
                        <SelectItem value="guest">访客</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                    <h4 className="text-sm font-medium mb-3">账户统计</h4>
                    <div className="grid grid-cols-3 gap-4 text-center">
                      <div>
                        <p className="text-2xl font-bold">{selectedUser.stats.totalPlays}</p>
                        <p className="text-xs text-muted-foreground">总播放次数</p>
                      </div>
                      <div>
                        <p className="text-2xl font-bold">{selectedUser.stats.totalWatchTime}</p>
                        <p className="text-xs text-muted-foreground">总观看时长</p>
                      </div>
                      <div>
                        <p className="text-sm font-medium">{selectedUser.stats.lastLogin}</p>
                        <p className="text-xs text-muted-foreground">最后登录</p>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="permissions" className="mt-0 space-y-4 pr-4">
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>允许下载</Label>
                        <p className="text-xs text-muted-foreground">用户可以下载媒体文件</p>
                      </div>
                      <Switch defaultChecked={selectedUser.settings.canDownload} />
                    </div>
                    
                    <Separator />
                    
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>允许删除</Label>
                        <p className="text-xs text-muted-foreground">用户可以删除媒体项目</p>
                      </div>
                      <Switch defaultChecked={selectedUser.settings.canDelete} />
                    </div>
                    
                    <Separator />
                    
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>管理用户</Label>
                        <p className="text-xs text-muted-foreground">用户可以管理其他用户账户</p>
                      </div>
                      <Switch defaultChecked={selectedUser.settings.canManageUsers} />
                    </div>
                    
                    <Separator />
                    
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>远程访问</Label>
                        <p className="text-xs text-muted-foreground">允许从外部网络访问</p>
                      </div>
                      <Switch defaultChecked={selectedUser.settings.remoteAccess} />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label>媒体库访问</Label>
                    <div className="space-y-2 rounded-lg border border-border/50 bg-secondary/20 p-3 max-h-[200px] overflow-y-auto scrollbar-none">
                      <div className="flex items-center space-x-3">
                        <Checkbox 
                          id="edit-all-libraries" 
                          checked={selectedUser.libraryAccess.includes("all")}
                        />
                        <label htmlFor="edit-all-libraries" className="text-sm font-medium cursor-pointer">
                          所有媒体库
                        </label>
                      </div>
                      <div className="pl-6 space-y-2 border-l-2 border-border/30 ml-2">
                        {allLibraries.map((lib) => (
                          <div key={lib.id} className="flex items-center space-x-3">
                            <Checkbox 
                              id={`edit-lib-${lib.id}`}
                              checked={selectedUser.libraryAccess.includes("all") || selectedUser.libraryAccess.includes(lib.id)}
                            />
                            <label htmlFor={`edit-lib-${lib.id}`} className="text-sm cursor-pointer text-muted-foreground">
                              {lib.name}
                            </label>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="playback" className="mt-0 space-y-4 pr-4">
                  <div className="space-y-2">
                    <Label>最大码率限制</Label>
                    <Select defaultValue={selectedUser.settings.maxBitrate?.toString() || "unlimited"}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="unlimited">无限制</SelectItem>
                        <SelectItem value="40000">40 Mbps（4K）</SelectItem>
                        <SelectItem value="20000">20 Mbps（1080p 高质量）</SelectItem>
                        <SelectItem value="10000">10 Mbps（1080p）</SelectItem>
                        <SelectItem value="8000">8 Mbps（720p）</SelectItem>
                        <SelectItem value="4000">4 Mbps（480p）</SelectItem>
                        <SelectItem value="2000">2 Mbps（低带宽）</SelectItem>
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">限制用户的最大播放码率，超出时将自动转码</p>
                  </div>

                  <div className="space-y-2">
                    <Label>转码策略</Label>
                    <Select defaultValue={selectedUser.settings.transcodePolicy}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="auto">自动（推荐）</SelectItem>
                        <SelectItem value="always">总是转码</SelectItem>
                        <SelectItem value="never">从不转码（仅 Direct Play）</SelectItem>
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">控制播放时是否进行转码</p>
                  </div>

                  <div className="space-y-2">
                    <Label>最大同时播放流数</Label>
                    <Select defaultValue={selectedUser.settings.maxStreams?.toString() || "unlimited"}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="unlimited">无限制</SelectItem>
                        <SelectItem value="1">1 个</SelectItem>
                        <SelectItem value="2">2 个</SelectItem>
                        <SelectItem value="3">3 个</SelectItem>
                        <SelectItem value="5">5 个</SelectItem>
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">限制用户同时播放的设备数量</p>
                  </div>
                </TabsContent>

                <TabsContent value="activity" className="mt-0 pr-4">
                  <div className="space-y-3">
                    {generateActivityLog(10).map((activity) => {
                      const Icon = activity.icon
                      return (
                        <div key={activity.id} className="flex items-center gap-3 py-2 border-b border-border/30 last:border-0">
                          <div className="p-1.5 rounded-lg bg-muted">
                            <Icon className="h-3.5 w-3.5 text-muted-foreground" />
                          </div>
                          <div className="flex-1">
                            <p className="text-sm">
                              {activity.actionText}
                              {activity.item && (
                                <span className="font-medium"> {activity.item}</span>
                              )}
                            </p>
                            <p className="text-xs text-muted-foreground">
                              {new Date(activity.timestamp).toLocaleString("zh-CN", {
                                month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit"
                              })}
                            </p>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </TabsContent>
              </ScrollArea>
            </Tabs>
          )}
          
          <DialogFooter className="mt-4">
            <Button variant="outline" onClick={() => setSelectedUser(null)}>
              取消
            </Button>
            <Button onClick={() => setSelectedUser(null)}>
              保存更改
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
