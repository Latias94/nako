# Addon Notification Provider Adapters — Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

Addon Notification Bridge proved host registration and scheduled delivery to an
ACK-only sidecar. Provider breadth is split here so the first real provider can
be selected deliberately with credential, template, retry, and redaction
boundaries recorded before implementation.

ANP-010 selected `http_webhook` as the first real provider target. It should be
implemented as an outbound HTTP webhook sink owned by the notification bridge
sidecar.

ANP-020 added the sidecar-owned `http_webhook` configuration contract and
redaction-safe diagnostics while the provider send path was still disabled.

ANP-030 implemented the first `http_webhook` send path. When explicitly
configured, the sidecar sends a fixed redaction-safe JSON summary to the
webhook target after receiving `library.scanned`. The default remains ACK-only.

ANP-040 finished integration/docs verification. The default smoke now asserts
the ACK output reports `http_webhook` as disabled. The runtime manifest and
official catalog facts remain unchanged, so no host contract change was needed.

ANP-050 closed this lane after review and final gates.

## Active Task

- Task ID: none
- Owner: planner
- Files:
  - `docs/workstreams/addon-notification-provider-adapters`
- Validation:
  - see `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: ANP-050 closeout found no blocking workstream-compliance or
  code-quality findings.
- Evidence: ANP-010 through ANP-050 evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Provider breadth is split from the ACK-only notification bridge lane.
- Nako core must remain provider-agnostic.
- Provider credentials, templates, provider calls, and provider-specific retry
  are sidecar-owned.
- First provider target is `http_webhook`.
- The first send path should use a fixed redaction-safe JSON summary rather
  than a user-defined template language.
- The first adapter should not add a sidecar background queue; retryable
  provider HTTP failures can surface as safe retryable sidecar failures so the
  existing Addon Event delivery retry is reused.
- `http_webhook` config is sidecar-owned through
  `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_*` environment variables.
- The addon manifest intentionally keeps `secret_reference_fields` empty for
  this slice so Nako core does not store provider URL or provider secret.
- Health and diagnostics report only safe booleans/status strings and expose
  whether the `http_webhook` send path is enabled.
- The send path posts a fixed JSON summary with event facts and sorted payload
  keys only.
- Provider HTTP `408`, `429`, `5xx`, and transport failures map to safe
  retryable sidecar failures so host delivery retry can run.
- Other provider HTTP `4xx` rejections map to a non-retryable safe sidecar
  failure.
- No sidecar background queue or provider attempt history exists yet.
- ANP-040 verified the host catalog facts still match the checked-in
  notification bridge manifest shape; no manifest/protocol change was made.
- The provider adapter lane is complete and closed.
- Named follow-ons:
  - `addon-notification-platform-adapters`
  - `addon-notification-template-controls`
  - `addon-notification-provider-attempt-history`
  - `addon-notification-provider-live-smoke`

## Blockers

- None currently known for ANP-050 closeout.

## Next Recommended Action

Do not restart provider breadth inside this closed lane. Open one named
follow-on only after the product priority is clear.
