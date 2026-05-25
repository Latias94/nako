# Addon Notification Provider Adapters — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Provider Selection Freeze

Status: Complete.

Exit criteria:

- first-provider criteria are explicit;
- credential, template, retry, and redaction ownership is recorded;
- one provider is selected or the lane is split again with a named reason.

Result: ANP-010 selected `http_webhook` as the first provider target and froze
the credential/template/retry/redaction boundary.

## M1 — Provider Configuration Contract

Status: Complete.

Exit criteria:

- sidecar configuration and secret reference docs exist for the selected
  provider;
- diagnostics are redaction-safe;
- tests prove invalid/missing configuration without raw secret leaks.

Result: ANP-020 added sidecar-owned environment configuration, safe status
diagnostics, operator docs, and no Nako-owned provider secret references.

## M2 — First Provider Send Path

Status: Complete.

Exit criteria:

- one provider send path is implemented behind the existing event ACK route;
- fixture-backed tests pass without live CI secrets;
- provider retry/rate-limit behavior is documented.

Result: ANP-030 implemented the fixture-backed `http_webhook` send path with
safe payloads, optional shared-secret header, retryable 408/429/5xx/transport
failure mapping, and non-retryable provider rejection mapping.

## M3 — Integration And Docs

Status: Complete.

Exit criteria:

- operator docs and smoke commands explain provider setup;
- host tests are added only if manifest/protocol behavior changes;
- no Nako core provider matrix is introduced.

Result: ANP-040 updated packaging docs and default smoke assertions, ran the
official addon full gate, and ran a focused host catalog gate. No host
manifest/protocol change was required.

## M4 — Closeout

Status: Complete.

Exit criteria:

- final evidence is recorded;
- remaining provider breadth is split into named follow-ons or explicitly
  deferred;
- `WORKSTREAM.json` and `HANDOFF.md` reflect final state.

Result: ANP-050 closed the lane and named follow-ons for platform adapters,
template controls, provider attempt history/background retry, and live provider
smoke.
