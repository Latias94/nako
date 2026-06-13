# Research: Nako 当前状态

- Query: Nako 当前代码和文档呈现出的产品定位、能力边界、架构优势、MVP 缺口。
- Scope: internal
- Date: 2026-06-13

## Findings

### 1. 当前产品定位摘要

Nako 当前定位是“开源、自托管媒体服务器后端”，面向希望在自有硬件上管理电影、剧集、动画和个人收藏的用户。README 明确写出产品标语 “Your media home, gently kept.”，并说明 Nako 是给自托管用户使用的 media server backend（`README.md:16`、`README.md:18`-`README.md:19`）。

当前发布状态仍是 `0.1.0-alpha.2` 技术预览。README 明确说它适合开发、自托管测试和早期 Addon 工作，但还不是稳定的 Jellyfin 或 Plex 替代品，且 Public API、Admin API、Addon Protocol、数据库 schema、生成 SDK 在 beta 前仍可能变化（`README.md:21`-`README.md:28`）。

产品北极星是 Jellyfin/Plex 级别的系统，但不是复制 Jellyfin/Plex 内部结构，也不急于接受 native plugin ABI。架构总览把长期目标定义为“Jellyfin/Plex-class system”，同时强调自托管、可检查、易扩展和避免过早 native 插件 ABI（`docs/ARCHITECTURE.md:5`-`docs/ARCHITECTURE.md:8`）。

当前阶段是 video-first，而不是永久 video-only。`CONTEXT.md` 定义 Nako 的长期 Media Server Scope 包括 video、audio、image、document、mixed、online media；同时当前 Video-First Phase 优先 movie、series、anime、home-video、playback、metadata、transcode 工作流（`CONTEXT.md:115`-`CONTEXT.md:121`）。

当前路线图的近期产品锚点是 M1 Product-Operator：单个自托管 operator 能配置一个 Media Library、扫描和索引媒体、浏览 catalog、播放视频，并从 Admin surface 诊断或修复常见失败。路线图强调后端 release ladder 是质量门禁，不是产品定义（`docs/ROADMAP.md:14`-`docs/ROADMAP.md:22`）。

### 2. 已实现能力

README 当前能力列表显示，Nako 已经有比较完整的后端技术预览面：媒体库扫描、本地推断、Source State、Media Source、Provisional Hierarchy；SQLite/PostgreSQL 持久化路径和 contract tests；本地 VFS 与 WebDAV-oriented pieces；TMDB/Bangumi/Douban provider runtime；NFO import/export；Managed Import、Nako-managed artwork、Library File Write、promotion/apply flows；播放源选择、remux/transcode planning、硬件加速策略、runtime diagnostics、bounded staging；Admin API/Admin Web diagnostics、operations、Addon onboarding、credentials、grants、runtime status；Addon Sidecar protocol；Docker/Compose、config preflight、release packaging 和 operator docs（`README.md:33`-`README.md:50`）。

代码层面也能看到这些边界已经存在，而不是纯路线图文字：

