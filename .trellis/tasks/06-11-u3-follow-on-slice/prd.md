# brainstorm: redacted incident bundle export

## Goal

Ship a one-day operator support slice that packages Nako's existing safe diagnostics into a redacted incident bundle export. The slice should give self-hosted operators one Admin-only support artifact for hard bugs without exposing raw paths, locators, tokens, credentials, FFmpeg command lines, provider payloads, backend URLs, or raw job payloads.

## What I already know

* The previous U3 first slice is complete, archived, and verified.
* Nako already ships `Playback Support Evidence` through Admin API and Admin Web.
* `docs/architecture/CONTROL_PLANE.md` still marks crash/fault bundles as not started.
* `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` keeps incident bundles deferred rather than shipped.
* Nako already exposes safe summaries for overview/system config, playback runtime/support evidence, jobs queue pressure, storage staging/VFS repair, and source hash/duplicate readiness.
* Jellyfin and Plex both validate that a mature self-hosted media server exposes operator-supportable diagnostic views as first-class capabilities.

## Assumptions (temporary)

* The first slice should be JSON-only, not a zip/archive package.
* The bundle should aggregate existing safe DTOs rather than introduce raw log, path, or command plumbing.
* The slice should stay inside Admin API/Admin Web boundaries and avoid schema or Public Client contract changes.
* The bundle is a read-only support artifact, not a repair action, backup engine, telemetry upload, or remote-access wizard.

## Open Questions

* None.

## Requirements (evolving)

* Add an Admin-only incident bundle DTO and route.
* Compose the bundle from existing safe summaries where possible: system/config posture, endpoint/network posture, playback readiness/support evidence, storage/VFS repair posture, and durable job queue pressure.
* Add an Admin Web read-only projection or export page that lets an operator inspect the bundle before sharing it manually.
* Include explicit redaction status so operators can see which sensitive field families were excluded.
* Keep raw paths, locators, tokens, credentials, FFmpeg command lines, provider payloads, backend URLs, query strings, raw job payloads, and unbounded logs out of the rendered and serialized artifact.
* Keep the view useful without turning it into a backup engine, upload workflow, remote access wizard, or generic log browser.

## Acceptance Criteria (evolving)

* [ ] Admin API exposes a JSON-only redacted incident bundle route.
* [ ] The bundle includes safe system/config, endpoint/network, playback, storage/VFS, and durable job pressure sections.
* [ ] Admin Web can request and inspect the bundle through a read-only operator surface.
* [ ] The route and page omit raw paths, locators, tokens, credentials, FFmpeg commands, provider payloads, backend URLs, query strings, raw job payloads, and unbounded logs.
* [ ] Tests prove the Admin contract, server assembly, and Admin Web projection remain redaction-safe.

## Definition of Done (team quality bar)

* Tests added/updated where behavior changes.
* Lint, format, and relevant checks pass.
* Docs/notes updated if the slice changes durable behavior or vocabulary.
* The work can be committed cleanly with a Conventional Commit.

## Out of Scope (explicit)

* Zip/archive generation.
* Upload, email, or sharing workflow for support artifacts.
* Backup/restore engine implementation.
* Remote access wizard or endpoint onboarding flow.
* User management, parental controls, or playback-policy overhaul.
* Raw server log download or client log ingestion.
* Realtime diagnostics streaming.

## Technical Approach

* Add a redacted Admin DTO in `nako-api` for the incident bundle.
* Add a thin `nako-server` Admin handler that composes existing safe diagnostic summaries.
* Add an Admin Web data-source method and read-only page/action for inspecting the bundle.
* Prefer existing redaction helpers and summary DTOs over new raw record plumbing.
* Add focused tests for each sensitive field family.

## Decision (ADR-lite)

**Context**: Jellyfin/Plex-class operators need a support artifact they can inspect and share during hard bugs without leaking host secrets.

**Decision**: Ship a JSON-only redacted incident bundle export next.

**Consequences**: The slice turns shipped diagnostics into an operator-supportable artifact while leaving zip packaging, upload/share transport, logs, and realtime diagnostics as explicit follow-ups.

## Research References

* [`research/jellyfin-gap-analysis.md`](research/jellyfin-gap-analysis.md) - Jellyfin controller patterns and the broader supportability gap.
* [`research/nako-gap-map.md`](research/nako-gap-map.md) - Nako's shipped safe diagnostic surfaces and residual incident-bundle gap.
* [`research/incident-bundle-plan.md`](research/incident-bundle-plan.md) - task-local implementation outline for the chosen export slice.
* [`research/plex-benchmark.md`](research/plex-benchmark.md) - Plex benchmark and predecessor support-view context.
* [`research/playback-support-evidence-plan.md`](research/playback-support-evidence-plan.md) - shipped predecessor note for the playback support evidence view.

## Technical Notes

* Task created after the U3 first slice closeout on 2026-06-11 and updated after the Admin Web playback support evidence route shipped.
* Good starting references: `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`, `CONTEXT.md`, `docs/architecture/CONTROL_PLANE.md`, `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`, `docs/architecture/OPERATIONS_RELEASE.md`.
