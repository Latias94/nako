# Managed Artwork Artifact Publish Generated Route Contract

## Goal

Move the existing Selected Artwork publication route into the generated Admin
API contract so Admin Web/client code can call it through
`NAKO_ADMIN_ROUTES` instead of leaving it as an Admin route-inventory
exclusion.

## What I Already Know

- Nako already implements
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- The route publishes a stored Managed Artwork Artifact as Selected Artwork.
- Existing server/API/DB workstream evidence says publication is idempotent and
  redaction-safe.
- The generated Admin TypeScript contract already contains
  `PublishSelectedArtworkResponse` because item artwork selection flows consume
  the same response shape.
- The route is currently excluded only because it has not been given a stable
  generated Admin Web route key.

## Reference-Code Boundary

- Jellyfin is reference material only. Do not copy code, comments, schemas, or
  tests.
- Jellyfin's image and remote-image controllers show that selecting/downloading
  item artwork is an operator workflow.
- Nako intentionally differs: Admin publication accepts only a stored
  `artifact_id`; it does not accept raw image URLs, local paths, provider
  payloads, or arbitrary upload bodies in this slice.

## Requirements

- Add a generated Admin route key for:
  - `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`
- Remove that route from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Regenerate both Admin TypeScript contract copies.
- Add `AdminApiClient.publishManagedArtworkArtifact(artifactId)` using the
  generated route key and `PublishSelectedArtworkResponse`.
- Add a focused Admin Web client test for route key, encoded path parameter,
  POST method, empty body, and safe response handling.
- Keep UI mutation behavior unchanged; do not add a new publish button or route
  workflow in this slice.
- Keep `storage_uri`, local paths, `managed-artwork://` handles, `source_uri`,
  `cache_uri`, provider URLs/query strings, tokens, credentials, content
  hashes, and raw backend payloads out of generated/Admin Web output.

## Acceptance Criteria

- [ ] `nako-api` generated route inventory includes
      `managedArtworkArtifactPublish`.
- [ ] `artwork/artifacts/{artifact_id}/publish` is no longer an explicit
      exclusion.
- [ ] Generated Admin Web contract copies match the generator.
- [ ] Admin Web client test covers the generated publish route and request
      body.
- [ ] Focused Rust/Admin Web gates pass before commit.

## Definition Of Done

- Code and generated artifacts are updated.
- Task evidence records commands run and results.
- Commit this slice with a Conventional Commit message.

## Out Of Scope

- Candidate acceptance.
- Ingest process-next or requeue controls.
- Stray-file remediation.
- Artifact cleanup/garbage collection.
- New Admin Web publish UI or confirmation workflow.