- Workspace 是 Rust modular monolith，`Cargo.toml` 使用 `resolver = "3"`、`members = ["crates/*"]`，版本 `0.1.0-alpha.2`，Rust `1.95`，描述为自托管 media home（`Cargo.toml:1`-`Cargo.toml:10`）。
- `nako-core` 聚合 domain records、identity、job、media、playback_policy、renderer、repository、storage_health、user_playback、webhook 等核心模块（`crates/nako-core/src/lib.rs:1`-`crates/nako-core/src/lib.rs:27`）。
- `nako-db` 同时有 `postgres` 和 `sqlite` adapter/facade，并暴露 `NakoDatabase`、backend capabilities 和 SQLite runtime options（`crates/nako-db/src/lib.rs:1`-`crates/nako-db/src/lib.rs:10`）。
- `nako-api` 明确分离 `admin`、`admin_contract`、`openapi`、`public_client`、`sdk` 模块，并有测试常量防止 provider governance / raw provider / source fingerprint 等敏感或 admin-only 概念泄漏到公共契约（`crates/nako-api/src/lib.rs:3`-`crates/nako-api/src/lib.rs:9`、`crates/nako-api/src/lib.rs:12`-`crates/nako-api/src/lib.rs:66`）。
- `nako-vfs` 已有 `StorageUri`、`StorageCapabilities`、local/WebDAV/cache 模块和可选 OpenDAL proof adapter；能力位覆盖 seekable、range-readable、watchable、linkable、writable、expensive listing、rate limited、remote latency（`crates/nako-vfs/src/lib.rs:17`-`crates/nako-vfs/src/lib.rs:29`、`crates/nako-vfs/src/lib.rs:31`-`crates/nako-vfs/src/lib.rs:109`）。
- `nako-playback` 暴露 Direct Play、Remux、Transcode、Denied 四种模式，PlaybackDecision 包含 mode、reason、selected_source、rendition、report、denial，并把请求输入拆成 source、probe、target、effective policy 和 selection context（`crates/nako-playback/src/lib.rs:31`-`crates/nako-playback/src/lib.rs:39`、`crates/nako-playback/src/lib.rs:83`-`crates/nako-playback/src/lib.rs:90`、`crates/nako-playback/src/lib.rs:138`-`crates/nako-playback/src/lib.rs:145`）。
- `nako-transcode` 已分出 artifact、engine、execution、ffmpeg、hardware、hls、pipeline、plan、policy、probe、profile、remux、runtime 等模块（`crates/nako-transcode/src/lib.rs:1`-`crates/nako-transcode/src/lib.rs:30`）。
- `nako-addon-protocol` 当前协议版本为 `0.1.0-alpha.1`，定义 runtime routes、manifest、install descriptor、hosted pages、configuration schema、secret references、event subscriptions、tasks、scopes，并把 runtime kind 限定为 HTTP sidecar（`crates/nako-addon-protocol/src/lib.rs:8`-`crates/nako-addon-protocol/src/lib.rs:39`、`crates/nako-addon-protocol/src/lib.rs:75`-`crates/nako-addon-protocol/src/lib.rs:126`、`crates/nako-addon-protocol/src/lib.rs:133`-`crates/nako-addon-protocol/src/lib.rs:189`）。
- `nako-server` router 组合了 system、account、admin、library、catalog、metadata、playback、renderer、user_playlist、user_playback、webhooks、automation、addons、jobs 等 protected routes，并单独挂载 addon runtime routes、API version header、network boundary、trace context（`crates/nako-server/src/http.rs:28`-`crates/nako-server/src/http.rs:70`）。
- `nako-server` app root 拆出 access、addons、artwork、automation、catalog、jobs、library、metadata、nfo、playback、renderer、runtime、source_duplicate、source_hash、storage、webhooks 等应用服务模块（`crates/nako-server/src/app.rs:30`-`crates/nako-server/src/app.rs:67`）。
- `nako-server` playback app 已经进一步拆出 direct、hls、hls_artifact、hls_flow、input、playlist、remux、remux_flow、renderer_flow、resource、runtime_session、selection、staging_policy、support 等模块，并通过 `PlaybackRuntimeStore` 把 media source/probe、policy、playback session、transcode session、outbox 访问统一起来（`crates/nako-server/src/app/playback/mod.rs:40`-`crates/nako-server/src/app/playback/mod.rs:58`、`crates/nako-server/src/app/playback/mod.rs:87`-`crates/nako-server/src/app/playback/mod.rs:184`）。

从架构进度看，文档对“已实现”有更细分的成熟度判断：

