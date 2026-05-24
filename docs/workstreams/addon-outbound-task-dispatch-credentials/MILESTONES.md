# Addon Outbound Task Dispatch Credentials - Milestones

Status: Completed
Last updated: 2026-05-25

## M0 - Boundary Frozen

Exit criteria:

- workstream docs agree on target state and non-goals;
- direct dispatch still host-owns task scheduling;
- outbound credentials are treated as a separate follow-on from task runtime.

## M1 - Credential Model

Exit criteria:

- a safe storage/resolution shape exists for `AddonAuth::Bearer` and
  `AddonAuth::SharedSecret`;
- missing credential behavior is defined;
- secret material is not pushed into public DTOs.

Status: achieved with `outbound_task_dispatch_secret_env` storage on addon
registrations, host-side env resolution, and redaction-safe failure tests.

## M2 - Direct Dispatch Injection

Exit criteria:

- direct task dispatch injects the resolved outbound credential;
- focused tests prove bearer and shared-secret paths;
- redaction-safe diagnostics cover missing or unresolved credentials.

Status: achieved with direct dispatch credential resolution, header injection,
and safe missing-secret failure tests.

## M3 - Closeout

Exit criteria:

- final gates are recorded;
- docs match the shipped behavior;
- any richer secret-provider or vault follow-ons are split explicitly.

Status: achieved on 2026-05-25. Final gates are recorded in
`EVIDENCE_AND_GATES.md`; the shipped boundary remains env-reference based; any
richer vault/provider design is deliberately deferred to a separate follow-on.
