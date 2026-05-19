# Managed Artwork Ingest Selection

Status: Active
Last updated: 2026-05-19

## Why This Lane Exists

`addon-managed-artwork-artifacts` intentionally stopped at internal Addon
Artwork Candidates. That boundary prevents addon-provided URLs from becoming
public client artwork, but users still need a path to accept a candidate into
Taru-managed artwork.

The next boundary is more complex than the Addon handler: network fetch,
content validation, cache/artifact storage, thumbnailing, selected artwork,
catalog hydration, and safe diagnostics all need one first-party ownership
model.

## Problem

Current public `ImageAsset` records expose `source_uri` and `cache_uri` through
Public Client DTOs. Directly copying a candidate's remote URL into
`ImageAsset` would create public hotlinks and leak addon/provider details.

Taru also does not yet have a cohesive managed artwork ingest service that can:

- fetch remote images with resource budgets and retry policy;
- validate content type, byte size, dimensions, and decodability;
- assign stable Taru-owned cache/artifact URIs;
- create thumbnails without blocking runtime routes;
- atomically update selected artwork and catalog projections;
- report failures without exposing raw URLs, local paths, or cache internals.

## Target State

- Candidate acceptance is a Taru-owned command, not direct Addon mutation.
- Remote fetch runs through a bounded worker or durable job with resource
  class, timeout, retry, and cancellation semantics.
- Managed artwork storage owns cache URI assignment and internal source
  provenance.
- Public Client artwork exposes only safe managed references and metadata.
- Selected artwork changes commit through a first-party artwork/catalog unit so
  image rows, selected flags, thumbnails, and search/catalog projections cannot
  diverge.

## In Scope

- Audit existing Artwork Candidate, `ImageAsset`, `ArtworkTask`, VFS/cache,
  staging, catalog hydration, and admin/public API seams.
- Decide the first acceptance path: admin-reviewed candidate accept, automatic
  trusted-addon policy, or queued managed ingest without selection.
- Define managed artwork artifact/cache ownership.
- Define selected artwork and public DTO redaction rules.
- Add tests and API docs for the selected first slice.

## Out Of Scope

- Addon sidecar proposal intake already shipped in AMAA.
- Artwork sidecar export remains `addon-library-file-write-policy`.
- Full image editing, AI generation, or arbitrary media transformations are out
  of scope.
- Marketplace/addon lifecycle automation is out of scope.

## Architecture Direction

Prefer a first-party service boundary, for example `ManagedArtworkService`, that
accepts candidate IDs or explicit internal commands and owns:

1. candidate lookup and policy checks;
2. network fetch scheduling or job creation;
3. content validation and managed artifact/cache write;
4. public `ImageAsset` publication with source/cache redaction policy;
5. selected artwork mutation and catalog/search refresh through one commit
   boundary.

If a slice cannot complete within a short request budget, record a queued job
or artwork task and make the HTTP/admin response truthful about queued vs
applied state. Do not reuse Addon Side Effect `applied` for deferred fetch work
unless a job association is recorded.

### First Slice Candidate

The recommended first executable slice is an audit and design pass:

- inventory whether `ArtworkTask` can represent candidate fetch/validate/cache
  or whether a new managed artwork job model is needed;
- decide where Taru-owned cache/artifact bytes live and how cache URIs are
  hidden or exposed;
- decide whether first acceptance creates an unselected managed artifact or a
  selected public `ImageAsset`;
- define a redacted admin/report shape before adding runtime behavior.

Do not implement remote fetch/cache in an Addon handler. Do not publish raw
candidate `source_uri` as public artwork.

### MAIS-020 Audit Decision

The first implementation target is a queued candidate-ingest boundary that
creates internal Managed Artwork state. It must not create selected public
`ImageAsset` rows during candidate acceptance.

The service shape should be:

1. `ManagedArtworkService::accept_candidate(candidate_id, policy)` or an
   equivalent first-party command validates the candidate, target item,
   library scope, status, and acceptance policy.
2. The command updates candidate acceptance state and creates a durable managed
   ingest record plus a durable job. The job input may include safe Taru IDs
   such as candidate, item, library, and image kind, but not the candidate's raw
   remote URL.
3. A worker loads the candidate source internally, fetches with artwork fetch
   resource budgets, validates content type, byte size, dimensions, and
   decodability, then writes a Taru-owned managed artifact record.
4. Public artwork publication is a later commit boundary. It may create or
   update `ImageAsset` only after the managed artifact exists and the public DTO
   redaction strategy is explicit.

Rejected first targets:

- Direct selected public `ImageAsset`: rejected because current Public Client
  DTOs expose `source_uri` and `cache_uri`, and catalog hydration can already
  auto-select provider image references. Candidate acceptance must not turn an
  unvalidated addon URL into a selected client-visible hotlink.
- Reusing `ArtworkTask` directly: rejected because `ArtworkTask` is keyed by
  `ImageAssetId`. Candidate fetch happens before a safe public asset should
  exist, so using it would force premature public row creation.
- Reusing staging manifest or VFS cache as Managed Artwork storage: rejected
  because staging is cleanup-oriented probe/FFmpeg input state, and VFS cache is
  remote storage fact cache. Neither is durable library artwork authority.

Boundaries for the next implementation slice:

- Add a dedicated managed artwork ingest/entity model rather than overloading
  public catalog image rows.
- Add a `JobKind` and resource class for managed artwork ingest if the existing
  generic job table is used. Persist only redacted job input and summaries,
  because `GET /jobs/{job_id}` exposes parsed job input and summary today.
- Keep raw candidate source URL, internal cache path, artifact storage URI,
  provider response details, and validation internals out of public and addon
  responses.
- Treat `cache_uri` as internal until a safe managed reference or image-serving
  route contract exists.
- Keep selected artwork mutation separate from ingest unless the same
  transaction can update managed artifact, public image, selected flags, and
  catalog/search projections without divergence.

ADR impact: ADR 0013 remains valid as the resource-class foundation for image
work, but it is not the right queue identity for candidate ingest. A future ADR
or ADR amendment is needed when Public Client artwork DTOs move from raw
`source_uri`/`cache_uri` exposure to managed image references.

## Closeout Condition

This lane can close when:

- one candidate acceptance/managed ingest path is implemented or explicitly
  split with evidence-backed reasoning;
- managed storage, redaction, selected-artwork semantics, and resource budgets
  are documented and tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
