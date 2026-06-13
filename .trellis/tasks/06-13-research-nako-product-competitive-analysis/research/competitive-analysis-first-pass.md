# Nako 竞品分析与调研：第一版判断

- Date: 2026-06-13
- Scope: Nako 当前仓库、`nako-official-addons`、本地 Jellyfin 参考仓库、官方/公开竞品资料
- Output role: 第一版产品判断，不替代后续逐项深研

## Executive Takeaway

Nako 不应该把目标定义成“Rust 版 Jellyfin”。更好的定位是：

> 面向重视自托管、可检查、数据可携带和安全扩展边界的用户，做一个 Jellyfin/Plex-class 的媒体后端与控制面；播放体验要达到基线，但真正差异化来自本地权威、可解释的媒体管线、Addon Sidecar 安全模型、资源预算和 operator-grade diagnostics。

这个定位会自然避开两种陷阱：

- 与 Plex 在客户端覆盖、远程账号体系、商业内容入口上正面硬拼。
- 与 Jellyfin 在 in-process plugin catalog 和既有客户端生态上做逐项复刻。

Nako 的机会在第三条路：把媒体服务器拆成更清晰的 host-owned workflows，让 Addon 负责发现、建议、执行受控动作，但让 Nako 保留权限、审计、持久化、文件写入、播放 runtime 和诊断权威。

## Market Map

### 1. 核心媒体服务器

| 产品 | 竞品角色 | 对 Nako 的意义 |
| --- | --- | --- |
| Jellyfin | 开源自托管基线 | 客户端、插件目录、Live TV、元数据、转码和社区生态是用户会默认比较的对象。 |
| Plex | 商业体验标杆 | 客户端覆盖、远程访问、家庭分享、Plex Pass、Plexamp/Dash 等塑造“好用”的期待。 |
| Emby | 商业 freemium 对照 | 功能门控、Premiere、Live TV、硬件转码、备份/恢复和客户端限制适合作为商业边界参照。 |

关键事实：Jellyfin 插件文档显示其插件覆盖 Authentication、Channels、Live TV、Metadata、Notifications、Reports、Subtitle 等类别，并支持 catalog/manual install；这意味着 Nako 的 Addon 生态不能只停在 metadata scraper。Plex 2025 以后把个人视频远程播放纳入 Plex Pass 或 Remote Watch Pass 条件，这让“纯自托管、无中心账号依赖的远程访问策略”成为 Nako 的潜在差异化，但也意味着 Nako 必须提供清晰的反代/VPN/Tailscale/Cloudflare Tunnel 指南。Emby Premiere 把硬件转码、HDR tone mapping、DVR、LDAP、下载同步、webhook、backup 等列为商业功能矩阵中的关键项，说明这些能力在成熟用户心智里不是边缘功能。

### 2. 客户端与 Addon 协议参照

Kodi 是 living-room/client UX 和 addon 生态参照，不是 server-only 直接竞品。Stremio 则证明了 catalog/meta/stream/subtitle addon 可以成为产品中心，但它也把 legal/policy boundary 推到最前线。Nako 如果做 Resource Search、External Acquisition、online catalog，必须把只读搜索、link check、用户选择、host-owned materialization、runner action 分层讲清楚。

### 3. 垂直媒体域

Immich 的 v2.0 stable 说明照片/视频自托管工具已经进入“现代产品体验 + 移动端 + ML/search + 社区规模”的阶段。Navidrome 证明轻量音乐服务器可以靠 OpenSubsonic API 和客户端生态长期生存。Audiobookshelf 说明 audiobook/podcast 有独立的章节、进度、离线和移动体验深度。

这对 Nako 的含义是：`Media Server Scope` 保留 video/audio/image/document/mixed/online 是对的，但产品路线不能靠一个泛化 `Media Item` UI 糊过去。每个媒体域都需要明确的 domain depth gate。

### 4. 自托管媒体生态组件

Servarr、Bazarr、Seerr、Kometa、Tdarr/FileFlows/Unmanic、Tautulli/Jellystat、Maintainerr、Tunarr/ErsatzTV/Dispatcharr、WatchState 等不是“媒体服务器”，但它们定义了真实用户的媒体栈。

Nako 如果只和 Jellyfin/Plex 功能表对齐，会漏掉用户真正的日常工作流：请求、审批、自动抓取、字幕补齐、重封装、清理、统计、跨系统 watched state、虚拟频道。第一版产品战略应该把这些看成生态集成对象，而不是全部内建。

## Nako 当前优势

### 1. 架构边界更现代

Nako 已经把 domain、persistence、VFS、library pipeline、metadata、playback planning、transcode runtime、events、automation、addon protocol、server composition 拆成 Rust workspace crate。这个边界比“先做功能，后清理架构”更适合长期扩展。

