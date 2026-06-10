# ADR / Spec Constraints For Media Server Gap Analysis

检查日期：2026-06-10

本研究用于约束 `.trellis/tasks/06-10-media-server-gap-analysis/research/product-benchmark.md`
中的成熟媒体服务器能力建议。判定口径：

- “已决定”表示 `CONTEXT.md`、ADR、架构地图或 `.trellis/spec/*/backend/index.md`
  已经给出稳定边界，后续建议应沿着该边界产品化。
- “冲突候选建议”表示该方向即使是 Plex/Jellyfin 常见能力，也会违反 Nako
  已接受的架构、授权、redaction、license 或 crate 边界。
- “开放缺口”表示方向有效，但当前文档将其标为 partial/deferred/follow-on，
  或只完成 foundation，不能写成已具备成熟媒体服务器能力。

主要来源：

- `CONTEXT.md`
- `docs/adr/0001-0055*.md`
- `docs/architecture/*.md`
- `.trellis/spec/*/backend/index.md`
- 重点补充读取：`.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`,
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`

## 总体结论

Nako 的成熟媒体服务器追赶路线不是“照搬 Plex/Jellyfin 功能形状”，而是沿
Nako 已确定的几个硬边界补齐用户闭环：

- Addon 必须是 out-of-process Addon Sidecar + Addon Protocol，不是 Jellyfin
  plugin compatibility 或 native in-process plugin ABI。
- Storage/VFS 是所有 scan、probe、streaming、NFO、artwork、subtitle 和 library
  file write 的入口，不能把本地路径或远端后端细节穿透到 app/API/job/addon。
- Metadata/artwork/provider/addon/AI 输出进入 canonical state 前必须经过 host-owned
  policy、plan/apply、field lock、acceptance 或显式 granted addon write path。
- Playback Runtime 归 Nako 所有；Addon 可建议资源，但不能替换 playback session、
  FFmpeg/transcode/runtime budget/error 行为。
- Public Client API 与 Admin API 是不同稳定性和 redaction 边界；成熟诊断应优先
  Admin-only，不能为了 Web/Admin UX 把内部治理面暴露到 Public Client。
- 长运行或重要后台工作必须是 durable job 或 supervised runtime task，并带 resource
  class、retry/cancel/redacted diagnostics；不能隐藏在 raw `tokio::spawn` 或 feature-local
  scheduler 中。
- Server 侧 AGPL 与 public protocol/helper crate permissive 的 license 边界是硬约束；
  `repo-ref/` 只能研究行为和架构，不允许复制 Jellyfin 或其他项目源码、schema、测试、
  assets 或注释。

## 1. Addon / Addon Protocol

### 已决定

- 当前 extension model 是 **Addon**：out-of-process **Addon Sidecar** 实现
  **Addon Protocol**，提供 Jellyfin-like extensibility experience，但明确不是
  Jellyfin Plugin Compatibility，也不是 native in-process plugin。
  来源：`CONTEXT.md`, ADR 0003, ADR 0015, ADR 0020, ADR 0053。
- Addon manifest 声明 protocol version、resources、entry points、event
  subscriptions、configuration schema、permissions、optional library-scoped grants；
  Nako 验证并保存 accepted registration。
  来源：ADR 0020, `.trellis/spec/nako-addon-protocol/backend/index.md`。
- Addon Sidecar 使用 revocable/rotatable scoped addon token；token 只包含被接受的
  Addon Permissions 和 library grants。Addon 不拿 admin token、database credential、
  raw filesystem authority 或 unmediated storage access。
  来源：`CONTEXT.md`, ADR 0020。
- Addon Protocol Version、Addon Version、Rust crate package version 三者分离；
  breaking wire changes 必须新 Addon Protocol Version。当前 protocol version 是
  `0.1.0-alpha.1`。
  来源：ADR 0033, `.trellis/spec/nako-addon-protocol/backend/index.md`。
- Addon Package / Addon Suite 是部署形态，不是 permission unit；官方 Addon 可以用
  suite 打包，但仍保留 per-Addon manifests、grants、tasks、events。
  来源：ADR 0034, `CONTEXT.md`。
- Official Addon Catalog 是 official addon manifest/install descriptor facts 的
  source of truth，不是 Addon Manager，也不是 sidecar runtime。
  来源：`.trellis/spec/nako-official-addon-catalog/backend/index.md`。
- Addon Client 只负责 HTTP request/response、protocol envelope validation、retry 和
  redaction；durable job persistence、registration、permission storage 在 server/control
  plane。
  来源：`.trellis/spec/nako-addon-client/backend/index.md`。
- Addon resource discovery 默认 read-only。Resource Search Selection、subtitle import、
  acquisition materialization 等后续 side effect 必须转为 host-owned selected reference、
  plan/apply/materialization/write flow。
  来源：ADR 0050, ADR 0051, ADR 0054,
  `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`。

### 冲突候选建议

- “兼容 Jellyfin 插件 API / server object model”冲突：Nako 只追求体验类似，不提供
  Jellyfin Plugin Compatibility。
- “在 Nako 进程内加载第三方插件 / native dynamic library ABI”冲突：当前模型是 HTTP
  Addon Sidecar。
- “Addon 直接写数据库、写 filesystem path、写远端 storage URL、修改 library sidecar
  文件”冲突：所有 Addon side effect 必须走 Nako API、Library File Write、VFS/NFO/artwork/subtitle
  boundary。
- “Addon Hosted Page 被当作可信 Admin UI，或接收 Nako admin credential”冲突：
  hosted page 不受信任，只能通过 scoped token/API 能力进入 Nako。
- “Addon Suite 粗粒度授权覆盖全部能力”冲突：suite 是 packaging convenience，
  permission/audit unit 仍是 Addon。
- “浏览器把 raw URL/password 从 Addon search result 回传给 Nako 触发下载/导入”冲突：
  必须使用 host-owned `search_id`/`selection_id`/selected refs。
- “第一个 Addon Manager 直接控制 Docker socket、容器生命周期或进程监督”冲突：
  ADR/Context 明确第一阶段偏 Addon Install Guide + Health Check，process lifecycle
  是 deferred/follow-on。

### 开放缺口

- Addon Manager lifecycle：package inventory、start/stop/restart、health checks、
  logs、port/env rendering、signature/update/rollback 仍是 `addon-manager-process-lifecycle`
  follow-on。
- Official Addon Suite 的真实生产 sidecar runtime、发布、安装指导、签名和兼容矩阵还未成熟。
- Addon Hosted Page 的更强 route policy、sandbox、CSP、secret/reference UX 仍需产品化。
- Addon task/event/subtitle/acquisition/resource flows 有基础，但 marketplace 级 catalog、
  一键安装、自动更新、评分/支付不在近期边界内。

## 2. Storage / VFS

### 已决定

- 先建内部 VFS，再考虑 OS-level mount/export。`nako-vfs` 是 scan、probe、streaming、
  remote storage、cache、staging、library file write 的抽象边界。
  来源：ADR 0002, ADR 0016, `docs/architecture/STORAGE_VFS.md`,
  `.trellis/spec/nako-vfs/backend/index.md`。
- Remote storage first target 是 WebDAV read-only preview before S3-compatible
  storage；remote backend 只能返回 Nako-owned `StorageUri` / metadata / virtual file
  records，不暴露 raw local paths。
  来源：ADR 0016。
- VFS/cache state 不是 catalog truth；cache staleness 不能直接删除 Media Source。
  Catalog/source tombstone 由 library/source state 处理。
  来源：ADR 0012, ADR 0016。
- Remote probe/FFmpeg 需要 explicit staging boundary：确定性 local cache path、
  etag/fingerprint validation、cleanup policy、disk budget、timeout/retry/concurrency。
  来源：ADR 0016, ADR 0017。
- Storage Backend Health 与 Storage Circuit Breaker 是 durable product state，用于限制
  scan/probe/playback staging/write work，不能变成隐藏重试计数器。
  来源：`CONTEXT.md`, `docs/architecture/STORAGE_VFS.md`。
- Source Locator 只在 Media Library 内唯一；Source Fingerprint 是证据，不是 Source
  identity；duplicate relationship 不自动 merge sources/items。
  来源：`CONTEXT.md`, ADR 0012,
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`。
- OpenDAL 只可作为 `StorageBackend` 背后的 optional implementation foundation，不能
  替换 Nako-owned `StorageUri`、capabilities、VFS cache/repair、health、fingerprint、
  staging、file write、API DTO 等概念。
  来源：ADR 0055。

