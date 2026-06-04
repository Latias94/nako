# VFS cache repair executable refresh action: minimal scope research

日期：2026-06-04

范围：只研究最小实现路径，不实现代码。

## 1. 当前 preview 数据流

当前 Admin preview 是只读链路，核心 authority 来自最新一条 VFS cache failure：

1. `CachedStorageBackend` 在 `stat` / `list_with_status` 调用 inner backend 失败时记录失败。
   - `crates/nako-vfs/src/cache.rs:77` 开始的 `stat`：先查 fresh cache；miss/过期后调用 `inner.stat`；成功 upsert object cache；失败时 `record_failure(uri, Stat, err, observed_at_ms)`，如果 transient 且存在 cached object，则返回 `StaleFallback`。
   - `crates/nako-vfs/src/cache.rs:145` 开始的 `list_with_status`：同样先查 fresh listing；成功 upsert listing；失败时 `record_failure(uri, List, err, observed_at_ms)`，可返回 stale listing。
   - `record_failure` 写入 `NewVfsCacheFailure { uri, scheme, operation, failed_at_ms, error }`，其中 `error` 使用 `safe_cache_failure_message(err)`，不是 raw backend error。

2. VFS repair classification 在 `nako-vfs` 内部生成。
   - `VfsCacheRepairDiagnostic::from_failure` 从 `VfsCacheFailure.operation/error/failed_at_ms/failure_count` 生成 diagnostic。
   - `classify_cache_repair` 将 `Timeout/Unavailable/RateLimited/StaleCache/PartialRead/Budget` 归为 `RetryableRefreshFailure`，推荐 `RefreshCache`。
   - `Permission/Security` 推荐 `FixBackendConfiguration`；unknown 或无 safe class 推荐 `InspectFailure`。
   - `redacted_cache_failure_message` 只返回 `StorageFailureClass::safe_message()` 或 `"storage failure"`。

3. `StorageDiagnosticsAppService` 目前只读最新 failure。
   - `crates/nako-server/src/app/storage.rs:286` 的 `latest_vfs_cache_repair_diagnostic()` 调用 `registry.store.get_latest_vfs_cache_failure()`，再 `map(VfsCacheRepairDiagnostic::from_failure)`。
   - 它不解析 raw URI、不定位 backend、不执行任何 cache mutation。

4. `/admin/v1/storage/staging` 将 preview 嵌入 staging diagnostics。
   - `crates/nako-server/src/http/admin.rs:1766` 的 `list_admin_storage_staging` 汇总 staging pressure、VFS cache summary，并把 `latest_vfs_cache_repair_diagnostic()` 映射成 Admin DTO。
   - `AdminVfsCacheRepairDiagnostic` 只暴露 classification、recommended_action、operation、failure_class、retryable、failed_at_ms、failure_count、safe_message、operator_action，不暴露 URI、etag、fingerprint、local path、backend URL。

5. 现有测试覆盖 preview redaction。
   - `crates/nako-api/src/admin/storage.rs:404` 验证 repair preview snake_case 序列化且不泄露 token/password/path。
   - `crates/nako-server/src/http/tests/system.rs:5083` 验证 `/admin/v1/storage/staging` 不泄露 `source_uri`、`local_path`、raw path、etag、fingerprint、raw error、token 等。
   - 同文件 `8600` 附近已有 Admin route 未认证返回 `401` 的系统测试，新增 POST route 应补进这类断言。

## 2. 可执行 `RefreshCache` 的最小可行入口

推荐最小入口：新增一个具体 Admin POST route，而不是通用 action body。

建议 route：

```text
POST /admin/v1/storage/vfs-cache/repair/refresh-cache
```

理由：

- 路径本身限定唯一可执行动作，避免客户端传入 `FixBackendConfiguration` / `InspectFailure` 这类 preview-only action。
- 不需要请求体携带 URI。target authority 继续来自 server 内部最新 failure，避免把 raw cache URI 变成公共输入。
- 放在 `admin_routes()` 当前 storage routes 附近，并位于 `.route_layer(require_admin_principal)` 之前，即可继承现有 Admin 权限。

