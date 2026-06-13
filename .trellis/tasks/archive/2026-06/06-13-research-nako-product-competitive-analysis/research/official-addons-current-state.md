# Research: official-addons-current-state

- Query: nako-official-addons 当前产品能力、Addon 协议落地程度、可对标 Jellyfin/Plex 插件生态的机会与缺口
- Scope: mixed
- Date: 2026-06-13

## Findings

### 1. 仓库结构和已有 Addon

`F:\SourceCodes\Rust\nako-official-addons` 是官方 Addon Sidecar 工作区，当前 release target 为 `v0.1.0-alpha.2`，面向 Nako Addon Protocol `0.1.0-alpha.1`，并依赖主仓库本地 `nako-addon-protocol` / `nako-addon-client` / `nako-official-addon-catalog` `0.1.0-alpha.2` 版本（`README.md:5`，`README.md:241-254`，`Cargo.toml:14-38`）。

目录分工：

- `Cargo.toml`: Rust workspace，包含 7 个官方 Addon crate（`Cargo.toml:1-10`）。
- `crates/`: 每个官方 Addon 的 Rust HTTP sidecar 实现；各 crate 使用 Axum/Tokio，并通过 `nako-addon-protocol` 和 `nako-official-addon-catalog` 生成/校验 manifest（`crates/*/Cargo.toml:16-24`，`crates/*/src/manifest.rs`）。
- `addons/`: 每个 Addon 的操作员分发材料，包括 `manifest.example.json`、`Dockerfile`、`compose.example.yml` 和 `smoke.local.ps1`；`metadata-scraper` 额外有 `systemd.example.service`，`notification-bridge` 额外有 `smoke.live.ps1`。
- `addons/browser-worker`: Node/Crawlee/Playwright 风格的浏览器渲染辅助服务，不是 Nako Addon manifest 单元，但被 metadata scraper 的 rendered providers 使用；暴露 `GET /health`、`GET /fixtures/rendered-page`、`POST /render`，并强调代理、cookie、错误响应脱敏（`addons/browser-worker/README.md:5-20`，`addons/browser-worker/README.md:34-100`）。
- `docs/workstreams/`: 旧 workstream 证据，覆盖 metadata provider hardening、resource search first-class protocol、browser-worker drift、external acquisition 等；本次只作为历史设计证据，不视为当前用户可用能力。
- `scripts/smoke_official_addon_container.py`: 容器 smoke 脚本，目前只覆盖 `metadata`、`notification`、`chromecast` 三类（`scripts/smoke_official_addon_container.py:12-33`）。

已有 Addon/辅助服务：

| Addon / 服务 | 当前形态 | 主要能力 | Manifest / route 证据 |
| --- | --- | --- | --- |
| `nako-metadata-scraper` | Rust HTTP sidecar | `metadata` resource、`bulk-metadata-scrape` task、`library.scanned` event proof、可选 metadata/artwork writeback | `addons/metadata-scraper/manifest.example.json:8-120`，`crates/nako-metadata-scraper/src/routes.rs:62-70` |
| `nako-notification-bridge` | Rust HTTP sidecar | `library.scanned` event ACK；可选单一 provider fan-out：HTTP webhook、Discord、Telegram；`/providers/test-send` 和 diagnostics | `addons/notification-bridge/manifest.example.json:8-39`，`crates/nako-notification-bridge/src/routes.rs:42-46` |
| `nako-chromecast-renderer` | Rust HTTP sidecar | `renderer_adapter` resource；Chromecast readiness、manual/live discovery gated、command envelope translation；默认偏 plan-only/安全控制 | `addons/chromecast-renderer/manifest.example.json:8-80`，`crates/nako-chromecast-renderer/src/routes.rs:30-35` |
| `nako-dlna-renderer` | Rust HTTP sidecar | `renderer_adapter` resource；DLNA plan-only foundation、manual targets、安全命令计划 | `addons/dlna-renderer/manifest.example.json:8-75`，`crates/nako-dlna-renderer/src/routes.rs:30-35` |
| `nako-resource-search` | Rust HTTP sidecar | `resource_search` 和 `resource_link_check`；fixture search/link-check；可选 PanSou-compatible provider | `addons/resource-search/manifest.example.json:8-125`，`crates/nako-resource-search/src/routes.rs:33-38` |
| `nako-external-acquisition-runner` | Rust HTTP sidecar | `external-acquisition-action` task；fixture/no-op action；可选 host materialization + Transmission profile | `addons/external-acquisition-runner/manifest.example.json:8-120`，`crates/nako-external-acquisition-runner/src/routes.rs:37-41` |
| `nako-subtitle-provider` | Rust HTTP sidecar | `subtitle` resource；fixture-backed read-only inline subtitle candidates | `addons/subtitle-provider/manifest.example.json:8-75`，`crates/nako-subtitle-provider/src/routes.rs:29-34` |
| `browser-worker` | Node 辅助服务 | rendered-page extraction/render drift for metadata providers；不是独立 Addon | `addons/browser-worker/README.md:5-20`，`crates/nako-metadata-scraper/README.md:306-313` |

