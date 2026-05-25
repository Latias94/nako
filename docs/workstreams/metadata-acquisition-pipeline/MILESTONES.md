# Metadata Acquisition Pipeline Milestones

## MAP-M1 Pipeline Boundary

Outcome: Library scan orchestration delegates metadata phases to a focused
application service.

Status: complete 2026-05-25.

Exit criteria:

- Existing scan output shape is unchanged.
- NFO import cancellation remains cooperative.
- Addon bulk task creation remains suggestion-only by default.

## MAP-M2 Explicit Addon Writeback Policy

Outcome: Media Library scan metadata policy can request Addon metadata writeback
without granting authority by itself.

Status: complete 2026-05-25.

Exit criteria:

- Public DTOs and SDKs expose the new policy field.
- Default remains false.
- Payload includes official `writeback` only when enabled.

## MAP-M3 Closed-Loop Proof

Outcome: A scan-triggered Addon Bulk Metadata Scrape can write Canonical
Metadata through the existing Addon Side Effect path.

Status: complete 2026-05-25.

Exit criteria:

- The sidecar path receives the bulk scrape request.
- The sidecar submits `/addon/v1/side-effects`.
- Nako applies metadata through merge policy and catalog/search commit.
- Task output remains diagnostic/result data, not the mutation source.

## MAP-M4 Real Directory Smoke

Outcome: The local and NAS scan/playback smoke procedure remains repeatable
after the refactor.

Status: complete 2026-05-25.

Exit criteria:

- Local test directory scan and byte-range playback pass.
- NAS subdirectory scan and byte-range playback pass.
- Any Addon sidecar prerequisites are documented if writeback smoke is skipped.
