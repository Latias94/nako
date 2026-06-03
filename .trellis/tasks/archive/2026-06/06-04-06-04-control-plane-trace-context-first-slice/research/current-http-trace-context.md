# Current HTTP trace context research

- Query: choose the smallest useful control-plane trace-context implementation
  slice for Nako.
- Scope: server HTTP boundary only.
- Date: 2026-06-04.

## Findings

### Architecture and ADR constraints

- `docs/architecture/CONTROL_PLANE.md` defines
  `control-plane-observability-and-trace-context` as request ID and trace
  context propagation across HTTP, durable jobs, VFS, FFmpeg, addons, and
  diagnostics.
- ADR 0053 requires request identity and trace context where useful, but also
  requires diagnostics to remain redacted and self-hosted telemetry to be
  explicit.
- The full architecture lane is cross-cutting. A first implementation should
  avoid jobs, VFS, FFmpeg, addon, webhook, API DTO, database, or Admin incident
  bundle scope.

### Current router shape

- `crates/nako-server/src/http.rs` centralizes root router assembly.
- Public health routes, unauthenticated account routes, protected routes, and
  addon runtime routes are merged into one root `Router`.
- All responses already pass through a top-level `add_api_version_header`
  middleware.
- Network boundary middleware can return preflight or forbidden responses before
  route handlers run, and auth middleware can return `401` before protected
  handlers run. A trace context middleware should therefore live at the root
  router level.
- CORS preflight currently allows `authorization`, `content-type`, and `range`;
  accepting browser-provided `x-request-id` requires adding that header to the
  allow list.
- Existing tests use Axum routers and `tower::ServiceExt`, so middleware
  behavior can be verified without a network listener.

### Existing safe ID patterns

- `nako-server` already depends on `uuid`.
- Existing server code uses `uuid::Uuid::new_v4().simple()` for opaque local
  identifiers such as tickets and addon helper IDs.
- A request ID can be generated without adding a new dependency.

### Recommended slice

Implement an HTTP-only typed trace context:

- Accept `x-request-id` only when it is short and made from a strict safe
  alphabet.
- Normalize accepted inbound IDs to lowercase.
- Generate a safe opaque value when the header is missing or invalid.
- Insert a typed context into request extensions.
- Echo the safe request ID through the `x-request-id` response header.

This establishes the first reusable seam for later propagation into playback,
durable jobs, VFS, FFmpeg, addons, and incident bundles without committing this
task to those broader changes.

## Risks

- If the middleware only wraps protected routes, public health, CORS preflight,
  or auth rejections may miss request IDs.
- If inbound IDs are accepted loosely, URLs, local paths, tokens, or raw user
  strings can become diagnostics.
- If the ID is added to response bodies or API DTOs in this first slice, the
  task becomes a public contract change and pulls in generated-contract gates.

## Verification candidates

- `/health` without inbound ID returns a generated `x-request-id`.
- `/health` with a valid mixed-case inbound ID returns the normalized ID.
- `/health` with an unsafe inbound ID does not echo the unsafe value.
- A protected request without auth still returns `x-request-id` and keeps the
  existing `x-nako-api-version` and `WWW-Authenticate` behavior.
