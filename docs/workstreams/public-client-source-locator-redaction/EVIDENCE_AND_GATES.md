# Public Client Source Locator Redaction Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http
```

Current known public exposure anchors include:

- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`

## Gate Set

### Audit Gate

```powershell
rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http
git diff --check
```

Proves the exposure inventory is current before public DTO changes.

### Public DTO Gate

```powershell
cargo check -p taru-client-protocol --tests
cargo check -p taru-api --tests
cargo nextest run -p taru-server <public-route-filter> --no-fail-fast
```

Proves protocol/server mapping changes compile and public route JSON tests
protect the redaction behavior.

### Contract Sync Gate

```powershell
cargo nextest run -p taru-api --no-fail-fast
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
- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-server/src/http/tests`

## Fresh Evidence

2026-05-18, PCLR-010:

- Workstream opened from ARF-005.
- Known locator exposure anchors identified with `rg`.
- First executable task set to exposure audit and contract decision before DTO
  field removal.

Fresh verification is required before marking implementation tasks or the lane
complete.

2026-05-18, PCLR-020:

- Audit command run:
  `rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http docs/api`.
- Extended audit also checked generated TypeScript SDK output with:
  `rg "locator|input_locator" sdk docs/api crates/taru-client-protocol crates/taru-api crates/taru-server/src/http`.
- Public Client contract exposures:
  - `crates/taru-client-protocol/src/catalog.rs`: `MediaSourceDto.locator`.
  - `crates/taru-client-protocol/src/catalog.rs`: `ClientTranscodePlan.input_locator`.
  - `crates/taru-api/src/public_client.rs`: `media_source_to_dto` maps full
    internal `MediaSource.locator` into public DTOs.
  - `crates/taru-api/src/public_client.rs`: `transcode_plan_to_dto` maps full
    internal `TranscodePlan.input_locator` into public DTOs.
  - `crates/taru-api/src/openapi.rs`: `MediaSourceDto.locator` and
    `ClientTranscodePlan.input_locator` are public schema fields.
  - `sdk/typescript/src/index.ts`: generated `MediaSourceDto.locator` and
    `ClientTranscodePlan.input_locator` mirror the OpenAPI leakage.
- Public routes affected by those DTOs:
  - `GET /libraries/{library_id}/sources` through `LibrarySourcesResponse`.
  - `GET /items/{item_id}` through `ItemDetailResponse.sources`.
  - `GET /sources/{source_id}/playback/decision` through
    `PlaybackDecisionResponse.source` and `ClientTranscodePlan`.
- Internal/server-only locator use is legitimate and must remain:
  - `crates/taru-server/src/http/playback.rs` uses `direct_play.source.locator`
    for response streaming.
  - `taru-streaming`, `taru-transcode`, playback input, remux, HLS, probe, and
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
- Compatibility posture: Taru is still pre-stable for this public shape, so
  PCLR-030 may remove fields directly instead of adding a deprecation period.
- `git diff --check` passed.

Fresh PCLR-030 validation is required before marking DTO changes complete.