### 冲突候选建议

- “把 SMB/NFS/rclone mount 当成本地文件随意 `std::fs` 调用”冲突：OS mounts may block；
  app logic 不能绕过 VFS。
- “用 OpenDAL Operator 替换 Nako StorageBackend/domain contract”冲突：ADR 0055 明确
  OpenDAL 只在 adapter 内部。
- “在 job input、diagnostics、Admin response、events 中持久化 raw `StorageUri`、Source
  Locator、path、backend URL、credential、etag/fingerprint/hash material”冲突。
- “扫描时为了准确性默认整文件 hash”冲突：现有策略是 layered evidence，partial/full hash
  由 explicit escalation/durable job 处理。
- “cache stale 直接删除 catalog/source”冲突：cache repair 与 source tombstone 是不同边界。
- “VFS repair 自动 purge/delete/invalidate 或修改 backend config”冲突，除非单独定义 cache
  invalidation、targeting、DB repository contract。

### 开放缺口

- WebDAV read path 仍是 partial，需要 retries、cache、operator diagnostics。
- VFS cache purge/delete/invalidation、backend configuration mutation、library file write
  repair、recurring automatic repair scheduling 仍是 follow-on。
- Source duplicate automatic reconciliation、confirm/reject/undo、Media Item merge 仍未作为
  自动化策略接受；当前偏 plan/apply/suggest。
