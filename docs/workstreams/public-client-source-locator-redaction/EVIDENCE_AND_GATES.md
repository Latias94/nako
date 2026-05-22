# Public Client Source Locator Redaction Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg "locator|input_locator" crates/nako-client-protocol crates/nako-api crates/nako-server/src/http
```

Current known public exposure anchors include:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`

## Gate Set

### Audit Gate

```powershell
rg "locator|input_locator" crates/nako-client-protocol crates/nako-api crates/nako-server/src/http
git diff --check
```

Proves the exposure inventory is current before public DTO changes.

### Public DTO Gate

```powershell
cargo check -p nako-client-protocol --tests
cargo check -p nako-api --tests
cargo nextest run -p nako-server <public-route-filter> --no-fail-fast
```

Proves protocol/server mapping changes compile and public route JSON tests
protect the redaction behavior.

### Contract Sync Gate

```powershell
cargo nextest run -p nako-api --no-fail-fast
```

Add existing OpenAPI and SDK generation checks from the client contract lanes
when DTO or generated artifacts change.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace gates if protocol changes affect SDK/client crates.

### Review Gate

Run `review-workstream` before accepting DTO changes and before closeout.
Record blocking findings, missing gates, and residual risks here.

## Evidence Anchors

- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/api/HTTP_API.md`
- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-server/src/http/tests`

## Fresh Evidence

2026-05-18, PCLR-010:

- Workstream opened from ARF-005.
- Known locator exposure anchors identified with `rg`.
- First executable task set to exposure audit and contract decision before DTO
  field removal.

Fresh verification is recorded below for the closeout claim.

2026-05-18, PCLR-020:

- Audit command run:
  `rg "locator|input_locator" crates/nako-client-protocol crates/nako-api crates/nako-server/src/http docs/api`.
- Extended audit also checked generated TypeScript SDK output with:
  `rg "locator|input_locator" sdk docs/api crates/nako-client-protocol crates/nako-api crates/nako-server/src/http`.
- Public Client contract exposures:
  - `crates/nako-client-protocol/src/catalog.rs`: `MediaSourceDto.locator`.
  - `crates/nako-client-protocol/src/catalog.rs`: `ClientTranscodePlan.input_locator`.
  - `crates/nako-api/src/public_client.rs`: `media_source_to_dto` maps full
    internal `MediaSource.locator` into public DTOs.
  - `crates/nako-api/src/public_client.rs`: `transcode_plan_to_dto` maps full
    internal `TranscodePlan.input_locator` into public DTOs.
  - `crates/nako-api/src/openapi.rs`: `MediaSourceDto.locator` and
    `ClientTranscodePlan.input_locator` are public schema fields.
  - `sdk/typescript/src/index.ts`: generated `MediaSourceDto.locator` and
    `ClientTranscodePlan.input_locator` mirror the OpenAPI leakage.
- Public routes affected by those DTOs:
  - `GET /libraries/{library_id}/sources` through `LibrarySourcesResponse`.
  - `GET /items/{item_id}` through `ItemDetailResponse.sources`.
  - `GET /sources/{source_id}/playback/decision` through
    `PlaybackDecisionResponse.source` and `ClientTranscodePlan`.
- Internal/server-only locator use is legitimate and must remain:
  - `crates/nako-server/src/http/playback.rs` uses `direct_play.source.locator`
    for response streaming.
  - `nako-streaming`, `nako-transcode`, playback input, remux, HLS, probe, and
    storage paths need full locators for execution.
- HTTP route tests currently seed `MediaSource.locator` values as fixtures.
  These are not contract exposures unless assertions or serialized public JSON
  keep locator fields after PCLR-030.
- Existing Admin API documentation already states catalog governance responses
  never return source locators. Any future Admin locator diagnostics should use
  redacted summaries and stay outside the Public Client DTO contract.
- Contract decision:
  - Remove raw `MediaSourceDto.locator` from Public Client DTOs.
  - Remove raw `ClientTranscodePlan.input_locator` from Public Client DTOs.
  - Keep `id`, `library_id`, `item_id`, `file_name`, `size_bytes`, and
    `fingerprint` as safe public source facts for now.
  - Public playback clients should use `MediaSourceId`, stream/remux/HLS
    routes, and playback session IDs, not Source Locator values.
  - Update OpenAPI and generated SDK artifacts in PCLR-040 after DTO/mapping
    changes land.
- Compatibility posture: Nako is still pre-stable for this public shape, so
  PCLR-030 may remove fields directly instead of adding a deprecation period.
- `git diff --check` passed.

2026-05-18, PCLR-030/PCLR-040:

- Added RED route assertions in:
  - `crates/nako-server/src/http/tests/catalog.rs`
  - `crates/nako-server/src/http/tests/playback.rs`
- RED check failed before DTO removal with public `locator` still present:
  `cargo nextest run -p nako-server browse_routes_return_catalog_graph playback_decision_and_direct_stream_routes_work --no-fail-fast`.
- Removed public fields:
  - `MediaSourceDto.locator`
  - `ClientTranscodePlan.input_locator`
- Updated `nako-api` mapping so internal `MediaSource.locator` and
  `TranscodePlan.input_locator` are not serialized into Public Client DTOs.
- Kept internal locator execution paths unchanged, including direct stream in
  `crates/nako-server/src/http/playback.rs`, streaming selection, remux, HLS,
  and storage workflows.
- Synchronized OpenAPI schema and generated TypeScript SDK output because
  `nako-api` has generator consistency tests for checked-in SDK artifacts.
- Updated `docs/api/HTTP_API.md` to state that public source and playback
  responses do not expose raw source locators or transcode input locators.
- Focused review result: no blocking public contract or leakage findings.
  Remaining `rg` hits are internal execution, test fixtures, documentation
  references, or negative assertions.
- Validation passed:
  - `cargo check -p nako-client-protocol --tests`
  - `cargo check -p nako-api --tests`
  - `cargo nextest run -p nako-api --no-fail-fast`
  - `cargo nextest run -p nako-server browse_routes_return_catalog_graph playback_decision_and_direct_stream_routes_work --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`

2026-05-18, PCLR-050:

- Closeout review found no blocking public contract or leakage findings.
- Fresh final gates rerun after lane documentation was updated:
  - `cargo check -p nako-client-protocol --tests`
  - `cargo check -p nako-api --tests`
  - `cargo nextest run -p nako-api --no-fail-fast`
  - `cargo nextest run -p nako-server browse_routes_return_catalog_graph playback_decision_and_direct_stream_routes_work --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Remaining `rg "locator|input_locator"` hits are internal execution paths,
  test fixtures, negative assertions, or documentation references.
- Workstream status updated to completed; no split follow-on is required for
  the redacted public client contract.
