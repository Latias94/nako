# Android Client QA Harness

Status: Completed
Last updated: 2026-05-18

This workstream owns the Android client testing and emulator smoke harness that
lets parallel UI, playback, and Public Client API work prove basic client
health without relying on manual ad hoc commands.

Closed on 2026-05-18 after the local harness gained deterministic
`empty-setup` and `profile-missing-token` emulator smoke states with named
screenshots, UI hierarchy dumps, pass/fail criteria, and documented evidence
paths.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

Related baseline:

- `docs/workstreams/android-client-foundation/`
- `docs/workstreams/android-material-expressive-ui/`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