- Per-backend staging budgets、remote FFmpeg input diagnostics、mount stall operator guidance
  仍需增强。
- OpenDAL production backend rollout 还未开始；下一步只是 proof adapter。

## 3. Library Intake / Scan / Local Inference

### 已决定

- Library 是管理边界，拥有 roots、scan behavior、metadata policy、refresh policy、
  presentation defaults；Library Preset 是配置模板，不是硬内容类型。
  来源：ADR 0010, ADR 0021, `CONTEXT.md`。
- Scan 持久化 scan snapshot、directory snapshot、source state，并用 tombstone 表达 missing
  source；重复 scan 必须幂等。
  来源：ADR 0012, ADR 0006。
- Local Inference 可创建 Provisional Hierarchy，但必须保留 Local Inference Evidence；
  confirmed canonical metadata 后，rescan 不得用 local inference 覆盖 canonical fields。
  来源：`CONTEXT.md`, `docs/architecture/LIBRARY_PIPELINE.md`。
- Watcher/debounce 目前只有 stable-candidate evidence foundation；watch events 不是稳定
  media events。
  来源：`docs/architecture/LIBRARY_PIPELINE.md`。
- Source fingerprint hash 的 scan-originated enqueue 在 source commit 后由 server app service
  处理，scan planning/commit 本身不读 bytes、不直接插 durable job。
  来源：`docs/architecture/STORAGE_VFS.md`,
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`。

### 冲突候选建议

- “文件系统 watcher 事件一到就 probe/metadata refresh”冲突：大文件 copy-in-progress 是明确风险。
- “scan commit 直接读 VFS full hash 或执行昂贵字节工作”冲突：hash 是 advisory -> durable job。
- “missing source 立即硬删除”冲突：使用 tombstone。
- “弱证据自动把 duplicate source 合并成同一个 Media Source/Media Item”冲突：Source Duplicate
  Relationship 不等于 merge。
- “命名 parser 直接确认 catalog/provider identity”冲突：`nako-naming` 只产出 deterministic hint。
- “把 path/file name inference 当作 metadata provider”冲突：它是 Local Inference Evidence。

### 开放缺口

- Watcher/debounce productization：OS watcher daemon、stable-size detection、copy-in-progress、
  storage pressure admission、scheduled reconciliation、per-library intake diagnostics。
- Anime/series path heuristics、confidence reporting、Unknown Media Item UX 仍需增强。
- Source duplicate repair M1 已有 suggestion/operator flow，但 automatic merge、undo、Media Item
  merge、cross-library variant UX 仍是后续设计。

## 4. Metadata / NFO / Artwork

### 已决定

- Canonical Metadata 是用户可见权威状态；provider payload、Generated Artifact、suggestion
  都不是 canonical metadata。
  来源：`CONTEXT.md`, ADR 0007, ADR 0021。
- Metadata merge 使用 explicit merge、field locks、local/NFO authority、provider raw response
  cache；user-locked fields 不被 automatic refresh 覆盖。
  来源：ADR 0007。
- NFO 是 local metadata boundary；import/export 走 `nako-nfo` codec，NFO Export opt-in per
  library，并尽量 round-trip preservation。
  来源：ADR 0008, `CONTEXT.md`, `.trellis/spec/nako-nfo/backend/index.md`。
- 所有 network metadata providers 必须走 `MetadataHttpRuntime`，它拥有 timeout、retry、
  provider interval、concurrency、User-Agent、proxy config、circuit breaker 和 secret redaction。
  来源：ADR 0018。
- Candidate Review、Provider Mapping application、related hierarchy application、Generated Artifact
  apply 是 Admin/governance/control-plane 边界；Public Client 暂不暴露这些 mutation/governance
  routes。
  来源：`docs/architecture/LIBRARY_PIPELINE.md`,
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-client-protocol/backend/index.md`。
- Addon/native metadata writeback 可以提交 canonical metadata-shaped patch，但 server 映射到
  `CanonicalMetadata`、合并 local locks 并重建 catalog projection。
  来源：ADR 0035。