- 顶层系统图认为 domain model、persistence、storage/VFS、scan、metadata、playback planning、browser transport、user state、addons/automation、control plane、deployment 都至少有 good/strong foundation；clients 是 mixed（`docs/ARCHITECTURE.md:80`-`docs/ARCHITECTURE.md:94`）。
- Library pipeline 中 Durable scan state、Local inference、Media probe、NFO authority、Metadata merge policy、TMDB/Douban/Bangumi provider、Addon-assisted metadata、Artwork lifecycle 均显示为 shipped 或 shipped foundation；watcher/debounce 仍 weak（`docs/architecture/LIBRARY_PIPELINE.md:22`-`docs/architecture/LIBRARY_PIPELINE.md:38`）。
- Playback 中 Direct Play byte ranges、Remux/Direct Stream、Playback decision model、browser playback tickets、renderer transport tickets、FFmpeg command planning、hardware detection/fallback、HLS MPEG-TS/fMP4/adaptive ladder、audio/subtitle sidecar、seek/restart first slice、progressive runtime、HDR tone mapping first slice、audio downmix/normalization first slice、runtime scheduler first slice、Web player first slice都有 shipped/partial 状态（`docs/architecture/PLAYBACK.md:28`-`docs/architecture/PLAYBACK.md:57`）。
- Storage/VFS 中 local backend、remote storage boundary、source locator/fingerprint、remote probe staging、remote FFmpeg input staging、VFS cache diagnostics/repair foundations、mount hang circuit foundation 均有已发货基础；WebDAV read path、library file writes 仍是 partial（`docs/architecture/STORAGE_VFS.md:22`-`docs/architecture/STORAGE_VFS.md:35`）。
- Control plane 中 HTTP addon protocol、addon token/grant scopes、durable jobs、runtime supervisor、resource classes/budgets、API version/error/page contracts 具备 shipped foundation；tracing/Admin diagnostics/HTTP cache/N+1 discipline 是 partial；crash bundle、endpoint discovery 尚未开始（`docs/architecture/CONTROL_PLANE.md:36`-`docs/architecture/CONTROL_PLANE.md:55`）。
- State/access 中 SQLite default、PostgreSQL-ready boundary、repository traits、local credential auth、library access、playback policy、user playback progress、transcode/playback sessions、search projection、event outbox 都有 shipped 或 foundation（`docs/architecture/STATE_ACCESS.md:19`-`docs/architecture/STATE_ACCESS.md:33`）。
- Operations/release 中 self-hosted docs、backup/restore docs、release checklist、release gate scripts、PostgreSQL contract harness、FFmpeg/ffprobe config、hardware readiness diagnostics 已有 shipped foundation 或 report baseline；runtime budgets、config mutation authority、observability 是 partial（`docs/architecture/OPERATIONS_RELEASE.md:20`-`docs/architecture/OPERATIONS_RELEASE.md:34`）。

### 3. 关键架构选择

Nako 的主要架构选择是“模块化单体 + 明确 crate 边界”。顶层架构的 North Star 要求可以索引大规模本地/远程媒体库、合并本地/NFO/provider/addon 元数据、基于 source/client/user facts 选择 Direct Play/Remux/Transcode、用 typed planning 管理 FFmpeg 硬件加速、通过 addon/webhook/automation surface 扩展而不信任任意进程内代码，并保持小规模部署简单、大规模部署可迁移 PostgreSQL 和外部服务（`docs/ARCHITECTURE.md:38`-`docs/ARCHITECTURE.md:53`）。

几个最重要的架构原则：

- Direct Play first：转码是 fallback，不是默认路径（`docs/ARCHITECTURE.md:55`-`docs/ARCHITECTURE.md:58`）。
- Planner before runtime：播放、转码、元数据、addon 决策先形成 typed plan，再变成进程、SQL、URL 或外部调用（`docs/ARCHITECTURE.md:57`-`docs/ARCHITECTURE.md:60`）。
- Manifest-backed artifacts：播放和 managed artifact URL 需要 typed manifest 证明生成、可服务且安全暴露（`docs/ARCHITECTURE.md:61`-`docs/ARCHITECTURE.md:63`）。
- FFmpeg CLI first：Rust 负责规划、监督、生命周期、策略和 serving，FFmpeg/ffprobe 负责解码、编码、mux、probe 和 HLS/DASH-like 输出（`docs/ARCHITECTURE.md:64`-`docs/ARCHITECTURE.md:67`）。
- Resource budgets 是产品行为：CPU/GPU/disk/network/staging/addon/webhook/scan work 必须 bounded 且 observable（`docs/ARCHITECTURE.md:68`-`docs/ARCHITECTURE.md:69`）。
- Local authority explicit：NFO、sidecars、用户编辑和字段锁是自托管产品的一等能力，不是 provider accident（`docs/ARCHITECTURE.md:70`-`docs/ARCHITECTURE.md:71`）。
- Addons out-of-process：Addon sidecars 通过 scoped HTTP API 和 token 通信，不采用进程内 plugin ABI（`docs/ARCHITECTURE.md:72`-`docs/ARCHITECTURE.md:73`）。
- Control plane explicit：durable jobs、runtime supervision、diagnostics、remote access、addon lifecycle、API scale 归共享 control-plane，而不是 feature helper 私藏（`docs/ARCHITECTURE.md:74`-`docs/ARCHITECTURE.md:76`）。

这些原则已经反映在 Trellis spec 中：

