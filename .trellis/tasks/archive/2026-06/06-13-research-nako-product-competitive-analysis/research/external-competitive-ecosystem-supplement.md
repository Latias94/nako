# Research: 外部竞品与自托管媒体生态补充

- Query: Nako 自托管媒体服务器竞品分析，需要覆盖 Jellyfin、Plex、Emby、Kodi、Stremio、Immich、Navidrome、Audiobookshelf 和相邻生态工具。
- Scope: external
- Date: 2026-06-13

## Supplementary Items

- **Servarr / *arr stack**：应加入。Sonarr、Radarr、Lidarr、Readarr、Prowlarr 已成为自托管影视、音乐、书籍库自动化的事实外围系统，覆盖监控、抓取、整理、命名、索引器同步，不是播放器但强影响媒体服务器工作流。
- **Bazarr**：应加入。它是 Sonarr/Radarr 的字幕自动化伴侣，覆盖多语言字幕、缺失字幕补齐、Jellyfin 集成刷新，能补足 Nako 的字幕生命周期与外部自动化对比。
- **Seerr**：应加入。它定位为 Overseerr 与 Jellyseerr 的统一后继，面向 Jellyfin、Plex、Emby 的媒体发现与请求管理，是多用户“请求-审批-入库”体验的关键外围竞品。
- **Kometa**：应加入。它覆盖集合、海报覆盖层、元数据和第三方榜单驱动的库策展，对 Nako 的 metadata、NFO、artwork pipeline 有参考价值。
- **Tdarr / FileFlows / Unmanic**：应作为“媒体处理自动化”类加入。它们聚焦 FFmpeg/HandBrake、分布式节点、规则流、转码、重封装和库优化。
- **Tautulli / Jellystat**：应作为“使用分析与可观测性”类加入。Tautulli 是 Plex 监控分析代表，Jellystat 是 Jellyfin 统计应用；Nako 的 Admin diagnostics 应单独和这类工具对标。
- **WatchState / JellyPlex-Watched**：应加入。它们解决 Plex、Jellyfin、Emby 之间 watched state 同步，是迁移、互操作、用户状态可携带性的直接信号。
- **Maintainerr**：应加入。它面向 Plex/Jellyfin/Emby 的规则化清理、删除、取消监控、清请求，代表“存储生命周期治理”。
- **Tunarr / ErsatzTV**：应加入。它们把现有媒体库编排成带 EPG 的线性频道，并通过 HDHomeRun/M3U 接入 Plex/Jellyfin/Emby，能启发 Nako 的“个人频道/连续播放/电视化体验”。
- **Dispatcharr**：应加入。它是 IPTV/M3U/EPG 管理和代理工具，面向 Plex/Jellyfin/Emby 等下游，适合补充 Live TV、流代理、EPG、失败切换维度。
- **Channels DVR**：应加入。它是商业化家庭 DVR/直播电视产品，强调本地服务器、客户端体验和平台支持策略，对 Nako 的 live TV/DVR 方向有参考价值。
- **Kavita / Komga / Calibre-Web**：应作为“阅读媒体服务器”类加入。现有框架已有 Audiobookshelf，但缺少 comics、manga、ebooks；这些项目能补齐非视频媒体域深度、OPDS、阅读进度、集合模型。
- **PhotoPrism / Ente Photos / LibrePhotos / Nextcloud Memories**：应作为 Immich 外的照片相邻基线加入。它们分别强调 AI 照片库、自托管/E2EE、开源 ML、Nextcloud 内照片体验，能避免只用 Immich 代表照片域。
- **Tube Archivist / Pinchflat / MeTube**：应作为“在线视频归档/导入”类加入。它们围绕 YouTube/yt-dlp、频道订阅、元数据索引、离线观看，适合分析 Nako 的 online/catalog resource 边界和合法性策略。
- **PeerTube**：可作为低优先级相邻对象加入。它不是个人媒体库服务器，而是 ActivityPub 联邦视频发布平台；若 Nako 未来考虑公开分享、社区目录或联邦分发，应纳入观察。

## Recommended Supplementary Fields