- Artwork 使用 Artwork Source / Managed Artwork / Artwork Candidate / Selected Artwork；
  client presentation 应使用 Managed Artwork，而不是 provider hotlink。
  来源：`CONTEXT.md`, `docs/architecture/LIBRARY_PIPELINE.md`。

### 冲突候选建议

- “Provider/Add-on/AI 结果直接覆盖 canonical metadata”冲突：必须经过 merge policy、
  acceptance workflow 或显式 granted addon write path。
- “Candidate Review 或 Provider Mapping governance 作为 Public Client route 暴露”冲突：
  当前是 Admin-only，Public exclusion 有测试要求。
- “NFO export 无条件重写整个文件”冲突：需要 opt-in、preview、preserve unknown fields/comments
  when supported。
- “把 Bangumi/TMDB/Douban provider-specific hierarchy 直接塞进 MediaItem identity”冲突：
  需要 Provider Subject / Provider Mapping / provider-neutral hierarchy。
- “客户端直接 hotlink TMDB 等 provider poster URL”冲突：使用 Managed Artwork。
- “把 provider API key/proxy URL/raw payload 写进 jobs/events/logs/diagnostics”冲突。

### 开放缺口

- NFO round-trip/writeback polish、backup policy、未知字段 preservation 的完整覆盖仍需加强。
- Provider depth/breadth 仍有后续：例如更深的 Douban season/episode graph、更多区域 provider、
  future intentional Public Client metadata read contract。
- Provider governance mutation undo、related hierarchy Web UX、durable/bulk related hierarchy
  execution 是后续。
- Artwork derivative policy、placeholder/Blurhash、selected-artwork invalidation、broader cache
  and cleanup automation、manual replacement UX 仍未完全成熟。

## 5. Catalog / Search

### 已决定

- Nako 持久化 normalized catalog graph records，并投影 search documents；
  `CanonicalMetadata` 保持 item-facing API shape。
  来源：ADR 0011。
- `nako-catalog` 是 pure orchestration/read-model crate：通过 core repository traits 读 facts，
  产生 graph/projection；不能直接 SQL、storage、HTTP DTO、search ranking、canonical mutation。
  来源：`.trellis/spec/nako-catalog/backend/index.md`。
- `nako-search` 当前拥有 transport-free search documents、query evaluation、ranking primitives，
  用 in-memory deterministic evaluation；external search adapters 等到真实 adapter 需求出现。
  来源：`.trellis/spec/nako-search/backend/index.md`。
- Browse facets 和 sort keys 是明确 Public Client contract，不是任意 DB column。
  来源：`CONTEXT.md`, ADR 0021。

### 冲突候选建议

- “Public Client 可按任意数据库字段筛选/排序”冲突：只能暴露 supported Browse Facets /
  Sort Keys。
