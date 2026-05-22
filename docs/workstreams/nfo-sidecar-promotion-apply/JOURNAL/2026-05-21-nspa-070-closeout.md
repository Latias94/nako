# 2026-05-21 — NSPA-070 Closeout

## Closeout Claim

NFO Sidecar Promotion Apply is complete.

The lane now has:

- durable accepted sidecar apply records and idempotent replay;
- export apply through `taru-nfo` round-trip preservation and VFS storage write
  policy;
- import apply through canonical metadata, local authority, field locks, and
  hierarchy confirmation;
- VFS backup restore and server rollback orchestration;
- terminal `Committed`, `FailedBeforeMutation`, `RollbackComplete`, and
  `RepairPending` outcomes with redacted diagnostics;
- focused and broader gates recorded in `EVIDENCE_AND_GATES.md`.

## Split Decisions

- Admin API/UI exposure is a follow-on consumer of the app-service apply
  boundary.
- Public Client API must not expose raw sidecar paths or direct sidecar writes.
- Addons may request sidecar effects only through scoped Taru-owned apply
  commands.
- Downloads/watch-folder acquisition must produce staged artifacts and consume
  Managed Import promotion plus NFO sidecar apply; it must not write sidecars or
  catalog state directly.

## Parent Handoff

Return to `post-rpd-product-hardening` PRPH-090. With local metadata, sidecar,
file-write, import, rollback, and repair boundaries now proven, the next
recommended mainline lane is Playback/Transcode Ops Hardening. Downloads/watch
folder, network, AI, and addon runtime remain downstream or parallel only if
they consume existing accepted boundaries.
