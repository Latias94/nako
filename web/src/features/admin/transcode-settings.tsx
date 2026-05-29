"use client"

import { useState } from "react"
import { 
  ArrowLeft, 
  Cpu, 
  HardDrive, 
  Zap, 
  Settings2, 
  ChevronRight,
  Monitor,
  Gauge,
  Film,
  Music,
  Image as ImageIcon,
  Save,
  RotateCcw,
  Info,
  AlertTriangle
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Checkbox } from "@/components/ui/checkbox"
import { Progress } from "@/components/ui/progress"
import { cn } from "@/lib/utils"

interface TranscodeSettingsProps {
  onBack?: () => void
}

// 硬件加速选项
const hwAccelOptions = [
  { value: "none", label: "无 (纯软件)", description: "使用CPU进行转码，兼容性最好但速度较慢" },
  { value: "nvenc", label: "NVIDIA NVENC", description: "使用NVIDIA显卡硬件加速，需要GTX 10系列或更新" },
  { value: "qsv", label: "Intel Quick Sync", description: "使用Intel核显硬件加速，需要6代酷睿或更新" },
  { value: "vaapi", label: "VA-API (Linux)", description: "Linux下的通用硬件加速接口" },
  { value: "videotoolbox", label: "VideoToolbox (macOS)", description: "macOS原生硬件加速" },
  { value: "amf", label: "AMD AMF", description: "使用AMD显卡硬件加速" },
]

// 视频编码器
const videoEncoders = [
  { value: "h264", label: "H.264 / AVC", description: "兼容性最好，几乎所有设备都支持" },
  { value: "hevc", label: "H.265 / HEVC", description: "更高压缩率，但部分老设备不支持" },
  { value: "av1", label: "AV1", description: "最新编码格式，压缩率最高，但硬件支持有限" },
  { value: "vp9", label: "VP9", description: "Google开发的开源编码格式" },
]

// 音频编码器
const audioEncoders = [
  { value: "aac", label: "AAC", description: "兼容性最好" },
  { value: "opus", label: "Opus", description: "更高质量，适合流媒体" },
  { value: "mp3", label: "MP3", description: "老格式，兼容性好" },
  { value: "flac", label: "FLAC", description: "无损格式，文件较大" },
]

// 预设质量
const qualityPresets = [
  { value: "auto", label: "自动", bitrate: "自适应" },
  { value: "4k", label: "4K Ultra HD", bitrate: "25-40 Mbps" },
  { value: "1080p", label: "1080p Full HD", bitrate: "8-12 Mbps" },
  { value: "720p", label: "720p HD", bitrate: "4-6 Mbps" },
  { value: "480p", label: "480p SD", bitrate: "1.5-2.5 Mbps" },
  { value: "360p", label: "360p", bitrate: "0.5-1 Mbps" },
]