- “在 catalog hydration 中做 search ranking 或 provider-specific search behavior”冲突。
- “在 `nako-catalog` 中直接访问 SQL/storage/HTTP”冲突。
- “搜索时 mutate catalog/canonical state”冲突。
- “为了规模直接把 Meilisearch/Tantivy 作为 domain contract”冲突：可替换 adapter 不能改变
  catalog writes/public contract。

### 开放缺口

- FTS/Tantivy/Meilisearch 等可替换 adapter、FTS/filter scale-up 仍未开始。
- Cursor pagination、large-library N+1/query budget tests、projection-backed list/read 深化是
  control-plane API scale follow-on。
- 更多 browse facets/sort keys 需要逐项作为 Public Client contract 明确。

## 6. Playback / Transcode / Streaming

### 已决定

- Playback Source Selection 运行时选择 allowed Media Source/Source Variant，必须尊重
  Library Access；不依赖 permanent default source。
  来源：`CONTEXT.md`, ADR 0038, ADR 0039。
- `nako-playback` 是 pure planner：根据 source/probe/target/policy/storage/preferences 选择
  Direct Play、Remux、Transcode 或 Denied；不执行 FFmpeg、不 serve bytes。
  来源：`.trellis/spec/nako-playback/backend/index.md`。
- `nako-transcode` 拥有 FFmpeg command planning、HLS/remux/transcode artifact modeling、
  hardware inventory、runtime primitives；`nako-streaming` 只做 range/direct byte response
  planning。
  来源：`.trellis/spec/nako-transcode/backend/index.md`,
  `.trellis/spec/nako-streaming/backend/index.md`。
- FFmpeg CLI-first media engine boundary 已接受：Rust 负责 decision/planning/process lifecycle/
  tickets/manifest/redaction，FFmpeg/ffprobe 负责 probe/decode/filter/encode/mux/segment work。
  来源：ADR 0052。
- Browser media transport 用 short-lived browser playback tickets；Renderer 使用独立 cast-safe
  tickets；ticket 不是 bearer token、Source Locator 或 permanent URL。
  来源：ADR 0036, ADR 0041。
- 外部 casting protocols 走 sidecar renderer adapters；Chromecast 是 first official renderer
  adapter sidecar，DLNA/AirPlay deferred。
  来源：ADR 0040-0043。
- Hardware capability report、FFmpeg probe inventory、CPU readiness、startup degradation、
  source-aware transcode runtime 已有 foundation；HLS transcode readiness 不是 server startup
  invariant。
  来源：ADR 0045-0049, `docs/architecture/PLAYBACK.md`。
- Playback byte/HLS artifacts 当前 cache policy 是 conservative `no-store`；selected artwork
  使用 authenticated private cache baseline。
  来源：`docs/architecture/CONTROL_PLANE.md`,
  `.trellis/spec/nako-server/backend/http-api-patterns.md`。

### 冲突候选建议

- “客户端推断 local path / Source Locator 直接播放”冲突：必须走 Playback Source Selection、
  Library Access、ticket/API transport。
- “把 bearer token 放进 media element src、HLS playlist、segment URL”冲突：必须使用 playback
  ticket。
- “Addon 替换 Playback Runtime、注入 FFmpeg args、作为 stream plugin 接管 session/budget/error”
  冲突：Addon 只能提供 Playback Resource Suggestion，runtime 归 Nako。
- “在 HTTP route 中拼 FFmpeg command 或绕过 `nako-transcode` planner”冲突。
- “HLS segment route 自己 lazy start FFmpeg process”冲突：当前 ADR 0052 的 segment route
  只 wait briefly for running sessions，不拥有 lazy process creation。
- “把 hardware acceleration 简化成 bool 并在 command builder 里读 config/probe 自行选择”
  冲突：需要 typed decode/filter/encode stages 和 planned policy。
- “把 Playback Transcode 当成 Optimized Version 长期缓存”冲突：二者已明确区分。

### 开放缺口

- Device capability profiles：浏览器、mobile、TV、renderer 的 codec/container/subtitle/HDR/audio
  facts 仍是重要 follow-on。
- HLS seek/restart、generation identity、FFmpeg seek flags、public `start_position_ms` query
  仍需设计/实现。
- LL-HLS/CMAF、DASH/CMAF、DRM/key delivery、remote transcode workers、durable queueing、player UX
  都是 follow-on。
