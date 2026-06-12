# Research: jellyfin-gap-analysis

- Query: Jellyfin 作为开源参考实现，当前 Nako 下一步最值得补的 operator/admin 能力差距是什么；同时覆盖设置/仪表盘、库扫描与计划任务、日志/诊断、修复/恢复、用户与权限、远程访问、备份/恢复、插件/扩展、客户端兼容性这些常见 operator 面。
- Scope: mixed
- Date: 2026-06-11

## Findings

### 1) Jellyfin 已验证的成熟能力模式
- Jellyfin 的 operator 面不是一个大而全的单页，而是一组清晰分工的控制器：`SystemController` 提供系统信息、存储、重启/关机、服务器日志和 endpoint 信息；`ActivityLogController` 提供分页活动日志；`ClientLogController` 提供客户端日志上报；`BackupController` 提供备份创建、枚举、manifest 与恢复；`ScheduledTasksController` 提供任务列表、启动、停止和 trigger 更新。证据见 `repo-ref/jellyfin/Jellyfin.Api/Controllers/SystemController.cs:67-206`、`ActivityLogController.cs:20-76`、`ClientLogController.cs:19-65`、`BackupController.cs:19-84`、`ScheduledTasksController.cs:17-151`。
- 库管理也是同样模式：`LibraryController` 负责刷新库、列出媒体文件夹、处理媒体增删改事件；`LibraryStructureController` 负责虚拟库、路径和 library options 的增删改，并在需要时触发刷新。见 `LibraryController.cs:335-344, 547-554, 576-648` 与 `LibraryStructureController.cs:29-351`。
- 插件/扩展能力在 Jellyfin 里已经是完整 lifecycle，而不只是“已安装列表”：`PluginsController` 覆盖安装后管理面，`PluginUpdateTask` 负责自动更新任务，`InstallationManager` 负责安装/更新事件、取消、完成后重启提示。见 `PluginsController.cs:26-190`、`PluginUpdateTask.cs:18-96`、`InstallationManager.cs:293-365`。
- 用户与权限是成熟的控制面核心：`UserController` 支持用户 CRUD、policy 更新、密码重置、Quick Connect、Forgot Password、设备/网络过滤列表；`UserPolicy` 显式建模 remote access、device、folder、playback、live TV、schedule 等权限。见 `UserController.cs:37-630` 与 `UserPolicy.cs:14-197`。
- 远程访问和客户端兼容性也不是隐式约定，而是显式模型：`StartupController` 有 remote access 初始配置；`SystemController` 有 endpoint info；`SessionController` 接收 client capabilities；`ClientCapabilitiesDto` 带有 playable media types、supported commands、media control、persistent identifier 和 `DeviceProfile`。见 `StartupController.cs:19-133`、`SystemController.cs:185-206`、`SessionController.cs:345-389`、`ClientCapabilitiesDto.cs:13-56`。

### 2) Nako 已经覆盖的相邻能力
- Nako 的 control plane 已经有明显的 operator 近邻：`CONTROL_PLANE.md` 里，Admin diagnostics 是 “good partial”，source hash durable job、VFS cache repair durable job、API version/error/page contracts、HTTP cache/ETag、N+1/list projection 都已经在进展矩阵里占位。最关键的是，`Crash/fault bundles` 明确写成 “not started”，`Remote access cookbook` 是 planned，`Endpoint discovery` 还是 not started。见 `docs/architecture/CONTROL_PLANE.md:48-55`。
- M1 诊断/修复矩阵进一步说明：operator readiness overview、source hash、duplicate repair、VFS cache repair、job queue、playback diagnostics、system/config diagnostics 都已有对应证据；而 `Incident bundles and realtime diagnostics` 仍是 deferred。见 `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:57-67`。
- 运营/发布侧已经有基础：self-hosted install docs、backup/restore docs、hardware readiness diagnostics、config mutation authority partial、observability partial。见 `docs/architecture/OPERATIONS_RELEASE.md:24-33`。
- library/intake 侧也不是空白：durable scan state、source tombstones、watcher/debounce foundation、metadata merge policy、addon-assisted metadata、artwork lifecycle 都已有相邻能力或已封闭工作流。见 `docs/architecture/LIBRARY_PIPELINE.md:26-37`。
- access/policy 侧已经有明显骨架：library access、playback permission、remote access、transcode permission 需要在昂贵工作之前先检查；这说明 Nako 的权限模型并不缺，只是 operator 面还缺更强的可观测/支持包。见 `docs/architecture/STATE_ACCESS.md:12,102`。
- Admin Web 也不是从零开始：它已经是 validation-oriented 的管理前端，并且现成有 Jobs、Settings、Media、Access、Addons 等页面/路由结构。见 `.trellis/spec/admin-web/frontend/index.md:1-10` 与 `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md:12-13,79,140,348`。

