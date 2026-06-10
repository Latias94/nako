# Plex / Jellyfin Product Benchmark For Nako

访问日期：2026-06-10

范围：基于公开官方资料调研 Plex 与 Jellyfin 作为成熟自托管/个人媒体服务器的用户可见能力面，并映射为 Nako 需要追赶、延后、或明确不做的能力分类。本笔记只做产品与能力面基准，不检查 Plex 源码，不复制 Jellyfin 实现。

## 执行摘要

Plex 的强项不是单点功能，而是完整用户旅程：库配置、自动扫描、匹配与海报、跨设备播放、远程访问、账号分享、Downloads、Live TV/DVR、Dashboard、广泛客户端和订阅打包。Plex 同时带有强云账号、集中服务、Premium 功能门槛和商业内容生态，这些不是 Nako 应照搬的方向。

Jellyfin 的强项是本地优先、开源、自托管可控、插件和客户端社区。它覆盖了媒体库、元数据、NFO、本地 artwork、Direct Play/Transcode、硬件加速、Live TV/DVR、用户访问控制、管理后台、插件和 Web/API 客户端生态。弱点是部分体验依赖客户端差异，远程访问需要用户自己配置网络，离线同步/优化版本不像 Plex 那样是一体化产品。

Nako 当前架构已经选择了与 Jellyfin/Plex 同量级的 server backend 方向：Video-First Phase、Media Library、Media Source、Canonical Metadata、Source Variant、Playback Source Selection、Addon Sidecar、Control Plane。追赶优先级应放在“一个自托管、视频优先、单管理员可完成的闭环”：配置 Media Library -> 扫描与识别 -> 元数据/Artwork 治理 -> 浏览 -> Direct Play/Remux/Transcode -> 远程访问文档 -> 用户/权限 -> Admin 诊断/修复 -> 备份/恢复。

## 产品基准矩阵

| 能力面 | Plex 用户可见能力 | Jellyfin 用户可见能力 | Nako 分类 |
| --- | --- | --- | --- |
| Library onboarding / scan | 创建 Library、选择类型与文件夹、扫描/刷新、自动库更新、匹配/修复匹配、metadata agents、本地 media assets、海报和背景管理。 | 创建 Libraries、内容类型、目录规则、扫描、Identify/Refresh Metadata、NFO、图片/本地 artwork、第三方 metadata 插件。 | 追赶 P0：M1 必须有清晰 onboarding、扫描状态、错误/修复、metadata/artwork 治理。 |
| Playback / transcode | Direct Play、Direct Stream、Transcode、硬件加速、HDR tone mapping、远程流质量、Relay、Plex Pass/Premium gating、Media Optimizer。 | Direct Play 优先、FFmpeg transcode、硬件加速、客户端 codec support matrix、reverse proxy/VPN 自托管远程访问。 | 追赶 P0/P1：Direct Play first、显式 client capability、HLS/remux/transcode、硬件能力与资源预算。明确不做 Plex 云 relay/paywall 形态。 |
| User / account / sharing | Plex account、Plex Home、Managed Accounts、PIN、Library Access/Sharing、restriction profiles、ratings/labels、下载/远程访问权限。 | 本地用户、密码、library access、remote access 开关、parental controls、访问时间/标签/评分限制。 | 追赶 P0：本地身份、角色、Library Access、parental controls、会话控制。延后社交化 sharing。 |
| Live TV / DVR / downloads / sync / optimized versions | Live TV/DVR、EPG、录制计划、Downloads、Media Optimizer/optimized versions。 | Live TV/DVR、tuner/M3U、guide data、录制后处理；官方文档没有把统一离线同步和 optimized versions 展示为 Plex 等价的一等产品。 | 延后 P1/P2：Live TV/DVR 与离线/optimized artifacts 要等播放与 Control Plane 稳定。 |
| Admin / diagnostics / monitoring / backup | Dashboard、active streams、bandwidth/history、logs、server data backup、scheduled tasks。 | Admin Dashboard、scheduled tasks、logs、troubleshooting、backup/restore 文档、metrics/monitoring 文档。 | 追赶 P0：Admin diagnostics/repair、Jobs、active sessions、logs、metrics、backup/restore 是自托管可信度核心。 |
| Plugins / addons / webhooks / automation | Webhooks 仍是官方能力；传统 plugin/channel 生态已被官方弱化/ sunset，更多依赖外部自动化和 Plex Pass 能力。 | Plugin catalog/repositories、官方/社区插件、Webhook plugin、REST API、scheduled tasks。 | 追赶 P1：Addon Protocol、Addon Sidecar、Webhook/Event、官方 Addon Catalog。明确不做 in-process native plugin ABI 和 Jellyfin plugin compatibility。 |
| Client ecosystem | Web、desktop、mobile、Apple TV、Android TV、Fire TV、Roku、Smart TV、game console、Plexamp、Plex Dash 等广泛官方客户端。 | Web、desktop、Android/iOS、Android TV/Fire TV、Roku、webOS/Tizen/Xbox/Kodi/Swiftfin/Infuse 等官方与社区客户端。 | 追赶路线：先稳定 Public Client API、Web/Media Web、mobile/shared core、capability profile，再扩 TV。延后广泛设备商店覆盖。 |