export function TranscodeSettings({ onBack }: TranscodeSettingsProps) {
  const [hwAccel, setHwAccel] = useState("nvenc")
  const [enableHwDecode, setEnableHwDecode] = useState(true)
  const [enableHwEncode, setEnableHwEncode] = useState(true)
  const [enableTonemapping, setEnableTonemapping] = useState(true)
  const [videoEncoder, setVideoEncoder] = useState("h264")
  const [audioEncoder, setAudioEncoder] = useState("aac")
  const [maxBitrate, setMaxBitrate] = useState([20])
  const [audioBitrate, setAudioBitrate] = useState([192])
  const [maxConcurrent, setMaxConcurrent] = useState([2])
  const [tempPath, setTempPath] = useState("/var/cache/nako/transcode")
  const [enableSubtitleBurn, setEnableSubtitleBurn] = useState(false)
  const [preserveOriginalAudio, setPreserveOriginalAudio] = useState(true)
  const [allowRemuxing, setAllowRemuxing] = useState(true)
  const [throttleBuffer, setThrottleBuffer] = useState([180])

  // 模拟系统信息
  const systemInfo = {
    cpu: "AMD Ryzen 9 5900X",
    gpu: "NVIDIA GeForce RTX 3080",
    gpuDriver: "535.154.05",
    memory: "32 GB",
    ffmpegVersion: "6.1.1",
    hwAccelStatus: {
      nvenc: true,
      nvdec: true,
      qsv: false,
      vaapi: false,
    }
  }

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
            <h1 className="text-lg font-semibold">转码设置</h1>
            <p className="text-xs text-muted-foreground">配置视频转码和硬件加速</p>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm">
              <RotateCcw className="mr-2 h-4 w-4" />
              重置默认
            </Button>
            <Button size="sm">
              <Save className="mr-2 h-4 w-4" />
              保存设置
            </Button>
          </div>
        </div>
      </header>

      <div className="p-4 lg:p-6">
        <Tabs defaultValue="hardware" className="space-y-6">
          <TabsList className="grid w-full max-w-2xl grid-cols-4">
            <TabsTrigger value="hardware" className="gap-2">
              <Cpu className="h-4 w-4" />
              <span className="hidden sm:inline">硬件加速</span>
            </TabsTrigger>
            <TabsTrigger value="video" className="gap-2">
              <Film className="h-4 w-4" />
              <span className="hidden sm:inline">视频</span>
            </TabsTrigger>
            <TabsTrigger value="audio" className="gap-2">
              <Music className="h-4 w-4" />
              <span className="hidden sm:inline">音频</span>
            </TabsTrigger>
            <TabsTrigger value="advanced" className="gap-2">
              <Settings2 className="h-4 w-4" />
              <span className="hidden sm:inline">高级</span>
            </TabsTrigger>
          </TabsList>

          {/* 硬件加速标签 */}
          <TabsContent value="hardware" className="space-y-6">
            {/* 系统信息卡片 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Monitor className="h-5 w-5" />
                  系统信息
                </CardTitle>
                <CardDescription>当前服务器的硬件配置</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                  <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                    <p className="text-xs text-muted-foreground">CPU</p>
                    <p className="mt-1 text-sm font-medium">{systemInfo.cpu}</p>
                  </div>
                  <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                    <p className="text-xs text-muted-foreground">GPU</p>
                    <p className="mt-1 text-sm font-medium">{systemInfo.gpu}</p>
                  </div>
                  <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                    <p className="text-xs text-muted-foreground">GPU驱动</p>
                    <p className="mt-1 text-sm font-medium">{systemInfo.gpuDriver}</p>
                  </div>
                  <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                    <p className="text-xs text-muted-foreground">FFmpeg版本</p>
                    <p className="mt-1 text-sm font-medium">{systemInfo.ffmpegVersion}</p>
                  </div>
                </div>

                <div className="mt-4 flex flex-wrap gap-2">
                  <Badge variant={systemInfo.hwAccelStatus.nvenc ? "default" : "secondary"}>
                    NVENC {systemInfo.hwAccelStatus.nvenc ? "可用" : "不可用"}
                  </Badge>
                  <Badge variant={systemInfo.hwAccelStatus.nvdec ? "default" : "secondary"}>
                    NVDEC {systemInfo.hwAccelStatus.nvdec ? "可用" : "不可用"}
                  </Badge>
                  <Badge variant={systemInfo.hwAccelStatus.qsv ? "default" : "secondary"}>
                    QSV {systemInfo.hwAccelStatus.qsv ? "可用" : "不可用"}
                  </Badge>
                  <Badge variant={systemInfo.hwAccelStatus.vaapi ? "default" : "secondary"}>
                    VA-API {systemInfo.hwAccelStatus.vaapi ? "可用" : "不可用"}
                  </Badge>
                </div>
              </CardContent>
            </Card>

            {/* 硬件加速设置 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Zap className="h-5 w-5" />
                  硬件加速
                </CardTitle>
                <CardDescription>选择用于视频转码的硬件加速方式</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-3">
                  <Label>硬件加速类型</Label>
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {hwAccelOptions.map((option) => (
                      <div
                        key={option.value}
                        className={cn(
                          "cursor-pointer rounded-lg border p-4 transition-all hover:border-primary/50",
                          hwAccel === option.value ? "border-primary bg-primary/5" : "border-border"
                        )}
                        onClick={() => setHwAccel(option.value)}
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-medium">{option.label}</span>
                          {hwAccel === option.value && (
                            <div className="h-2 w-2 rounded-full bg-primary" />
                          )}
                        </div>
                        <p className="mt-1 text-xs text-muted-foreground">{option.description}</p>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="grid gap-6 sm:grid-cols-2">
                  <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                    <div className="space-y-0.5">
                      <Label>启用硬件解码</Label>
                      <p className="text-xs text-muted-foreground">使用GPU解码原始视频</p>
                    </div>
                    <Switch checked={enableHwDecode} onCheckedChange={setEnableHwDecode} />
                  </div>
                  <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                    <div className="space-y-0.5">
                      <Label>启用硬件编码</Label>
                      <p className="text-xs text-muted-foreground">使用GPU编码输出视频</p>
                    </div>
                    <Switch checked={enableHwEncode} onCheckedChange={setEnableHwEncode} />
                  </div>
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="space-y-0.5">
                    <Label>HDR到SDR色调映射</Label>
                    <p className="text-xs text-muted-foreground">将HDR内容转换为SDR以兼容更多设备</p>
                  </div>
                  <Switch checked={enableTonemapping} onCheckedChange={setEnableTonemapping} />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* 视频设置标签 */}
          <TabsContent value="video" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">视频编码设置</CardTitle>
                <CardDescription>配置视频转码输出格式和质量</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-3">
                  <Label>视频编码器</Label>
                  <Select value={videoEncoder} onValueChange={setVideoEncoder}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {videoEncoders.map((encoder) => (
                        <SelectItem key={encoder.value} value={encoder.value}>
                          <div>
                            <span>{encoder.label}</span>
                            <span className="ml-2 text-xs text-muted-foreground">{encoder.description}</span>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label>最大视频码率</Label>
                    <span className="text-sm text-muted-foreground">{maxBitrate[0]} Mbps</span>
                  </div>
                  <Slider
                    value={maxBitrate}
                    onValueChange={setMaxBitrate}
                    max={50}
                    min={1}
                    step={1}
                  />
                  <p className="text-xs text-muted-foreground">更高的码率意味着更好的画质，但会增加带宽和存储需求</p>
                </div>

                <div className="space-y-3">
                  <Label>质量预设</Label>
                  <div className="grid gap-2 sm:grid-cols-3 lg:grid-cols-6">
                    {qualityPresets.map((preset) => (
                      <div
                        key={preset.value}
                        className="cursor-pointer rounded-lg border border-border/50 p-3 text-center transition-all hover:border-primary/50"
                      >
                        <p className="text-sm font-medium">{preset.label}</p>
                        <p className="text-xs text-muted-foreground">{preset.bitrate}</p>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="space-y-0.5">
                    <Label>允许直接串流 (Remux)</Label>
                    <p className="text-xs text-muted-foreground">当客户端支持原始格式时，直接传输而不转码</p>
                  </div>
                  <Switch checked={allowRemuxing} onCheckedChange={setAllowRemuxing} />
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="space-y-0.5">
                    <Label>烧录字幕</Label>
                    <p className="text-xs text-muted-foreground">将字幕硬编码到视频中（对于不支持字幕的设备）</p>
                  </div>
                  <Switch checked={enableSubtitleBurn} onCheckedChange={setEnableSubtitleBurn} />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* 音频设置标签 */}
          <TabsContent value="audio" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">音频编码设置</CardTitle>
                <CardDescription>配置音频转码输出格式</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-3">
                  <Label>音频编码器</Label>
                  <Select value={audioEncoder} onValueChange={setAudioEncoder}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {audioEncoders.map((encoder) => (
                        <SelectItem key={encoder.value} value={encoder.value}>
                          <div>
                            <span>{encoder.label}</span>
                            <span className="ml-2 text-xs text-muted-foreground">{encoder.description}</span>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label>音频码率</Label>
                    <span className="text-sm text-muted-foreground">{audioBitrate[0]} kbps</span>
                  </div>
                  <Slider
                    value={audioBitrate}
                    onValueChange={setAudioBitrate}
                    max={320}
                    min={64}
                    step={32}
                  />
                </div>

                <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                  <div className="space-y-0.5">
                    <Label>保留原始音轨</Label>
                    <p className="text-xs text-muted-foreground">在转码时保留所有原始音频轨道（多语言、评论音轨等）</p>
                  </div>
                  <Switch checked={preserveOriginalAudio} onCheckedChange={setPreserveOriginalAudio} />
                </div>

                <div className="rounded-lg border border-border/50 p-4">
                  <Label className="mb-3 block">音频通道下混</Label>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-3">
                      <Checkbox id="downmix-stereo" defaultChecked />
                      <label htmlFor="downmix-stereo" className="text-sm">
                        将环绕声下混为立体声（对于不支持多声道的设备）
                      </label>
                    </div>
                    <div className="flex items-center space-x-3">
                      <Checkbox id="normalize-audio" />
                      <label htmlFor="normalize-audio" className="text-sm">
                        音量标准化
                      </label>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* 高级设置标签 */}
          <TabsContent value="advanced" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">转码性能</CardTitle>
                <CardDescription>配置转码任务的资源使用</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label>最大并发转码数</Label>
                    <span className="text-sm text-muted-foreground">{maxConcurrent[0]} 个</span>
                  </div>
                  <Slider
                    value={maxConcurrent}
                    onValueChange={setMaxConcurrent}
                    max={8}
                    min={1}
                    step={1}
                  />
                  <p className="text-xs text-muted-foreground">同时进行的转码任务数量，过多会影响系统性能</p>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label>转码节流缓冲</Label>
                    <span className="text-sm text-muted-foreground">{throttleBuffer[0]} 秒</span>
                  </div>
                  <Slider
                    value={throttleBuffer}
                    onValueChange={setThrottleBuffer}
                    max={600}
                    min={30}
                    step={30}
                  />
                  <p className="text-xs text-muted-foreground">预先转码的视频缓冲时长，更长的缓冲可以减少卡顿</p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-base">临时文件</CardTitle>
                <CardDescription>转码过程中的临时文件存储位置</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label>转码缓存路径</Label>
                  <Input value={tempPath} onChange={(e) => setTempPath(e.target.value)} />
                  <p className="text-xs text-muted-foreground">建议使用SSD以获得最佳性能</p>
                </div>

                <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">当前缓存使用</span>
                    <span className="text-sm font-medium">2.4 GB / 50 GB</span>
                  </div>
                  <Progress value={4.8} className="mt-2 h-2" />
                  <Button variant="outline" size="sm" className="mt-3">
                    清理缓存
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="border-destructive/50">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base text-destructive">
                  <AlertTriangle className="h-5 w-5" />
                  实验性功能
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>启用AV1编码</Label>
                    <p className="text-xs text-muted-foreground">实验性支持，可能不稳定</p>
                  </div>
                  <Switch />
                </div>
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>启用实时转码优化</Label>
                    <p className="text-xs text-muted-foreground">使用更激进的编码参数以降低延迟</p>
                  </div>
                  <Switch />
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