建议最小执行链路：

1. HTTP handler 保持薄：
   - 无 body 或空 request DTO。
   - 调用 `app.storage().refresh_latest_vfs_cache_repair()`。
   - 将 app/service 层返回的 redaction-safe report 映射为 Admin DTO。

2. `StorageDiagnosticsAppService` 增加一个同步方法：
   - 读取 `get_latest_vfs_cache_failure()`。
   - 没有 latest failure：返回 safe `NotFound` 或 `InvalidInput`。
   - 用 `VfsCacheRepairDiagnostic::from_failure(&failure)` 重新计算当前 recommended action。
   - 只有 `recommended_action == RefreshCache` 时继续；其他 action 返回 safe `InvalidInput`/`Conflict`，且不得触碰 backend。
   - 内部解析 `failure.uri`，但解析失败必须转换成不含 raw URI 的 safe error。`StorageUri::parse` 当前错误消息会包含输入值，不能直接向 HTTP 传播。
   - 按 failure URI 定位对应 cached backend，然后按 `failure.operation` 执行一次强制 refresh。

3. backend 定位建议放在 `StorageBackendRegistry` 内：
   - 当前 registry 只有 `backend_for_media_source` / `backend_for_library_root`，没有任意 VFS URI 到 library backend 的公开入口。
   - 最小实现可增加私有 helper：按 configured library WebDAV root 与 failure URI 做 prefix/root-boundary 匹配，然后复用 `backend_for_library_config` 构建或获取 `LibraryStorageBackend`。
   - missing/ambiguous match 必须返回 redaction-safe error，不包含 failure URI、library root、base_url 或 path 片段。
   - 当前生产缓存包装只在 `webdav_storage_backend()` 中创建，local backend 默认不走 `CachedStorageBackend`。若 latest failure 指向非 cached backend，应 safe reject。

4. VFS/cache 边界需要一个小的强制刷新方法。
   - 直接调用现有 `StorageBackend::stat/list_with_status` 不够：`CachedStorageBackend` 会在 fresh cache 命中时早退，不一定触碰 inner backend。
   - 建议在 `nako-vfs` 增加 VFS-owned 方法，例如 `refresh_cache_entry(uri, operation)`，由 `CachedStorageBackend` 实现。
   - 该方法应绕过 fresh-cache 早退和 stale fallback，只做一次 inner `stat` 或 `list`：
     - 成功：复用现有 object/listing upsert 逻辑，返回 redaction-safe report。
     - 失败：复用 `record_failure` 写 safe failure，然后返回原 `NakoError`，让 HTTP 走现有 error mapping。
   - 若需要从 `LibraryStorageBackend` 调用 inner cached backend，最小方式是在 `StorageBackend` trait 上加带默认 unsupported 的 async 方法，再让 `CachedStorageBackend` 和 `LibraryStorageBackend` override。这样不需要 downcast，也不会要求所有 backend 改实现。

5. `LibraryStorageBackend` 的 backoff 是关键风险点。
   - 普通 `LibraryStorageBackend::stat/list` 会先 `reject_if_backing_off()`；一次 retryable storage failure 后可能立即进入 process-local/durable backoff。
   - 如果 refresh action 复用普通 `stat/list`，它可能只返回 `StorageRateLimited`，不会执行 underlying backend，无法满足“operator 手动 refresh 以确认 backend 恢复”的目标。
   - 建议给 repair refresh 一个明确的单次 override 路径：不在 refresh 前做 backoff reject，但仍在 refresh 后调用 `record_result`。成功会通过真实成功关闭健康 backoff；失败会继续按现有健康/错误分类记录。不要额外做 circuit breaker reset。