- HEVC/AV1 目前是 typed policy value，H264/AAC 仍是主要 executable HLS output；HEVC/AV1
  FFmpeg execution、client compatibility、hardware selection 后续。
- PGS/image subtitle burn-in、external subtitle burn-in、hardware-filter burn-in、richer subtitle
  capability profiles 仍未完成。
- Hardware tone mapping、one-frame GPU smoke、container device pass-through matrix、operator smoke
  matrices 是 operations/playback 后续。
- Optimized Version/offline sync/download-to-go 需要独立 artifact lifecycle、quota、expiry、
  access revocation 和 resource policy，不能复用 transient HLS session directory。

## 7. Public / Admin API

### 已决定

- Public Client HTTP contract 是 versioned `v1`，wire DTO source of truth 在
  `nako-client-protocol`；`nako-api` 做 OpenAPI aggregation/server-domain mapping。
  来源：ADR 0023, ADR 0025, `.trellis/spec/nako-client-protocol/backend/index.md`。
- Admin-only surfaces 使用 `/admin/v1/*`，与 Public Client API 分离。
  来源：ADR 0027, `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`。
- Public route inventory 不包含 Provider Governance、Metadata Candidate Review、batch apply、
  raw provider payload、related hierarchy application 等 governance/mutation surfaces。
  来源：`.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-client-protocol/backend/index.md`。
- HTTP handler 应薄：Axum extraction、query/path/body parsing、trace context、DTO mapping；
  app services 拥有 workflow/access/policy。
  来源：ADR 0019, `.trellis/spec/nako-server/backend/http-api-patterns.md`。
- Public DTO 不暴露 principal ID、raw source locators、server paths、bearer tokens、
  transcode internals；Admin diagnostics 也必须 redaction-safe。
  来源：ADR 0023, ADR 0027, ADR 0053。
- Generated Admin Web contract 必须从 `nako-api` 生成，不能手改。
  来源：`.trellis/spec/nako-api/backend/admin-and-public-contracts.md`。

### 冲突候选建议

- “为了 Admin Web 方便，把 Admin diagnostics/governance route 加到 Public Client SDK/OpenAPI”
  冲突。
- “HTTP route 直接返回 DB/domain/internal structs”冲突：必须显式 DTO。
- “新增 admin route 但不继承 admin principal guard”冲突。
- “API response 暴露 raw provider payload、job input/summary/error、paths、locators、
  FFmpeg command/stderr、tokens、backend URLs、credentials”冲突。
- “Public browse/search/list 返回 unbounded list 或 page total 语义漂移”冲突；v1 最小分页
  是 `limit`/`offset`/`returned`。
- “手工编辑 Admin Web generated contract”冲突。

### 开放缺口

- Cursor pagination、大库 API response budget、N+1/list projection discipline 仍是
  `api-scale-and-cache-contracts` follow-on。
- Catalog/image/artifact cache semantics 尚未系统化：HLS/media bytes no-store、selected artwork
  private cache 已有，immutable artifact/CDN/shared-cache 仍未接受。
- Public metadata governance/read contract 如果未来要开放，必须先有 PRD/ADR/spec 更新。
- Admin Web 对所有已 shipped repair/diagnostics route 的可达性需要由 release ladder 失败证据驱动，
  不能泛化成“大修 Admin repair platform”。

## 8. Auth / User / Library Access

### 已决定

- 第一层 inbound HTTP auth 是 bearer-token boundary；`GET /health` public，其它受保护路由
  默认需要 auth。token value 不进 logs/API/jobs/diagnostics。
  来源：ADR 0024。
- 本地用户认证使用 password hash + opaque session token hash；auth middleware 同时支持 bootstrap
  admin token 和 active local session token。
  来源：ADR 0037。
- Single-Admin Mode 可作为 first implementation，但不能抹掉 User、Role、Library Access 概念。
  来源：`CONTEXT.md`。
- User Playback State 按 server-internal stable principal + item/source 记录；Public route 使用
  `/users/me/...`，不暴露 arbitrary user id 或 internal principal id。
  来源：ADR 0028。
- Public library/catalog/playlist/playback access 应由 app service 使用 `AuthenticatedPrincipal`
  和 Library Access/Playback Policy 判定，不应由 HTTP route 自己循环过滤或绕过。
  来源：`.trellis/spec/nako-server/backend/http-api-patterns.md`。
