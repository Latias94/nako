# Research: Jellyfin 源码结构与 Nako 可参考点

- Query: 从 Jellyfin 源码结构和文档中提炼 Nako 可参考的产品/架构启发，明确不能复制 GPL 源码实现，只能观察行为/边界/用户工作流。
- Scope: mixed
- Date: 2026-06-13

## Findings

### 1. Jellyfin 高层模块划分

- `README.md` 把仓库定位成后端服务器，默认可托管 web client，也支持分离部署；开发说明里还把 ffmpeg 作为运行依赖之一（`repo-ref/jellyfin/README.md:42,71,83,95,122,142,184`）。
- `Jellyfin.sln` 把核心域拆成 `MediaBrowser.Controller`, `MediaBrowser.Common`, `MediaBrowser.Model`, `MediaBrowser.Providers`, `MediaBrowser.LocalMetadata`, `MediaBrowser.XbmcMetadata`, `MediaBrowser.MediaEncoding`, `Jellyfin.Api`, `Jellyfin.Server`, `Emby.Server.Implementations`, `Emby.Naming`, `Emby.Photos` 和测试项目，结构上是“模型 / 控制器 / 实现 / 提供者 / 编码 / API”分层（`repo-ref/jellyfin/Jellyfin.sln:6-100`）。
- `ApplicationHost` 负责把 `ILibraryManager`, `IMediaSourceManager`, `ICollectionManager`, `IInstallationManager` 等注册进 DI，并把 `IResolverIgnoreRule`, `IItemResolver`, `ILibraryPostScanTask`, `IMediaSourceProvider` 以扩展部件方式挂入库扫描与媒体源流程；同时也会装载插件程序集（`repo-ref/jellyfin/Emby.Server.Implementations/ApplicationHost.cs:508,542,561,574,695-709,846,972`）。
- API 端按用户工作流拆 controller：`Library*`, `Items`, `MediaInfo`, `Videos`, `Audio`, `Subtitle`, `Playstate`, `DynamicHls`, `HlsSegment`, `Plugins`, `UserLibrary` 等，暴露的是产品动作而不是底层对象树（`repo-ref/jellyfin/Jellyfin.Api/Controllers/*.cs`）。
- 对 Nako 的对照是：Jellyfin 这套分层和 Nako 现有的 `Media Library`、`Canonical Metadata`、`Playback Runtime`、`Addon Sidecar` 边界很接近，但 Nako 已明确要把边界做成 Rust 主机 + 进程外 sidecar，而不是 .NET 进程内扩展（`CONTEXT.md:35-36,433-485,489-490,455-480`; `docs/ARCHITECTURE.md:57-74,85-91`; `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md:18-33,60-79`）。

### 2. 插件模型和 Nako Addon Sidecar 的关键差异

- Jellyfin 插件是进程内 assembly 插件：`BasePlugin` / `BasePlugin<T>` 绑定当前程序集和 XML 配置文件，`IPluginManager` 负责 `LoadAssemblies`、`EnablePlugin`、`DisablePlugin`、`RemovePlugin`、`PopulateManifest`，`PluginsController` 直接读写插件配置并调用安装/卸载（`repo-ref/jellyfin/MediaBrowser.Common/Plugins/BasePlugin.cs:13-83`; `repo-ref/jellyfin/MediaBrowser.Common/Plugins/BasePluginOfT.cs:18-171`; `repo-ref/jellyfin/MediaBrowser.Common/Plugins/IPluginManager.cs:14-94`; `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs:28-244`）。
- Jellyfin 还提供 `IHasWebPages.GetPages()` 让插件把页面挂进服务器 UI，说明它默认接受“宿主内嵌扩展 UI”的模式（`repo-ref/jellyfin/MediaBrowser.Model/Plugins/IHasWebPages.cs:7-9`; `repo-ref/jellyfin/MediaBrowser.Model/Plugins/PluginPageInfo.cs:6`; `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs:163-244`）。
- Jellyfin 的 metadata provider 里也有大量 `Plugin : BasePlugin<PluginConfiguration>, IHasWebPages, IHasEmbeddedImage` 这种形态，插件和功能实现更像宿主内模块，而不是独立服务（如 `repo-ref/jellyfin/MediaBrowser.Providers/Plugins/Tmdb/Plugin.cs:16`; `repo-ref/jellyfin/MediaBrowser.Providers/Plugins/AudioDb/Plugin.cs:14`; `repo-ref/jellyfin/MediaBrowser.Providers/Plugins/MusicBrainz/Plugin.cs:21`）。
- Nako 这边的约束更硬：Addon 是 `Addon Sidecar`，通过 `Addon Protocol` 和 `Addon Token` 调用 Nako API；不能假设 `Jellyfin Plugin Compatibility`，也不把 `Native Plugin` 当主线（`CONTEXT.md:35-40,437-485,489-490,455-480`; `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md:18-33,60-79`）。
- 结论：Jellyfin 可借鉴的是“可发现、可启用/停用、可配置、可展示页面”的产品体验；Nako 不该复制的是“进程内程序集加载 + 宿主对象模型直连 + 服务器 UI 内嵌页面”的信任边界。