### 2. 当前能力 / 分发形态

当前产品能力已经超过“hello world Addon”，但仍明显是 alpha 自托管实验形态：

- Metadata 是最成熟能力。`nako-metadata-scraper` 默认启用 fixture，TMDB、Bangumi、Douban、AniList 以及一批 AV providers 默认关闭并通过 env/preset 启用；provider registry、provider execution、field policy、ranking/fusion、external ID direct lookup、bulk scrape resume/cooldown、render drift 等均已有实现或文档化路径（`crates/nako-metadata-scraper/README.md:12-102`，`crates/nako-metadata-scraper/README.md:126-194`，`crates/nako-metadata-scraper/README.md:259-304`）。
- Metadata side effects 已落到 Nako runtime API，而不是 sidecar 直写数据库/文件系统。metadata writeback 和 artwork writeback 只有在 request payload 包含对应 writeback 对象且 Nako 授权时才提交（`crates/nako-metadata-scraper/README.md:244-256`，`crates/nako-metadata-scraper/src/nako_runtime.rs:82-100`）。
- Notification bridge 已有真实 provider proof，但产品策略是“精确一个 provider 显式启用”；默认 ACK-only，多个 provider 同时启用 fail-closed，避免 host retry 下重复发送（`crates/nako-notification-bridge/README.md:7-11`，`crates/nako-notification-bridge/README.md:112-116`，`crates/nako-notification-bridge/src/routes.rs:121-166`）。
- Renderer adapter 有 Chromecast 和 DLNA 两条线。Chromecast 支持可选 live LAN discovery/control gate，DLNA 当前 plan-only；两者都强调不泄漏 media URL、ticket、bearer token、LAN host（`crates/nako-chromecast-renderer/README.md:5-15`，`crates/nako-dlna-renderer/README.md:5-9`）。
- Resource Search 和 External Acquisition 被刻意拆开：search/link-check 只返回候选和安全检查，执行下载/入库是 separate action addon。这个分层避免搜索 provider 直接获得执行权限（`README.md:25-31`，`README.md:177-181`，`crates/nako-resource-search/README.md:5-7`）。
- Subtitle provider 当前是只读 fixture foundation，不写字幕文件、不调用 live subtitle provider（`crates/nako-subtitle-provider/README.md:5-9`）。
- 分发方式主要是源码 workspace、crate binary、Dockerfile、Compose 示例、systemd 示例、manual smoke。Nako 主仓库也明确当前 Addons 是外部运行 sidecar，Nako 尚不安装、更新、启动或停止 Addon（`F:\SourceCodes\Rust\nako\README.md:52-64`，`docs/guides/ADDON_AUTHOR_GUIDE.md:225-263`）。
- 官方 catalog 的事实源在主仓库 `nako-official-addon-catalog`，包含每个官方 Addon 的 manifest builder、binary/container install descriptor，并通过 protocol validation 测试防漂移（`.trellis/spec/nako-official-addon-catalog/backend/index.md:1-29`，`crates/nako-official-addon-catalog/src/lib.rs:1442-1506`）。

分发成熟度判断：

- 已有：每 Addon 的 manifest、Dockerfile、Compose 示例、本地 smoke、部分容器 smoke、Nako Admin-mediated smoke 入口。
- 部分已有：Install Guide / Manager Plan 是 Nako host 侧协议和 Admin API 读模型，不是自动生命周期管理（`docs/guides/ADDON_AUTHOR_GUIDE.md:225-263`）。
- 尚缺：稳定官方 Addon catalog/marketplace UI、包签名、版本兼容矩阵、自动更新、统一 Addon Suite 镜像/compose、生命周期控制、第三方开发者 SDK/模板和发布流程。

