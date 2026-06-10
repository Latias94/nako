# Nako Current State And Gap Map

研究日期：2026-06-10

范围：检查 Nako 当前 Rust workspace、测试标记、`CONTEXT.md`、`docs/architecture/` 与关键 ADR，形成面向媒体服务器成熟度对标的当前状态与缺口地图。本文只做研究记录，不修改业务代码。

## 阅读来源

- 领域语言：`CONTEXT.md`
- 任务上下文：`.trellis/tasks/06-10-media-server-gap-analysis/prd.md`
- 架构地图：`docs/ARCHITECTURE.md`、`docs/architecture/README.md`、`docs/architecture/LIBRARY_PIPELINE.md`、`docs/architecture/STORAGE_VFS.md`、`docs/architecture/PLAYBACK.md`、`docs/architecture/STATE_ACCESS.md`、`docs/architecture/CONTROL_PLANE.md`、`docs/architecture/REALTIME_SYNC.md`、`docs/architecture/OPERATIONS_RELEASE.md`、`docs/architecture/LANES.md`
- 关键 ADR：`docs/adr/0001-modular-monolith-rust-workspace.md`、`0002-internal-vfs-before-os-mounting.md`、`0003-http-addons-before-in-process-plugins.md`、`0005-bounded-async-pipelines-and-resource-budgets.md`、`0006-persist-job-inputs-and-explicit-retry-policy.md`、`0011-normalized-catalog-graph-and-search-projection.md`、`0012-durable-scan-state-and-source-tombstones.md`、`0016-remote-storage-and-vfs-cache-boundary.md`、`0017-playback-streaming-and-remote-hardening-boundaries.md`、`0018-playback-client-capability-and-profile-contract.md`、`0021-public-client-contract-and-sdk-surface.md`、`0023-admin-api-contract-drift-guard.md`、`0024-local-client-core-and-uniffi-bindings.md`、`0027-control-plane-and-client-api-scale-baseline.md`、`0028-addon-sidecar-management-boundary.md`、`0032-addon-external-streaming-and-acquisition-intake.md`、`0038-addon-renderer-adapter-boundary.md`、`0044-postgres-backend-parity-baseline.md`、`0047-runtime-supervisor.md`、`0049-postgres-ci-and-operations-baseline.md`、`0052-official-addon-catalog.md`、`0053-control-plane-and-runtime-foundation.md`、`0055-control-plane-runtime-foundation-follow-through.md`
- 代码重点：`crates/nako-core/`、`crates/nako-db/`、`crates/nako-vfs/`、`crates/nako-library/`、`crates/nako-metadata/`、`crates/nako-playback/`、`crates/nako-transcode/`、`crates/nako-server/`、`crates/nako-api/`、`crates/nako-addon-*`、`crates/nako-client-*`

## 执行摘要

Nako 当前不是空壳或 demo skeleton。它已经有可执行的 Media Library、Media Source、Media Item、Canonical Metadata、User Playback State、Playback Runtime、Addon、Public Client API/Admin API、Storage/VFS、Jobs/Control Plane 基础，并且 DB、VFS、metadata provider、addon client、public client transport 等多个 seam 已经有两个以上 Adapter 或真实替代实现。

最大的风险不在“缺少所有模块”，而在三类结构性问题：

- 深度集中：`nako-server`、`nako-db`、`nako-api` 是明显重心，`nako-server` 同时承担 HTTP、composition、app service、runtime、playback orchestration、admin diagnostics，未来扩展容易继续变成 god crate。
- 协议面过宽：`nako-api`、`nako-client-protocol`、`nako-addon-protocol`、`nako-addon-client` 已经形成大合同面，新增字段和 route 时需要更强 drift guard、兼容性测试和 SDK 同步流程。
- 产品闭环仍缺：成熟 media server 期待的库 onboarding、watcher/debounce、识别修复、metadata/artwork 治理、播放兼容矩阵、远程访问/运维文档、备份恢复、实时同步、客户端生态还没有形成“一条用户能顺畅跑完”的产品链路。

## Crate 职责与成熟度

成熟度口径：

- 成熟：有领域模型、持久化/运行时实现、HTTP/API 或调用面、较多测试，且 seam 至少部分真实。
- 已实现：核心路径可跑，测试存在，但产品覆盖或运维闭环仍不足。
- Foundation：抽象或基础能力已建立，实际场景/Adapter/产品面还薄。
- Shallow：职责合理但深度很浅，或当前只是薄包装/fixture/helper。
- 风险：实现深但边界或合同面有明显扩展压力。