- `nako-server` 是 composition、app orchestration、runtime-supervision、HTTP boundary crate；handler 应保持薄，业务流程放到 app services（`.trellis/spec/nako-server/backend/directory-structure.md:3`-`.trellis/spec/nako-server/backend/directory-structure.md:5`、`.trellis/spec/nako-server/backend/directory-structure.md:22`-`.trellis/spec/nako-server/backend/directory-structure.md:30`）。
- `nako-server` spec 明确 long-running scan、metadata、playback、addon、webhook、artifact workflows 必须使用 durable job/runtime boundaries，而不是 raw `tokio::spawn`（`.trellis/spec/nako-server/backend/directory-structure.md:36`-`.trellis/spec/nako-server/backend/directory-structure.md:42`）。
- `nako-playback` 是纯播放规划边界，只从 source、probe、target、policy、storage、preference facts 选择 Direct Play、Remux、Transcode 或 Denied，不执行 FFmpeg、不服务字节（`.trellis/spec/nako-playback/backend/index.md:3`-`.trellis/spec/nako-playback/backend/index.md:6`）。
- `nako-transcode` 专注 FFmpeg command planning、remux/HLS/transcode artifact modeling、hardware capability inventory、runtime primitives（`.trellis/spec/nako-transcode/backend/index.md:3`-`.trellis/spec/nako-transcode/backend/index.md:5`）。
- `nako-api` 要求 Admin 和 Public contract 分离，Public Client API 从 `nako-api` 生成，不从 Axum handler 反推；Admin `/admin/v1/*` 不进入 Public SDK（`.trellis/spec/nako-api/backend/admin-and-public-contracts.md:17`-`.trellis/spec/nako-api/backend/admin-and-public-contracts.md:31`）。
- `nako-db` 明确 SQLite 是默认 runtime target，PostgreSQL parity 通过 adapter shape 和 contract tests 保持（`.trellis/spec/nako-db/backend/index.md:3`-`.trellis/spec/nako-db/backend/index.md:5`）。

### 4. 与 Jellyfin/Plex 类产品对比时可作为差异化的点

1. 自托管、可检查、backend-first 的产品气质。Nako 不把目标写成“马上替代 Jellyfin/Plex”，而是先建立一个可检查、可扩展、边界清晰的后端基础，再逐步靠 M1/M2/M3/M4/M5 完成产品体验（`README.md:23`-`README.md:28`、`docs/ROADMAP.md:27`-`docs/ROADMAP.md:36`）。

2. 本地权威优先。NFO、sidecars、用户编辑、字段锁、本地推断证据、NFO round trip 被作为一等产品模型，适合重视本地收藏可迁移性和可解释性的用户。`CONTEXT.md` 明确 Metadata Source Priority 默认 local/NFO 优先于 external providers，并要求 NFO Export opt-in 且避免破坏性重写（`CONTEXT.md:672`-`CONTEXT.md:675`、`CONTEXT.md:690`-`CONTEXT.md:695`、`CONTEXT.md:774`-`CONTEXT.md:775`）。

3. Provider-neutral domain model。Bangumi/TMDB/Douban 等 provider subject 不直接污染核心 Media Item Hierarchy，而是通过 Provider Mapping 进入 Nako 自己的 provider-neutral 模型（`CONTEXT.md:171`-`CONTEXT.md:177`、`CONTEXT.md:642`-`CONTEXT.md:643`）。

4. Addon 选择 out-of-process sidecar，而不是进程内 plugin ABI。它追求 Jellyfin-like extensibility experience，但不追求 Jellyfin Plugin Compatibility；这降低核心进程信任边界风险，也让 Addon token、grants、health check、hosted pages、task/event/resource boundaries 可以被审计（`CONTEXT.md:9`-`CONTEXT.md:16`、`CONTEXT.md:35`-`CONTEXT.md:40`、`CONTEXT.md:437`-`CONTEXT.md:443`）。

5. 播放和转码强调可解释决策。Direct Play/Remux/Transcode 不只是 runtime 尝试，而是从 probe、client capability、policy、user preference、storage facts 产生 PlaybackRenditionPlan；这个模式有助于未来做设备兼容矩阵、诊断、降级和 UI 解释（`docs/architecture/PLAYBACK.md:9`-`docs/architecture/PLAYBACK.md:26`、`crates/nako-playback/src/lib.rs:31`-`crates/nako-playback/src/lib.rs:39`）。

