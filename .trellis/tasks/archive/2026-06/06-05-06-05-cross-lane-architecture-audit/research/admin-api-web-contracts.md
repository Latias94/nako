# Admin API / Admin Web / Generated Contract Lane Audit

> Audit date: 2026-06-05
> Scope: `nako-api`, Admin HTTP routes, Admin Web, generated Admin TypeScript
> Constraints: research only; no production code changes; no commits

## Summary

This lane is valuable but should be treated as a shared serializing surface for
parallel development. The current Admin API and Admin Web implementation has
good redaction habits and useful generator drift checks, but the Interface is
getting shallow in two places:

- route facts are spread across Axum route registration, `nako-api`
  `ADMIN_ROUTE_SUFFIXES`, generated TypeScript route constants, and Admin Web
  client helpers;
- Admin Web route state, query normalization, DTO projection, mock data, and
  redaction tests are hand-maintained across large files.

The immediate risk is not one known data leak. The risk is contract drift when
playback/decoding, remote access, Addon, storage/VFS, and Admin Web follow-ons
all touch `nako-api`, `admin_contract.rs`, generated contracts, route state, and
redaction tests in parallel.

Recommended lane decision: do one bounded contract-hardening task before broad
Admin Web or diagnostics feature work, then choose one fearless refactor cleanup
for Admin contract generation or route state if parallel queues need this lane
to absorb more change.

## Authority Read

- `CONTEXT.md` distinguishes **Client Application**, **Public Client API**,
  **Admin API**, **Generated Artifact**, and **Acceptance Workflow**. The Admin
  API is for server administration, diagnostics, configuration, and operational
  workflows; it is not a Public Client API contract.
- ADR 0023 keeps Public Client API versioning and error envelopes stable.
- ADR 0027 defines `/admin/v1/*` as the versioned Admin API namespace and keeps
  Admin DTOs in `nako-api`, not `nako-client-protocol`.
- ADR 0053 makes redacted diagnostics, durable work, resource policy, endpoint
  configuration, and API scale part of the control plane.