源码规模快照：workspace 有 28 个 crate、约 412 个 Rust 源文件。`nako-server` 约 138 个 Rust 文件/115k 行，`nako-db` 约 71 个文件/58k 行，`nako-api` 约 23 个文件/22k 行。测试标记最多的是 `nako-server`、`nako-transcode`、`nako-api`、`nako-db`、`nako-vfs`。

| Crate | 当前职责 | 成熟度 | 证据路径 | 主要缺口/风险 |
| --- | --- | --- | --- | --- |
| `nako-core` | Domain records、ID、repository traits、Media Library/Source/Item、metadata、jobs、playback、addon、VFS 等核心合同。 | 成熟/风险 | `crates/nako-core/src/media/`、`crates/nako-core/src/repository/`、`crates/nako-core/src/job.rs` | Repository trait 数量很多，Interface 面向外暴露较宽；核心合同稳定后需要防止继续把 app-layer shape 下沉到 core。 |
| `nako-db` | SQLite/Postgres adapter、migration、repository contract implementation、DB facade。 | 成熟 | `crates/nako-db/src/facade.rs`、`crates/nako-db/src/sqlite/`、`crates/nako-db/src/postgres/` | 两个真实 Adapter 已存在，但行数大，新增 repository 容易双实现成本高；Postgres parity 需要持续 CI/contract tests。 |
| `nako-vfs` | StorageBackend、local/WebDAV/OpenDAL/cache/staging/health primitives。 | 成熟 | `crates/nako-vfs/src/lib.rs`、`local.rs`、`webdav.rs`、`opendal.rs`、`cache.rs` | 后端 breadth 仍有限；multi-backend config、remote write/import promotion、destructive cache cleanup policy 还需产品化。 |
| `nako-library` | Scan/index/probe orchestration、local inference、source hash、library ingestion workflow。 | 已实现 | `crates/nako-library/src/` | Watcher/debounce、large-library incremental scan、anime/series path heuristics、scan repair UX 仍是缺口。 |
| `nako-media-probe` | 从 VFS-backed source 提取媒体技术事实。 | Shallow/合理 | `crates/nako-media-probe/src/lib.rs` | 单文件、职责集中是合理的；真实外部 probe/ffprobe 失败矩阵和 remote staging 覆盖仍要看播放/库流水线集成测试。 |
| `nako-naming` | 文件名和路径解析 helper。 | Shallow/薄弱 | `crates/nako-naming/src/lib.rs` | 命名规则是 media server 高风险区；当前适合 foundation，不足以支撑复杂 anime/series/special/edition 识别。 |
| `nako-metadata` | Provider runtime、TMDB/Bangumi/Douban adapters、matching、candidate review、hierarchy confirmation。 | 已实现/成熟 | `crates/nako-metadata/src/types.rs`、`registry.rs`、`strategy.rs`、`candidate_review.rs`、`confirmation.rs`、`tests.rs` | Provider breadth 仍有限；accepted hierarchy automation、provider conflict UX、bulk repair policy 仍需深化。 |
| `nako-nfo` | NFO codec、import/export、preview、sidecar apply workflow。 | 已实现 | `crates/nako-nfo/src/`、`crates/nako-server/src/app/nfo.rs` | 需要更多 round-trip 真实样本、跨 provider metadata lock 行为、remote storage sidecar 写入策略。 |
| `nako-catalog` | Catalog graph hydration/search projection foundation。 | Foundation | `crates/nako-catalog/src/lib.rs` | 单文件、Adapter breadth 有限；catalog query scale、N+1、cursor pagination 和 rebuild 操作需要压力测试。 |
| `nako-search` | Search document/projection primitives。 | Foundation/Shallow | `crates/nako-search/src/lib.rs` | 更像 projection primitive；还不是 mature search engine seam。需要 ranking、language/anime aliases、incremental rebuild 证据。 |
| `nako-playback` | Pure playback decision planning、policy/capability matching、selected playback requirements。 | 已实现/聚焦 | `crates/nako-playback/src/` | 边界正确：不放 FFmpeg process execution；device profile breadth、subtitle/tone mapping policy 还需更多真实客户端矩阵。 |
| `nako-transcode` | FFmpeg command planning、hardware capability inventory、transcode/remux/HLS request/artifact modeling。 | 成熟 | `crates/nako-transcode/src/` | HLS/fMP4/adaptive 已有深度；实际 GPU scheduling、tone mapping execution、seek restart、LL-HLS/CMAF 仍是后续缺口。 |
| `nako-streaming` | Direct byte/range transport 和 streaming response mechanics。 | Shallow/合理 | `crates/nako-streaming/src/` | 当前职责小而清晰；需要端到端 remote range、HEAD/Range/cache header、abort/backpressure 测试补强。 |
| `nako-events` | Event/webhook envelopes、signing、delivery attempts/transport helpers。 | Foundation | `crates/nako-events/src/lib.rs`、`crates/nako-server/src/app/webhooks.rs` | Event outbox 基础存在；实时订阅/SSE/WebSocket/offline sync 仍未形成。 |
| `nako-automation` | Automation provider configuration 和 generated artifact workflow contracts。 | Foundation | `crates/nako-automation/src/lib.rs`、`crates/nako-server/src/app/automation.rs` | 外部自动化和 AI-generated artifact 已有治理入口，但真实 provider breadth、权限和回滚 UX 仍薄。 |
| `nako-addon-protocol` | Addon manifest、resource、event、task、health、hosted surface、permission contract。 | 已实现/风险 | `crates/nako-addon-protocol/src/lib.rs` | 4246 行单文件，协议面很宽；需要拆分 by surface/resource 并强化兼容性/versioning 测试。 |
| `nako-addon-client` | HTTP Addon Sidecar client、transport seam、manifest/resource/task/event/health validation。 | 已实现/风险 | `crates/nako-addon-client/src/lib.rs` | `AddonTransport` + `ReqwestAddonTransport` + mock 是真实 seam；但 2870 行单文件，错误语义和 retry/timeout policy 外泄风险高。 |
| `nako-official-addon-catalog` | Official addon catalog descriptors/package facts。 | Shallow/Foundation | `crates/nako-official-addon-catalog/src/lib.rs` | 作为 catalog facts 合理；需要 manager lifecycle/package install 状态落地后才算成熟。 |
| `nako-reference-addon` | Local reference addon fixture。 | Shallow | `crates/nako-reference-addon/src/lib.rs` | 适合协议 fixture；不能替代真实 sidecar lifecycle/process manager 测试。 |
| `nako-client-protocol` | Public Client wire contract、route inventory、catalog DTO。 | 已实现/风险 | `crates/nako-client-protocol/src/lib.rs`、`catalog.rs` | `PUBLIC_CLIENT_ROUTES` 当前约 49 条；DTO/route inventory 宽，SDK/UniFFI/Web drift guard 需要持续强化。 |
| `nako-client-core` | Transport-neutral client request builders/response mapping。 | 已实现 | `crates/nako-client-core/src/` | 架构方向正确；需要保证移动端实际使用路径覆盖到所有 public route 类型。 |
| `nako-client` | Async HTTP client adapter、request/response mapping、streaming builder。 | 已实现/风险 | `crates/nako-client/src/lib.rs` | `ClientTransport` + `ReqwestTransport` + mock 是真实 seam；单文件 2452 行，route 增长会提高维护成本。 |
| `nako-client-cli` | CLI client surface。 | Shallow | `crates/nako-client-cli/src/` | 当前更像验证面；不是完整用户 CLI 管理体验。 |
| `nako-client-uniffi` | UniFFI binding surface for mobile/shared core。 | 已实现/风险 | `crates/nako-client-uniffi/src/lib.rs` | 绑定面已暴露很多 builder；需要与 `nako-client-core` 和 route inventory 的 contract drift guard 持续同步。 |
| `nako-api` | Public/Admin DTO、admin contract、public client mapping。 | 成熟/风险 | `crates/nako-api/src/public_client.rs`、`admin_contract.rs`、`admin.rs` | Admin contract suffix 当前 114 个，HTTP/admin DTO 面非常宽；contract generation 和 TS/web sync 是必守门。 |
| `nako-server` | Composition、HTTP、app services、jobs/runtime、playback runtime、addons、admin diagnostics。 | 成熟/高风险 | `crates/nako-server/src/app/`、`crates/nako-server/src/http/` | 最大 god crate。`http/admin.rs` 约 158k bytes，`app/playback/mod.rs` 约 71k bytes；继续加能力前要拆深模块而不是加更多 helper。 |
| `nako` | Package-level addon/protocol convenience crate。 | Shallow | `crates/nako/src/lib.rs` | 合理薄层；无明显问题。 |
| `nako-uniffi-bindgen` | UniFFI binding generation helper。 | Shallow | `crates/nako-uniffi-bindgen/src/` | 工具型 crate，成熟度取决于 binding CI。 |

