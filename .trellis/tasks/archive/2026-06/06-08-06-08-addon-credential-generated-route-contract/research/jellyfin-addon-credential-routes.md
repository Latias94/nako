# Jellyfin Comparison: API Key Routes vs Nako Addon Credential Routes

## Reference Studied

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ApiKeyController.cs`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Security/AuthenticationManager.cs`

## Findings

Jellyfin exposes API key management as explicit admin-facing HTTP routes:
listing keys, creating a key for an application name, and deleting a key. The
controller delegates persistence and token lifecycle to an authentication
manager rather than letting callers compose hidden URLs.

Nako's model is intentionally different. An Addon Token is scoped to an Addon
Sidecar and the Addon Permissions accepted by the operator. Nako also separates
Addon grants from token lifecycle, while Jellyfin's API key route is a broader
application key management surface.

## Nako Decision

Keep Nako's existing Addon token and grant semantics. The architectural lesson is
not to copy Jellyfin's route shape; it is that credential-management operations
are explicit management-plane contracts. Because Admin Web already consumes
these Nako routes, they should be generated route keys instead of derived string
suffixes.

## Redaction Boundary

Generated route keys may expose route templates only. Admin Web may hold a
one-time raw token returned by issue/rotation responses in mutation state, but
load data and route inventory must not expose token hashes, raw stored
credentials, backend URLs, local paths, or Addon Sidecar secrets.
