# Jellyfin Managed Artwork Publish Comparison

## Reference Findings

- Jellyfin exposes item image management as explicit operator-facing routes in
  `ImageController`.
- Jellyfin exposes remote image search/download as explicit operator-facing
  routes in `RemoteImageController`.
- Jellyfin's image flow can operate directly on item image types, indexes,
  uploaded image data, and remote provider images.

## Nako Gap

- Nako already has a narrower Selected Artwork publication route:
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- The route is implemented and tested, but it remains an Admin route inventory
  exclusion.
- Generated Admin Web consumers cannot call it via `NAKO_ADMIN_ROUTES`.

## Chosen Slice

- Generate only the existing artifact publish route.
- Reuse the existing `PublishSelectedArtworkResponse` DTO.
- Add only a typed Admin Web client method and focused client test.

## Nako Boundary

- Publication accepts only a stored Managed Artwork Artifact ID.
- The artifact already carries library, item, kind, and storage authority.
- The request body stays empty in this slice.
- Nako must not accept raw provider URLs, local paths, source/cache URIs,
  arbitrary public image URLs, upload bodies, or storage handles through this
  generated client method.

## Validation Implications

- API contract tests should prove generated contracts contain the route while
  route inventory exclusions remain honest.
- Existing server tests remain the source of truth for publication idempotency
  and redaction.
- Admin Web client tests should prove the generated route is used with an
  encoded `artifact_id` and empty POST body.