## 领域能力状态矩阵

| 能力面 | 已实现 | 薄弱/缺失 | 关键路径 |
| --- | --- | --- | --- |
| Media Library | Library domain、preset/profile、scan snapshot、scan job、public/admin route 和 repository 已存在；ADR 0010/0012 把 Library 作为管理边界而不是 media kind。 | Onboarding 体验、watcher/debounce、scan repair、large-library incremental behavior、自动库更新产品化不足。 | `crates/nako-core/src/media/library.rs`、`crates/nako-library/src/`、`crates/nako-server/src/app/library.rs`、`crates/nako-server/src/http/library.rs`、`docs/architecture/LIBRARY_PIPELINE.md` |
| Media Source | `MediaSource`、VFS URI、probe facts、fingerprint hash、duplicate relationship/tombstone 基础已存在；Source 与 Item 分离符合 Nako 领域语言。 | Source Variant/quality edition、remote source promotion、rename/move reconciliation、duplicate repair UX 仍需深化。 | `crates/nako-core/src/media/source.rs`、`crates/nako-core/src/repository/media.rs`、`crates/nako-server/src/app/source_hash.rs`、`crates/nako-db/src/*/media*` |
| Media Item | `MediaItem` + `CanonicalMetadata` 是 item-facing API shape；catalog/public DTO、artwork、provider mapping、candidate review 连接起来。 | Series/season/episode hierarchy、collections、people graph、home videos/photos/music 等非 video-first 类型仍不够完整；item detail scale 和 N+1 需要压测。 | `crates/nako-core/src/media/item.rs`、`crates/nako-core/src/media/catalog.rs`、`crates/nako-api/src/public_client.rs`、`crates/nako-server/src/http/catalog.rs` |
| Local Inference | Scan/probe 中保存 `LocalInferenceEvidence`；命名与路径解析已有 foundation；catalog governance 可看到 best local inference。 | Heuristics 明显还浅：anime absolute episode、season specials、multi-edition、multi-part、language/subtitle sidecars、confidence explain/override 都未成熟。 | `crates/nako-core/src/media/source.rs`、`crates/nako-core/src/repository/metadata.rs`、`crates/nako-library/src/`、`crates/nako-naming/src/lib.rs` |
| Canonical Metadata | Metadata merge policy、field locks、provider mapping、candidate review、batch apply、related hierarchy application、NFO/addon/generated artifact apply 都有实现。 | Provider breadth、自动 accepted hierarchy 的安全策略、conflict UX、metadata/artwork repair journey 仍缺产品闭环。 | `crates/nako-core/src/media/merge.rs`、`candidate.rs`、`metadata.rs`、`crates/nako-metadata/src/candidate_review.rs`、`confirmation.rs`、`crates/nako-server/src/app/metadata.rs` |
| User Playback State | `/users/me/playback-state/...` public routes 已实现 progress/watched/continue watching；repository SQLite/Postgres 双实现；public DTO 映射存在。 | 多设备冲突/last-write 策略、实时 invalidation、favorites/ratings/hidden、profile-level history/privacy、离线同步合并仍缺。 | `crates/nako-core/src/user_playback.rs`、`crates/nako-core/src/repository/user_playback.rs`、`crates/nako-server/src/app/user_playback.rs`、`crates/nako-server/src/http/user_playback.rs`、`crates/nako-db/src/*/user_playback.rs` |
| Playback Runtime | Direct/Remux/HLS、session records、tickets、resource admission、FFmpeg CLI boundary、HLS artifact serving、renderer/casting bridge、admin runtime diagnostics 已经较深。 | Device profile breadth、seek restart、LL-HLS/CMAF、subtitle burn-in、HDR tone mapping execution、GPU scheduling、active stream UX、客户端兼容矩阵仍是 gap。 | `crates/nako-playback/src/`、`crates/nako-transcode/src/`、`crates/nako-server/src/app/playback/`、`crates/nako-server/src/http/playback.rs`、`docs/architecture/PLAYBACK.md` |
| Addon | HTTP addon 优先已落地：protocol/client/server routes、token/grants、health、events/tasks、resource search、subtitle、external acquisition、renderer adapter、official catalog/reference addon。 | Addon Manager lifecycle/process/package management 仍是 boundary/deferred；sidecar install/update/rollback/health supervision 未成熟；协议文件偏大。 | `crates/nako-addon-protocol/src/lib.rs`、`crates/nako-addon-client/src/lib.rs`、`crates/nako-server/src/app/addons.rs`、`crates/nako-server/src/http/addons.rs`、`crates/nako-official-addon-catalog/src/lib.rs` |
| Public Client API/Admin API | Public Client route inventory 约 49 条；Admin contract suffix 114 个；Rust client、core builders、UniFFI、admin TS contract guard 都存在。 | API scale/cache/cursor contracts、large-library pagination budget、OpenAPI/SDK sync breadth、breaking-change policy、mobile/TV client compatibility matrix 仍不足。 | `crates/nako-client-protocol/src/lib.rs`、`crates/nako-client/src/lib.rs`、`crates/nako-client-core/src/`、`crates/nako-client-uniffi/src/lib.rs`、`crates/nako-api/src/admin_contract.rs` |
| Storage/VFS | `StorageBackend` 有 Local FS、WebDAV、OpenDAL proof、Cached backend；server 有 backend registry、health/staging/cache repair/source hash。 | Multi-backend config、S3/SMB/NAS breadth、remote write/import promotion、recurring repair、destructive cleanup policy、backup/restore story 仍不足。 | `crates/nako-vfs/src/lib.rs`、`local.rs`、`webdav.rs`、`opendal.rs`、`cache.rs`、`crates/nako-server/src/app/storage.rs`、`docs/architecture/STORAGE_VFS.md` |
| Jobs/Control Plane | `JobKind` 15 类；DurableJobRuntime、lease/heartbeat/retry、RuntimeSupervisor、resource class mapping、admin jobs/diagnostics 存在；ADR 0053/0055 是 baseline。 | 并非所有 JobKind 都等深迁入 durable runtime；Transcode/Automation 部分仍有 migration 压力；distributed execution、recurring schedules、observability/metrics/backup 还需补。 | `crates/nako-core/src/job.rs`、`crates/nako-server/src/app/job_runtime.rs`、`jobs.rs`、`runtime.rs`、`docs/architecture/CONTROL_PLANE.md` |