- Playback permission policy 与 Renderer Target 已明确：Library Access 是基本 browse/play/manage
  gate，Playback Permission Policy 决定 direct/remux/transcode/remote/cast 等能力。
  来源：ADR 0039。

### 冲突候选建议

- “Nako 云账号是本地服务器登录前提”冲突：当前是 local credential/session auth，remote access
  也不依赖 central relay/account。
- “把 bearer token value 当 user id/principal 存储”冲突。
- “Addon 或 Addon Hosted Page 使用 admin token”冲突。
- “跨 library source variants 播放时忽略每个 source 的 library context/access”冲突。
- “Public route 只在 HTTP 层检查一次 access，然后调用 raw app service side effect”冲突；
  app service 边界要拥有 manage/play/browse decision。
- “播放 ticket 只在 issuance 检查 access，使用时不检查”冲突：ticket validation 需在 issuance
  和 use 时执行 current Library Access/playback policy。

### 开放缺口

- Parental controls/restriction profiles、rating/tag/time-window policies、PIN/managed profile
  仍未成熟。
- Account recovery、SSO、invitation/self-registration 是 future/onboarding follow-ons。
- Playback access policy/session limits：per-user remote playback、max bitrate、allow/deny transcode、
  active session limit、idle termination 仍是 `playback-access-policy-and-session-limits`。
- Playback heartbeat buffering、multi-device conflict semantics、favorites/hidden/user rating first-class
  routes 仍是后续。

## 9. Control Plane / Jobs / Events / Realtime

### 已决定

- ADR 0053 把 application control plane 定义为 first-class boundary：policy、identity、
  authorization、durable work、supervision、resource accounting、diagnostics、endpoint config、
  addon mediation、API scale contracts。
- Long-running 或 important background work 必须进入 durable job 或 supervised runtime boundary；
  raw `tokio::spawn` 只适合 request-local/explicitly disposable work。
  来源：ADR 0019, ADR 0053。
- Durable job 必须持久化 safe `input_json`、明确 lifecycle、lease/run token、retry/cancel/backoff、
  idempotency、resource class；retry 创建新 job row，不原地重置 failed job。
  来源：ADR 0006, ADR 0030, `docs/architecture/CONTROL_PLANE.md`。
- Resource budgets/resource classes 是控制 playback、scan、probe、provider、addon、webhook、
  automation、storage remote、transcode 竞争的手段。
  来源：ADR 0005, ADR 0053。
- Event outbox 持久化 domain events；payload 必须 safe，不含 plaintext secrets、local paths 或
  large binary data；webhook/addon/automation/realtime 是不同 consumers。
  来源：ADR 0014, `docs/architecture/REALTIME_SYNC.md`。
- Webhook delivery 有签名、attempt persistence、retry/backoff；不能把 disabled subscription
  deliver，也不能 raw spawn delivery loop。
  来源：`.trellis/spec/nako-events/backend/index.md`。
- Request identity/trace context 已有 HTTP request ID first slice；后续应跨 jobs/VFS/FFmpeg/addons
  传播。
  来源：`docs/architecture/CONTROL_PLANE.md`。

### 冲突候选建议

- “每个 feature 自己起 scheduler/background loop/retry log”冲突：会绕过 control plane。
- “后台任务没有 resource class，与 playback/scan/transcode 争抢资源”冲突。
- “durable job input 存 raw path/locator/credential/provider payload/materialization refs”冲突。
- “retry 直接把 failed job 改回 queued”冲突：必须新建 retry job 并保留 failed audit。
- “事件 payload 带 raw local path、provider payload、binary artifact、secret”冲突。
- “Realtime client updates 直接复用 durable webhook payload 且不按 principal/library access 过滤”
  冲突。

### 开放缺口

- Broader job-kind scheduler migration、recurring VFS cache repair scheduling/execution policy、
  source duplicate automatic reconciliation 仍是 follow-on。
- Client SSE/WebSocket gateway 未开始；需要 principal-scoped filtering、scan/playback/transcode/catalog
  updates、reconnect/backfill。