## 1. Library Onboarding / Scan / Metadata / Artwork

### Plex 观察

Plex 把媒体库视为用户 onboarding 的第一步：创建 Library 时选择类型、语言、文件夹和高级选项；之后提供手动扫描、刷新 metadata、自动扫描、定期扫描、空垃圾、匹配/修复匹配等管理动作。Plex 的公开文档还把命名规则、本地海报/背景/字幕等 Local Media Assets、agent 或 metadata 行为分散成用户可操作的指南。

产品意义：

- 用户不只需要“能扫目录”，还需要知道当前扫描在做什么、为什么某个条目没出现、为什么匹配错、怎么修复。
- Metadata 与 artwork 是 library onboarding 的一部分，不是后续可选 polish。
- Plex 的模型默认用户愿意接受 provider metadata 和云账号服务；Nako 应保留 Local Authority，把 NFO、sidecar、用户编辑和 provider 结果都展示为可解释的来源。

### Jellyfin 观察

Jellyfin 覆盖多种 library 类型，提供 metadata 下载、NFO、图片、本地 artwork、Identify/Refresh、扫描媒体库、插件化 metadata provider 等用户可见能力。它的强项是本地优先和可控性：文件命名、NFO、图片布局、metadata provider 的启用/禁用对自托管用户很重要。

产品意义：

- Nako 的 Video-First Phase 可以先窄后深，但不能让命名、NFO、本地 artwork、provider diagnostics 缺位。
- Jellyfin 用户预期已经包括“我能纠错”和“我能控制本地文件权威性”。

### Nako 分类

追赶 P0：

- Media Library 创建向导：Library Preset、Media Domain、Media Source 路径/Source Locator、扫描策略、metadata provider 策略、artwork 策略。
- 扫描可观测性：扫描队列、进度、失败条目、重试、跳过原因、Source tombstone、Source Duplicate Relationship 预览。
- Metadata 治理：Canonical Metadata 来源解释、Candidate Review、field lock、本地 NFO/sidecar 优先级、provider payload 诊断。
- Artwork 治理：poster/backdrop/logo/thumbnail 分类、来源、缓存状态、手动替换、失效刷新。

延后 P1/P2：

- 音乐、图片、书籍、文档等非视频域的同等深度。
- Provider 市场级广度，例如所有区域性 metadata provider 和 fanart provider。

明确不做：

- 把 provider/cloud metadata 作为不可解释的黑盒权威。
- 为了兼容 Jellyfin 插件而引入进程内 metadata agent ABI。

## 2. Playback / Transcode / Client Capability / Remote Access

### Plex 观察

Plex 的播放旅程以 Direct Play、Direct Stream、Transcode 的差异解释为核心，并把硬件加速、HDR to SDR tone mapping、带宽、远程质量、Relay、客户端能力和 Premium 功能放进用户可理解的配置页面。Plex 还提供 Media Optimizer，把常见转码结果预先生成成 optimized versions，降低实时转码压力。

产品意义：

- Plex 让普通用户理解“为什么这台设备要转码”。这是播放器 UX、服务端规划和诊断 UI 的共同问题。
- Plex 的远程访问体验很强，但它依赖 Plex 账号、集中服务、Relay 和 Premium/Pass 打包。Nako 可以借鉴用户旅程，不应复制商业/云控制面。

### Jellyfin 观察