## Module / Interface / Seam 观察

### 真实 seam

- DB seam 真实：`NakoDatabase` facade 后有 SQLite 与 Postgres；大量 repository trait 同时由 `SqliteStore` 和 `PostgresStore` 实现。路径：`crates/nako-db/src/facade.rs`、`crates/nako-db/src/sqlite/`、`crates/nako-db/src/postgres/`。
- VFS seam 真实：`StorageBackend` 后有 `LocalFsBackend`、`WebDavBackend`、`OpenDalStorageBackend`、`CachedStorageBackend`。路径：`crates/nako-vfs/src/lib.rs`、`local.rs`、`webdav.rs`、`opendal.rs`、`cache.rs`。
- Metadata provider seam 真实：`MetadataProvider` trait、registry、TMDB/Bangumi/Douban provider 和 fake/mock provider 都存在。路径：`crates/nako-metadata/src/types.rs`、`registry.rs`、`tests.rs`。
- Addon transport seam 真实：`AddonTransport` + `ReqwestAddonTransport` + tests/mock 支撑 protocol validation。路径：`crates/nako-addon-client/src/lib.rs`。
- Public client transport seam 真实：`ClientTransport` + `ReqwestTransport` + `MockTransport`，另有 transport-neutral `nako-client-core` 和 UniFFI binding。路径：`crates/nako-client/src/lib.rs`、`crates/nako-client-core/src/`、`crates/nako-client-uniffi/src/lib.rs`。
- Renderer/casting seam 已有 foundation：Renderer Adapter Bridge、casting command transport、official Chromecast/DLNA descriptor facts 已存在。路径：`crates/nako-server/src/app/renderer_adapter.rs`、`casting.rs`、`crates/nako-official-addon-catalog/src/lib.rs`。

