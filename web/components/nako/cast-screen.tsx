"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState } from "react"
import {
  ArrowLeft,
  Cast,
  Tv2,
  Smartphone,
  Wifi,
  RefreshCw,
  Check,
  X,
  Airplay,
  MonitorSpeaker,
  Settings2,
  Info,
  Play,
  Pause,
  Volume2,
  VolumeX,
  SkipForward,
  ChevronRight
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Slider } from "@/components/ui/slider"
import { cn } from "@/lib/utils"

interface CastScreenProps {
  onBack?: () => void
  currentMedia?: {
    title: string
    poster: string
    progress: number
    duration: string
  }
}

// 模拟设备数据
const availableDevices = [
  {
    id: "1",
    name: "客厅电视",
    type: "chromecast" as const,
    model: "Chromecast with Google TV",
    status: "available",
    icon: Tv2
  },
  {
    id: "2",
    name: "卧室 Apple TV",
    type: "airplay" as const,
    model: "Apple TV 4K",
    status: "available",
    icon: Airplay
  },
  {
    id: "3",
    name: "书房音响",
    type: "dlna" as const,
    model: "Sonos Beam",
    status: "available",
    icon: MonitorSpeaker
  },
  {
    id: "4",
    name: "小米电视",
    type: "dlna" as const,
    model: "Mi TV 4S 55\"",
    status: "busy",
    icon: Tv2
  },
  {
    id: "5",
    name: "iPad Pro",
    type: "airplay" as const,
    model: "iPad Pro 12.9\"",
    status: "available",
    icon: Smartphone
  },
]

const castHistory = [
  { deviceName: "客厅电视", mediaTitle: "沙丘2", time: "今天 20:30", duration: "2小时15分钟" },
  { deviceName: "卧室 Apple TV", mediaTitle: "真探 S01E05", time: "昨天 22:00", duration: "58分钟" },
  { deviceName: "客厅电视", mediaTitle: "星际穿越", time: "3天前", duration: "2小时49分钟" },
]