Jellyfin 公开文档强调 Direct Play 优先、FFmpeg 转码、硬件加速配置、client codec support matrix、networking/reverse proxy。它不提供 Plex 式托管 relay，而是要求用户配置端口、HTTPS、reverse proxy、VPN 或类似网络方案。

产品意义：

- Jellyfin 证明自托管产品可以不做云 relay，但必须给出清晰的网络部署文档。
- 客户端 codec support matrix 是服务器决策可解释性的基础，Nako 的 Playback Source Selection 需要客户端主动上报能力，而不是服务端猜测。

### Nako 分类

追赶 P0：

- Direct Play first：先证明无需转码的播放路径稳定。
- Playback Source Selection：用 Media Source、Source Variant、client capability、user policy 生成可解释计划。
- Remux/HLS/Transcode：分清 container 改封装、音频转码、视频转码、字幕烧录、HDR tone mapping。
- Client capability profile：codec、container、subtitle、audio channel、HDR、network、screen、browser/player engine。
- 远程访问基础：reverse proxy、HTTPS、base URL、CORS、token redaction、range/HLS 访问、诊断检查。

追赶 P1：

- 硬件转码矩阵：Intel/AMD/NVIDIA/Apple/VAAPI/QSV/NVENC/VideoToolbox 这类主流能力的 inventory、admission control、operator diagnostics。
- Optimized Source Variant：把预转码输出作为 Nako-Managed Artifact，而不是 ad hoc 文件。

延后 P2：

- Plex Relay 等中心化穿透服务。
- 复杂 ABR、LL-HLS/CMAF、跨设备 Watch Together。

明确不做：

- Subscription-gated local playback 或 remote playback。
- 默认把用户流量导向 Nako 官方中转服务。
- 不可观测的后台转码进程。

## 3. User / Account / Sharing / Parental Controls

### Plex 观察

Plex 以 Plex account 为中心：服务器所有者、家庭成员、managed accounts、PIN、Library Access、sharing、restrictions、ratings/labels、下载权限等都围绕账号体系展开。优点是跨设备和外部分享顺滑，缺点是个人媒体服务器的身份边界依赖 Plex 服务。

### Jellyfin 观察

Jellyfin 使用本地用户模型，管理员可控制 library access、remote access、devices、parental controls、标签/评分限制等。它更符合纯自托管预期，但跨站点分享和社交体验不如 Plex 一体化。

### Nako 分类

追赶 P0：

- 本地用户、session auth、admin/operator 分离。
- Library Access：按 Media Library、Media Domain、rating、tag、collection、Source Variant 或未来 policy 控制。
- Parental controls：评分、标签、时间窗口、隐藏/允许规则、PIN 或快速切换用户。
- Active sessions：谁在看、看什么、从哪里、Direct Play/Transcode、带宽、可停止会话。

追赶 P1：

- 家庭成员/managed profile 的轻量模型。
- 分享邀请的受控版本，但不要绑定 Nako 云账号。

延后：

- Plex 式好友网络、公开资料页、跨服务器社交。

明确不做：

- 中心化 Nako account 作为本地服务器登录前提。
- 服务器之间的开放转分享链，除非未来有明确的信任与审计设计。

## 4. Live TV / DVR / Downloads / Sync / Optimized Versions

### Plex 观察

Plex 将 Live TV/DVR、EPG、录制、Downloads、Media Optimizer 打包成高可见、高留存能力。用户层面的价值是：电视源和本地媒体共用同一客户端生态，离线与预优化降低网络和转码成本。

### Jellyfin 观察

Jellyfin 有 Live TV/DVR、tuner/M3U、guide data、录制和录制后处理。离线下载/同步与 optimized versions 没有在官方文档中呈现为与 Plex Downloads/Media Optimizer 对等的一体化 server product，这一点是基于 2026-06-10 官方文档面向用户的能力呈现所作推断。

### Nako 分类

追赶 P1：

- Optimized Source Variant：复用 transcode runtime，生成有 manifest、生命周期、磁盘预算和删除策略的 Nako-Managed Artifact。
- Downloads/offline：必须先有 Public Client API、User Playback State、Device identity、artifact manifest、license-free storage policy 和冲突处理。

延后 P2：

- Live TV/DVR：涉及 tuner discovery、EPG、录制排程、冲突处理、转码/Direct Stream、保留策略、权限和客户端 UI，范围应独立规划。
- SyncPlay / Watch Together：需要 realtime/session 基础成熟。