### 3) 最值得补的一天切片
- 我现在最看好的 one-day 切片是 `safe incident bundle / redacted diagnostics export`。它比“再做一个设置页”更能补 Jellyfin 那类成熟 operator 体验，也比直接开全量用户管理、插件市场、备份引擎、远程访问向导更小。
- 原因很直接：Nako 已经有了大量可拼装的安全事实来源，但缺少一个把这些事实打包成支持/排障工件的出口。`CONTROL_PLANE.md` 直接把 crash/fault bundles 标成 not started，`M1` 也把 incident bundles 留在 deferred。说明缺口不是“没有数据”，而是“没有 operator-supportable 的诊断成品”。
- 这个切片的合理边界应当是：一个 Admin API 只读导出 + 一个 Admin Web 只读投影，聚合当前安全事实（系统/存储摘要、最近失败、job queue pressure、endpoint posture、必要时再加日志索引），但不暴露 raw path、token、locator、FFmpeg 命令、backend URL、provider payload。它可以先是 JSON bundle，不必一开始就做成完整 zip/support package。
- 如果必须再收窄，次优切片是 “server log list/download + activity diagnostics”；但我认为 bundle 更值，因为它把日志、修复、网络与作业诊断统一到一个 operator 动作里。

### 4) 不适合现在开的超大项
- 全量 Jellyfin 风格用户/家庭权限系统。Nako 已经有 access/policy 骨架，但要做成可用的家庭/家长控制/会话管理体系，跨度会跨到多个路由、DTO、会话与播放策略，不是一日切片。
- 全量插件/扩展生命周期或市场。Nako 的 addon 现在是 sidecar-first 的另一条 lane，继续往 marketplace、安装、升级、健康、回滚推进会非常快变成独立项目。
- 完整备份/恢复引擎。Nako 现在只有 readiness/doc foundation；真正的备份/恢复会碰到存储分类、schema、重启、升级和权限，超出 one-day。
- 全量客户端兼容性矩阵/播放诊断 UI。那是播放 planner、capability profile、API contract、Android/客户端适配的组合 lane，更像 U2/U4 的跨层工作，不适合作为当前 follow-on 的小切片。
- 完整 remote access wizard。它值得做，但更像一个独立 control-plane/documentation lane，不如 incident bundle 这条线能更快补足 operator 诊断闭环。

## Files Found

- `.trellis/tasks/06-11-u3-follow-on-slice/prd.md` - 当前 task 的目标、约束、验收与技术备注。
- `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md` - U1-U8 roadmap，定义下一阶段的产品化切片。
- `docs/architecture/CONTROL_PLANE.md` - control plane 进度矩阵，明确 diagnostics / crash bundles / remote access / endpoint discovery 的状态。
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` - M1 operator 诊断与修复覆盖矩阵，标出 shipped / adequate / deferred。
- `docs/architecture/OPERATIONS_RELEASE.md` - install、backup/restore、hardware readiness、observability 的运维基线。
- `docs/architecture/LIBRARY_PIPELINE.md` - scan/intake/source tombstones/metadata/artwork 的相邻能力状态。
- `docs/architecture/STATE_ACCESS.md` - library access / playback permission / remote access / transcode permission 的策略边界。
- `.trellis/spec/nako-server/backend/index.md` - server 端开发入口规范与应读子规范。
- `.trellis/spec/nako-server/backend/http-api-patterns.md` - HTTP 路由、admin boundary、access check、trace context 规范。
- `.trellis/spec/nako-server/backend/logging-guidelines.md` - tracing / redaction-safe diagnostics / durable job trace context 规范。
- `.trellis/spec/nako-server/backend/error-handling.md` - HTTP 错误映射与 redaction 约束。
- `.trellis/spec/nako-server/backend/quality-guidelines.md` - route test、bounded list、admin web contract 质量门。
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` - Admin/Public contract 分层与 DTO 约束。
- `.trellis/spec/admin-web/frontend/index.md` - Admin Web 是 validation-oriented 前端的总说明。
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` - Admin Web 路由、表单、data source、测试模式。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/SystemController.cs` - 系统信息、storage、restart/shutdown、server logs、endpoint info。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ActivityLogController.cs` - 分页活动日志查询。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ClientLogController.cs` - 客户端日志上报。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/BackupController.cs` - 备份创建/枚举/manifest/restore。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs` - 计划任务列表、启动、停止、trigger 更新。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/LibraryController.cs` - 库刷新、媒体文件夹、库统计与变更入口。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/LibraryStructureController.cs` - 虚拟库/路径/options 维护。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs` - 插件启停、卸载、配置、manifest/image。
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/Tasks/PluginUpdateTask.cs` - 插件自动更新任务。
- `repo-ref/jellyfin/Emby.Server.Implementations/Updates/InstallationManager.cs` - 插件安装/更新/取消/事件/重启提示。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/UserController.cs` - 用户 CRUD、policy、password、Quick Connect、forgot password、filtered list。
- `repo-ref/jellyfin/MediaBrowser.Model/Users/UserPolicy.cs` - 用户权限和 remote access / device / playback 模型。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/StartupController.cs` - 首次配置、remote access、first user。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/DashboardController.cs` - dashboard 插件配置页。
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/SessionController.cs` - client capabilities 上报。
- `repo-ref/jellyfin/MediaBrowser.Model/Dto/ClientCapabilitiesDto.cs` - client capability / device profile DTO。