6. 控制面和数据面显式分离。Nako 把“谁可以做、何时做、如何被监督、如何诊断、如何让客户端规模化消费”称为 control plane，而不是把 scan/transcode/addon/webhook/job 的调度逻辑散在 feature 内部（`docs/architecture/CONTROL_PLANE.md:5`-`docs/architecture/CONTROL_PLANE.md:24`）。

7. 对 operator diagnostics/redaction 的重视程度较高。当前 M1 coverage matrix 显示 Product-Operator M1 已经有 overview、release ladder、source identity、duplicate repair、VFS cache repair、job queue、playback runtime、catalog governance、generated artifact recovery、artwork repair、system config 等诊断/修复覆盖，并且强调 raw path、source locator、token、provider payload、FFmpeg command 等不能泄漏（`docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:51`-`docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:67`、`docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:81`-`docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:94`）。

8. 长期不局限视频。虽然现在是 video-first，术语和路线图保留了 audio/image/document/mixed/online 等 Media Domain 扩展空间，避免从一开始把数据库和 API 写死成 video-only（`CONTEXT.md:115`-`CONTEXT.md:121`、`CONTEXT.md:754`-`CONTEXT.md:755`）。

### 5. 明显短板和风险

1. 产品仍是 alpha 技术预览，不应按稳定 Jellyfin/Plex 替代品宣传。README 明确 API、Admin API、Addon Protocol、schema、SDK 都可能变化（`README.md:23`-`README.md:28`）。

2. 客户端产品面仍弱于后端。顶层架构把 Clients 标为 Mixed，下一压力是 Public media client parity、player UX、TV/casting clients（`docs/ARCHITECTURE.md:93`）。`apps/admin-web` 自述为历史 Admin Web validation console，并明确“不再是 release product frontend”（`apps/admin-web/README.md:1`-`apps/admin-web/README.md:9`）。Android app 有 browse/search/detail/source picker/player smoke 等基础，但仍排除 downloads/offline playback、external player handoff、CI device-farm/golden screenshot（`apps/android/README.md:8`-`apps/android/README.md:35`）。

3. Addon 生态有协议基础，但缺 Addon Manager 生命周期。README 明确 Nako 还不安装、更新、启动、停止、移除、记录或监督 addon processes（`README.md:52`-`README.md:55`）。Control plane 也把 Addon process supervision 标为 Deferred，并要求等协议、权限和官方 catalog 稳定后再做 lifecycle management（`docs/architecture/CONTROL_PLANE.md:41`-`docs/architecture/CONTROL_PLANE.md:44`、`docs/architecture/CONTROL_PLANE.md:468`-`docs/architecture/CONTROL_PLANE.md:483`）。

4. 远程访问还主要是 policy/readiness 和 cookbook 方向。README 明确没有内置 NAT traversal 或 relay service，并建议 local/private-network/VPN/reverse-proxy/tunnel-bounded auth enabled，不要把 placeholder config 暴露公网（`README.md:67`-`README.md:72`）。Control plane 中 Remote access cookbook 是 planned，Built-in tunnel provider deferred，Endpoint discovery not started（`docs/architecture/CONTROL_PLANE.md:49`-`docs/architecture/CONTROL_PLANE.md:52`）。

5. Storage/VFS 对真实远程/挂载环境仍有风险。WebDAV read path 是 Partial，library file writes 是 Partial，mount hang protection 虽有 durable circuit foundation，但文档仍提醒 OS-level mount stalls 需要 bounded adapters 和 operator guidance，不能宣称 syscall preemption（`docs/architecture/STORAGE_VFS.md:28`、`docs/architecture/STORAGE_VFS.md:34`-`docs/architecture/STORAGE_VFS.md:35`）。

6. Watcher/debounce 和大文件稳定摄取还没有产品化完成。Library pipeline 把 watcher/debounce 标为 Weak，只有 stable-candidate evidence foundation，并明确未加入 OS watcher daemon、storage-pressure admission 或 scan scheduler behavior（`docs/architecture/LIBRARY_PIPELINE.md:37`、`docs/architecture/LIBRARY_PIPELINE.md:442`-`docs/architecture/LIBRARY_PIPELINE.md:459`）。

7. 播放成熟度仍有后续压力。Playback map 已有大量 shipped first slices，但 VFS/remote playback resilience 是 Partial，release/packaging 是 Partial；设备 profiles、精确 codec/container/subtitle/HDR capability、ABR refinement、hardware tone-map execution、seek restart refinement、LL-HLS/CMAF、remote workers、player UX 仍是后续 lane（`docs/architecture/PLAYBACK.md:52`-`docs/architecture/PLAYBACK.md:57`、`docs/architecture/PLAYBACK.md:64`-`docs/architecture/PLAYBACK.md:82`）。