核心原则已经成形：Direct Play first、Planner before runtime、Manifest-backed artifacts、FFmpeg CLI first、Resource budgets as product behavior、Local authority explicit、Addons out-of-process、Control plane explicit。

### 2. Addon Sidecar 是真差异化

Jellyfin 的插件体验成熟，但本地源码观察显示其插件是 in-process assembly 模型，能装载、配置、暴露页面并直接进入宿主对象模型。Nako 的 Addon Sidecar 模型更适合把权限、崩溃隔离、协议版本、library-scoped grants、side effects、health check 和诊断做成一等契约。

这不是“插件能力弱”，而是产品取舍不同：Nako 应该把它包装成“可扩展，但核心进程和媒体库写入权不交给任意插件”。

### 3. 本地权威和可携带性更符合收藏用户

Nako 已把 NFO import/export、NFO round trip、本地推断证据、Provider Mapping、Metadata Source Priority、Managed Artwork、Library File Write 建成明确语言。对重视个人收藏可迁移的用户，这是和 Plex 云账号体验、Jellyfin provider/plugin 混合模型不同的卖点。

### 4. Operator diagnostics 有机会领先

Nako 已把 storage health、VFS cache repair、durable jobs、playback runtime diagnostics、addon health/grants、release gate、hardware report 等放进控制面。成熟媒体服务器的实际痛点常常不是“有没有功能”，而是“坏了以后怎么知道为什么坏”。这里是 Nako 值得继续压强的方向。

## Nako 当前短板

### 1. 客户端和首屏体验不足

Jellyfin/Plex 用户首先感知的是 TV/mobile/web player，而 Nako 目前更像 backend/control-plane alpha。后端能力强，但如果 Web/Android/TV/casting 的首轮体验不顺，用户不会感知架构价值。

### 2. Addon 生态入口还不够产品化

官方 addons 已有 7 个 sidecar 和 browser-worker，但安装仍偏工程化：manifest、Dockerfile、Compose、smoke、手动注册。缺 marketplace/catalog UI、版本兼容矩阵、Addon Suite 分发、签名/更新/rollback、第三方模板和 conformance gate。

### 3. Jellyfin parity 缺口明显

Nako 需要优先补足用户一眼能识别的生态类别：OpenSubtitles/live subtitle provider、Trakt/scrobble、LDAP/SSO/OIDC、playback reporting/reports、Live TV/tuner、local intros/preroll、skin/theme 或至少 client customization、books/music/photo provider depth。

### 4. Remote access 还只是边界和文档方向

Plex 的远程播放订阅化给了 Nako 差异化机会，但也提高了用户期待。Nako 不需要内建 relay，但必须提供端到端、安全、可诊断的远程访问 cookbook 和 endpoint discovery 模型。

### 5. 高风险生态需要更强 policy

Resource Search、external acquisition、yt-dlp、IPTV、torrent/debrid、成人/AV provider、browser-rendered scraping 都可能带来 ToS、隐私、滥用和运营风险。Nako 现在的 host-owned flow 是正确方向，但需要产品层显式分 trust tier、默认关闭策略、secret handling、redaction 和审计。

## Recommended Positioning

### Primary Positioning

Nako 应定位为：

> 可审计、可扩展、数据可携带的自托管媒体服务器后端，优先服务重视本地收藏、元数据治理、插件安全边界和运维可诊断性的用户。

不要主打“比 Plex 更商业化体验好”或“比 Jellyfin 插件更多”。短期更可信的说法是：

- 比 Plex 更少中心账号和功能门控依赖。
- 比 Jellyfin 更明确插件信任边界和 host-owned side effects。
- 比普通自托管工具更重视资源预算、失败诊断和可恢复工作流。

### Target Segments

优先用户：

- 有 NAS/家庭服务器经验，愿意用 Docker/Compose。
- 有 Jellyfin/Plex/Emby 使用经验，但对远程账号依赖、插件安全、元数据混乱或迁移成本不满意。
- 动画、亚洲影视、AV、本地 NFO、字幕、多 provider 元数据需求强的收藏用户。
- 想把媒体库治理、资源搜索、下载执行、字幕、通知、casting 拆成可审计工作流的 power user。

暂不优先：

- 只想要开箱即用 TV app 的普通家庭用户。
- 依赖 Plex 社交/云端 discovery/商业流媒体聚合的用户。
- 不愿处理反代、VPN、端口、Docker 的用户。

## 12-18 个月建议路线

1. **MVP polish：先证明“一个 operator 能稳定用起来”**
   - 强化 Web/Android 基本浏览和播放。
   - 做远程访问 cookbook 和诊断。
   - 把 release ladder 变成用户可理解的安装/升级信心。