### 3. 与 Nako host 的契约

Nako 的 Addon 模型不是 in-process plugin，而是 sidecar protocol。关键契约：

- 协议版本：`ADDON_PROTOCOL_VERSION = "0.1.0-alpha.1"`，`SUPPORTED_ADDON_PROTOCOL_VERSIONS` 当前只含这一版（`crates/nako-addon-protocol/src/lib.rs:8-9`）。Addon 自身 version、protocol version、Rust crate version 是三层不同版本（`nako-official-addons/README.md:252-257`，`docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md:50-73`）。
- Manifest：必填 id/name/version/protocol_version/base_url/resources/scopes；Nako 注册时存 validated snapshot，Addon 默认 disabled，启用需要显式 grant 每个 resource/task/event 需要的 scope（`docs/guides/ADDON_AUTHOR_GUIDE.md:5-21`，`docs/guides/ADDON_AUTHOR_GUIDE.md:179-192`）。
- Runtime calls：sidecar 提供 `GET /manifest.json` 和 `POST /health`，resource/task/event 使用 envelope，并必须 echo protocol/addon/resource/request identity；Nako 校验 identity 后才信任响应（`docs/guides/ADDON_AUTHOR_GUIDE.md:81-168`，`crates/nako-addon-client/src/lib.rs:935-989`）。
- Scope 和 grant 分离：manifest scope 是“可请求能力”，accepted grants 是 Nako 允许执行的能力。`nako-addon-client` 在 HTTP 调用前验证 manifest 和 granted scopes，specialized helpers 还校验 `resource_search`、`resource_link_check`、`subtitle` schema（`.trellis/spec/nako-addon-client/backend/index.md:1-39`，`crates/nako-addon-client/src/lib.rs:1191-1263`）。
- Protected side effects：Addon 可做 metadata/artwork/subtitle 等强副作用，但必须通过 Nako-owned APIs、Addon Token、permissions、audit、resource boundary；协议 crate 只定义 wire payload，不拿 Nako 内部 domain 类型（`CONTEXT.md:417-480`，`docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md:39-50`，`docs/adr/0035-addon-native-metadata-writeback.md:23-42`）。
- Host-owned resource flow：Resource Search、Subtitle Import、External Acquisition 都要求先由 Addon 做只读 discovery，再由 Nako 存短期 selection session、生成 apply/materialization plan、执行 VFS/Library write/durable job，并对 Admin 响应脱敏（`.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md:1-53`）。
- External acquisition materialization：sidecar 执行 action 时只能拿 host-owned opaque refs；如需 raw material，调用 Nako runtime `/addon/v1/acquisition/materialize`，且 Nako 校验 running task/token/idempotency/audit 后才返回 material（`crates/nako-addon-protocol/src/lib.rs:33-37`，`crates/nako-addon-client/src/lib.rs:1026-1048`，`crates/nako-external-acquisition-runner/src/materialization.rs:121-180`）。
- Install descriptor/guide：descriptor 可包含 binary/container runtime reference 和 Secret Reference binding，但 protocol validation 拒绝本地绝对路径、明文 secret value；install guide 是 inert operator instructions，不含 raw token/secret/process-control 指令（`crates/nako-addon-protocol/src/lib.rs:165-192`，`crates/nako-addon-protocol/src/lib.rs:2383-2447`，`docs/guides/ADDON_AUTHOR_GUIDE.md:225-263`）。
- UI 扩展：Addon Entry Point 和 Hosted Page 可用于设置/诊断，但 Hosted Page 不可信，不接收 Nako admin bearer token 或 Addon Token（`CONTEXT.md:91-101`，`CONTEXT.md:481-482`，`docs/guides/ADDON_AUTHOR_GUIDE.md:38-49`）。

协议落地程度评估：

- 已落地较完整：manifest validation、health、resource call、task call、event delivery proof、scope grant、runtime side effects、specialized schema helpers、install descriptor/guide、official catalog builder、redaction-safe diagnostics。
- 部分落地：Addon Manager plan/read model 已有 guide 约束，但生命周期 automation 不是当前产品能力；Addon Suite 概念有 ADR，但仓库目前仍主要按一个 sidecar 一个 Addon 分发。
- 仍需硬化：多协议版本适配、第三方 Addon conformance tests、host 与 official-addons 的跨仓库 release gate、容器 smoke 覆盖所有 Addon、package signing/source catalog。

### 4. 插件生态差异化机会

