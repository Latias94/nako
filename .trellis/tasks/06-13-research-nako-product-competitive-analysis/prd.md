# research: Nako product competitive analysis

## Goal

基于 Nako 当前代码、文档、官方 Addon 仓库、Jellyfin 参考仓库和公开竞品资料，形成一套可继续深研的产品竞品分析框架，帮助判断 Nako 应如何区别于 Jellyfin、Plex、Emby 等核心媒体服务器，以及 Servarr、Seerr、Bazarr、Kometa、Tdarr、Tautulli 等自托管媒体生态组件。

## What I Already Know

- Nako 当前定位是开源自托管媒体服务器后端，处于 `0.1.0-alpha.2` 技术预览阶段，不是稳定的 Jellyfin/Plex 替代品。
- Nako 是 Video-First Phase，但长期 Media Server Scope 包含 video、audio、image、document、mixed、online 等媒体域。
- Nako 重点架构方向包括：Local Inference、Provider Mapping、NFO/local authority、Managed Artwork、VFS/storage abstraction、Direct Play/Remux/Transcode planning、durable jobs、runtime budgets、Admin diagnostics。
- Nako Addon 不是 Jellyfin Plugin Compatibility；当前模型是 out-of-process Addon Sidecar，使用 Addon Protocol、scoped tokens、grants、health check、resource calls、tasks、events 和 host-owned side effects。
- `nako-official-addons` 已有 metadata scraper、notification bridge、Chromecast renderer、DLNA renderer、resource search、external acquisition runner、subtitle provider 等官方 sidecar。
- 外部竞品不应只看核心媒体服务器，还应区分媒体请求、自动化下载、字幕、转码整理、统计、清理、虚拟频道、照片、音乐、阅读、有声书和在线视频归档等生态角色。

## Requirements

- 研究对象必须同时覆盖核心媒体服务器、相邻垂直媒体服务器、自托管媒体生态工具和 Nako 自身。
- 研究字段必须能支持产品定位、功能差异、生态策略、技术边界、运营成熟度、商业/授权模式和风险分析。
- 竞品分析应优先使用官方站点、官方文档、GitHub 仓库/发布页、Nako 仓库文档和官方 Addon 仓库资料。
- Jellyfin 参考仓库只用于行为、架构边界和用户工作流观察；不得复制、翻译或派生实现代码、测试、迁移、资源或生成文件。
- 输出应先形成可执行的 `outline.yaml` 和 `fields.yaml`，后续可交给 `/research-deep` 或子代理分批深研。

## Acceptance Criteria

- [x] `research/outline.yaml` 列出研究主题、对象清单、分组、默认执行参数和输出目录。
- [x] `research/fields.yaml` 列出竞品分析字段、字段说明、详细程度和不确定信息记录规则。
- [x] 至少覆盖 Jellyfin、Plex、Emby、Kodi、Stremio、Immich、Navidrome、Audiobookshelf、Nako。
- [x] 至少覆盖 Servarr/Bazarr/Seerr/Kometa/Tdarr/Tautulli/Maintainerr 等生态组件分组。
- [x] 字段中明确包含 Addon/Plugin trust boundary、remote access/account dependency、data portability、operations maturity、legal/policy boundary。
- [x] 记录本轮使用的主要本地资料和外部来源入口，方便后续深研追溯。

## Definition Of Done

- 研究大纲文件已创建在任务目录。
- 子代理产出的仓库内研究文件若完成，纳入后续深研参考。
- 不修改业务代码。
- 不提交或回退用户未要求的变更。

## Out Of Scope

- 本轮不做 Nako 功能实现。
- 本轮不创建最终市场报告或定位宣言，只创建调研框架和第一轮判断。
- 本轮不复制 Jellyfin/Plex/其他项目源码或实现细节。
- 本轮不评估具体法律结论，只记录需要法律/政策审查的风险维度。

## Technical Notes

- 本地已读取：`README.md`、`CONTEXT.md`、`docs/ARCHITECTURE.md`、`docs/architecture/PLAYBACK.md`、`docs/architecture/CONTROL_PLANE.md`、`docs/architecture/LIBRARY_PIPELINE.md`、`docs/architecture/STORAGE_VFS.md`、`docs/architecture/OPERATIONS_RELEASE.md`、`crates/nako-addon-protocol/README.md`、`docs/guides/ADDON_AUTHOR_GUIDE.md`。
- 官方 Addon 仓库已读取：`../nako-official-addons/README.md`、`addons/metadata-scraper/README.md`、`addons/resource-search/README.md`，并枚举了 addon/crate README。
- 已派发子代理：
  - `research/nako-current-state.md`：Nako 当前产品/架构状态。
  - `research/official-addons-current-state.md`：官方 Addon 仓库状态。
  - `research/jellyfin-source-architecture-notes.md`：Jellyfin 源码结构观察。
  - `research/external-competitive-ecosystem-supplement.md`：外部竞品与生态工具补充。
- 第一版产品判断已整理到 `research/competitive-analysis-first-pass.md`。
