# Addon Resource Flow Patterns

Use this spec before changing server-side Addon flows that turn read-only Addon
discovery into host-owned selection, planning, materialization, or writes.

## Scenario: Host-Owned Addon Resource Flow

### 1. Scope / Trigger

Trigger this spec when server work touches any of:

- Addon resource search selections.
- Addon subtitle selected references, import plans, or import apply.
- External acquisition selected-link, intake-candidate, or materialization
  handoff.
- New Addon resource types that need Admin-visible candidate selection followed
  by host-owned side effects.

This is a `nako-server` app/HTTP boundary pattern. Do not move host-owned
selection sessions, Admin apply plans, grant storage, VFS writes, or durable
job policy into `nako-addon-protocol`.

### 2. Signatures

Existing signatures that define the pattern:

- `AdminAddonResourceSearchResponse` exposes `search_id`, redacted result
  summaries, provider diagnostics, and optional `safe_error_code`.
- `AdminAddonResourceSearchLinkSummary` exposes `selection_id`,
  `link_type`, safe display source, `source_ref_redacted`, and safe flags.
- `AdminAddonSubtitleSelectedReference` exposes addon, manifest, search, and
  selection identity without exposing subtitle delivery material.
- `AdminSubtitleImportPlan` exposes host-derived idempotency key, status,
  reasons, target summary, sidecar plan, conflict policy, and backup policy.
- `AddonExternalAcquisitionMaterializationRequest` accepts only protocol target
  refs plus job/declaration/runner/idempotency/audit facts; the server validates
  the running host task before materializing raw link data.
- `resource_flow::SelectionSessionStore<TSelection, TContext>` is the
  server-local helper for transient Addon selection sessions. Resource modules
  insert `SelectionSession::new(search_id, addon_id, manifest_id, context,
  created_at_ms, selections)` and resolve with
  `get_selection(addon_id, manifest_id, search_id, selection_id, now_ms)`.

### 3. Contracts

- Discovery Addon calls are read-only unless a separate action scope and host
  handoff exists.
- Admin callers receive opaque `search_id` and `selection_id` values, not raw
  URLs, passwords, addon secrets, local paths, or materialization refs.
- Selection sessions are short-lived host memory. They must carry `addon_id`,
  `manifest_id`, `created_at_ms`, `expires_at_ms`, and typed selections.
- New Addon resource flows must reuse the server-local
  `SelectionSessionStore` for TTL pruning, oldest-session eviction, addon
  validation, manifest validation, and selection lookup. Do not copy another
  bespoke `HashMap<search_id, Session>` store with its own TTL/max-count logic.
- `SelectionSessionStore` owns only session mechanics. Resource-specific
  payload snapshots, safe summaries, selected-reference responses, link-check
  contexts, subtitle import plans, acquisition intake handoff, and error entity
  names stay in the resource module.
- The default transient selection-session policy is 15 minutes TTL and 64 stored
  sessions. Changing that policy affects all host-owned Addon resource flows
  and requires focused helper tests plus Resource Search and Subtitle Search
  regression tests.
- Apply plans are host-derived and idempotency-keyed. Resource-specific modules
  keep domain validation such as subtitle language/format checks.
- Apply/materialization results report host decisions and redacted facts, not
  raw sidecar/provider material.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Missing selection session or selection id | Return not-found for the resource-specific selected entity. |
| Addon id mismatch | Treat as missing and return not-found for the resource-specific selected entity. |
| Manifest id mismatch after a selection exists | Return conflict before handoff. |
| Expired selection session | Prune it and return not-found for the resource-specific selected entity. |
| Session count exceeds helper max-count | Evict oldest sessions by `created_at_ms`. |
| Read-only discovery tries to imply a side effect | Require a separate action/link-check/materialization flow. |
| Apply request idempotency key differs from plan | Reject as invalid input. |
| External acquisition materialization is not for a running aligned task | Reject as invalid materialization request. |
| Unsupported materialized link type | Reject without exposing raw source ref. |
| Addon/client failure | Map to stable `safe_error_code` and redacted facts. |