6. latest failure 成功后不会自动消失，这是当前 schema 的限制。
   - `VfsCacheRepository` 只有 `record/get/latest/summarize`，没有 clear/delete failure 方法；PRD 又要求不改 schema/repository contract。
   - 最小实现不应新增 failure delete。
   - 为避免 refresh 成功后 `/admin/v1/storage/staging` 继续显示旧 failure 的 `RefreshCache`，可在 `latest_vfs_cache_repair_diagnostic()` 中用现有 `get_vfs_cache_object/get_vfs_cache_listing` 判断 failure 是否已被更新的 cache entry 覆盖：如果对应 cache 的 `fetched_at_ms >= failure.failed_at_ms`，则把 repair 视为 resolved（返回 `None` 或 Healthy diagnostic）。这不需要新 repository 方法。

## 3. 必须避免的扩张

本 slice 不应扩张到以下内容：

- 不做 purge/delete/invalidate/retry-all/bulk cache 操作。
- 不新增 durable job、repair queue、progress polling、history、cancel/retry job。
- 不新增 DB schema、migration、repository clear/delete contract。
- 不新增 Admin Web 业务逻辑；DTO/route 变化只刷新 generated TypeScript contracts。
- 不新增 Public Client API。
- 不在 server 层直接访问 filesystem/WebDAV/raw backend URL 来修 cache；server 只做 target 校验和调用 VFS/cache boundary。
- 不让请求体接受 raw URI、source locator、etag、fingerprint、backend URL、credential、token。
- 不把 `FixBackendConfiguration`、`InspectFailure`、`None` 做成可执行 action。
- 不把 storage backend health reset/circuit breaker reset 混入本 action；refresh 成败只通过真实 backend 调用的 `record_result` 影响 health。
- 不引入后台 `tokio::spawn`、supervisor、control-plane job。同步 POST 足够。

## 4. 建议修改文件和测试

建议实现文件：

- `crates/nako-vfs/src/lib.rs`
  - 增加 VFS-owned refresh report 类型，或在 `StorageBackend` trait 增加默认 unsupported 的 `refresh_cache_entry` 方法。
- `crates/nako-vfs/src/cache.rs`
  - 在 `CachedStorageBackend` 内实现强制 `RefreshCache`：绕过 fresh-cache 早退，复用 inner call、upsert、safe failure recording。
- `crates/nako-server/src/app/storage.rs`
  - 在 `StorageDiagnosticsAppService` 增加 `refresh_latest_vfs_cache_repair`。
  - 在 `StorageBackendRegistry` 增加 latest failure URI 到 cached library backend 的私有解析 helper。
  - 在 `LibraryStorageBackend` 增加单次 repair refresh delegate，明确处理 backoff/health 记录策略。
- `crates/nako-server/src/http/admin.rs`
  - 增加 POST route、thin handler、VFS/service report 到 Admin DTO 的映射。
- `crates/nako-api/src/admin/storage.rs`
  - 增加 redaction-safe response DTO；如果使用 request DTO，也应为空或只含固定 `RefreshCache`，不要含 URI。
- `crates/nako-api/src/admin_contract.rs`
  - 增加 Admin route constant 和 response type 输出。
- `apps/admin-web/src/adminApi/generated/contract.ts`
  - 若 DTO/route shape 变化，由 generator 刷新。
- `web/src/api/admin/generated/contract.ts`
  - 若当前 contract test 仍要求双端生成，也由 generator 刷新。

建议测试：

- `crates/nako-vfs/src/cache.rs`
  - `refresh_cache_stat_bypasses_fresh_cache_and_upserts_object`
  - `refresh_cache_list_bypasses_fresh_listing_and_upserts_listing`
  - `refresh_cache_failure_records_redacted_failure_without_stale_fallback`
  - `refresh_cache_default_backend_is_unsupported`（如果 trait default added）
