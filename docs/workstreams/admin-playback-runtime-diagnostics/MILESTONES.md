# Admin Playback Runtime Diagnostics Milestones

Status: Completed
Last updated: 2026-05-18

## M56: Admin Playback Runtime Diagnostics

Objective:

- Add a read-only Admin API v1 playback runtime diagnostics surface for the web
  console.
- Explain hardware acceleration policy, selected acceleration, FFmpeg
  capability evidence, transcode budgets, remote playback budgets, and staging
  cleanup configuration.
- Preserve Public Client API, public OpenAPI/SDK, and `taru-client-protocol`
  boundaries.

Deliverables:

- Admin-owned playback runtime diagnostics DTOs in `taru-api::admin`.
- Playback app diagnostics snapshot support in `taru-server`.
- `GET /admin/v1/playback/runtime`.
- Focused API/server tests for shape, redaction, auth, and public leakage.
- Updated admin-web-console data-source notes after route support lands.

Non-goals:

- No Public Client API route or DTO changes.
- No `taru-client-protocol` changes.
- No public OpenAPI or TypeScript SDK expansion.
- No playback session mutations.
- No playback source selection deepening.
- No adaptive HLS ladder or FFmpeg runner behavior changes.
- No frontend UI implementation.

Exit criteria:

- Admin Console can read playback runtime diagnostics through
  `/admin/v1/playback/runtime`.
- The response includes hardware policy, selected acceleration, FFmpeg
  capabilities, transcode budgets, remote stream/stage budget summaries, and
  staging cleanup summaries.
- The response does not expose local paths, staging roots, transcode
  `output_path`, secrets, tokens, or process-local runner handles.
- Existing Public Client API playback routes remain compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- Focused API and server validation gates pass.
