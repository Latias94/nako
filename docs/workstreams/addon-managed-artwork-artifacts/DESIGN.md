# Addon Managed Artwork Artifacts

Status: Active
Last updated: 2026-05-19

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

### Core Architecture Alignment

`artwork_write` must not become a side channel that writes catalog-visible image
state, cache state, or search projections independently from Taru-owned artwork
and catalog services. If the first artwork slice needs a stronger durable write
unit than the current artwork/catalog seams provide, AMAA should add that
first-party commit boundary instead of embedding multi-step persistence inside
the Addon handler.

Artwork export to media-library sidecar files is a **Library File Write** and
belongs in `addon-library-file-write-policy`; AMAA may produce Managed Artwork
or Taru-Managed Artifacts, but it should not invent a separate file-write path.

### AMAA-020 Selected First Target

The first AMAA-030 apply target should be an addon-initiated Artwork Candidate
proposal for an existing Media Item. The addon may request that Taru consider a
poster, backdrop, logo, banner, or thumbnail candidate, but it must not
directly create selected artwork, public client artwork references, or library
sidecar files. Arbitrary `other` artwork kinds are deferred until key naming,
display, and selection semantics are explicit.

The first payload should be typed around candidate intent, for example:

```json
{
  "intent": "propose_artwork",
  "kind": "poster",
  "source": {
    "kind": "remote_url",
    "url": "https://addon.example/poster.jpg"
  },
  "language": "en",
  "width": 1000,
  "height": 1500
}
```

The first slice should accept only HTTP(S) remote URL sources. It should reject
filesystem paths, Source Locators, remote storage handles, raw image bytes,
data URIs, `cache_uri`, `selected`, and sidecar export fields. Candidate source
details may be stored internally, but the Addon response/report must expose
only redacted aggregate facts and stable IDs.

Do not write the current public `ImageAsset` table directly for this first
slice. Existing Public Client DTOs expose `source_uri` and `cache_uri`; writing
addon-provided URLs there would turn unverified addon output into
client-visible artwork and risk unstable hotlinks. AMAA-030 should introduce
or reuse a first-party candidate boundary that is not treated as selected
public artwork.

Managed Artwork fetch/cache, thumbnail generation, selected-artwork changes,
and sidecar export remain follow-on work. If a later slice makes candidates
public or selected, it must add artifact fetching/storage, content validation,
resource budgets, cache URI assignment, and redacted diagnostics first.

## Closeout Condition

This lane can close when:

- the artwork/artifact seam audit is recorded;
- the first `artwork_write` behavior is implemented or explicitly split with
  evidence-backed reasoning;
- resource budgets, artifact provenance, storage/cache policy, and redaction
  guarantees are documented and tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
