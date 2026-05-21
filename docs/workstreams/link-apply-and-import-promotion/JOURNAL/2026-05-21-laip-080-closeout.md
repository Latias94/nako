# LAIP-080 Journal — Closeout

Date: 2026-05-21
Task: LAIP-080
Status: Complete

## Closeout Claim

`link-apply-and-import-promotion` is complete as the mutating Managed Import
promotion lane.

## Shipped Boundary

The lane now proves:

- durable promotion acceptance/apply/audit persistence;
- app-service acceptance and idempotent replay;
- VFS-mediated copy/hardlink/symlink target creation;
- server-side plan revalidation before mutation;
- catalog commits only after target creation;
- duplicate relationship persistence from accepted promotion evidence;
- cleanup-complete or cleanup-pending audit after catalog failure following
  target creation;
- no NFO sidecar mutation hidden inside Managed Import promotion.

## Follow-Ons

- `nfo-sidecar-promotion-apply` owns accepted NFO sidecar import/export
  mutation.
- Move/delete source behavior remains deferred until source-retention and
  rollback semantics are explicitly designed.
- Downloader/watch-folder acquisition remains downstream of staged artifact
  intake and accepted promotion apply.

## Verification

Fresh closeout gates passed on 2026-05-21:

- `cargo fmt --all -- --check`
- `cargo nextest run -p taru-db promotion_apply --no-fail-fast`
- `cargo nextest run -p taru-vfs cleanup --no-fail-fast`
- `cargo nextest run -p taru-server managed_import --no-fail-fast`
- `cargo nextest run -p taru-vfs link --no-fail-fast`
- `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`
- `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`
- `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`
- `git diff --check`

## Handoff

Return to `post-rpd-product-hardening` for PRPH-080 lane scoring.