### 偏 shallow 但当前合理的 module

- `nako-media-probe`、`nako-naming`、`nako-streaming` 是小而集中的 module；浅不是问题，问题在它们承载的场景未来会变复杂，需要 tests 先行。
- `nako-events`、`nako-automation`、`nako-search`、`nako-catalog` 是 foundation 性质。当前不应急着抽更多 crate，而应补真实 workflow、数据规模和第二 adapter/consumer 证据。
- `nako-client-cli`、`nako-reference-addon`、`nako-uniffi-bindgen` 是 helper/fixture/tooling；成熟度应从端到端流程判断，而不是自身代码量。

### Interface 复杂度外泄风险

- `nako-core/src/repository/` trait 很多，说明 domain/persistence seam 明确，但新增 capability 会同时扩大 core 合同、DB adapter、server app service、API DTO。建议未来每个新 capability 明确 owner crate 和 contract tests，避免 app workflow shape 直接进入 core。
- `nako-api/src/admin_contract.rs` 和 `crates/nako-server/src/http/admin.rs` 已经很宽。Admin route 增长要依赖 generator/drift guard，不能靠人工同步。
- `nako-addon-protocol/src/lib.rs` 和 `nako-addon-client/src/lib.rs` 单文件偏大。协议 contract 与 client runtime policy 继续增长时，应按 manifest/resource/task/event/health/permissions 分层。
- `nako-server/src/app/composition.rs` 和 `crates/nako-server/src/app/playback/mod.rs` 聚合太多服务。Composition 是边界，不能变成所有 lifecycle 的默认归宿；Playback 已深，后续应持续内部分模块化。

