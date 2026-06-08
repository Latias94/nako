# Jellyfin Addon Install-Guide Preview Comparison

## Reference Findings

- Jellyfin has a plugin manager and installation manager that parse plugin
  manifests, reconcile package metadata, and expose plugin install/update
  operator workflows.
- Jellyfin's plugin model is in-process and filesystem/package oriented.
- Nako intentionally differs: Addons are out-of-process sidecars with scoped
  HTTP contracts, explicit grants, and install guides rather than a native ABI.

## Nako Gap

- Nako already has a non-persistent install-guide preview route for an
  `AddonInstallDescriptor`.
- The route is server-tested for validation and redaction, but it is still an
  Admin route inventory exclusion.
- Generated Admin Web consumers cannot call it via `NAKO_ADMIN_ROUTES`.

## Chosen Slice

- Generate only the existing preview route and wrapper DTOs.
- Keep registration/install/lifecycle mutation flows unchanged.
- Keep Admin Web UI behavior unchanged except for typed client access and tests.

## Redaction Boundary

The generated route/client/test surface must not render or return unsafe UI
material such as:

- raw manifest JSON text;
- raw secret values;
- local runtime paths;
- filesystem URLs;
- bearer tokens;
- addon token material.

## Validation Implications

- API contract tests should prove generated contracts contain the route and DTO
  wrappers while route inventory exclusions remain honest.
- Server Addon install-guide preview tests remain the source of truth for
  validation and redaction behavior.
- Admin Web client tests should prove the generated route is used with a POST
  body.