### 3. 媒体库 / 元数据 / 播放 / 转码对 Nako 的参考价值

- 媒体库：`ILibraryManager` 把解析、路径映射、虚拟库、扫描、后处理任务、排序、增删改查、扫描队列统一收口；`IMediaSourceManager` 负责媒体源、直播源、探测和默认音轨/字幕索引（`repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryManager.cs:31-771`; `repo-ref/jellyfin/MediaBrowser.Controller/Library/IMediaSourceManager.cs:19-143`）。
- 配置层：`LibraryOptions` 和 `VirtualFolderInfo` 说明“库是配置边界，不只是目录”；库级选项会直接影响扫描、图像、刷新节奏和状态回传（`repo-ref/jellyfin/MediaBrowser.Model/Configuration/LibraryOptions.cs:9-74`; `repo-ref/jellyfin/MediaBrowser.Model/Entities/VirtualFolderInfo.cs:12-56`）。
- 元数据：`ProviderManager` 把本地元数据、远程元数据、图片、NFO 保存统一编排；`ILocalMetadataProvider` / `IRemoteMetadataProvider` / `IImageProvider` / `IMetadataSaver` 让“搜索、拉取、选图、落盘”分成不同角色（`repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:443-941`; `repo-ref/jellyfin/MediaBrowser.Controller/Providers/IRemoteMetadataProvider.cs:12-31`; `repo-ref/jellyfin/MediaBrowser.Controller/Providers/ILocalMetadataProvider.cs:9-23`; `repo-ref/jellyfin/MediaBrowser.XbmcMetadata/Savers/BaseNfoSaver.cs:30-129`）。
- NFO 工作流：`MediaBrowser.LocalMetadata` 与 `MediaBrowser.XbmcMetadata` 分别承担本地 XML/NFO 的读取与写回，`NfoUserDataSaver` 通过 `SaveMetadataAsync` 把用户数据回写到宿主管理的元数据流程里（`repo-ref/jellyfin/MediaBrowser.LocalMetadata/BaseXmlProvider.cs:14-45`; `repo-ref/jellyfin/MediaBrowser.LocalMetadata/Savers/BaseXmlSaver.cs:23-317`; `repo-ref/jellyfin/MediaBrowser.XbmcMetadata/Parsers/BaseNfoParser.cs:37-653`; `repo-ref/jellyfin/MediaBrowser.XbmcMetadata/Savers/BaseNfoSaver.cs:30-990`; `repo-ref/jellyfin/MediaBrowser.XbmcMetadata/NfoUserDataSaver.cs:23-79`）。
- 播放：`DeviceProfile`、`DirectPlayProfile`、`TranscodingProfile`、`StreamBuilder`、`MediaSourceInfo`、`PlayMethod`、`TranscodeReason` 这套模型先算“能不能直放 / 直流 / 转码”，再下发播放 URL 或转码理由（`repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs:13-55`; `repo-ref/jellyfin/MediaBrowser.Model/Dlna/StreamBuilder.cs:19-1413`; `repo-ref/jellyfin/MediaBrowser.Model/Dto/MediaSourceInfo.cs:14-132`; `repo-ref/jellyfin/MediaBrowser.Model/Session/PlayMethod.cs:5`; `repo-ref/jellyfin/MediaBrowser.Model/Session/TranscodeReason.cs:8-45`）。
- 转码 / HLS：`MediaEncoder` 负责 FFmpeg 输入参数、探测归一化、封面 / 字幕抽取，`TranscodeManager` 负责转码作业生命周期，`DynamicHlsController` 和 `HlsSegmentController` 负责动态 HLS 播放与分段，`DynamicHlsPlaylistGenerator` 负责 playlist 生成，`KeyframeExtractionScheduledTask` 说明 HLS 还有异步预处理任务（`repo-ref/jellyfin/MediaBrowser.MediaEncoding/Encoder/MediaEncoder.cs:42-1345`; `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs:34-670`; `repo-ref/jellyfin/Jellyfin.Api/Controllers/DynamicHlsController.cs:42-1639`; `repo-ref/jellyfin/Jellyfin.Api/Controllers/HlsSegmentController.cs:24-39`; `repo-ref/jellyfin/src/Jellyfin.MediaEncoding.Hls/Playlist/DynamicHlsPlaylistGenerator.cs:17-112`; `repo-ref/jellyfin/src/Jellyfin.MediaEncoding.Hls/ScheduledTasks/KeyframeExtractionScheduledTask.cs:18-33`; `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Probing/ProbeResultNormalizer.cs:26-49`）。
- 对 Nako 的直接启发：可以吸收“库级配置 + 扫描 / 后处理分层 + 元数据提供者编排 + 播放决策先行 + HLS / 转码作业受控”的产品形状，但实现应落在 Nako 自己的术语和契约上，尤其要维持 `Playback Runtime` 与 `Addon Sidecar` 的边界（`CONTEXT.md:421-486,506-548,768-780`; `docs/architecture/LIBRARY_PIPELINE.md`; `docs/architecture/PLAYBACK.md`; `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`）。