8. SQLite 默认合理，但播放写压力仍需要证明。State/access 风险提示 playback 会产生 heartbeat、session state、metrics、cleanup、scan/provider job 等频繁写入，仍需要 SQLite WAL、busy timeout、pool sizing、transaction scope 的压力测试（`docs/architecture/STATE_ACCESS.md:43`-`docs/architecture/STATE_ACCESS.md:60`、`docs/architecture/STATE_ACCESS.md:88`-`docs/architecture/STATE_ACCESS.md:92`）。

9. 诊断/观测还有未启动或 partial 项。Control plane 中 crash/fault bundles not started，tracing/request identity partial，Admin diagnostics good partial，HTTP cache/ETag narrow partial，N+1/list projection discipline partial（`docs/architecture/CONTROL_PLANE.md:47`-`docs/architecture/CONTROL_PLANE.md:55`）。

10. Metadata governance 已强，但 undo 和更多 provider depth 仍不是 MVP 完整态。Roadmap 把 provider governance undo、future intentional Public Client metadata exposure、Douban Season/Episode graph depth、broader scheduler migration 等列为后续；M1 中 provider-governance mutation undo 只有当 M1 evidence 证明 local authority 会被伤害时才打开（`docs/ROADMAP.md:64`-`docs/ROADMAP.md:70`、`docs/ROADMAP.md:171`-`docs/ROADMAP.md:180`）。

### 6. 读取的文件清单

#### 核心产品和架构文档

- `README.md`: 产品状态、当前能力、当前边界、快速启动、发布版本和 license。
- `CONTEXT.md`: Nako 领域词汇、长期 media server scope、video-first phase、addon/metadata/playback/storage/access 等术语边界。
- `docs/ARCHITECTURE.md`: 顶层系统图、North Star、架构原则、成熟度表、相关 ADR/workstream 入口。
- `docs/architecture/README.md`: architecture deep dive 索引和 linkage policy。
- `docs/architecture/LIBRARY_PIPELINE.md`: scan、watcher、probe、metadata、artwork、addon-assisted intake 状态和风险。
- `docs/architecture/PLAYBACK.md`: Direct Play、Remux、HLS、transcode、hardware、runtime scheduler、player 等播放能力地图。
- `docs/architecture/STORAGE_VFS.md`: local/remote storage、WebDAV、source identity、VFS cache、library writes、mount hang 风险。
- `docs/architecture/CONTROL_PLANE.md`: addon lifecycle、durable jobs、runtime supervisor、diagnostics、remote access、API scale、cache contracts。
- `docs/architecture/STATE_ACCESS.md`: SQLite/PostgreSQL、identity/access、playback/user state、event outbox、write pressure。
- `docs/architecture/OPERATIONS_RELEASE.md`: self-hosted docs、release gates、backup/restore、FFmpeg/hardware readiness、operator diagnostics。
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`: Product-Operator M1 Admin diagnostics/repair coverage matrix。
- `docs/ROADMAP.md`: M0-M5 milestone ladder、M1 product-operator target、release convergence queue。
- `docs/GOALS.md`: 当前 roadmap reconciliation / M1 convergence goal、non-goals、evidence。

#### Workspace 和代码入口

- `Cargo.toml`: workspace 成员、版本、Rust 版本、描述、license、核心依赖。
- `crates/nako-core/src/lib.rs`: core domain/repository module exports。
- `crates/nako-db/src/lib.rs`: SQLite/PostgreSQL facade 和 contract-test 入口。
- `crates/nako-api/src/lib.rs`: Admin/Public/OpenAPI/SDK/contract module exports 和公共契约防泄漏测试常量。
- `crates/nako-addon-protocol/src/lib.rs`: Addon Protocol version、runtime routes、manifest/install/sidecar contract。
- `crates/nako-playback/src/lib.rs`: playback decision、mode、planning request、selection context、rendition plan。
- `crates/nako-transcode/src/lib.rs`: transcode/FFmpeg/HLS/hardware/runtime module map。
- `crates/nako-library/src/lib.rs`: library index/source identity tests 和 scan/ingestion/probe/local inference exports。
- `crates/nako-vfs/src/lib.rs`: StorageUri、StorageCapabilities、local/WebDAV/cache backend boundary。
- `crates/nako-server/src/http.rs`: Axum route assembly、protected/public/addon runtime routes、auth/network/trace middleware。
- `crates/nako-server/src/app.rs`: NakoApp composition root 和 app services。
- `crates/nako-server/src/app/runtime.rs`: runtime supervisor/control-plane helper 入口。
- `crates/nako-server/src/app/playback/mod.rs`: playback app orchestration modules 和 `PlaybackRuntimeStore`。
- `apps/admin-web/README.md`: Admin Web 当前是 validation console，不是 release product frontend。
- `apps/android/README.md`: Android client foundation、UniFFI/client-core boundary、已包含和排除范围。

#### Trellis workflow 和相关 spec

- `.trellis/workflow.md`: Trellis 研究必须持久化到 task `research/`，以及任务/上下文流程。
- `.trellis/spec/nako-server/backend/index.md`: server spec 总入口和 authority/evidence。
- `.trellis/spec/nako-server/backend/directory-structure.md`: server composition/app/HTTP/runtime boundary、long-running work 禁止 raw `tokio::spawn`。
- `.trellis/spec/nako-server/backend/http-api-patterns.md`: HTTP route/auth/access patterns。
- `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`: Addon resource flow host-owned pattern。
- `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`: source fingerprint hash trigger、scheduler、redaction、duplicate reconciliation 边界。
- `.trellis/spec/nako-library/backend/index.md`: library scan/source ingestion/probe/local inference spec 入口。
- `.trellis/spec/nako-vfs/backend/index.md`: storage/VFS adapter boundary 入口。
- `.trellis/spec/nako-playback/backend/index.md`: pure playback planner boundary 入口。
- `.trellis/spec/nako-transcode/backend/index.md`: FFmpeg/HLS/transcode runtime boundary 入口。
- `.trellis/spec/nako-api/backend/index.md`: API contract patterns 入口。
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`: Admin/Public contract split、redaction、generated contract rules。
- `.trellis/spec/nako-core/backend/index.md`: core domain/repository boundary 入口。
- `.trellis/spec/nako-db/backend/index.md`: persistence boundary、SQLite default、PostgreSQL parity。