明确不做：

- Plex 式广告流媒体、租赁/购买、商业节目聚合。
- 未经 operator 明确选择的 acquisition/download 自动化。

## 5. Admin / Diagnostics / Monitoring / Backup

### Plex 观察

Plex 的 Dashboard、active streams、带宽/历史、logs、backup/server data 文档提供了 operator 需要的基本答案：服务器是否健康、谁在使用、为什么转码、数据怎么迁移或恢复。

### Jellyfin 观察

Jellyfin 的 Admin Dashboard、scheduled tasks、logs、troubleshooting、backup/restore、monitoring/metrics 文档体现了开源自托管产品的运维基线。用户需要能看到任务、日志、ffmpeg 问题、路径配置和备份边界。

### Nako 分类

追赶 P0：

- Admin Overview：健康、版本、存储、数据库、FFmpeg、网络、jobs、队列、最近失败。
- Jobs：scan、probe、metadata、artwork、transcode、addon、webhook、backup 都应有统一 durable job 视图。
- Playback diagnostics：每个会话的 Direct Play/Remux/Transcode 原因、client capability、source fact、错误。
- Log bundles：可下载、可脱敏、带 trace context。
- Backup/restore：SQLite/PostgreSQL、配置、secrets reference、metadata/artwork cache、addon registrations、Nako-Managed Artifacts 策略。

延后 P1：

- Prometheus/Grafana 级可观测性模板。
- 多节点/远程 worker dashboard。

明确不做：

- 每个功能自己隐藏 scheduler、重试和日志。
- 只能靠数据库手工修复的 operator experience。

## 6. Plugins / Addons / Webhooks / Automation

### Plex 观察

Plex 的公开资料显示 Webhooks 是官方可见的自动化能力；传统 plugin/channel 生态已被官方弱化或 sunset。Plex 的成熟方向更像封闭核心产品加外部 webhook/automation，而不是开放服务器内插件平台。

### Jellyfin 观察

Jellyfin 有 plugin catalog/repositories、官方与社区插件、Webhook plugin、API 和 scheduled tasks。这对自托管用户很有吸引力，但也带来 ABI、版本兼容、server trust boundary 和故障隔离压力。

### Nako 分类

追赶 P1：

- Addon Protocol：manifest、resources、tasks、events、configuration schema、health、permissions。
- Addon Sidecar：out-of-process、scoped token、Secret Reference、grants、operator install guide。
- Webhooks/Event：签名、重试、delivery attempts、redaction、event schema version。
- Official Addon Catalog：只作为发现和安装指导，不把第三方代码塞入 Nako 进程。

延后：

- 一键托管 addon runtime。
- 大规模 marketplace、评分、支付、自动更新。

明确不做：

- Jellyfin plugin compatibility。
- Native dynamic library / in-process plugin ABI。
- Addon 私下轮询数据库或直接写 Library File。

## 7. Mobile / TV / Web Client Ecosystem

### Plex 观察

Plex 的客户端生态是最大护城河：Web、desktop、mobile、TV、streaming devices、game consoles、Plexamp、Plex Dash 等产品覆盖让服务器能力成为跨设备体验。

### Jellyfin 观察

Jellyfin 有 Web、desktop、Android/iOS、Android TV/Fire TV、Roku、webOS/Tizen/Xbox/Kodi/Swiftfin/Infuse 等官方与社区客户端。优点是广泛，缺点是 codec support、UI polish、离线能力和商店更新节奏不一致。

### Nako 分类

追赶 P0：

- Public Client API：浏览、搜索、播放票据、resume、progress、user state、artwork URLs、capability report。
- Media Web：作为能力闭环和 API 真实性测试，不只是 admin demo。
- Player diagnostics：客户端能解释“为什么转码/为什么失败”。

追赶 P1：

- Mobile/shared client core：离线、resume、download queue、push/realtime 的协议基础。
- TV-first navigation model：遥控器/焦点/大屏播放能力。

延后 P2：

- Roku/webOS/Tizen/game console 等多商店并行覆盖。
- Plexamp 级独立音乐体验，除非 Nako 音乐域进入正式阶段。

明确不做：

- 为追求设备数量而冻结后端 API 的错误抽象。
- 把 server roadmap 绑定到单个闭源客户端。

