# official-addon-catalog-crate

## 现状摘要

`crates/nako-official-addon-catalog` 已经不是空壳，它是官方 addon
事实的共享来源。当前 `src/lib.rs` 里按 addon 分了 6 个模块：

- `metadata_scraper`
- `resource_search`
- `external_acquisition_runner`
- `subtitle_provider`
- `dlna_renderer`
- `chromecast_renderer`
- `notification_bridge`

每个模块都已经定义了：

- `ADDON_ID`
- `ADDON_NAME`
- `ADDON_VERSION`
- 默认 base URL 或 container base URL
- runtime binary/image
- description
- manifest 构造函数
- binary/container install descriptor
- 一些 manifest 级别的单元测试

这说明 catalog 第一版不需要重新发明 addon 事实模型，只需要把现有模块事实汇总成可发布、可验证的 catalog artifact。

## 已存在的契约

### 1. 事实来源已经模块化

`manifest_with_version(...)` 是主要事实拼装器。各模块自己定义：

- resource kind/path/input/output schema
- scopes
- event subscriptions
- tasks
- hosted pages
- configuration schema
- secret reference fields
- default timeout / max attempts
- auth

### 2. 安装指引已经存在

每个 addon 都有：

- `binary_install_descriptor()`
- `container_install_descriptor()`

它们可被 `nako_addon_protocol::addon_install_guide(...)` 直接转换为安装指导信息。

### 3. 现有测试已经能做事实守门

`src/lib.rs` 底部已经有一组测试，验证：

- manifest shape 和 protocol validation
- container descriptor shape
- `addon_install_guide(...)` 输出的 runtime reference / resource / task / page 计数

这意味着 catalog 第一版可以依赖这些事实做“生成或验证”，不需要新引入复杂的 addon manager。

## 缺口

1. 还没有一个统一的 catalog 汇总层。
2. 还没有一个对 operator 可见的 catalog artifact。
3. 还没有一个把 addon 事实、安装指引、trust tier、smoke status 聚合成单一表面的入口。
4. 现有 crate 更像“事实工厂”，还不是“目录输出器”。

## 最小实现路径

### 建议输出形态

第一版优先做一个生成型文档或 JSON artifact，而不是 manager：

- 生成一个 markdown catalog 表格，适合 docs 入口
- 同时可选生成 machine-readable JSON，方便后续 CI 校验

### 建议 catalog 字段

第一版至少应包含：

- addon id
- addon name
- addon version
- Addon Protocol Version
- compatible Nako version range
- default base URL
- container base URL
- runtime kind
- runtime binary / image
- resources / tasks / events / hosted pages
- required scopes
- trust tier
- install guide reference
- smoke status

### 生成策略

1. 直接从现有模块导出 manifest/install descriptor facts。
2. 汇总成单一 catalog artifact。
3. 用 `nako-addon-protocol::validate_manifest` 和
   `validate_install_descriptor` 保证事实不漂移。
4. 后续再决定是否把 catalog 变成 CI 产物或 docs 产物。

## 验证建议

- `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`
- `cargo nextest run -p nako-addon-protocol -p nako-official-addon-catalog --no-fail-fast`

## 结论

这个 crate 已经具备 catalog 最小闭环所需的事实来源。第一版应
聚焦“生成/验证官方 addon catalog artifact”，而不是做 addon manager。
