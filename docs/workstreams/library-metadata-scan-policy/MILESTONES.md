# Library Metadata Scan Policy - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Frozen

- Workstream docs exist and use Nako domain language.
- First execution slice is NFO-only and does not hide provider/addon breadth.

## M1 - Plan Model Exists

- `MetadataProfile` or library config can express scan-time metadata
  acquisition behavior.
- Defaults preserve current preset intent: local-first NFO-enabled video
  libraries should import NFO during scan unless disabled.
- Disabled local metadata does not schedule NFO import.

## M2 - Scan Executes Local Metadata Step

- Scan command/job output includes metadata acquisition evidence.
- NFO Import runs after source index and probe.
- Existing manual NFO import/export commands still work.

## M3 - Real Smoke Proven

- Local test directory scan imports NFO metadata without a separate command.
- NAS single-directory SMB scan and Range streaming still pass.
- Full NAS root is explicitly deferred unless progress and cancellation
  diagnostics are ready.

## M4 - Closeout

- Focused tests, formatting, and smoke evidence are fresh.
- Follow-ons are split for provider/addon metadata, embedded readers, artwork,
  and large-library progress controls.

Status: Complete on 2026-05-25.
