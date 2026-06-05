# Host-owned Addon resource flow pattern audit

## Goal

Define the server-owned pattern that turns read-only Addon Resource discovery
into explicit Nako-owned selection, apply planning, safe diagnostics, and
side-effect handoff without letting Addon Sidecars mutate host state directly.

## Requirements

- Compare existing Resource Search, subtitle import, and external acquisition
  flows.
- Define common vocabulary for selection sessions, selected references,
  apply-plan/apply-result shapes, safe error codes, grant checks, redaction, and
  host-owned side-effect authority.
- Keep the pattern server-local first; do not move host policy into
  `nako-addon-protocol`.
- Identify whether Addon task/event execution policy needs a separate follow-on.
- Identify Admin/API surfaces that must serialize with future Addon Manager
  work.

## Acceptance Criteria

- [ ] The audit identifies duplicated or divergent host-owned resource-flow
      behavior across current Addon flows.
- [ ] The audit proposes a server-local pattern and names what should remain in
      the permissive Addon Protocol.
- [ ] Safe diagnostic and redaction requirements are listed.
- [ ] Product decisions blocking Addon Manager implementation are listed.
- [ ] A first bounded implementation/refactor follow-on is recommended or
      explicitly deferred.

## Definition of Done

- Research output is written under this task or linked from the parent audit.
- No production code changes are made in this audit task.
- `git diff --check` passes.

## Out of Scope

- No Addon Manager implementation.
- No Addon process lifecycle, Docker socket, auto-update, or signing behavior.
- No mechanical split of `nako-addon-protocol`, `nako-addon-client`, or
  `nako-official-addon-catalog`.
- No Admin Web Addon UI changes.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/addon-boundary-automation.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Important docs/ADRs:
  - ADR 0003, 0015, 0020, 0050, 0051, 0053