对标 Jellyfin/Plex 的现实差异：

- Jellyfin 有成熟 plugin catalog 和官方/第三方 repository URL 机制，文档列出 Metadata、Authentication、Channels、Live TV、Notifications、Reports、Subtitle 等类别，并支持从 catalog 或手动放入 plugin directory 安装（Jellyfin docs: https://jellyfin.org/docs/general/server/plugins/）。
- Jellyfin 生态覆盖了 Anilist/Anidb/Kitsu/Fanart/Open Subtitles/Trakt/LDAP/TVHeadend/Playback Reporting 等官方插件，也有多个第三方 repository（Jellyfin docs lines 54-79, 83-360）。
- Plex 在 2018 年公告逐步 phase out browsable plugins、移除 Plugin Directory，但 scanners 和 metadata agents 不受该公告影响，并建议 utility plugins 转为 standalone apps；这反而印证 Nako sidecar/standalone utility 方向更符合长期维护边界（Plex forum: https://forums.plex.tv/t/discontinuation-of-plugins-watch-later-recommended-and-cloud-sync/312312）。

Nako 可差异化的机会：

1. **安全边界作为卖点**：Jellyfin 插件强在 catalog 和生态广度，但 in-process/服务器内部模型会带来 ABI、权限、崩溃隔离和升级耦合成本。Nako 的 sidecar + scoped grants + host-owned side effects 可以作为“自托管可扩展但不把核心进程交给插件”的定位。
2. **Addon Suite 分发**：ADR 0034 已定义 Suite：部署上可一组服务/一个包，授权上仍按 per-Addon manifest/grants/tasks/events 审计。可把 metadata + browser-worker + resource-search + acquisition-runner 打成官方媒体扩展套件，降低 Compose 复杂度，同时保留权限细粒度（`docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md:39-62`）。
3. **Provider breadth + policy depth**：metadata scraper 已有多 provider registry、AV provider presets、field policy、render drift。相比常见单 provider 插件，Nako 可以突出“多源融合、字段级来源策略、可诊断 provider wave”的差异化（`crates/nako-metadata-scraper/README.md:126-194`）。
4. **Host-owned acquisition workflow**：Resource Search 只读、External Acquisition action 单独授权、materialization 由 host gate；这比“搜索插件直接下载/写库”更适合私有库、云盘链接、BT/下载器等高风险场景。
5. **Diagnostics-first operations**：每个 Addon 有 `/health` 和 `/ui/diagnostics`，health response/report 只暴露 safe facts；可形成“插件健康、权限、token、provider readiness、drift case”的管理员体验。
6. **现代 Addon authoring SDK**：当前 Rust protocol/client 已有，但第三方开发门槛仍高。可做 TypeScript/Python sidecar template + conformance test harness，吸收 Plex/Jellyfin 历史上大量脚本型生态作者。
7. **真实缺口优先级**：Jellyfin 类别中 Nako 当前缺 `auth/LDAP/SSO`、`Trakt/scrobble`、`Live TV/tuner`、`reports/playback analytics`、`OpenSubtitles live provider`、`theme/preroll/local intros`、`books/music/provider` 等；这些比继续扩 AV provider 更能扩大生态观感。

### 5. 生态风险和下一步建议

主要风险：

- **生态入口缺失**：没有 Jellyfin 式稳定 catalog/marketplace，用户需要手工跑 sidecar、粘 manifest 或用 compose；这会限制非开发者 adoption。
- **生命周期边界容易误解**：Nako 文档明确 Addon Manager 不是 first protocol slice，且不应直接管理容器/进程；但用户会自然期待“一键安装/更新/启动/日志”。必须把 Install Guide、Manager Plan 和未来 Manager 的边界讲清楚（`CONTEXT.md:490-492`，`docs/guides/ADDON_AUTHOR_GUIDE.md:249-263`）。
- **跨仓库漂移**：official-addons 依赖主仓库 protocol/catalog 本地 path；manifest.example、catalog builder、runtime routes、smoke scripts 三者容易漂移。已有 manifest drift tests，但容器 smoke 覆盖不全。
- **Provider 合规和可靠性风险**：多 AV/rendered provider、代理/cookie、站点 drift、成人内容、第三方 ToS/反爬和地区可达性都可能使官方生态背负运营风险。默认关闭和 manual live drift 是正确方向，但需要更明确 provider policy。
- **Secret / token / URL 泄漏风险**：仓库已有大量 redaction tests；随着 live provider、host materialization、notification fan-out 扩展，必须把“响应、诊断、日志、Debug 不泄漏”作为 conformance gate。
- **任务/事件幂等和重试语义复杂**：Notification provider fan-out 目前禁止多个 provider 是保守正确；未来多 provider/多动作必须避免 host retry 导致重复副作用。External acquisition/materialization 同样依赖 idempotency 和 host task alignment。
- **协议版本 alpha**：`0.1.0-alpha.1` 不代表长期兼容，生态还没到第三方稳定承诺阶段；现在发布过多第三方指南会带来迁移负担。