### Seam 还不够真实或 Adapter 不足

- Search/catalog projection 当前更像 internal projection foundation，不是有多个 backend 的搜索 seam。
- Automation provider 仍以 generated artifact workflow 为主，外部 provider breadth 与 user-facing governance 不足。
- Addon Manager lifecycle 是明确 deferred：protocol/client/route 很多，但 install/update/process supervision 不是 mature runtime。
- Storage backend 有多个 adapter，但生产级远程后端 breadth 仍偏少；OpenDAL 当前更像 proof。
- Realtime sync 没有形成 SSE/WebSocket/offline sync adapter seam。

## 测试覆盖与明显缺口

当前测试标记分布显示测试不是空白：`nako-server` 约 740 个 test marker，`nako-transcode` 约 126，`nako-api` 约 116，`nako-db` 约 88，`nako-vfs` 约 68，`nako-core` 约 62，`nako-metadata` 约 58，`nako-library` 约 54。`nako-playback`、`nako-addon-client`、`nako-addon-protocol`、`nako-nfo`、client/binding crate 也都有测试标记。

已有覆盖较强的区域：

- Playback/transcode planning、HLS artifact manifest、resource admission、runtime diagnostics。
- API contract drift guard，尤其 Admin contract 和 client protocol route inventory。
- DB repository adapter 双实现的基础 contract/parity。
- VFS local/WebDAV/cache 行为。
- Metadata candidate review、provider registry、merge/hierarchy confirmation。
- User Playback State 的 repository/app/http 基础路径。

明显测试缺口：

- 大库规模：catalog browse/search、continue watching、admin governance 在 10k/100k items 下的 query budget、N+1、pagination 稳定性。
- 真实媒体样本：ffprobe/ffmpeg 失败矩阵、subtitle/audio track 多样性、HDR/tone mapping、multi-version、bad container。
- Watcher/debounce：rename/move/delete/flapping remote storage、partial copy、case-only rename、scan cancellation/resume。
- Metadata/local inference：复杂 anime/series/special naming、provider 冲突、manual override 后 refresh 不回退、bulk apply conflict。
- Remote storage：range abort、network timeout/retry、cache eviction、staging cleanup、WebDAV inconsistent metadata、multi-backend config。
- Control plane：所有 `JobKind` 的 durable runtime parity、stale lease recovery、retry backoff、idempotency、job cancellation side-effect boundaries。
- Addon：sidecar process lifecycle、package install/update/rollback、manifest version compatibility、permission denial matrix、malformed response fuzzing。
- Client API：Rust client、client-core、UniFFI、Admin TS/Web 的 contract round-trip 和 route coverage gate。
- Realtime/offline sync：当前基本是缺失项，暂无可评估覆盖。

## Repo-relative 路径索引