- Redacted incident bundle/crash/fault bundle 未开始。
- Unified trace context across jobs/FFmpeg/VFS/addons、metrics/export profile、runtime diagnostics
  仍是 partial。
- Remote access cookbook、endpoint discovery、trusted proxy/public URL/LAN URL model 仍未完全产品化。

## 10. Licensing / Reference Code

### 已决定

- Nako server implementation crates 默认 `AGPL-3.0-or-later`；public protocol/SDK boundary
  crate 使用 permissive license，Apache-2.0 是默认选择。
  来源：ADR 0022, AGENTS.md。
- Public protocol crates 必须 dependency-light，不能依赖 AGPL server crates 或 internal server
  domain models；必要时把 neutral wire types 移到 permissive crate，或在 public boundary
  复制很小 wire types。
  来源：ADR 0022, `.trellis/spec/nako-client-protocol/backend/index.md`,
  `.trellis/spec/nako-addon-protocol/backend/index.md`。
- `nako` facade 只 re-export public protocol/helper crates，不能暴露 server/API/database/catalog/
  playback/metadata/storage/runtime internals，并保留 permissive surface。
  来源：`.trellis/spec/nako/backend/index.md`。
- `repo-ref/` 只能作为 reference material，用来研究 behavior、architecture boundaries、user
  workflows；不能 copy/translate line by line/import source/comments/migrations/tests/schemas/assets/
  generated code。
  来源：AGENTS.md, ADR 0038。

### 冲突候选建议

- “直接移植 Jellyfin plugin API、schema、tests、DLNA profiles、assets、comments 或 generated code”
  冲突 license/reference-code 规则。
- “为了 Addon authors 方便，让 permissive protocol crate 依赖 `nako-server`/`nako-core`/DB/storage”
  冲突。
- “把 server internals 暴露到 `nako` facade 或 public client/addon protocol crate”冲突。
- “复制 official addon facts 到 server/docs 多处维护”冲突：official addon catalog 是 source of
  truth。

### 开放缺口

- Official addon sidecar packages、third-party marketplace、license compatibility/signature/update
  policy 仍需后续定义。
- Nako-Managed Artifacts、optimized versions、offline sync、Generated Artifacts 的 backup/restore
  license/retention/classification 需要随着功能增加持续更新。

## 对产品基准建议的约束化路由

### 可以直接沿现有边界推进

- Library onboarding、scan diagnostics、metadata/artwork governance：
  沿 Library/VFS/metadata/NFO/artwork/Admin governance 边界做产品闭环。
- Direct Play/Remux/HLS/Transcode、hardware report、playback diagnostics：
  沿 `nako-playback` planner、`nako-transcode` FFmpeg boundary、Playback Runtime、ticketed
  transport 推进。
- Local users/access、Public Client API、Media Web：
  沿 local session auth、Library Access、`/users/me`、Public Client DTO/route inventory 推进。
- Admin Jobs、storage repair、source hash/duplicate diagnostics：
  沿 ADR 0053 durable job/control-plane/Admin redaction 边界推进。

### 必须拆成独立 follow-on 设计

- Optimized Versions / Downloads / Offline Sync：
  需要 durable artifact lifecycle、quota、expiry、access revocation、resource scheduling，不能
  复用 playback transcode cache 或 HLS session output。
- Live TV / DVR：
  涉及 tuner/EPG/recording schedule/conflict/transcode/client UI，应独立 PRD/ADR。
- Parental Controls / Household Profiles：
  需要 User/Role/Library Access/Playback Policy 的更细规则，不应硬编码在 route。
- Addon Manager / Marketplace：
  先稳定 Addon Protocol、permissions、official catalog，再讨论 lifecycle/update/signature。
- Realtime / Push / Watch Together：
  先完成 SSE/WebSocket gateway、principal filtering、runtime/session semantics。

### 明确不应做或不应照搬

- Plex-style central account as login prerequisite、subscription-gated local playback、first-party
  relay as default remote access path。
- Jellyfin plugin compatibility、native in-process plugin ABI、direct import of Jellyfin code/schema/tests/assets。
- Addon/database/filesystem direct mutation、raw URL/password/browser-resubmitted acquisition、provider/AI
  direct canonical mutation。
- Hidden unbounded background schedulers、opaque FFmpeg helpers、unredacted diagnostics。