下一步建议：

1. **先做官方 Addon Catalog 最小闭环**：提供一个 machine-readable official catalog，列出 addon id、version、protocol version、image、compose snippet、required scopes、health path、docs URL、compat Nako version；由 `nako-official-addon-catalog` 生成，避免手写漂移。
2. **把容器 smoke 扩到 7 个 Addon + browser-worker 集成**：`scripts/smoke_official_addon_container.py` 当前只有 metadata/notification/chromecast，应补 resource-search、external-acquisition-runner、subtitle-provider、dlna-renderer，并加 metadata + browser-worker 的 rendered fixture smoke。
3. **打造 Addon Suite 分发体验**：先不做生命周期控制，先给一个官方 Compose suite：metadata-scraper + browser-worker + resource-search + subtitle-provider + notification-bridge，配套 install guide 和 Manager Plan。
4. **建立第三方 author kit**：Rust/TypeScript/Python 三个最小 sidecar template，附 manifest validation、health/resource/task/event conformance tests、redaction test helper、Dockerfile 模板。
5. **补 Jellyfin 感知的高价值类别**：短期优先 OpenSubtitles live provider、Trakt/scrobble、LDAP/SSO 或 OIDC auth bridge、playback reporting/reports、local intros/preroll。它们比更多同类 metadata provider 更能体现“生态”。
6. **把 host-owned flow 文档产品化**：Resource Search -> Selection -> Acquisition Intake -> Materialization -> Runner 的链路是 Nako 差异化核心，应有一张用户/开发者流程图和端到端 smoke。
7. **定义 provider trust tier**：官方 provider、community provider、adult/rendered provider、private-network runner、notification provider 分层展示默认启用策略、所需 secret、网络权限、失败类型和用户风险。
8. **延后“插件内 UI”野心**：当前 Hosted Page 不可信是正确边界。先把 diagnostics/settings schema 做好，不要承诺嵌入式前端插件执行。

### 6. Files found

主仓库 `F:\SourceCodes\Rust\nako`：

- `CONTEXT.md`: Nako Addon 术语、Sidecar/Protocol/Suite/Token/Task/Event/Side Effect/Manager 边界。
- `README.md`: Nako 当前 alpha 能力、Addon Sidecar 边界、官方 E2E smoke 状态。
- `.trellis/workflow.md`: Trellis research 必须持久化到 task research 文件。
- `.trellis/spec/nako-addon-protocol/backend/index.md`: Addon wire contract crate 边界和验证要求。
- `.trellis/spec/nako-addon-client/backend/index.md`: sidecar HTTP caller、scope grant、schema validation、redaction 规则。
- `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`: host-owned discovery/selection/materialization/apply pattern。
- `.trellis/spec/nako-server/backend/index.md`: server 层相关 Addon flow spec 路由。
- `.trellis/spec/nako-official-addon-catalog/backend/index.md`: official addon fact source of truth。
- `.trellis/spec/guides/index.md`: cross-layer / reuse thinking guide 索引。
- `crates/nako-addon-protocol/README.md`: 协议 crate 简述。
- `crates/nako-addon-protocol/src/lib.rs`: manifest/resource/task/event/health/side-effect/install descriptor DTO 和 validation。
- `crates/nako-addon-client/src/lib.rs`: resource/task/event/health 调用、specialized helpers、Nako runtime side-effect/materialization client。
- `crates/nako-official-addon-catalog/src/lib.rs`: 官方 Addon manifest/install descriptor builder。
- `docs/guides/ADDON_AUTHOR_GUIDE.md`: Addon author manifest、resource、health、registration、grant、install guide、protected writes 文档。
- `docs/adr/0003-http-addons-before-in-process-plugins.md`: 先 HTTP Addon、后 native plugin 的架构决策。
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`: capability-scoped HTTP addon 决策。
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`: Jellyfin-like experience 但采用 sidecar + scoped Nako API。
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`: Addon Protocol Version 与 Addon/crate version 分离。
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`: Addon Package/Suite 分发边界。
- `docs/adr/0035-addon-native-metadata-writeback.md`: canonical metadata-shaped writeback。
- `docs/adr/0053-application-control-plane-boundary.md`: durable jobs、runtime supervision、diagnostics/control-plane 基线。

