# Managed Artwork Ingest Selection

Status: Proposed
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

## Closeout Condition

This lane can close when:

- one candidate acceptance/managed ingest path is implemented or explicitly
  split with evidence-backed reasoning;
- managed storage, redaction, selected-artwork semantics, and resource budgets
  are documented and tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