- `.trellis/spec/nako-api/backend/*` requires DTO source first, generator
  second, generated artifacts last, with Admin/Public separation and redaction
  tests.
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` requires
  TanStack Router-owned search state, generated Admin API client use, mock
  fallback, and tests for URL params, data-source calls, localization, and
  unsafe-field omission.

## Current Strengths

- Admin/Public separation is explicit in ADRs and specs.
- `nako-api/src/admin/operations.rs` redacts job input, summary, and error
  payloads into presence booleans for `AdminJobListItem` and
  `AdminJobCancelRequestResponse`.
- Admin Web Jobs route now keeps source fingerprint hash drill-down in URL
  search params and maps directly to `AdminJobsQuery`.
- `admin_contract.rs` tests compare generator output against both
  `apps/admin-web/src/adminApi/generated/contract.ts` and
  `web/src/api/admin/generated/contract.ts`.
- Network, playback support, storage staging, generated artifact, Addon, and
  Admin Web tests already contain many sensitive-term negative assertions.
- Media Web uses `@nako/sdk` for Public Client API access, which keeps the
  Client Application surface separate from the Admin API contract.

## Architecture Pressure

### 1. Route Facts Are Not Single-Source

Evidence:

- `crates/nako-server/src/http/admin.rs` registers broad `/admin/v1/*` routes.
- `crates/nako-server/src/http/addons.rs` registers additional `/admin/v1/addons/*`
  routes.
- `crates/nako-api/src/admin_contract.rs` owns `ADMIN_ROUTE_SUFFIXES` and emits
  `NAKO_ADMIN_ROUTES`.
- `apps/admin-web/src/adminApi/client.ts` consumes generated route constants but
  still appends some subpaths manually.

Observed drift pressure from a route comparison across `crates/nako-server/src/http/*.rs`
and the generated Admin Web route constants:

- server-only route examples include:
  - `/admin/v1/jobs/{job_id}/cancel`
  - `/admin/v1/access/invitations`
  - `/admin/v1/access/invitations/{invitation_id}/revoke`
  - `/admin/v1/settings/playback/runtime`
  - `/admin/v1/addons/{addon_id}/tokens`
  - `/admin/v1/addons/{addon_id}/tokens/{token_id}/rotate`
  - `/admin/v1/addons/{addon_id}/tokens/{token_id}/revoke`
  - `/admin/v1/addons/{addon_id}/grants`
  - `/admin/v1/addons/{addon_id}/task-runs`
- generated route constants use `:addon_id` for many Addon paths while server
  route registration uses `{addon_id}`.

Some of this is intentional because Admin Web does not consume every route yet,
and ADR 0027 allows route migration to be incremental. The architectural issue
is that there is no explicit distinction between:

- generated/public-to-Admin-Web routes;
- implemented but intentionally not generated routes;
- implemented routes missing from generation by accident;
- generated routes whose placeholder syntax differs from server route syntax.

### 2. Generated Contract Generator Is a Wide Static Module

Evidence:

- `crates/nako-api/src/admin_contract.rs` is about 4006 lines.
- `CONTRACT_BODY` is a giant TypeScript string, not a typed projection from
  Rust DTO metadata.
- The generator drift test proves generated files match the static generator,
  but it cannot by itself prove the static generator still matches every Rust
  DTO or every intended route.

The module currently provides leverage because it blocks hand-edited generated
artifacts. Its Interface is becoming shallow because every Admin DTO addition
requires knowing the Rust DTO, static TypeScript body, route suffix table,
expected-name assertions, generated copies, Admin Web mocks, and UI tests.

### 3. Admin Web Route State Is Consistent But Repetitive

Evidence:

- `apps/admin-web/src/App.tsx` is about 1011 lines and contains route
  declarations plus many `validate*Search` / `normalize*Search` functions.
- Similar paged search shapes recur for Jobs, Catalog, Acquisition Intake,
  Generated Artifacts, Item Artwork, Playback Sessions, Storage Staging, and
  Media pages.
- Page components repeat `onSearchChange({ field, offset: 0 })`,
  `toAdmin*Query`, active-filter counting, mock fallback, and route tests.

This is not a correctness failure. The pressure is parallel development cost:
each new diagnostics page must touch route definitions, search normalization,
page-local query mapping, i18n, mock data, and `App.test.tsx`.

### 4. Admin Client Still Bridges Public Client Routes Manually

Evidence:

- `apps/admin-web/src/adminApi/client.ts` has methods such as
  `getPublicCatalogItemsBridge`, `getPublicItemDetailBridge`, and
  `getPublicSourceProbeBridge`.
- `apps/admin-web/src/adminApi/types.ts` defines Public Client response shapes
  manually for those bridge methods.
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts` already uses
  `@nako/sdk`, the generated Public Client SDK.

ADR 0027 allows Admin Web to read Public Client API routes when the information
is genuinely client-facing. The pressure is that the Admin API client module is
also acting as a Public Client route adapter, with hand-written Public Client
types beside generated `@nako/sdk` types.

### 5. Redaction Is Broad But Distributed

Evidence:

- `nako-api` DTO modules contain local serialization tests for sensitive terms.
- `admin_contract.rs` has a forbidden-term test for generated Admin contracts.
- `nako-server` route tests seed sensitive payloads for many Admin responses.
- `apps/admin-web/src/App.test.tsx` and `adminApi/dataSource.test.ts` assert
  that rendered output and projected data omit unsafe fields.

This is the right instinct, but it is not yet a deep redaction Module. Future
diagnostics from playback/decoding, remote access, Addon sidecars, and VFS jobs
will likely duplicate sensitive-term fixtures unless a shared test vocabulary or
route redaction matrix is added.

## Candidate Tasks

### P0: Admin Route Inventory Parity Gate

Classification: ready bounded implementation follow-on.

Files / modules:

- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- focused tests in `nako-api` and/or `nako-server`

Problem:

Route facts are split across server registration and the generated Admin Web
contract. The current tests prevent generated-file drift but do not classify
implemented routes as generated, intentionally excluded, or missing.

Solution:

Add an explicit Admin route inventory classification in `nako-api` or a small
test helper:

- normalize `{param}` and `:param` or standardize on one syntax;
- assert all generated routes correspond to implemented `/admin/v1/*` routes;
- assert selected implemented routes are either generated or explicitly
  excluded with a reason;
- add missing generated constants for routes already consumed by Admin Web
  through manual suffix appends, especially Addon tokens/grants and job cancel,
  or document them as intentionally not generated.

Benefits:

- Locality: route compatibility errors surface in one contract test.
- Leverage: every future Admin diagnostics lane gets route drift protection.
- Parallel safety: playback, Addon, storage, access, and Admin Web tasks can
  coordinate through a visible route inventory instead of implicit string
  knowledge.

Recommended priority: first.

### P1: Admin Contract Generator Deepening

Classification: needs architecture audit before implementation; likely
fearless refactor cleanup after P0.

Files / modules:

- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-api/src/admin/*.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`

Problem:

The generated Admin contract is generated from a static TypeScript string. It
keeps generated artifacts honest, but the generator itself has become a wide
manual contract surface.

Solution options:

- Split the contract body into per-domain generator fragments matching
  `nako-api::admin` modules.
- Keep static TypeScript but add per-domain route/type tests and a route
  classification table.
- Longer term: generate TypeScript definitions from serde/ts metadata or a
  narrow schema intermediate, if the dependency and build complexity are worth
  it.

Benefits:

- Locality: DTO contract updates stay near the Admin domain module.
- Leverage: generator tests can fail on the domain slice that drifted.
- Test improvement: route/type redaction assertions can be grouped by
  operations, storage, playback, Addon, and network diagnostics.

Recommended priority: second, but design first; do not start this in parallel
with major Admin DTO feature work.

### P1: Separate Admin Client From Public Client Bridges

Classification: ready fearless refactor cleanup candidate.

Files / modules:

- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/types.ts`
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

Problem:

The Admin Web Admin client contains Public Client bridge methods and
hand-written Public Client response types even though `@nako/sdk` is already
available and used by Media Web.

Solution:

Move Public Client reads used by Admin Web management views behind a separate
Public Client data adapter backed by `@nako/sdk`, or make the dependency
explicit in `AdminDataSource` without putting Public Client route strings inside
`AdminApiClient`.

Benefits:

- Locality: Admin API client remains Admin-only.
- Leverage: Public Client route and DTO changes flow through the generated SDK.
- Parallel safety: future Client Application and Admin Web work conflict less
  often in `adminApi/client.ts` and `adminApi/types.ts`.

Recommended priority: after P0 if web-product and client-surface lanes will run
in parallel.

### P2: Admin Web Route Search State Helper

Classification: ready fearless refactor cleanup candidate.

Files / modules:

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/*`
- `apps/admin-web/src/App.test.tsx`

Problem:

Paged search validation and route-owned filter updates repeat across many Admin
Web pages. The pattern is consistent but shallow: each page caller must remember
the same `limit`, `offset`, `emptyToUndefined`, and `offset: 0` rules.

Solution:

Extract a small route-search helper Module for paged route search:

- reusable `normalizePagedSearch`;
- declarative string / enum / bounded integer field definitions;
- test helper for "filter changes reset offset";
- no new form library.

Benefits:

- Locality: search behavior changes once.
- Leverage: new diagnostics pages start with the correct URL contract.
- Test improvement: page tests can focus on route-specific filters instead of
  re-proving pagination normalization every time.

Recommended priority: useful but not blocking; avoid during large page feature
work unless the page churn is already high.

### P2: Redaction Test Vocabulary And Matrix

Classification: bounded architecture/spec task, then implementation.

Files / modules:

- `.trellis/spec/nako-api/backend/quality-guidelines.md`
- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-api/src/admin/*.rs`
- `crates/nako-server/src/http/tests/*`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`

Problem:

Sensitive terms are asserted in many places, but each slice reinvents the
fixtures. This increases the chance that one lane tests `source_uri` while
another forgets `playback ticket`, `Source Fingerprint`, `FFmpeg stderr`, or
Addon token forms.

Solution:

Create a shared redaction vocabulary and route matrix for Admin diagnostics:

- common forbidden strings by category;
- route families and allowed exception notes, such as one-time `raw_token`
  issue/rotation responses;
- reusable Rust and TypeScript test helpers where practical;
- spec guidance for playback, network, Addon, storage/VFS, and job diagnostics.

Benefits:

- Locality: redaction policy is visible and reusable.
- Leverage: new Admin DTOs inherit a stronger negative-test checklist.
- Parallel safety: lane-specific diagnostics can use the same forbidden-term
  vocabulary.

Recommended priority: pair with P0 or the next diagnostics-heavy implementation
queue.

### P3: Admin Diagnostics Module Sizing Review

Classification: architecture audit first; not ready for direct implementation.

Files / modules:

- `crates/nako-api/src/admin/metadata_candidate_review.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-api/src/admin/managed_artwork.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-server/src/http/admin.rs`
- `apps/admin-web/src/App.test.tsx`

Problem:

Several Admin DTO modules and tests are now very wide. This is acceptable while
shipping first slices, but it makes future parallel edits expensive.

Solution:

Audit one domain at a time for deepening opportunities:

- split by durable workflow or diagnostic family only when the new Module hides
  meaningful behavior;
- avoid pass-through files that only move names;
- keep tests at the Interface where callers cross the seam.

Benefits:

- Locality: domain-specific Admin diagnostics become easier to reason about.
- Leverage: future lanes can own smaller files without inventing new crates.

Recommended priority: only after the route inventory and generator pressure are
reduced.

## Cross-Lane Conflict Surface

High-conflict files and contracts:

- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/mockData.ts`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/addons.rs`

Lanes likely to conflict here:

- playback/decoding/transcode: Admin playback runtime/support/renderers DTOs,
  runtime settings, FFmpeg redaction, resource pressure.
- remote access/network tunnel: Admin network diagnostics, endpoint readiness,
  tunnel provider redaction.
- Addon Protocol / Addon Sidecar: Addon routes, tokens/grants, task runs,
  hosted surfaces, Addon Resource Search, Addon Task diagnostics.
- storage/VFS/control-plane: Jobs, VFS cache repair, source fingerprint hash,
  source-read pressure, durable job diagnostics.
- web-product/client-surfaces: route state, Public Client bridge use, generated
  SDK boundaries, Admin Web page tests.

Parallelization recommendation:

- Do not run two lanes that both change `admin_contract.rs` unless one is only
  reading.
- Keep one "contract lane owner" responsible for regenerating Admin TypeScript
  contracts and resolving route inventory conflicts.
- Feature lanes may add DTOs, but generated contract refresh and route parity
  should be serialized through the contract owner.

## Recommended Priority

1. P0 Admin route inventory parity gate.
2. P1 Admin contract generator deepening design.
3. P1 Separate Admin client from Public Client bridges, if client/web lanes run
   together.
4. P2 Redaction test vocabulary and matrix, paired with next diagnostics queue.
5. P2 Admin Web route search helper, when page churn is active.
6. P3 Admin diagnostics module sizing review, one domain at a time.

## Ready / Not Ready Summary

| Candidate | Ready for implementation? | Needs architecture audit? | Fearless refactor candidate? |
| --- | --- | --- | --- |
| Admin route inventory parity gate | Yes | No | No |
| Admin contract generator deepening | No | Yes | Yes, after design |
| Separate Admin client from Public bridges | Yes | No | Yes |
| Admin Web route search helper | Yes | No | Yes |
| Redaction vocabulary and matrix | Partially | Light spec/audit first | No |
| Admin diagnostics module sizing review | No | Yes | Yes, per domain |

## Final Recommendation For This Lane

Do architecture audit first at the cross-lane level, but for this lane the
highest-value next action is not another product feature. It is a bounded
contract-hardening task: add Admin route inventory parity and placeholder
normalization. That task makes later Admin diagnostics, Addon, playback,
network, and storage/VFS work safer to parallelize.

After that, choose one cleanup based on the global queue shape:

- if many Admin DTOs are coming, deepen `admin_contract.rs`;
- if web-product and client-surface work run together, split Admin client from
  Public Client bridges;
- if many Admin pages are coming, extract route search helpers.