## External References

未做外部网页检索。本次调研只根据仓库内 README、CONTEXT、architecture docs、Roadmap/Goals、Trellis specs 和代码入口判断。

版本信息来自仓库内文件：

- Nako workspace/package version: `0.1.0-alpha.2`（`Cargo.toml:5`-`Cargo.toml:8`）。
- README status: `0.1.0-alpha.2` technical preview（`README.md:21`-`README.md:28`）。
- Addon Protocol Version: `0.1.0-alpha.1`（`README.md:30`-`README.md:31`、`crates/nako-addon-protocol/src/lib.rs:8`-`crates/nako-addon-protocol/src/lib.rs:9`）。

## Related Specs

- `.trellis/spec/nako-server/backend/index.md`
- `.trellis/spec/nako-server/backend/directory-structure.md`
- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`
- `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- `.trellis/spec/nako-library/backend/index.md`
- `.trellis/spec/nako-vfs/backend/index.md`
- `.trellis/spec/nako-playback/backend/index.md`
- `.trellis/spec/nako-transcode/backend/index.md`
- `.trellis/spec/nako-api/backend/index.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-core/backend/index.md`
- `.trellis/spec/nako-db/backend/index.md`

## Caveats / Not Found

- `python3 ./.trellis/scripts/task.py current --source` 在本环境中超时，未能确认 active task runtime state；用户已明确给出任务目录，因此本研究按指定目录写入。
- `.agents/skills/trellis-research/SKILL.md` 不存在；本次按开发者消息中的 Trellis researcher agent 规则执行。
- 本报告没有运行测试、构建或启动服务；它是只读代码/文档调研。
- 没有阅读全部 crate 源码和全部 ADR/workstream closeout；结论以 README、CONTEXT、顶层/相关架构地图、Roadmap/Goals、相关 spec 和关键代码入口为依据。
- 没有对 Jellyfin/Plex 做外部事实核查；对比仅使用仓库内“Jellyfin/Plex-class”“not a stable Jellyfin or Plex replacement yet”“not Jellyfin Plugin Compatibility”等自述，和通用产品类别层面的推断。
