# Addon Install-Guide Preview Generated Route Contract

## Goal

Move the existing non-persistent Addon install-guide preview route into the
generated Admin API contract, so Admin Web/client code can call it through
`NAKO_ADMIN_ROUTES` instead of leaving it as a route-inventory exclusion.

## What I Already Know

- Nako already implements `POST /admin/v1/addons/install-guide-preview`.
- The server validates `AddonInstallDescriptor` and returns an
  `AdminAddonInstallGuidePreviewResponse { guide }`.
- Existing server tests prove the route rejects local runtime paths/raw secrets
  and does not echo raw secret material, local paths, bearer tokens, or
  filesystem URLs.
- The generated contract already includes `AdminAddonInstallDescriptor`,
  `AdminAddonRuntimeRequirement`, `AdminAddonSecretReferenceBinding`, and the
  Addon install-guide DTOs through Addon catalog resolve/install-guide flows.
- The route is currently excluded only because it lacked a stable generated
  Admin Web route key.

## Reference-Code Boundary

- Jellyfin is reference material only. Do not copy code, comments, schemas, or
  tests.
- Jellyfin's plugin manager and installation manager show that plugin manifest
  validation/preview is an operator workflow.
- Nako's addon model remains out-of-process Addon Sidecars with generated,
  redaction-safe Admin contracts.

## Requirements

- Add a generated Admin route key for:
  - `POST /admin/v1/addons/install-guide-preview`
- Remove that route from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Add generated TypeScript wrapper DTOs:
  - `AdminAddonInstallGuidePreviewRequest`
  - `AdminAddonInstallGuidePreviewResponse`
- Regenerate both Admin TypeScript contract copies.
- Add `AdminApiClient.previewAddonInstallGuide(request)` using
  `NAKO_ADMIN_ROUTES.addonInstallGuidePreview`.
- Keep the existing local JSON preview helper behavior intact; do not replace
  Legacy dashboard onboarding in this slice.
- Add client tests for route key, POST body, and safe response handling.
- Add data-source tests if adding a public data-source method.
- Keep raw manifest JSON, raw secret values, local runtime paths, bearer tokens,
  and filesystem URLs out of rendered Admin Web/UI output.

## Acceptance Criteria

- [ ] `nako-api` generated route inventory includes
      `addonInstallGuidePreview`.
- [ ] `addons/install-guide-preview` is no longer an explicit exclusion.
- [ ] Generated Admin Web contract copies contain preview request/response DTOs
      and match the generator.
- [ ] Admin Web client test covers the generated preview route and request body.
- [ ] Focused Rust/Admin Web gates pass before commit.

## Definition Of Done

- Code and generated artifacts are updated.
- Task evidence records commands run and results.
- Commit this slice with a Conventional Commit message.

## Out Of Scope

- Registering or installing addons.
- Reworking Legacy dashboard onboarding UI.
- Adding a new Addon install preview page.
- Addon package download, process supervision, plugin marketplace mutation, or
  lifecycle execution.