## Code Patterns

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/SystemController.cs:67-206` - system info, storage, restart/shutdown, logs, endpoint info all live in one admin/system rail.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ActivityLogController.cs:20-76` - paged activity log with filters and sort support.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ClientLogController.cs:19-65` - client log upload is gated by config and upload size.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/BackupController.cs:19-84` - backup create/list/manifest/restore are explicit operator actions.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs:17-151` - task list, start, stop, trigger updates, and per-task lookup.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/LibraryController.cs:335-344, 547-554, 576-648` - library refresh plus media folder and update notifications.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/LibraryStructureController.cs:29-351` - library topology and path maintenance with refresh hooks.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs:26-190` - installed plugin lifecycle, configuration, image, manifest.
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/Tasks/PluginUpdateTask.cs:18-96` - scheduled auto-update job for plugins.
- `repo-ref/jellyfin/Emby.Server.Implementations/Updates/InstallationManager.cs:293-365` - installation/update lifecycle emits install/updated/cancelled/failed events and pending restart.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/UserController.cs:37-630` - rich user and permission management, plus device/network filtered user listing.
- `repo-ref/jellyfin/MediaBrowser.Model/Users/UserPolicy.cs:14-197` - explicit remote access, device, playback, folder, schedule, live TV permission model.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/StartupController.cs:19-133` - first-time setup includes remote access toggle and initial user setup.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/DashboardController.cs:24-112` - dashboard exposes plugin configuration pages.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/SessionController.cs:345-389` and `repo-ref/jellyfin/MediaBrowser.Model/Dto/ClientCapabilitiesDto.cs:13-56` - client capability reporting with a full device-profile DTO.
- `docs/architecture/CONTROL_PLANE.md:48-55` - Nako still lacks crash/fault bundles and endpoint discovery.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:57-67` - Nako already shipped several repair lanes; incident bundles remain deferred.
- `docs/architecture/OPERATIONS_RELEASE.md:24-33` - backup docs and remote access cookbook are foundations/partials, not a mature operator workflow yet.

## External References

- Jellyfin source tree under `repo-ref/jellyfin` as the open-source operator/admin reference implementation.
- Local Nako architecture and ADR docs under `docs/architecture/` and `.trellis/spec/` as the current boundary set.
- No internet sources were used for this pass.

## Related Specs

- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/nako-server/backend/logging-guidelines.md`
- `.trellis/spec/nako-server/backend/error-handling.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/admin-web/frontend/index.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`

## Caveats / Not Found

- `python3 ./.trellis/scripts/task.py current --source` returned no active task in session state; the research was written to the user-specified task directory instead of guessing a new task path.
- Jellyfin here is the server/reference slice available in `repo-ref/jellyfin`; I did not inspect a full Jellyfin web UI tree, so UI conclusions come mainly from controller and capability evidence.
- The recommendation for the next one-day slice is an inference from the current docs/spec state, not a formal roadmap decision already recorded in the repo.
- I did not browse the internet; all evidence is from the local repo and spec set requested by the user.
