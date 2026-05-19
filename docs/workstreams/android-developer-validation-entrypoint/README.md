# Android Developer Validation Entrypoint

Status: Closed
Last updated: 2026-05-19

This workstream owns the developer-facing Android local validation command. It
wraps existing Gradle and smoke regression gates so future Android UI,
playback, and client API work has one clear handoff entrypoint.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

Primary command:

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1
```

No-emulator command:

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke
```

Closed on 2026-05-19 after both default and no-emulator commands passed.