插件仓库 `F:\SourceCodes\Rust\nako-official-addons`：

- `README.md`: 官方 Addon 总览、当前 release target、默认端口、provider defaults、smoke、Nako relationship、license/reference policy。
- `Cargo.toml`: workspace 成员、统一版本、依赖。
- `CHANGELOG.md`: alpha.1/alpha.2 能力演进。
- `LICENSE`: AGPL-3.0-or-later。
- `scripts/smoke_official_addon_container.py`: 容器 smoke，当前仅 metadata/notification/chromecast。
- `addons/metadata-scraper/*`: metadata scraper manifest、Docker/Compose/systemd/smoke。
- `addons/notification-bridge/*`: notification bridge manifest、Docker/Compose/local/live smoke。
- `addons/chromecast-renderer/*`: Chromecast renderer manifest、Docker/Compose/smoke。
- `addons/dlna-renderer/*`: DLNA renderer manifest、Docker/Compose/smoke。
- `addons/resource-search/*`: resource search manifest、Docker/Compose/smoke。
- `addons/external-acquisition-runner/*`: external acquisition runner manifest、Docker/Compose/smoke。
- `addons/subtitle-provider/*`: subtitle provider manifest、Docker/Compose/smoke。
- `addons/browser-worker/README.md`: browser worker render/extract/health/drift contract。
- `addons/browser-worker/package.json`: Node helper package scripts/dependencies。
- `addons/browser-worker/src/*.mjs`: render/extract server implementation。
- `addons/browser-worker/test/*.mjs`: browser-worker tests。
- `crates/nako-metadata-scraper/**`: metadata sidecar routes/config/provider registry/engine/runtime/writeback/providers/tests。
- `crates/nako-notification-bridge/**`: event route/provider registry/provider send/template/diagnostics/attempt history。
- `crates/nako-chromecast-renderer/**`: renderer adapter route/config/chromecast implementation。
- `crates/nako-dlna-renderer/**`: DLNA plan-only route/config implementation。
- `crates/nako-resource-search/**`: resource_search/link_check route/domain/engine/provider/fusion/PanSou adapter。
- `crates/nako-external-acquisition-runner/**`: acquisition action route/runner/materialization/transmission adapter。
- `crates/nako-subtitle-provider/**`: subtitle resource route/config/fixture subtitles。
- `docs/workstreams/official-resource-search-first-class-protocol/DESIGN.md`: resource_search first-class protocol historical design evidence。
- `docs/workstreams/official-resource-search-pansou-compatible-provider/DESIGN.md`: PanSou-compatible provider design evidence。
- `docs/workstreams/official-metadata-*`: metadata provider breadth/hardening/live drift/writeback historical evidence。
- `docs/workstreams/official-media-extension-addons/*`: media extension addon historical evidence。
- `docs/workstreams/official-external-acquisition-*`: acquisition runner/materialization/transmission historical evidence。

### Code patterns

