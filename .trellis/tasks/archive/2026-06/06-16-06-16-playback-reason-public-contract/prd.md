# playback reason public contract

## Problem

Nako can already decide Direct Play, Remux, Transcode, or Denied, but the public and admin playback surfaces do not yet expose a stable, shared compatibility-reason vocabulary that is clearly bounded from runtime execution details. That makes playback support harder to explain across clients and Admin diagnostics.

## Goal

Make playback decisions explainable through a redaction-safe reason contract that can be shared by public client responses, Admin playback diagnostics, and focused server tests.

## Scope

In scope:

- Stabilize playback compatibility reason codes for Direct Play, Remux, Transcode, and Denied outcomes.
- Keep reason codes redaction-safe and free of FFmpeg command lines, raw paths, Source Locators, and secret payloads.
- Update public/Admin contracts and focused HTTP tests when DTOs or serialized shapes change.
- Keep playback planning in `nako-playback`, transcode execution in `nako-transcode`, and route mapping in `nako-server`.

Out of scope:

- New FFmpeg runtime behavior.
- Android or UI playback explainability work.
- Broad device-profile database expansion.
- Remote worker or durable playback queueing.

## Requirements

1. Playback decisions expose stable compatibility reasons for Direct Play, Remux, Transcode, and Denied outcomes.
2. Public/Admin-facing payloads keep those reasons redaction-safe.
3. Route mappings preserve existing playback behavior and do not leak runtime details.
4. Contract artifacts stay in sync when DTOs change.
5. Focused tests prove the reason vocabulary and serialized shape.

## Acceptance Criteria

- A source that cannot Direct Play yields a stable, safe reason code.
- A source that can Remux but not Direct Play yields a stable, safe reason code.
- A source that must Transcode yields a stable, safe reason code.
- A denied playback request yields a stable, safe reason code.
- Responses do not expose FFmpeg command lines, host paths, Source Locators, bearer tokens, or provider payloads.

## Notes

- Prefer the existing capability planner and public playback decision path.
- Keep the first slice server/API first so the contract can stabilize before UI work.
