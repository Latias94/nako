# Playback Runtime Resource Admission

## Goal

Deepen playback runtime resource admission so Direct Play, Remux, HLS, staging,
and transcode work expose bounded, operator-visible pressure behavior instead
of relying on hidden per-feature limits.

## Requirements

- Audit current playback resource handling before changing code.
- Preserve Direct Play first behavior; admission should not force transcode as
  a fallback unless policy allows it.
- Model resource admission in the playback runtime/control-plane boundary,
  not inside ad hoc HTTP handlers.
- Keep denial and queue/pressure errors typed and client-safe.
- Add focused tests for admission, denial, and unchanged happy paths.
- Coordinate with Media Web lane before changing public playback session
  response semantics.

## Acceptance Criteria

- [ ] Current resource permits and pressure behavior are mapped.
- [ ] A bounded admission decision exists for at least one high-pressure
  playback path selected by the worker after audit.
- [ ] Denied or deferred playback work produces stable, redaction-safe error or
  status evidence.
- [ ] Existing Direct Play/Remux/HLS happy-path tests continue to pass.
- [ ] Operator or diagnostic follow-on is recorded if the first slice does not
  expose UI/API status.

## Definition of Done

- Focused playback/server/transcode tests pass.
- Public error or status behavior is covered where changed.
- Architecture/evidence notes identify any follow-on for remote workers,
  LL-HLS/CMAF, hardware smoke, subtitle burn-in, or GPU scheduling.

## Out of Scope

- No distributed transcode queue in this first slice.
- No LL-HLS/CMAF implementation unless selected as the narrow admission target.
- No Web player UI changes except compile/test updates needed by contract
  changes.
- No schema migration without planner approval.

## Technical Notes

- Likely files: `crates/nako-server/src/app/playback/resource.rs`,
  `crates/nako-server/src/app/playback/*`, `crates/nako-transcode/src/runtime.rs`,
  and playback HTTP tests.
- ADR 0053 is the baseline for runtime/control-plane behavior.
- Stop for planner coordination if this needs global job scheduler semantics.