## Nako Prioritization

### P0：M1/近期必须追赶

1. Library onboarding：Library Preset、路径、扫描策略、metadata/artwork policy。
2. Scan diagnostics：扫描进度、失败原因、重试、repair action。
3. Metadata/artwork governance：Candidate Review、NFO/local authority、field locks、手动 artwork。
4. Playback core：Direct Play、Remux/HLS、Transcode fallback、client capability profile。
5. Remote access baseline：reverse proxy/HTTPS 文档、base URL、token/redaction、外部可达性诊断。
6. Local users/access：admin/operator、Library Access、parental controls 基础。
7. Admin diagnostics：Jobs、active sessions、logs、health checks、backup/restore runbook。
8. Public Client API + Media Web：用真实客户端闭环证明 server 能力。

### P1：追赶但可在 M1 后推进

1. Hardware transcode operator matrix 和资源 admission control。
2. Optimized Source Variant / Media Optimizer 等价物。
3. Downloads/offline sync 的 protocol-first 设计。
4. Webhooks/Event delivery 和 Addon Protocol productization。
5. Managed profile / household profile。
6. Monitoring metrics 与可下载诊断包。
7. TV client foundation。

### P2：延后，等核心闭环稳定

1. Live TV/DVR。
2. Watch Together / SyncPlay。
3. 广泛智能电视和游戏主机客户端。
4. 多节点远程转码 workers。
5. Marketplace 级 Addon 生态。
6. 非视频域同等深度，尤其音乐/图片/书籍/文档。

### 明确不做

1. Plex 式中心化账号作为本地服务器登录前提。
2. Subscription-gated local/remote playback。
3. 官方流量 relay 作为默认远程访问路径。
4. 广告流媒体、租赁/购买、商业内容聚合。
5. Jellyfin plugin compatibility 或 in-process plugin ABI。
6. Addon 直接数据库写入或绕过 Nako 授权写 Library File。
7. 不可观测的后台任务、转码进程、扫描器或 provider mutation。

## 对 Nako 架构的直接启发

产品追赶不等于功能堆叠。Plex/Jellyfin 的共同点是用户能完成闭环：导入、识别、浏览、播放、修复、分享或限制、诊断。Nako 已有很多后端边界，但每个边界都需要对应 operator/user 可见面，否则用户会感觉“服务器在工作，但不知道发生了什么”。

建议把下一个产品化视角压到三条验收线上：

1. 新管理员 30 分钟闭环：安装、创建 Media Library、扫描、修复一个匹配、播放一个视频、查看 Dashboard、备份配置。
2. 播放可解释闭环：任一播放失败或转码都能回答 Source、client capability、policy、FFmpeg/runtime、network 五类原因。
3. 扩展可控闭环：Addon/Webhook 能发现、授权、执行、失败重试、撤销，并且不会突破 Nako trust boundary。

## Sources

所有来源均为公开资料，访问日期均为 2026-06-10。

### Plex