### 5. Good/Base/Bad Cases

- Good: Resource Search stores a selected link in a host session, Admin selects
  by `selection_id`, and Nako records an acquisition intake candidate with
  redacted diagnostics.
- Good: Subtitle Search stores a selected candidate, Nako derives a sidecar file
  plan, writes through Library File Write/VFS, and refreshes subtitle facts.
- Good: a new Addon resource flow wraps
  `SelectionSessionStore<MySelection, MyContext>` with a small module-local
  store that maps `SelectionSessionLookup::Missing` to the resource-specific
  not-found entity and `ManifestMismatch` to the resource-specific conflict
  message.
- Good: External acquisition action receives a selected-link or intake-candidate
  ref, then calls Nako runtime materialization to receive raw material only
  after task/token/idempotency/audit validation.
- Base: Read-only discovery succeeds but no side effect happens until the Admin
  caller selects, plans, or applies.
- Bad: Browser resubmits a raw link/password from a search result.
- Bad: Addon Protocol DTOs grow fields for Nako Admin session TTL or apply-plan
  policy.
- Bad: Product responses include raw source URIs, local paths, bearer tokens,
  materialization refs, or idempotency keys.
- Bad: a future resource type copies the Resource Search or Subtitle Search
  session-store implementation instead of reusing `SelectionSessionStore`.

### 6. Tests Required

When changing these flows, keep or add tests that assert:

- Response shapes still expose `search_id`/`selection_id` or selected refs
  without raw source material.
- Selection lookup rejects missing, expired, wrong-addon, or wrong-manifest
  references.
- Apply plans are deterministic and idempotency-keyed.
- Apply/materialization responses do not contain raw paths, raw URLs, bearer
  tokens, addon tokens, candidate IDs that grant raw access, materialization
  refs, or idempotency keys.
- Resource-specific behavior remains in the resource module after shared helper
  extraction.
- Helper tests cover selected handoff, expired-session pruning, oldest-session
  eviction, addon mismatch as missing, and manifest mismatch as conflict.
- Resource Search and Subtitle Search focused tests continue to prove public
  response shape, selected-reference lookup, link-check/import handoff, and
  redaction after helper changes.

### 7. Wrong vs Correct

#### Wrong

```text
Addon search result -> Admin response includes raw URL -> browser posts raw URL
back to Nako -> server starts downloader/import.
```

This gives a read-only Addon or browser request implicit side-effect authority.

#### Correct

```text
Addon search result -> Nako stores a host-owned selected reference -> Admin
posts selection_id -> Nako derives plan/materialization/write through host
policy -> response contains only redacted diagnostics.
```

This preserves Nako ownership of grants, audit, idempotency, storage, durable
work, and diagnostics.

#### Wrong

```rust
struct MyResourceSessionStore {
    sessions: HashMap<String, MyResourceSession>,
}

impl MyResourceSessionStore {
    fn prune(&mut self, now_ms: i64) { /* copied TTL logic */ }
    fn enforce_max_count(&mut self) { /* copied eviction logic */ }
}
```

This forks host-owned session policy and lets Resource Search, Subtitle Search,
and future Addon resource flows drift.

#### Correct

```rust
struct MyResourceSessionStore {
    sessions: SelectionSessionStore<MySelection, MyContext>,
}

impl MyResourceSessionStore {
    fn insert(&mut self, session: MyResourceSessionInput) {
        self.sessions.insert(SelectionSession::new(
            session.search_id,
            session.addon_id,
            session.manifest_id,
            session.context,
            session.created_at_ms,
            session.selections,
        ));
    }
}
```

The shared helper owns transient session mechanics while the resource module
keeps its typed payloads, response mapping, handoff behavior, and redaction
tests local.