- **Ecosystem Role & Integration Point**：区分核心媒体服务器、客户端、请求管理、自动化、转码、统计、清理、直播/IPTV、照片/书籍/音频垂直工具，避免把外围工具误判为直接竞品。
- **Remote Access & Account Dependency**：记录是否需要云账号、远程访问是否付费、是否支持纯局域网、反向代理/Tailscale/VPN 友好度。Plex 2025 后远程个人视频播放订阅化，使此维度必须单列。
- **Monetization & Feature Gates**：记录免费版、订阅、终身授权、可选捐赠/product key，以及硬件转码、下载、DVR、远程播放、移动端是否被门控。
- **Client Surface & Store Distribution**：单独比较 Web、iOS、Android、Apple TV、Android TV、Roku、Tizen、webOS、Kodi 插件、第三方客户端，以及应用商店可得性。
- **Automation Workflow Compatibility**：记录与 Servarr、Seerr、Bazarr、Kometa、Tdarr、Tautulli/Jellystat、Maintainerr 等生态的 API 适配程度。
- **Extension Trust Boundary**：比较 in-process plugin、HTTP addon、sidecar、SDK、权限模型、分发目录、审核机制、崩溃隔离和副作用控制；这是 Nako Addon Sidecar 的核心差异化字段。
- **Data Portability & Migration**：记录 NFO、sidecar artwork、provider IDs、watched state、用户进度、播放列表、集合、评分、备份/恢复、跨系统迁移路径。
- **Media Processing Pipeline**：比较扫描、探测、转码、重封装、HDR tone mapping、字幕提取、章节、音轨选择、硬件能力发现、队列、失败重试、分布式 worker。
- **Library Lifecycle & Storage Policy**：增加去重、多版本/多质量、edition/variant、缓存、清理、归档、保留策略、空间预警、自动删除与安全护栏。
- **ML / Semantic Metadata**：对照片、视频、音乐和文本媒体分别记录 face/object/OCR/CLIP/search、模型本地化、远程 ML、资源消耗、隐私边界。
- **Live TV / Linear Experience**：记录 DVR、EPG、HDHomeRun、M3U、IPTV proxy、虚拟频道、连续播放、广告/间隔片段、频道调度。
- **Operations Maturity**：记录 Docker Compose、TrueNAS/unRAID/Proxmox/NAS 包、升级破坏性、备份、健康检查、日志、指标、诊断包、安全公告与 CVE 响应。
- **Legal & Policy Boundary**：对 Stremio addons、IPTV、yt-dlp、torrent/debrid、metadata providers、插件目录审核进行单独评估，避免产品定位和合规风险混在功能表里。
- **Community & Release Health**：记录 release cadence、贡献者、issue/PR 活跃度、官方 vs 社区客户端、文档质量、插件生态规模和维护风险。

## Sources

- [Jellyfin Documentation](https://jellyfin.org/docs/)；[Jellyfin plugins](https://jellyfin.org/docs/general/server/plugins/)；[Jellyfin 10.11 release](https://jellyfin.org/posts/jellyfin-release-10.11.0/)
- [Plex Pass plans](https://www.plex.tv/plans/)；[Plex 2025 updates](https://www.plex.tv/blog/important-2025-plex-updates/)；[Plex remote playback requirements](https://support.plex.tv/articles/requirements-for-remote-playback-of-personal-media/)
- [Emby Premiere feature matrix](https://emby.media/support/articles/Premiere-Feature-Matrix.html)
- [Kodi addons](https://kodi.tv/addons/)
- [Stremio Addon SDK](https://www.stremio.com/addon-sdk)
- [Immich stable release](https://immich.app/blog/stable-release)
- [Navidrome](https://www.navidrome.org/)
- [Audiobookshelf](https://www.audiobookshelf.org/)
- [Servarr Wiki](https://wiki.servarr.com/)
- [Bazarr](https://www.bazarr.media/)
- [Seerr](https://seerr.dev/)
- [Kometa Wiki](https://kometa.wiki/)
- [Tdarr](https://tdarr.io/)；[FileFlows](https://fileflows.com/)；[Unmanic](https://github.com/unmanic/unmanic)
- [Tautulli](https://tautulli.com/)；[Jellystat](https://github.com/CyferShepard/Jellystat)；[WatchState](https://github.com/arabcoders/watchstate)
- [Maintainerr](https://maintainerr.info/)
- [Tunarr](https://tunarr.com/)；[ErsatzTV](https://ersatztv.org/)；[Dispatcharr Docs](https://dispatcharr.github.io/Dispatcharr-Docs/)；[Channels DVR](https://getchannels.com/)
- [Kavita](https://www.kavitareader.com/)；[Komga](https://komga.org/)；[Calibre-Web](https://github.com/janeczku/calibre-web)
- [PhotoPrism](https://www.photoprism.app/)；[Ente self-hosting](https://ente.com/help/self-hosting/)；[LibrePhotos](https://docs.librephotos.com/)；[Memories](https://memories.gallery/)
- [Tube Archivist](https://www.tubearchivist.com/)；[Pinchflat](https://github.com/kieraneglin/pinchflat)；[MeTube](https://github.com/alexta69/metube)；[PeerTube](https://joinpeertube.org/)