export function CastScreen({ onBack, currentMedia }: CastScreenProps) {
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null)
  const [isCasting, setIsCasting] = useState(false)
  const [isScanning, setIsScanning] = useState(false)
  const [volume, setVolume] = useState([80])
  const [isMuted, setIsMuted] = useState(false)
  const [isPlaying, setIsPlaying] = useState(true)

  // 设置状态
  const [enableDLNA, setEnableDLNA] = useState(true)
  const [enableAirPlay, setEnableAirPlay] = useState(true)
  const [enableChromecast, setEnableChromecast] = useState(true)
  const [autoConnect, setAutoConnect] = useState(false)
  const [showOnLockScreen, setShowOnLockScreen] = useState(true)

  const handleScan = () => {
    setIsScanning(true)
    setTimeout(() => setIsScanning(false), 2000)
  }

  const handleConnect = (deviceId: string) => {
    setSelectedDevice(deviceId)
    setIsCasting(true)
  }

  const handleDisconnect = () => {
    setSelectedDevice(null)
    setIsCasting(false)
  }

  const connectedDevice = availableDevices.find(d => d.id === selectedDevice)

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b border-border/50 bg-background/95 backdrop-blur-sm">
        <div className="flex h-14 items-center gap-4 px-4 lg:px-6">
          {onBack && (
            <Button variant="ghost" size="icon" onClick={onBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
          )}
          <div className="flex-1">
            <h1 className="text-lg font-semibold">投屏</h1>
            <p className="text-xs text-muted-foreground">将媒体投射到其他设备</p>
          </div>
          <Button variant="outline" size="sm" onClick={handleScan} disabled={isScanning}>
            <RefreshCw className={cn("mr-2 h-4 w-4", isScanning && "animate-spin")} />
            {isScanning ? "扫描中..." : "扫描设备"}
          </Button>
        </div>
      </header>

      <div className="p-4 lg:p-6">
        {/* 当前投屏状态 */}
        {isCasting && connectedDevice && (
          <Card className="mb-6 border-primary/50 bg-primary/5">
            <CardContent className="p-4">
              <div className="flex items-start gap-4">
                {currentMedia && (
                  <img
                    src={resolveArtwork(currentMedia.poster)}
                    alt={currentMedia.title}
                    className="h-24 w-16 rounded-lg object-cover"
                  />
                )}
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <Cast className="h-4 w-4 text-primary" />
                    <span className="text-sm text-primary">正在投屏到</span>
                  </div>
                  <h3 className="mt-1 font-semibold">{connectedDevice.name}</h3>
                  {currentMedia && (
                    <p className="text-sm text-muted-foreground">{currentMedia.title}</p>
                  )}

                  {/* 播放控制 */}
                  <div className="mt-4 space-y-3">
                    <div className="flex items-center gap-2">
                      <Button
                        variant="outline"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => setIsPlaying(!isPlaying)}
                      >
                        {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                      </Button>
                      <Button variant="outline" size="icon" className="h-8 w-8">
                        <SkipForward className="h-4 w-4" />
                      </Button>
                      <div className="flex flex-1 items-center gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => setIsMuted(!isMuted)}
                        >
                          {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
                        </Button>
                        <Slider
                          value={isMuted ? [0] : volume}
                          onValueChange={setVolume}
                          max={100}
                          className="w-24"
                        />
                      </div>
                      <Button variant="destructive" size="sm" onClick={handleDisconnect}>
                        断开连接
                      </Button>
                    </div>

                    {/* 进度条 */}
                    {currentMedia && (
                      <div className="space-y-1">
                        <Slider value={[currentMedia.progress]} max={100} className="w-full" />
                        <div className="flex justify-between text-xs text-muted-foreground">
                          <span>1:23:45</span>
                          <span>{currentMedia.duration}</span>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        <Tabs defaultValue="devices" className="space-y-6">
          <TabsList>
            <TabsTrigger value="devices" className="gap-2">
              <Wifi className="h-4 w-4" />
              可用设备
            </TabsTrigger>
            <TabsTrigger value="history" className="gap-2">
              <Cast className="h-4 w-4" />
              投屏历史
            </TabsTrigger>
            <TabsTrigger value="settings" className="gap-2">
              <Settings2 className="h-4 w-4" />
              设置
            </TabsTrigger>
          </TabsList>

          {/* 可用设备 */}
          <TabsContent value="devices">
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {availableDevices.map((device) => {
                const Icon = device.icon
                const isConnected = selectedDevice === device.id
                const isBusy = device.status === "busy"

                return (
                  <Card
                    key={device.id}
                    className={cn(
                      "cursor-pointer transition-all hover:border-primary/50",
                      isConnected && "border-primary bg-primary/5",
                      isBusy && "opacity-50"
                    )}
                    onClick={() => !isBusy && !isConnected && handleConnect(device.id)}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-3">
                          <div className={cn(
                            "flex h-12 w-12 items-center justify-center rounded-lg",
                            isConnected ? "bg-primary/20" : "bg-muted"
                          )}>
                            <Icon className={cn(
                              "h-6 w-6",
                              isConnected ? "text-primary" : "text-muted-foreground"
                            )} />
                          </div>
                          <div>
                            <h3 className="font-medium">{device.name}</h3>
                            <p className="text-xs text-muted-foreground">{device.model}</p>
                          </div>
                        </div>
                        {isConnected ? (
                          <Badge variant="default">已连接</Badge>
                        ) : isBusy ? (
                          <Badge variant="secondary">使用中</Badge>
                        ) : (
                          <Badge variant="outline">可用</Badge>
                        )}
                      </div>

                      <div className="mt-3 flex items-center gap-2">
                        <Badge variant="secondary" className="text-[10px]">
                          {device.type === "chromecast" && "Chromecast"}
                          {device.type === "airplay" && "AirPlay"}
                          {device.type === "dlna" && "DLNA"}
                        </Badge>
                      </div>
                    </CardContent>
                  </Card>
                )
              })}
            </div>

            {availableDevices.length === 0 && (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <Wifi className="mb-4 h-12 w-12 text-muted-foreground/50" />
                <h3 className="font-medium">未发现设备</h3>
                <p className="mt-1 text-sm text-muted-foreground">
                  确保你的设备与服务器在同一网络中
                </p>
                <Button variant="outline" className="mt-4" onClick={handleScan}>
                  <RefreshCw className="mr-2 h-4 w-4" />
                  重新扫描
                </Button>
              </div>
            )}
          </TabsContent>

          {/* 投屏历史 */}
          <TabsContent value="history">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">最近投屏</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {castHistory.map((item, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between rounded-lg border border-border/50 p-4"
                    >
                      <div className="flex items-center gap-4">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                          <Tv2 className="h-5 w-5 text-muted-foreground" />
                        </div>
                        <div>
                          <p className="font-medium">{item.mediaTitle}</p>
                          <p className="text-sm text-muted-foreground">
                            {item.deviceName} · {item.time}
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="text-sm text-muted-foreground">{item.duration}</p>
                        <Button variant="ghost" size="sm" className="mt-1 h-7 text-xs">
                          再次投屏
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* 投屏设置 */}
          <TabsContent value="settings" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">投屏协议</CardTitle>
                <CardDescription>选择启用的投屏协议</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                      <MonitorSpeaker className="h-5 w-5" />
                    </div>
                    <div>
                      <Label>DLNA / UPnP</Label>
                      <p className="text-xs text-muted-foreground">通用协议，支持大多数智能电视</p>
                    </div>
                  </div>
                  <Switch checked={enableDLNA} onCheckedChange={setEnableDLNA} />
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                      <Airplay className="h-5 w-5" />
                    </div>
                    <div>
                      <Label>AirPlay</Label>
                      <p className="text-xs text-muted-foreground">Apple设备专用协议</p>
                    </div>
                  </div>
                  <Switch checked={enableAirPlay} onCheckedChange={setEnableAirPlay} />
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                      <Cast className="h-5 w-5" />
                    </div>
                    <div>
                      <Label>Chromecast</Label>
                      <p className="text-xs text-muted-foreground">Google Cast协议</p>
                    </div>
                  </div>
                  <Switch checked={enableChromecast} onCheckedChange={setEnableChromecast} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-base">行为设置</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label>自动连接上次设备</Label>
                    <p className="text-xs text-muted-foreground">启动时自动连接到上次使用的设备</p>
                  </div>
                  <Switch checked={autoConnect} onCheckedChange={setAutoConnect} />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <Label>锁屏时显示控制</Label>
                    <p className="text-xs text-muted-foreground">在移动设备锁屏时显示播放控制</p>
                  </div>
                  <Switch checked={showOnLockScreen} onCheckedChange={setShowOnLockScreen} />
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
