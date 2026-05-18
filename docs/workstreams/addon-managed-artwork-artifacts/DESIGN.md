# Addon Managed Artwork Artifacts

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

`addon-protected-writes` proved the protected-write apply model with
Canonical Metadata. Artwork is a different boundary: remote image URLs, addon
generated images, thumbnails, cache files, selected artwork, sidecar exports,
and diagnostics all involve artifact and storage policy that should not be
hidden inside the metadata lane.

## Problem

Taru has `ImageAsset` and `ArtworkTask` primitives, but it does not yet have a
cohesive Addon-owned artwork apply service. If `artwork_write` simply accepted
provider URLs or addon file paths, Taru would risk:

- serving unstable hotlinks instead of Managed Artwork;
- exposing Source Locators or filesystem paths to Addon Sidecars;
- bypassing resource budgets for external fetches, thumbnailing, and cache
  storage;
- losing provenance between Artwork Source, Artwork Candidate, Selected
  Artwork, and Taru-Managed Artifact;
- returning unsafe diagnostics or unredacted payloads.

## Target State

- `artwork_write` requests move through Addon Side Effect intake and explicit
  apply outcome state.
- Addon artwork output is normalized into a Taru-owned artwork/artifact command.
- Taru owns artifact storage, cache URI assignment, selected-artwork state, and
  redacted diagnostics.
- Remote artwork is fetched by a bounded Taru worker or represented as a
  candidate until fetched; Addon Sidecars do not give Taru raw library paths.
- Public Client surfaces only safe Managed Artwork references.

## In Scope

- Audit current `ImageAsset`, `ArtworkTask`, storage/VFS, catalog, and Addon
  Side Effect seams.
- Decide the first concrete `artwork_write` payload and target model.
- Decide whether first apply is synchronous, queued, or candidate-only.
- Define artifact storage, thumbnail/cache policy, resource budgets, and
  provenance.
- Update HTTP API docs and workstream evidence for shipped behavior.

## Out Of Scope

- Full Addon Manager lifecycle.
- Direct addon database, filesystem, or remote storage access.
- Public Client write routes.
- Subtitle, NFO, or other Library File Write behavior.
- General image-processing pipeline replacement outside the first artwork
  apply path.

## Architecture Direction

Reuse the APW three-stage model:

1. Addon runtime route authenticates, validates permission/library/target,
   persists the side-effect record, and returns redacted summaries.
2. Artwork-specific validation normalizes payload into a Taru artwork/artifact
   command.
3. Taru applies the command through Managed Artwork, artifact storage, catalog,
   and task seams, then records a safe apply outcome.

If fetching, resizing, hashing, or exporting can exceed a short request budget,
prefer a queued Addon Task or durable job rather than blocking the runtime
request.

## Closeout Condition

This lane can close when:

- the artwork/artifact seam audit is recorded;
- the first `artwork_write` behavior is implemented or explicitly split with
  evidence-backed reasoning;
- resource budgets, artifact provenance, storage/cache policy, and redaction
  guarantees are documented and tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
