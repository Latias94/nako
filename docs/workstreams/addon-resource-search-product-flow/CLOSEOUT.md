# Addon Resource Search Product Flow - Closeout

Status: Closed
Closed: 2026-05-28

## Closeout Claim

The addon resource-search product-flow lane is complete. Nako now exposes a
host-owned Admin API flow for running a read-only resource search, returning
display-safe result cards, and explicitly selecting one opaque link into
acquisition intake without round-tripping raw URLs, passwords, request context,
provider messages, downloader commands, or cloud-drive transfer authority
through the browser.

## Delivered

- `nako-server::app::addons` owns product resource-search execution over the
  existing typed addon client.
- Host limits are applied before addon calls, matching diagnostic behavior.
- Product search responses return `search_id`, result fingerprints, display
  metadata, redacted link summaries, and opaque `selection_id` values.
- Raw selected links live only in a bounded transient host session.
- The transient session stores only result metadata/count snapshots plus the
  selected raw link, avoiding repeated storage of unrelated raw links.
- Diagnostic resource search remains counts/provider-summary only.
- `nako-server::app::acquisition_intake` reports true idempotent replay for
  `resource_search_selection` candidates.
- A selected opaque link creates or replays a ready acquisition intake candidate
  through the host-owned intake service.
- Admin HTTP routes exist for product search and selected-link intake candidate
  creation.
- HTTP tests prove product search and selection responses do not expose raw
  URLs, passwords, request context, provider messages, image URLs, or selected
  result internals.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `RSPF-010` through `RSPF-060` are complete.
- The lane stays Nako-only and does not modify `nako-official-addons`.
- The read-only `acquisition_search_read` boundary remains separate from
  downloader, link-check, cloud-drive save, and password persistence behavior.
- Diagnostic and product routes remain separate.

### Code Quality

- Blocking: none.
- Important: none.
- Product search reuses the typed addon client and existing manifest/scope
  validation instead of creating a second transport path.
- Search session storage is transient, bounded, and owned by `AddonAppService`.
- Selection records intake candidates through `AcquisitionIntakeAppService`
  rather than letting a read-only addon write candidates directly.
- Tests cover the public HTTP seam and verify redaction/leak boundaries.

### Missing Gates

- None for this lane.
- Full workspace nextest was not run because the touched surfaces are bounded to
  `nako-api` contracts and `nako-server` addon/intake behavior. Focused nextest
  gates plus broad `cargo check --tests` covered the changed surfaces.

## Final Verification

- `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`
  passed with 2 tests.
- `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` passed
  with 8 tests.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests.
- `cargo fmt --all -- --check` passed.
- `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`
  passed.
- `git diff --check` passed; Git reported expected Windows line-ending warnings
  only.

## Follow-Ons

1. **Admin UI**
   - Add an operator search screen, result cards, source/link filters, and an
     explicit select action using the product routes from this lane.
2. **Official addon migration**
   - In `nako-official-addons`, migrate resource-search manifests/providers to
     the first-class `resource_search` resource and `acquisition_search_read`
     grant.
3. **Link checking**
   - Define a separate availability/check contract. Keep this read-only and
     distinct from search and downloader execution.
4. **Downloader / external acquisition runner**
   - Define execution scopes, audit, cancellation, resource limits, and output
     handoff separately from resource search.
5. **Cloud-drive save / transfer**
   - Treat cloud-drive transfer as separate authority and policy, not as part of
     search or selection.
6. **Password/code persistence**
   - Decide the secret-reference model before persisting extraction codes or
     dispatching them to future runners.

## Evidence Anchors

- `docs/workstreams/addon-resource-search-product-flow/EVIDENCE_AND_GATES.md`
- `crates/nako-api/src/extension.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/acquisition_intake.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`