- `crates/nako-server/src/app/storage.rs` 或 `crates/nako-server/src/app/tests/storage.rs`
  - no latest failure -> safe not found/invalid input。
  - latest diagnostic action 不是 `RefreshCache` -> reject，backend call count 为 0。
  - retryable stat failure -> refresh 调用 cached backend inner stat，成功更新 cache。
  - retryable list failure -> refresh 调用 cached backend inner list，成功更新 listing。
  - parse/missing/ambiguous backend target -> safe error，不含 raw URI/root/base_url/token。
  - process-local backoff 存在时，repair refresh 仍执行一次 underlying probe，并按结果 record health。
- `crates/nako-api/src/admin/storage.rs`
  - response DTO snake_case，且 body 不含 URI/path/etag/fingerprint/token/password/raw backend error。
- `crates/nako-api/src/admin_contract.rs`
  - route constant 包含新 POST path。
  - generated contract drift test 覆盖新 DTO。
- `crates/nako-server/src/http/tests/system.rs`
  - POST route 未认证 -> `401`，非 admin -> `403`（如已有 non-admin helper，可补一例）。
  - successful POST response redaction-safe。
  - failed backend refresh 走现有 storage HTTP mapping，response 不含 raw target。
  - `/admin/v1/storage/staging` 在 refresh 前显示 `recommended_action=refresh_cache`，refresh 成功后不继续暴露旧 actionable repair（如果实现 resolved-latest-failure 判断）。

建议验证命令：

```text
cargo fmt --all -- --check
cargo check -p nako-core -p nako-vfs -p nako-api -p nako-server --tests
cargo nextest run -p nako-vfs cache --no-fail-fast
cargo nextest run -p nako-api admin_vfs admin_contract --no-fail-fast
cargo nextest run -p nako-server storage admin_v1_storage --no-fail-fast
cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts
cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts
git diff --check
python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-vfs-cache-repair-executable-refresh-action
```

## 5. Redaction / 权限 / 错误映射风险

Redaction 风险：

- `VfsCacheFailure.uri` 是内部 raw target。可以作为 service 内部 authority，但不能出现在 request、response、Admin contract、public error message。
- `StorageUri::parse` 的 invalid input message 会包含原始输入；action service 解析 latest failure URI 时必须捕获并替换成 safe message。
- `ObjectMetadata` / `ObjectListing` 包含 URI、etag、fingerprint 等字段；refresh response 不应直接返回这些类型。
- `NakoError::Storage` 内部 Display 包含 URI，但 `ApiError::public_message` 对 storage errors 做了 safe mapping。不要在 Admin response 中调用 `err.to_string()`。
- backend resolution 错误不要包含 WebDAV root、base_url、library path、token、credential env name。

权限风险：

- 新 route 必须加入当前 Admin router，并位于 `.route_layer(require_admin_principal)` 保护范围内。
- route 不能挂到 public/client router，也不能加入 Public OpenAPI path inventory。
- auth disabled 时会注入 bootstrap admin，这是现有开发/本地行为；auth enabled 时要验证 missing token `401`、non-admin `403`。

错误映射风险：

- no latest target：用 `NakoError::NotFound` 或 safe `InvalidInput`，不要返回空成功。
- latest target 不是 `RefreshCache`：用 safe `InvalidInput`/`Conflict`，并断言 backend 未调用。
- backend refresh 失败：传播原 storage `NakoError`，让 `crates/nako-server/src/http/error.rs` 映射到现有 `StorageTimeout`、`StorageRateLimited`、`StorageUnauthorized`、`StorageError` 等 code/status；不要包成 unclassified string。
- manual refresh 与 backoff 的关系必须显式测试。若不做单次 override，retryable failure 后 action 很可能被 backoff 拦截，表现为没有真正 refresh cache。
- 成功后旧 failure 仍在 repository 中。若 preview 继续只看 latest failure，会出现 action 成功但 staging diagnostics 仍推荐 `refresh_cache` 的状态漂移；建议用现有 cached object/listing fetched time 在 preview 层过滤已解决 failure。