- 每个 Rust Addon 都遵循 `GET /manifest.json`、`POST /health`、resource/task/event route、`GET /ui/diagnostics` 的 Axum router 形态：metadata (`crates/nako-metadata-scraper/src/routes.rs:62-70`)、notification (`crates/nako-notification-bridge/src/routes.rs:42-46`)、resource search (`crates/nako-resource-search/src/routes.rs:33-38`)、subtitle (`crates/nako-subtitle-provider/src/routes.rs:29-34`)、renderer (`crates/nako-chromecast-renderer/src/routes.rs:30-35`，`crates/nako-dlna-renderer/src/routes.rs:30-35`)。
- Manifest 不在 Addon 里手写全部事实，而是从 `nako-official-addon-catalog` builder 生成并和 checked-in example 比对：metadata (`crates/nako-metadata-scraper/src/manifest.rs:15-18`)、resource search (`crates/nako-resource-search/src/manifest.rs:28-31`)、catalog tests (`crates/nako-official-addon-catalog/src/lib.rs:1442-1506`)。
- Typed protocol resources 已覆盖 `metadata`、`webhook`、`resource_search`、`resource_link_check`、`subtitle`、`renderer_adapter`，scopes 覆盖 metadata suggest/read/write、automation、webhook、renderer、acquisition、subtitle（`crates/nako-addon-protocol/src/lib.rs:274-300`，`crates/nako-addon-protocol/src/lib.rs:553-587`）。
- Response/request Debug 和 diagnostics 多处显式脱敏：resource link、subtitle delivery、renderer transport、external acquisition refs、Transmission credentials、notification provider secrets（`crates/nako-addon-protocol/src/lib.rs:702-720`，`crates/nako-addon-protocol/src/lib.rs:1275-1277`，`crates/nako-addon-protocol/src/lib.rs:1515-1606`，`crates/nako-external-acquisition-runner/src/transmission.rs:211-256`，`crates/nako-notification-bridge/src/routes.rs:1430-1635`）。
- `nako-addon-client` 负责调用前 manifest/scope/schema validation，避免 server 或 Addon 直接拼 reqwest 调用：resource search/subtitle helpers (`crates/nako-addon-client/src/lib.rs:398-647`)、task/event/health helpers (`crates/nako-addon-client/src/lib.rs:665-989`)、Nako runtime side-effect/materialization (`crates/nako-addon-client/src/lib.rs:1018-1080`)。
- Notification bridge 当前避免多 provider fan-out，多个 send path 直接 fail closed（`crates/nako-notification-bridge/src/routes.rs:121-166`，`crates/nako-notification-bridge/src/routes.rs:1005-1041`）。
- External acquisition runner 对 raw material 采用 materializer 抽象，fixture no-op 与 Nako runtime materializer 可替换，runtime materializer Debug redacts token（`crates/nako-external-acquisition-runner/src/materialization.rs:56-80`，`crates/nako-external-acquisition-runner/src/materialization.rs:121-180`，`crates/nako-external-acquisition-runner/src/materialization.rs:238-248`）。

### External references

- Jellyfin Plugins docs: https://jellyfin.org/docs/general/server/plugins/ 。用于对标 catalog/manual install、official/third-party repository、plugin category breadth。检索日期：2026-06-13。
- Jellyfin plugin template repository: https://github.com/jellyfin/jellyfin-plugin-template 。用于确认 Jellyfin 提供 plugin authoring template。检索日期：2026-06-13。
- Plex forum announcement: https://forums.plex.tv/t/discontinuation-of-plugins-watch-later-recommended-and-cloud-sync/312312 。用于对标 Plex browsable plugins phased out、manual install residual、scanners/metadata agents unaffected、utility plugins 转 standalone app 的历史经验。检索日期：2026-06-13。
- Plex Plug-ins GitHub org: https://github.com/plexinc-plugins 。用于确认 Plex 历史 `.bundle` plugin/channel 生态形态。检索日期：2026-06-13。

### Related specs

- `.trellis/spec/nako-addon-protocol/backend/index.md`
- `.trellis/spec/nako-addon-client/backend/index.md`
- `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`
- `.trellis/spec/nako-official-addon-catalog/backend/index.md`
- `docs/guides/ADDON_AUTHOR_GUIDE.md`
- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `docs/adr/0035-addon-native-metadata-writeback.md`
- `docs/adr/0053-application-control-plane-boundary.md`

## Caveats / Not Found

- 未运行测试或 smoke；本文件是只读调研结论，不验证当前工作区是否可构建。
- 没有写入插件仓库或主仓库代码；唯一写入是本研究文件。
- 当前会话 `task.py current --source` 返回无 active task；本次按用户明确提供的任务目录作为研究写入边界。
- Jellyfin/Plex 对比只使用官方文档/官方论坛/GitHub 组织公开信息；未做全量插件数量统计或第三方生态质量评分。
- `addons/browser-worker/node_modules/` 目录存在大量第三方依赖 README，本次没有逐个分析依赖源码；只读取 browser-worker 自身 README/source/test 层面的能力。
- `docs/workstreams/` 含大量历史设计和 journal，本次只抽取与当前产品能力直接相关的设计证据，未把未合并或历史 TODO 当成已交付能力。
- `scripts/smoke_official_addon_container.py` 目前只支持 metadata/notification/chromecast；README 中列出的所有 local smoke 脚本不等于统一发布 gate 全覆盖。
