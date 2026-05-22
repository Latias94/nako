# Managed Artwork Module Deepening Design

Status: Completed
Last updated: 2026-05-19

## Problem

Managed Artwork behavior is correct but too much implementation is concentrated
in broad files:

- `crates/nako-server/src/app/artwork.rs` mixes candidate acceptance, ingest
  claiming and processing, remote fetch, image validation, local artifact file
  storage, Selected Artwork publication, image variant serving, gallery
  assembly, lifecycle diagnostics, drift inventory, and remediation cleanup.
- `crates/nako-db/src/artwork.rs` mixes task, candidate, ingest, artifact,
  selection, gallery, lifecycle, and remediation SQL adapters.
- `crates/nako-api/src/admin.rs` carries many Admin DTOs and redaction tests for
  Managed Artwork alongside unrelated Admin API surfaces.

The concentration makes it harder to reason about the real seams. A maintainer
changing variant serving must keep artifact cleanup, ingest failure handling,
and gallery redaction in working memory even when those concerns should be
local.

## Target State

Managed Artwork keeps the existing product contract but gains deeper internal
Modules:

- a variant Module that owns variant request policy, resizing, media type, and
  opaque presentation validators;
- an artifact store Module that owns local artifact paths, file inventory,
  stray-file classification, and delete outcomes;
- an ingest pipeline Module that owns fetch, validation, artifact write, and
  failure mapping behind a small orchestration interface;
- lifecycle/remediation Modules that separate diagnostics and cleanup planning
  from HTTP/Admin DTO projection;
- repository adapter modules that keep trait implementations visible while
  hiding SQL row mapping and query constants by concern.

The external app interface should remain stable unless a later task proves that
changing it removes real complexity. The redaction contract is load-bearing and
must not be weakened.

## Architecture Direction

### Variant Module

Move Selected Artwork variant behavior out of the broad app file first. The
caller should ask for selected image bytes with a parsed variant request and
receive a `ManagedArtworkImageBytes` value. The implementation owns:

- dimension policy;
- no-upscale behavior;
- output media type decisions;
- presentation ETag construction;
- storage-error mapping for invalid requests or derivation failures.

This is the safest first slice because public route behavior and tests already
exist.

### Artifact Store Module

Move local Managed Artwork Artifact file storage and inventory into a private
module. It should be the only implementation that knows:

- local artifact root layout;
- managed-artwork storage URI construction/parsing;
- file discovery and classification;
- safe delete outcomes;
- path prefix checks.

Callers should not assemble artifact paths or infer store layout.

### Ingest Pipeline Module

Separate processing mechanics from job orchestration. The app service may still
own durable job claiming and commits, but fetch, validate, store, and failure
summary creation should live behind a narrower Module interface.

This lane does not add retry, cancel, repair, or backoff behavior.

### Repository Adapter Modules

Keep core repository traits stable while splitting the SQLite adapter by
domain concern. SQL constants, row mapping, and transaction helpers should be
local to their concern instead of growing a single `artwork.rs` adapter file.

### API Surface

Admin and Public Client DTOs remain explicit. Redaction tests remain at the API
surface. Managed Artwork Admin DTOs now live in a focused module while
preserving the exported public names.

## Redaction Invariants

The following values are internal and must not be returned in public or Admin
DTOs, OpenAPI examples, SDK-visible fields, HTTP headers, or error messages:

- raw source URLs;
- `managed-artwork://` storage URIs;
- local filesystem paths;
- cache URIs;
- artifact content hashes;
- raw provider/addon payloads.

Internal domain records may carry these values when they are needed for Nako
authority, storage, or validation. Redaction is enforced at API and log/error
surfaces.

## Assumptions

- Existing behavior and tests are trusted as the compatibility baseline.
- Work proceeds through small refactor slices with focused validation.
- Reference projects under `repo-ref/` are for behavior and architecture study
  only; implementation remains original Nako code.