### 4. Nako 应避免照搬的地方

- 不能照搬 GPL 代码、注释、测试、schema、资产或生成物；Jellyfin 仓库许可证是 GPL-2.0，README 也明确标识这一点（`repo-ref/jellyfin/LICENSE:1-2`; `repo-ref/jellyfin/README.md:11`）。
- 不能复制 Jellyfin 的 in-process 插件 ABI、AssemblyLoad 方式、`BasePlugin` / `PluginManifest` / `PluginsController` 这类宿主内部耦合形态；Nako 已选择 sidecar + HTTP + token 模式（`repo-ref/jellyfin/MediaBrowser.Common/Plugins/*`; `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs:28-244`; `CONTEXT.md:455-490`）。
- 不能直接沿用 Jellyfin 的 DLNA / `DeviceProfile` 作为 Nako 的域模型；Nako 应保留自己的 `Playback Source Selection`、`Media Technical Facts`、`Audio Output Requirement`、`Color Pipeline Requirement` 语言（`repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs:13-55`; `repo-ref/jellyfin/MediaBrowser.Model/Dlna/StreamBuilder.cs:1273-1413`; `CONTEXT.md:149-160,311-329,433-435,543-548`）。
- 不能照抄 Jellyfin 的路由名、DTO 形状、provider 名称、错误码枚举和 FFmpeg 命令拼接逻辑；对 Nako 来说这些都属于“行为参考”，不是实现模板（`repo-ref/jellyfin/Jellyfin.Api/Controllers/*.cs`; `repo-ref/jellyfin/MediaBrowser.Model/Dto/*.cs`; `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs:371-537`）。
- 不能把 provider 特定术语直接升格为核心域；Jellyfin 里 `Tmdb`、`Omdb`、`MusicBrainz`、`AudioDb`、`Zap2It` 等已经深度进入实现，但 Nako 的核心应维持 provider-neutral（`repo-ref/jellyfin/MediaBrowser.Providers/Plugins/*`; `CONTEXT.md:227-275,529-535,762-773`）。

### 5. 可转化为竞品分析字段的问题

- Jellyfin 的“插件体验”到底拆成哪些用户可感知能力：安装、启用 / 停用、配置、页面入口、健康状态、卸载、更新？
- Jellyfin 的库配置是如何按库 / 按类型分层的：虚拟库、路径、内容类型、刷新频率、图像 / 扫描开关、用户访问？
- Jellyfin 的元数据工作流里，哪些字段是本地优先、哪些是远程补全、哪些可手工覆写、哪些会回写到 NFO？
- Jellyfin 如何把 `ProviderIds`、外部 ID、图像候选、NFO 读写和用户数据保存串成单条工作流？
- Jellyfin 的播放决策里，哪些是“客户端能力”，哪些是“源技术事实”，哪些是“转码原因”？
- Jellyfin 的 HLS / 转码边界里，哪些能力在 API 层、哪些在媒体编码层、哪些在计划层？
- Jellyfin 对外暴露的控制平面能力有哪些：插件管理、系统信息、库管理、播放状态、字幕、图片、动态 HLS、活动日志？
- Jellyfin 的“web client 可分离部署”对产品集成与开发体验有什么影响？
- Jellyfin 的 in-process 扩展模式在安全性、调试便利、版本耦合和发布节奏上各自带来什么成本？
- 对 Nako 来说，这些问题应该进一步转写成：是否需要 sidecar 入口、是否需要库级 grant、是否需要用户可见健康检查、是否需要独立 addon 页面、是否需要 NFO round-trip、是否需要可解释的播放 / 转码理由。