2. **Addon ecosystem：先 catalog，再 manager**
   - 生成 official addon catalog。
   - 扩容器 smoke 覆盖 7 个 Addon + browser-worker。
   - 做官方 Addon Suite Compose。
   - 提供 Rust/TypeScript/Python author kit。
   - 暂缓自动 lifecycle manager，避免过早接管 Docker/systemd。

3. **Jellyfin-visible Addon gaps**
   - OpenSubtitles live provider。
   - Trakt/scrobble。
   - LDAP/OIDC auth bridge。
   - Playback reporting/reports。
   - Live TV/tuner 或先接入 Tunarr/ErsatzTV/Dispatcharr。

4. **Interoperability and migration**
   - NFO/artwork/provider ID import/export 文档化。
   - watched state migration/sync 对接 WatchState 类工具。
   - Plex/Jellyfin/Emby library import mapping 研究。

5. **Policy-first resource workflows**
   - Resource Search -> Selection -> Acquisition Intake -> Materialization -> Runner 画成产品流程图。
   - 定义 provider trust tiers。
   - 为 high-risk addons 建默认关闭、权限提示、审计和 redaction gate。

## Differentiation Matrix

| 维度 | Jellyfin | Plex | Nako 应采取的方向 |
| --- | --- | --- | --- |
| 核心价值 | 免费开源媒体服务器 | polished commercial media platform | 可审计、自托管、host-owned workflows |
| 插件/扩展 | 成熟 plugin catalog，进程内模型 | 历史 plugin 弱化，utility 外部化 | Sidecar Addon + scoped grants + health/diagnostics |
| 远程访问 | 自行反代/VPN/网络配置 | 中心账号和付费远程播放条件 | 不做 relay 承诺，做好 cookbook + endpoint discovery |
| 元数据 | provider/plugin 丰富 | 自动化和商业体验强 | local/NFO authority + provider-neutral mapping + review/apply |
| 播放 | 成熟客户端和转码 | 客户端覆盖最强 | Direct Play first + typed planning + clear diagnostics |
| 运维 | 社区成熟，但复杂故障仍需经验 | 用户体验强，控制权较少 | admin diagnostics / release gates / redaction-safe repair |
| 数据可携带 | 较好，依赖实践 | 相对平台化 | NFO/artwork/provider IDs/user state portability 作为卖点 |

## Deep Research Backlog

- 核心服务器对比：Jellyfin / Plex / Emby 的客户端、远程访问、转码、Live TV、插件、数据迁移。
- Addon/Plugin 生态：Jellyfin plugin catalog、Stremio addon protocol、Plex plugin 退场、Nako Addon Sidecar。
- 自托管媒体工作流：Servarr、Bazarr、Seerr、Kometa、Tdarr/FileFlows、Tautulli/Jellystat、Maintainerr。
- 媒体域扩展：Immich、PhotoPrism、Navidrome、Audiobookshelf、Kavita/Komga。
- 合规和高风险资源：Stremio、IPTV、yt-dlp、PanSou-compatible、torrent/debrid、AV providers。
- 迁移和互操作：NFO、artwork、provider IDs、watched state、playlists、collections、user ratings。

## Sources And Local Evidence

Local evidence:

- `research/nako-current-state.md`
- `research/official-addons-current-state.md`
- `research/jellyfin-source-architecture-notes.md`
- `research/external-competitive-ecosystem-supplement.md`
- `README.md`
- `CONTEXT.md`
- `docs/ARCHITECTURE.md`
- `docs/architecture/*.md`
- `docs/guides/ADDON_AUTHOR_GUIDE.md`
- `../nako-official-addons/README.md`

External source highlights:

- Jellyfin plugin catalog and categories: <https://jellyfin.org/docs/general/server/plugins/>
- Jellyfin 10.11.0 release notes and upgrade caveats: <https://jellyfin.org/posts/jellyfin-release-10.11.0/>
- Plex 2025 remote playback change: <https://www.plex.tv/blog/important-2025-plex-updates/>
- Plex remote playback requirements: <https://support.plex.tv/articles/requirements-for-remote-playback-of-personal-media/>
- Plex Pass feature/pricing page: <https://www.plex.tv/plans/>
- Emby Premiere feature matrix: <https://emby.media/support/articles/Premiere-Feature-Matrix.html>
- Stremio Addon SDK: <https://www.stremio.com/addon-sdk>
- Immich v2.0 stable release: <https://immich.app/blog/stable-release>
- Navidrome product page: <https://www.navidrome.org/>
- Audiobookshelf product page: <https://www.audiobookshelf.org/>
- Servarr wiki: <https://wiki.servarr.com/>
- Seerr product page: <https://seerr.dev/>
