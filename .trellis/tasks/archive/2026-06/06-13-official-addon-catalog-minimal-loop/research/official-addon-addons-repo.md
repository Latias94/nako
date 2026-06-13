# official-addon-addons-repo

## 仓库现状摘要

`../nako-official-addons` 已经有完整的官方 addon 包装结构，不只是
源码目录。每个 addon 目录普遍具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`
- 某些 addon 还有 `systemd.example.service`

这说明 catalog 第一版可以把 addon repo 当成“安装与 smoke 事实源”，
而不是只看 Rust crate。

## 覆盖矩阵

### 1. metadata-scraper

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`
- `systemd.example.service`

关键事实：

- addon id: `nako.official.metadata-scraper`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-metadata-scraper:9100`
- 有 metadata resource、bulk task、library.scanned event subscription、diagnostics hosted page
- smoke 既支持纯 sidecar，也支持注册到 Nako 后的 admin-mediated smoke

### 2. resource-search

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`

关键事实：

- addon id: `nako.official.resource-search`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-resource-search:9130`
- 有 `resource_search` 和 `resource_link_check`
- 只读资源搜索与 link check 闭环很清楚

### 3. notification-bridge

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`
- `smoke.live.ps1`

关键事实：

- addon id: `nako.official.notification-bridge`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-notification-bridge:9110`
- 当前是 ACK-only proof，provider fan-out 仍然是显式配置

### 4. chromecast-renderer

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`

关键事实：

- addon id: `nako.official.chromecast-renderer`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-chromecast-renderer:9120`
- 有 renderer_adapter resource、diagnostics hosted page

### 5. dlna-renderer

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`

关键事实：

- addon id: `nako.official.dlna-renderer`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-dlna-renderer:9150`
- 明确是 plan-only foundation

### 6. subtitle-provider

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `smoke.local.ps1`

关键事实：

- addon id: `nako.official.subtitle-provider`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-subtitle-provider:9140`
- 是只读 subtitle candidate discovery

### 7. external-acquisition-runner

已具备：

- `manifest.example.json`
- `README.md`
- `Dockerfile`
- `compose.example.yml`
- `smoke.local.ps1`

关键事实：

- addon id: `nako.official.external-acquisition-runner`
- version: `0.1.0-alpha.2`
- protocol version: `0.1.0-alpha.1`
- base URL: `http://nako-external-acquisition-runner:9160`
- 有 task 资源，且声明了 secret reference field

### 8. browser-worker

已具备：

- `README.md`
- `package.json`
- `package-lock.json`
- `Dockerfile`
- `scripts/smoke.mjs`
- `scripts/live-render-drift.mjs`
- `test/*.mjs`

关键事实：

- 它不是 Addon manifest 单元
- 它是 metadata provider 使用的 browser-render helper
- 不应被 catalog 当成官方 addon 目录项

## 适合 catalog 的事实集合

Catalog 第一版可稳定覆盖：

- addon id / name / version / protocol version
- base URL / container base URL
- runtime kind / binary / image
- resources / tasks / events / hosted pages
- required scopes
- configuration schema id
- secret reference field ids
- install guide reference
- smoke status

## 缺口

1. 官方 addons 仓库里虽然每个 addon 都有 manifest 和 smoke 事实，但没有一个统一的 catalog 文件输出。
2. browser-worker 需要明确排除，不然 catalog 会把 helper 误当 addon。
3. 还没有统一的 trust tier / smoke status 字段用于 operator 读表。
4. 现有 README 里更多是单个 addon 说明，不是统一目录入口。

## 第一版建议

第一版 catalog 应该：

- 只收录真正的官方 addon sidecar
- 排除 browser-worker
- 使用 addon repo 的 manifest.example.json / README / smoke 作为事实源
- 用 Nako 仓库里的 catalog crate 作为聚合与验证源

## 验证建议

- 读 `manifest.example.json` 与 `README.md` 作为 catalog 输入
- 对每个 addon 的 manifest 做 protocol validation
- 确认 smoke 脚本存在且路径稳定
- 检查 browser-worker 是否被明确排除在 catalog 外

## 结论

官方 addons 仓库已经具备 catalog 所需的安装与 smoke 事实。第一版不
需要做 addon manager，只需要把这些事实聚合成一个稳定目录输出即可。
