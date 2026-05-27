# Nako Renderer Cast-Safe Transport Milestones

Status: Active
Last updated: 2026-05-27

## Milestone 1 - Boundary Accepted

Exit criteria:

- ADR 0041 records that renderer cast-safe transport tickets are distinct from
  browser playback tickets.
- Workstream docs define scope, non-goals, task order, and gates.
- Stale ADR/workstream indexes point to the new active lane.

Tasks:

- `NRCT-010`

## Milestone 2 - Current Gap Locked By Tests

Exit criteria:

- Tests characterize direct-only renderer playback behavior.
- Tests characterize current `nako_remote_client` media transport-auth
  registration semantics.
- Tests prove browser tickets do not have renderer/playback-session/network
  scope.

Tasks:

- `NRCT-020`

## Milestone 3 - Ticket Primitive Implemented

Exit criteria:

- Renderer transport tickets can be issued and validated with all required
  scope fields.
- Expiry and scope mismatch behavior is covered.
- Debug/error output stays redaction-safe.

Tasks:

- `NRCT-030`

## Milestone 4 - Public Contract Carries Safe Transport

Exit criteria:

- Renderer command DTOs expose a typed transport envelope.
- Public OpenAPI and SDK outputs include the new safe shape.
- Tests prevent raw payload, bearer token, source locator, path, and transcode
  session credential leakage.

Tasks:

- `NRCT-040`

## Milestone 5 - Nako Remote Non-Direct Playback Works

Exit criteria:

- Nako remote-client renderer playback can use direct, remux, and HLS decisions.
- Control routes remain bearer-authenticated.
- Media URLs validate renderer tickets.
- Denied policy paths create no runtime side effects.

Tasks:

- `NRCT-050`

## Milestone 6 - Ready For Protocol Casting Workstreams

Exit criteria:

- Admin diagnostics accurately report Nako remote transport readiness without
  leaking ticket material.
- Chromecast/DLNA/AirPlay remain explicit follow-on lanes.
- Final gates pass and the lane is closed.

Tasks:

- `NRCT-060`
- `NRCT-070`