| 主题 | URL |
| --- | --- |
| Quick start / setup guides | https://support.plex.tv/articles/200264746-quick-start-step-by-step-guides/ |
| Creating libraries | https://support.plex.tv/articles/200288926-creating-libraries/ |
| Scanning vs refreshing a library | https://support.plex.tv/articles/200289306-scanning-vs-refreshing-a-library/ |
| Editing libraries | https://support.plex.tv/articles/200289266-editing-libraries/ |
| Naming and organizing TV files | https://support.plex.tv/articles/naming-and-organizing-your-tv-show-files/ |
| Local Media Assets for movies | https://support.plex.tv/articles/200220677-local-media-assets-movies/ |
| Direct Play and Direct Stream | https://support.plex.tv/articles/200250387-streaming-media-direct-play-and-direct-stream/ |
| Transcoding media | https://support.plex.tv/articles/200250377-transcoding-media/ |
| Hardware-accelerated streaming | https://support.plex.tv/articles/115002178853-using-hardware-accelerated-streaming/ |
| HDR to SDR tone mapping | https://support.plex.tv/articles/hdr-to-sdr-tone-mapping/ |
| Remote access | https://support.plex.tv/articles/200289506-remote-access/ |
| Relay | https://support.plex.tv/articles/216766168-accessing-a-server-through-relay/ |
| Remote playback requirements | https://support.plex.tv/articles/requirements-for-remote-playback-of-personal-media/ |
| Plex Pass feature overview | https://support.plex.tv/articles/201751006-plex-pass-feature-overview/ |
| Downloads overview | https://support.plex.tv/articles/201018507-downloads-overview/ |
| Media Optimizer overview | https://support.plex.tv/articles/214079318-media-optimizer-overview/ |
| Live TV & DVR FAQ | https://support.plex.tv/articles/226463767-frequently-asked-questions-dvr-live-tv/ |
| Setting up DVR recordings | https://support.plex.tv/articles/226074728-setting-up-recordings/ |
| Supported DVR tuners and antennas | https://support.plex.tv/articles/225877427-supported-dvr-tuners-and-antennas/ |
| Plex Home | https://support.plex.tv/articles/200288286-what-is-plex-home/ |
| Creating a Plex Home | https://support.plex.tv/articles/204234323-creating-a-plex-home/ |
| Managing library access / shares | https://support.plex.tv/articles/201105738-creating-and-managing-server-shares/ |
| Restricting library access | https://support.plex.tv/articles/204232573-restricting-the-shares/ |
| Status and dashboard | https://support.plex.tv/articles/200871837-status-and-dashboard/ |
| Plex Media Server log files | https://support.plex.tv/articles/200250417-plex-media-server-log-files/ |
| Backing up Plex Media Server data | https://support.plex.tv/articles/201539237-backing-up-plex-media-server-data/ |
| Webhooks | https://support.plex.tv/articles/115002267687-webhooks/ |
| Plugin/channel sunset context | https://www.plex.tv/blog/subtitles-and-sunsets-big-improvements-little-housekeeping/ |
| Plex apps and devices | https://www.plex.tv/apps-devices/ |
| Plex Media Server downloads | https://www.plex.tv/media-server-downloads/ |

### Jellyfin

| 主题 | URL |
| --- | --- |
| Libraries | https://jellyfin.org/docs/general/server/libraries/ |
| Movie media organization | https://jellyfin.org/docs/general/server/media/movies/ |
| TV show media organization | https://jellyfin.org/docs/general/server/media/shows/ |
| External files / local sidecars | https://jellyfin.org/docs/general/server/media/external-files/ |
| Metadata overview | https://jellyfin.org/docs/general/server/metadata/ |
| NFO metadata | https://jellyfin.org/docs/general/server/metadata/nfo/ |
| Transcoding | https://jellyfin.org/docs/general/server/transcoding/ |
| Hardware acceleration | https://jellyfin.org/docs/general/administration/hardware-acceleration/ |
| Client codec support | https://jellyfin.org/docs/general/clients/codec-support/ |
| Networking | https://jellyfin.org/docs/general/networking/ |
| Reverse proxy | https://jellyfin.org/docs/general/networking/reverse-proxy/ |
| Users | https://jellyfin.org/docs/general/server/users/ |
| Live TV | https://jellyfin.org/docs/general/server/live-tv/ |
| Live TV setup guide | https://jellyfin.org/docs/general/server/live-tv/setup-guide/ |
| Live TV post-processing | https://jellyfin.org/docs/general/server/live-tv/post-process/ |
| Scheduled tasks | https://jellyfin.org/docs/general/server/tasks/ |
| Troubleshooting | https://jellyfin.org/docs/general/administration/troubleshooting/ |
| Logging | https://jellyfin.org/docs/general/administration/logging/ |
| Backup and restore | https://jellyfin.org/docs/general/administration/backup-and-restore/ |
| Monitoring | https://jellyfin.org/docs/general/post-install/networking/advanced/monitoring/ |
| Plugins | https://jellyfin.org/docs/general/server/plugins/ |
| Jellyfin API | https://api.jellyfin.org/ |
| Jellyfin Webhook plugin repository | https://github.com/jellyfin/jellyfin-plugin-webhook |
| Clients overview | https://jellyfin.org/docs/general/clients/ |
| Client downloads | https://jellyfin.org/downloads/clients/ |

### Nako Context Checked

| 主题 | 路径 |
| --- | --- |
| Domain vocabulary | `CONTEXT.md` |
| Architecture map | `docs/ARCHITECTURE.md` |
| Current architecture lanes | `docs/architecture/LANES.md` |
| Task PRD | `.trellis/tasks/06-10-media-server-gap-analysis/prd.md` |
