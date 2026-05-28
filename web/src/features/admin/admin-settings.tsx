"use client"

import { useEffect, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { 
  Settings,
  Server,
  HardDrive,
  Cpu,
  Network,
  Shield,
  Bell,
  Palette,
  Globe,
  Clock,
  Save,
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  Play,
  Gauge,
  Info,
  Monitor
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Slider } from "@/components/ui/slider"
import { Separator } from "@/components/ui/separator"
import {
  ADMIN_SETTINGS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
} from "@/src/api/admin/read-models-data-source"

export function AdminSettings() {
  const { data: settingsData = ADMIN_SETTINGS_READ_MODEL_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "settings"],
    queryFn: () => createAdminReadModelsDataSource().loadSettings(),
    staleTime: 30 * 1000,
    retry: 0,
  })
  const [hasChanges, setHasChanges] = useState(false)
  const [hwAccelType, setHwAccelType] = useState(
    settingsData.transcode.hardwareAcceleration || "none",
  )

  const markChanged = () => setHasChanges(true)

  useEffect(() => {
    setHwAccelType(settingsData.transcode.hardwareAcceleration || "none")
  }, [settingsData.transcode.hardwareAcceleration])
  
  const handleHwAccelChange = (value: string) => {
    setHwAccelType(value)
    markChanged()
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">高级设置</h1>
          <p className="text-muted-foreground">
            配置服务器、转码和存储选项
            <span className="ml-2 text-xs">
              {settingsData.source === "live" ? "Live Admin API" : "Fixture fallback"}
              {settingsData.error ? ` · ${settingsData.error}` : ""}
            </span>
          </p>
        </div>
        <div className="flex items-center gap-2">
          {hasChanges && (
            <Badge variant="secondary" className="bg-warning/10 text-warning gap-1">
              <AlertTriangle className="h-3 w-3" />
              未保存的更改
            </Badge>
          )}
          <Button variant="outline" className="gap-2">
            <RefreshCw className="h-4 w-4" />
            重启服务
          </Button>
          <Button className="gap-2" disabled={!hasChanges}>
            <Save className="h-4 w-4" />
            保存设置
          </Button>
        </div>
      </div>

      <Tabs defaultValue="general" className="space-y-6">
        <TabsList className="grid w-full grid-cols-5 lg:w-auto lg:inline-flex">
          <TabsTrigger value="general" className="gap-2">
            <Server className="h-4 w-4" />
            <span className="hidden sm:inline">常规</span>
          </TabsTrigger>
          <TabsTrigger value="transcode" className="gap-2">
            <Play className="h-4 w-4" />
            <span className="hidden sm:inline">转码</span>
          </TabsTrigger>
          <TabsTrigger value="storage" className="gap-2">
            <HardDrive className="h-4 w-4" />
            <span className="hidden sm:inline">存储</span>
          </TabsTrigger>
          <TabsTrigger value="network" className="gap-2">
            <Network className="h-4 w-4" />
            <span className="hidden sm:inline">网络</span>
          </TabsTrigger>
          <TabsTrigger value="advanced" className="gap-2">
            <Settings className="h-4 w-4" />
            <span className="hidden sm:inline">高级</span>
          </TabsTrigger>
        </TabsList>

        {/* 常规设置 */}
        <TabsContent value="general" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Server className="h-5 w-5" />
                服务器信息
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>服务器名称</Label>
                  <Input defaultValue={settingsData.general.serverName} onChange={markChanged} />
                </div>
                <div className="space-y-2">
                  <Label>服务器 ID</Label>
                  <Input defaultValue={settingsData.general.serverId} disabled className="font-mono" />
                </div>
              </div>
              
              <div className="space-y-2">
                <Label>外部访问地址</Label>
                <Input defaultValue="https://media.example.com" onChange={markChanged} />
                <p className="text-xs text-muted-foreground">
                  用于生成外部分享链接
                </p>
              </div>
              <div className="rounded-lg border border-border/50 bg-muted/30 p-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">监听地址</span>
                  <span className="font-mono">{settingsData.general.listenAddr}</span>
                </div>
                <div className="mt-2 flex items-center justify-between">
                  <span className="text-muted-foreground">Admin API</span>
                  <span>{settingsData.general.adminApiVersion}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Globe className="h-5 w-5" />
                语言和区域
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>界面语言</Label>
                  <Select defaultValue="zh-CN" onValueChange={markChanged}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="zh-CN">简体中文</SelectItem>
                      <SelectItem value="zh-TW">繁體中文</SelectItem>
                      <SelectItem value="en">English</SelectItem>
                      <SelectItem value="ja">日本語</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>元数据语言</Label>
                  <Select defaultValue="zh-CN" onValueChange={markChanged}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="zh-CN">简体中文</SelectItem>
                      <SelectItem value="zh-TW">繁體中文</SelectItem>
                      <SelectItem value="en">English</SelectItem>
                      <SelectItem value="ja">日本語</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              
              <div className="space-y-2">
                <Label>时区</Label>
                <Select defaultValue="Asia/Shanghai" onValueChange={markChanged}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Asia/Shanghai">Asia/Shanghai (UTC+8)</SelectItem>
                    <SelectItem value="Asia/Tokyo">Asia/Tokyo (UTC+9)</SelectItem>
                    <SelectItem value="America/New_York">America/New_York (UTC-5)</SelectItem>
                    <SelectItem value="Europe/London">Europe/London (UTC+0)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 转码设置 */}
        <TabsContent value="transcode" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Cpu className="h-5 w-5" />
                硬件加速
              </CardTitle>
              <CardDescription>
                配置转码时使用的硬件加速方式
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>硬件加速方式</Label>
                <Select value={hwAccelType} onValueChange={handleHwAccelChange}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">无（纯软件转码）</SelectItem>
                    <SelectItem value="qsv">Intel Quick Sync Video (QSV)</SelectItem>
                    <SelectItem value="nvenc">NVIDIA NVENC</SelectItem>
                    <SelectItem value="vaapi">VA-API (Linux)</SelectItem>
                    <SelectItem value="videotoolbox">VideoToolbox (macOS)</SelectItem>
                    <SelectItem value="amf">AMD AMF</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* 针对不同硬件加速类型的设置 */}
              {hwAccelType === "none" && (
                <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                  <div className="flex items-start gap-3">
                    <Info className="h-5 w-5 text-muted-foreground mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">软件转码模式</p>
                      <p className="text-xs text-muted-foreground">
                        使用 CPU 进行转码，兼容性最好但性能较低。推荐在没有独立显卡或 Intel 核显的环境下使用。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="space-y-2">
                      <Label>编码线程数</Label>
                      <Select defaultValue="auto" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="auto">自动（推荐）</SelectItem>
                          <SelectItem value="2">2 线程</SelectItem>
                          <SelectItem value="4">4 线程</SelectItem>
                          <SelectItem value="8">8 线程</SelectItem>
                          <SelectItem value="16">16 线程</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>编码预设</Label>
                      <Select defaultValue="medium" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="ultrafast">极快（质量较低）</SelectItem>
                          <SelectItem value="veryfast">很快</SelectItem>
                          <SelectItem value="fast">快速</SelectItem>
                          <SelectItem value="medium">中等（推荐）</SelectItem>
                          <SelectItem value="slow">慢速（质量较高）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>
              )}

              {hwAccelType === "qsv" && (
                <div className="rounded-lg border border-blue-500/20 bg-blue-500/5 p-4">
                  <div className="flex items-start gap-3">
                    <Monitor className="h-5 w-5 text-blue-500 mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">Intel Quick Sync Video</p>
                      <p className="text-xs text-muted-foreground">
                        适用于 Intel 处理器（第6代及以上）的核心显卡。低功耗高效率。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="space-y-2">
                      <Label>QSV 设备</Label>
                      <Select defaultValue="auto" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="auto">自动检测</SelectItem>
                          <SelectItem value="/dev/dri/renderD128">/dev/dri/renderD128</SelectItem>
                          <SelectItem value="/dev/dri/renderD129">/dev/dri/renderD129</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>QSV 编码模式</Label>
                      <Select defaultValue="la_icq" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="icq">ICQ（恒定质量）</SelectItem>
                          <SelectItem value="la_icq">LA-ICQ（前瞻恒定质量，推荐）</SelectItem>
                          <SelectItem value="vbr">VBR（可变码率）</SelectItem>
                          <SelectItem value="cbr">CBR（恒定码率）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>低功耗模式</Label>
                        <p className="text-xs text-muted-foreground">降低 GPU 占用，适合后台转码</p>
                      </div>
                      <Switch onChange={markChanged} />
                    </div>
                  </div>
                </div>
              )}

              {hwAccelType === "nvenc" && (
                <div className="rounded-lg border border-green-500/20 bg-green-500/5 p-4">
                  <div className="flex items-start gap-3">
                    <Monitor className="h-5 w-5 text-green-500 mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">NVIDIA NVENC</p>
                      <p className="text-xs text-muted-foreground">
                        适用于 NVIDIA GTX 10 系列及以上显卡。高性能硬件编码。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="space-y-2">
                      <Label>CUDA 设备</Label>
                      <Select defaultValue="0" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="0">GPU 0（默认）</SelectItem>
                          <SelectItem value="1">GPU 1</SelectItem>
                          <SelectItem value="2">GPU 2</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>NVENC 编码预设</Label>
                      <Select defaultValue="p4" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="p1">P1（最快，质量较低）</SelectItem>
                          <SelectItem value="p2">P2</SelectItem>
                          <SelectItem value="p3">P3</SelectItem>
                          <SelectItem value="p4">P4（平衡，推荐）</SelectItem>
                          <SelectItem value="p5">P5</SelectItem>
                          <SelectItem value="p6">P6</SelectItem>
                          <SelectItem value="p7">P7（最慢，质量最高）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>码率控制模式</Label>
                      <Select defaultValue="vbr" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="cqp">CQP（恒定 QP）</SelectItem>
                          <SelectItem value="vbr">VBR（可变码率，推荐）</SelectItem>
                          <SelectItem value="cbr">CBR（恒定码率）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>B 帧支持</Label>
                        <p className="text-xs text-muted-foreground">启用 B 帧可提高压缩效率</p>
                      </div>
                      <Switch defaultChecked onChange={markChanged} />
                    </div>
                  </div>
                </div>
              )}

              {hwAccelType === "vaapi" && (
                <div className="rounded-lg border border-orange-500/20 bg-orange-500/5 p-4">
                  <div className="flex items-start gap-3">
                    <Monitor className="h-5 w-5 text-orange-500 mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">VA-API (Video Acceleration API)</p>
                      <p className="text-xs text-muted-foreground">
                        Linux 通用硬件加速接口，支持 Intel/AMD/NVIDIA 显卡。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="space-y-2">
                      <Label>渲染设备</Label>
                      <Select defaultValue="/dev/dri/renderD128" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="/dev/dri/renderD128">/dev/dri/renderD128</SelectItem>
                          <SelectItem value="/dev/dri/renderD129">/dev/dri/renderD129</SelectItem>
                          <SelectItem value="/dev/dri/card0">/dev/dri/card0</SelectItem>
                          <SelectItem value="/dev/dri/card1">/dev/dri/card1</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>VA-API 配置</Label>
                      <Select defaultValue="auto" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="auto">自动检测</SelectItem>
                          <SelectItem value="iHD">iHD（Intel 推荐）</SelectItem>
                          <SelectItem value="i965">i965（旧版 Intel）</SelectItem>
                          <SelectItem value="radeonsi">RadeonSI（AMD）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>
              )}

              {hwAccelType === "videotoolbox" && (
                <div className="rounded-lg border border-purple-500/20 bg-purple-500/5 p-4">
                  <div className="flex items-start gap-3">
                    <Monitor className="h-5 w-5 text-purple-500 mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">Apple VideoToolbox</p>
                      <p className="text-xs text-muted-foreground">
                        macOS 原生硬件加速，支持 Apple Silicon 和 Intel Mac。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>实时编码</Label>
                        <p className="text-xs text-muted-foreground">优化延迟，适合直播转码</p>
                      </div>
                      <Switch onChange={markChanged} />
                    </div>
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>允许软件回退</Label>
                        <p className="text-xs text-muted-foreground">当硬件不支持时使用软件编码</p>
                      </div>
                      <Switch defaultChecked onChange={markChanged} />
                    </div>
                    <div className="space-y-2">
                      <Label>配置文件</Label>
                      <Select defaultValue="main" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="baseline">Baseline（兼容性最好）</SelectItem>
                          <SelectItem value="main">Main（推荐）</SelectItem>
                          <SelectItem value="high">High（质量最高）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>
              )}

              {hwAccelType === "amf" && (
                <div className="rounded-lg border border-red-500/20 bg-red-500/5 p-4">
                  <div className="flex items-start gap-3">
                    <Monitor className="h-5 w-5 text-red-500 mt-0.5" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">AMD Advanced Media Framework</p>
                      <p className="text-xs text-muted-foreground">
                        适用于 AMD RX 400 系列及以上显卡。支持 VCE/VCN 编码。
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 space-y-3">
                    <div className="space-y-2">
                      <Label>AMF 编码器质量</Label>
                      <Select defaultValue="balanced" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="speed">速度优先</SelectItem>
                          <SelectItem value="balanced">平衡（推荐）</SelectItem>
                          <SelectItem value="quality">质量优先</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>码率控制</Label>
                      <Select defaultValue="vbr_latency" onValueChange={markChanged}>
                        <SelectTrigger className="w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="cqp">CQP（恒定质量）</SelectItem>
                          <SelectItem value="cbr">CBR（恒定码率）</SelectItem>
                          <SelectItem value="vbr_peak">VBR Peak（峰值码率）</SelectItem>
                          <SelectItem value="vbr_latency">VBR Latency（低延迟，推荐）</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>
              )}

              <Separator className="my-4" />

              {/* 通用硬件加速选项 */}
              {hwAccelType !== "none" && (
                <>
                  <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                      <Label>硬件解码</Label>
                      <p className="text-sm text-muted-foreground">
                        使用硬件加速解码视频
                      </p>
                    </div>
                    <Switch defaultChecked onChange={markChanged} />
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                      <Label>硬件编码</Label>
                      <p className="text-sm text-muted-foreground">
                        使用硬件加速编码视频
                      </p>
                    </div>
                    <Switch defaultChecked onChange={markChanged} />
                  </div>
                </>
              )}

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>HDR 色调映射</Label>
                  <p className="text-sm text-muted-foreground">
                    转码 HDR 内容时转换为 SDR
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Gauge className="h-5 w-5" />
                转码质量
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label>默认最大码率</Label>
                  <span className="text-sm text-muted-foreground">20 Mbps</span>
                </div>
                <Slider defaultValue={[20]} max={50} step={1} onValueChange={markChanged} />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>2 Mbps</span>
                  <span>50 Mbps</span>
                </div>
              </div>

              <Separator />

              <div className="space-y-2">
                <Label>默认视频编码器</Label>
                <Select defaultValue="h264" onValueChange={markChanged}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="h264">H.264 (AVC)</SelectItem>
                    <SelectItem value="h265">H.265 (HEVC)</SelectItem>
                    <SelectItem value="av1">AV1</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>默认音频编码器</Label>
                <Select defaultValue="aac" onValueChange={markChanged}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="aac">AAC</SelectItem>
                    <SelectItem value="ac3">AC3</SelectItem>
                    <SelectItem value="opus">Opus</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>转码临时文件</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>临时文件目录</Label>
                <div className="flex gap-2">
                  <Input defaultValue="/var/cache/nako/transcode" className="flex-1 font-mono" onChange={markChanged} />
                  <Button variant="outline" size="icon">
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                </div>
              </div>

              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label>最大缓存大小</Label>
                  <span className="text-sm text-muted-foreground">50 GB</span>
                </div>
                <Slider defaultValue={[50]} max={200} step={10} onValueChange={markChanged} />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 存储设置 */}
        <TabsContent value="storage" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <HardDrive className="h-5 w-5" />
                存储概览
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {[
                  { path: "/media", used: 12.4, total: 16, label: "媒体存储" },
                  {
                    path: "staging://admin-api",
                    used: bytesToTiB(settingsData.storage.stagingUsedBytes),
                    total: Math.max(0.1, bytesToTiB(settingsData.storage.stagingMaxBytes)),
                    label: "暂存空间",
                  },
                  { path: "/var/lib/nako", used: 2.1, total: 50, label: "数据库和配置" },
                ].map((storage) => (
                  <div key={storage.path} className="space-y-2">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium">{storage.label}</p>
                        <p className="text-sm text-muted-foreground font-mono">{storage.path}</p>
                      </div>
                      <span className="text-sm">
                        {storage.used} TB / {storage.total} TB
                      </span>
                    </div>
                    <div className="h-2 bg-secondary rounded-full overflow-hidden">
                      <div 
                        className={`h-full ${storage.used / storage.total > 0.9 ? "bg-destructive" : storage.used / storage.total > 0.7 ? "bg-warning" : "bg-primary"}`}
                        style={{ width: `${(storage.used / storage.total) * 100}%` }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>数据库</CardTitle>
              <CardDescription>
                管理媒体数据库和元数据
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50 border border-border">
                <div className="flex items-center gap-3">
                  <CheckCircle2 className="h-5 w-5 text-success" />
                  <div>
                    <p className="font-medium">数据库状态正常</p>
                    <p className="text-sm text-muted-foreground">SQLite 3.45.0 | 大小: 156 MB</p>
                  </div>
                </div>
                <Button variant="outline" size="sm">
                  备份
                </Button>
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>自动备份</Label>
                  <p className="text-sm text-muted-foreground">
                    每天自动备份数据库
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>

              <div className="space-y-2">
                <Label>备份保留数量</Label>
                <Select defaultValue="7" onValueChange={markChanged}>
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="3">3 份</SelectItem>
                    <SelectItem value="7">7 份</SelectItem>
                    <SelectItem value="14">14 份</SelectItem>
                    <SelectItem value="30">30 份</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 网络设置 */}
        <TabsContent value="network" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Network className="h-5 w-5" />
                网络配置
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>HTTP 端口</Label>
                  <Input type="number" defaultValue="8096" onChange={markChanged} />
                </div>
                <div className="space-y-2">
                  <Label>HTTPS 端口</Label>
                  <Input type="number" defaultValue="8920" onChange={markChanged} />
                </div>
              </div>

              <div className="space-y-2">
                <Label>绑定地址</Label>
                <Select defaultValue="0.0.0.0" onValueChange={markChanged}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="0.0.0.0">所有网络接口 (0.0.0.0)</SelectItem>
                    <SelectItem value="127.0.0.1">仅本地 (127.0.0.1)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>启用 HTTPS</Label>
                  <p className="text-sm text-muted-foreground">
                    需要配置 SSL 证书
                  </p>
                </div>
                <Switch onChange={markChanged} />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                远程访问
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>允许远程连接</Label>
                  <p className="text-sm text-muted-foreground">
                    允许从局域网外访问
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>自动端口映射 (UPnP)</Label>
                  <p className="text-sm text-muted-foreground">
                    自动配置路由器端口转发
                  </p>
                </div>
                <Switch onChange={markChanged} />
              </div>

              <div className="space-y-2">
                <Label>远程访问最大码率</Label>
                <Select defaultValue="20000" onValueChange={markChanged}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="0">无限制</SelectItem>
                    <SelectItem value="20000">20 Mbps</SelectItem>
                    <SelectItem value="10000">10 Mbps</SelectItem>
                    <SelectItem value="8000">8 Mbps</SelectItem>
                    <SelectItem value="4000">4 Mbps</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 高级设置 */}
        <TabsContent value="advanced" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Clock className="h-5 w-5" />
                定时任务
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>自动扫描媒体库</Label>
                  <p className="text-sm text-muted-foreground">
                    定期扫描新增和变更的文件
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>自动刷新元数据</Label>
                  <p className="text-sm text-muted-foreground">
                    定期更新媒体元数据和图片
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>自动清理缓存</Label>
                  <p className="text-sm text-muted-foreground">
                    删除过期的转码缓存文件
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Bell className="h-5 w-5" />
                通知
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>扫描完成通知</Label>
                  <p className="text-sm text-muted-foreground">
                    媒体库扫描完成后发送通知
                  </p>
                </div>
                <Switch onChange={markChanged} />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>新增媒体通知</Label>
                  <p className="text-sm text-muted-foreground">
                    添加新媒体时发送通知
                  </p>
                </div>
                <Switch onChange={markChanged} />
              </div>

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>错误警报</Label>
                  <p className="text-sm text-muted-foreground">
                    发生错误时发送警报
                  </p>
                </div>
                <Switch defaultChecked onChange={markChanged} />
              </div>
            </CardContent>
          </Card>

          <Card className="border-destructive/50">
            <CardHeader>
              <CardTitle className="text-destructive flex items-center gap-2">
                <AlertTriangle className="h-5 w-5" />
                危险操作
              </CardTitle>
              <CardDescription>
                这些操作可能导致数据丢失，请谨慎使用
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <p className="font-medium">重置所有元数据</p>
                  <p className="text-sm text-muted-foreground">
                    清除所有媒体的元数据并重新获取
                  </p>
                </div>
                <Button variant="outline" className="text-destructive hover:text-destructive">
                  重置
                </Button>
              </div>

              <Separator />

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <p className="font-medium">清空转码缓存</p>
                  <p className="text-sm text-muted-foreground">
                    删除所有转码临时文件
                  </p>
                </div>
                <Button variant="outline" className="text-destructive hover:text-destructive">
                  清空
                </Button>
              </div>

              <Separator />

              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <p className="font-medium">恢复出厂设置</p>
                  <p className="text-sm text-muted-foreground">
                    重置所有设置为默认值
                  </p>
                </div>
                <Button variant="destructive">
                  恢复出厂
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}

function bytesToTiB(bytes: number) {
  return Math.round((bytes / 1024 ** 4) * 10) / 10
}