| 主题 | 路径 |
| --- | --- |
| 领域语言 | `CONTEXT.md` |
| 全局架构 | `docs/ARCHITECTURE.md`、`docs/architecture/README.md` |
| Library pipeline | `docs/architecture/LIBRARY_PIPELINE.md`、`crates/nako-library/src/`、`crates/nako-server/src/app/library.rs` |
| Storage/VFS | `docs/architecture/STORAGE_VFS.md`、`crates/nako-vfs/src/`、`crates/nako-server/src/app/storage.rs` |
| Playback | `docs/architecture/PLAYBACK.md`、`crates/nako-playback/src/`、`crates/nako-transcode/src/`、`crates/nako-server/src/app/playback/` |
| State/access | `docs/architecture/STATE_ACCESS.md`、`crates/nako-server/src/app/access.rs`、`crates/nako-db/src/*/identity.rs` |
| Control plane | `docs/architecture/CONTROL_PLANE.md`、`crates/nako-server/src/app/job_runtime.rs`、`jobs.rs`、`runtime.rs` |
| Public client contract | `crates/nako-client-protocol/src/lib.rs`、`crates/nako-client/src/lib.rs`、`crates/nako-client-core/src/`、`crates/nako-client-uniffi/src/lib.rs` |
| Admin API contract | `crates/nako-api/src/admin_contract.rs`、`crates/nako-server/src/http/admin.rs`、`apps/admin-web/src/adminApi/generated/contract.ts`、`web/src/api/admin/generated/contract.ts` |
| Metadata | `crates/nako-core/src/media/metadata.rs`、`merge.rs`、`candidate.rs`、`crates/nako-metadata/src/` |
| NFO | `crates/nako-nfo/src/`、`crates/nako-server/src/app/nfo.rs` |
| User playback state | `crates/nako-core/src/user_playback.rs`、`crates/nako-server/src/app/user_playback.rs`、`crates/nako-server/src/http/user_playback.rs` |
| Addon | `crates/nako-addon-protocol/src/lib.rs`、`crates/nako-addon-client/src/lib.rs`、`crates/nako-server/src/app/addons.rs`、`crates/nako-server/src/http/addons.rs` |
| Renderer/casting | `crates/nako-server/src/app/renderer_adapter.rs`、`casting.rs`、`crates/nako-server/src/http/renderer.rs` |
| DB adapters | `crates/nako-db/src/facade.rs`、`crates/nako-db/src/sqlite/`、`crates/nako-db/src/postgres/` |

## 优先缺口地图

P0：产品闭环与可诊断性

- Library onboarding：创建/扫描/状态/错误/修复/重新识别/metadata/artwork 治理需要一条完整 Admin/Web journey。
- Control Plane parity：把 durable runtime、resource class、retry/cancel/idempotency 贯彻到仍薄的 JobKind，避免后台任务回到 one-off helper。
- API scale baseline：Public/Admin list/detail/search/continue watching/catalog governance 的 pagination、cache、N+1 与 contract drift guard。
- Playback operational UX：active sessions、failure evidence、client capability profile、admin diagnostics 和 remote playback guidance 需要闭环。

P1：媒体语义深度

- Local Inference/name parsing：优先补 anime/series/special/edition/multi-part/subtitle sidecar 的测试驱动规则，而不是先扩 provider。
- Canonical Metadata governance：provider conflict、accepted hierarchy、bulk apply、manual override 后 refresh 行为需要更强可解释性。
- Storage/VFS remote maturity：multi-backend config、remote cache/staging repair、safe cleanup、import promotion。
- Addon Manager：从 protocol/client 进入 sidecar lifecycle、install/update/rollback/supervision。

P2：生态与高级能力

- Realtime/SSE/WebSocket/offline sync。
- Broad client/device compatibility matrix、TV client profile、Downloads/optimized versions。
- Live TV/DVR、advanced media optimizer、full metrics/backup/restore UX。

## 结论

Nako 已经具备成熟 media server backend 的骨架和不少深实现，尤其 Storage/VFS、Playback Runtime、Metadata Governance、Public/Admin contract、DB adapter parity、Addon protocol/client。下一阶段不应只“新增更多 capability”，而应优先把现有能力串成可诊断、可恢复、可扩展的产品闭环，并把 `nako-server`、`nako-api`、`nako-addon-*` 的宽界面继续向更深的内部 module 和 contract tests 收敛。