## External References

- Jellyfin README 指向的官方文档与相关项目：`https://jellyfin.org/docs/`、`https://github.com/jellyfin/jellyfin-web`、`https://github.com/jellyfin/jellyfin-ffmpeg`
- Jellyfin 许可证：GPL-2.0（`repo-ref/jellyfin/LICENSE`）
- Nako 参考文档：`CONTEXT.md`、`docs/ARCHITECTURE.md`、`docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`、`docs/adr/0038-playback-planning-and-transcode-policy-seams.md`

## Files Read

- `repo-ref/jellyfin/README.md` - 仓库定位、开发方式、web client / ffmpeg 依赖、运行与文档入口。
- `repo-ref/jellyfin/Jellyfin.sln` - solution 级项目分层与模块边界。
- `repo-ref/jellyfin/LICENSE` - GPL-2.0 许可证确认。
- `repo-ref/jellyfin/Emby.Server.Implementations/ApplicationHost.cs` - DI 注册、插件程序集装载、扩展点挂载。
- `repo-ref/jellyfin/MediaBrowser.Common/Plugins/*.cs` - 进程内插件模型、配置持久化、装载与启停。
- `repo-ref/jellyfin/MediaBrowser.Model/Plugins/*.cs` - 插件状态、页面入口、配置 DTO。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs` - 插件安装 / 启用 / 停用 / 卸载 / 配置 API。
- `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryManager.cs` - 库扫描、路径解析、虚拟库、后处理与查询接口。
- `repo-ref/jellyfin/MediaBrowser.Controller/Library/IMediaSourceManager.cs` - 媒体源、直播源、探测与默认轨道选择。
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/LibraryOptions.cs` - 库级扫描 / 图像 / 刷新选项。
- `repo-ref/jellyfin/MediaBrowser.Model/Entities/VirtualFolderInfo.cs` - 虚拟库状态与刷新进度模型。
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/*.cs` - 元数据 / 图片 / 外部 ID 的抽象接口。
- `repo-ref/jellyfin/MediaBrowser.Providers/*.cs` - 远程元数据服务与 provider 插件实现。
- `repo-ref/jellyfin/MediaBrowser.LocalMetadata/*.cs` - 本地 XML / NFO、局部图片读取与写回。
- `repo-ref/jellyfin/MediaBrowser.XbmcMetadata/*.cs` - Xbmc / NFO 解析与保存实现。
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs` - 元数据与图片提供者编排。
- `repo-ref/jellyfin/MediaBrowser.Model/Dlna/*.cs` - 播放能力与转码判定模型。
- `repo-ref/jellyfin/MediaBrowser.Model/Dto/MediaSourceInfo.cs` - 媒体源与转码输出信息。
- `repo-ref/jellyfin/MediaBrowser.Model/Session/*.cs` - 播放方法、转码原因、会话能力与转码信息。
- `repo-ref/jellyfin/MediaBrowser.MediaEncoding/*.cs` - FFmpeg 参数、探测、转码、字幕与附件抽取。
- `repo-ref/jellyfin/src/Jellyfin.MediaEncoding.Hls/*.cs` - 动态 HLS 播放列表与 keyframe 抽取。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/*.cs` - library / items / playstate / mediainfo / video / audio / subtitle / HLS 等控制器。
- `CONTEXT.md` - Nako 术语、Addon Sidecar、Playback Runtime、Metadata / NFO 边界。
- `docs/ARCHITECTURE.md` - Nako 总体架构、插件 / 转码 / 库 / 控制平面边界。
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md` - Nako 侧车式 addon 的直接约束。
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md` - 播放与转码分层参考。
- `docs/adr/0003-http-addons-before-in-process-plugins.md` - 早期 addon 采用 HTTP / 进程外边界的决策。
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md` - 能力范围与自动化提供者方向。

## Caveats / Not Found

- 本次只做高层结构调研，没有逐文件深挖实现，也没有把 Jellyfin 的具体算法、错误分支、测试数据完整展开。
- 未在线核验 Jellyfin 官方文档页面，只使用了仓库内 README 和本地源码结构作为依据。
- Jellyfin 源码是 GPL-2.0；后续任何 Nako 实现都必须保持独立实现，不能沿用源码、注释、测试、schema、assets 或生成内容。
