# Post-RPD Product Hardening — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Gate Set

This umbrella is documentation-only. Code gates belong to execution
workstreams.

### Roadmap Gate

```powershell
git diff --check
```

### First Execution Lane Gate

```powershell
cargo fmt --all -- --check
cargo nextest run -p taru-metadata --no-fail-fast
cargo nextest run -p taru-server metadata --no-fail-fast
git diff --check
```

The exact closeout gate for metadata breadth is authoritative in
`docs/workstreams/metadata-provider-breadth/EVIDENCE_AND_GATES.md`.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | PRPH-010 | Reviewed `CONTEXT.md`, RPD closeout docs, metadata/NFO/playback/auth/addon ADRs, and existing workstreams | Pass. Metadata provider breadth is the next highest-leverage lane after packaging. |
| 2026-05-21 | PRPH-020 | `docs/workstreams/metadata-provider-breadth` | Pass. First execution lane opened with a capability/conflict/matching-first plan. |
| 2026-05-21 | PRPH-030 | Reviewed `metadata-provider-breadth` closeout docs and re-scored NFO/link, playback/transcode, managed import, network, AI, and addon lanes in `DESIGN.md` | Pass. `nfo-link-authority` is the next mainline execution lane; playback/transcode ops is the safest parallel sidecar candidate. |
| 2026-05-21 | child closeout | `cargo check --workspace --tests`; `cargo nextest run -p taru-api --no-fail-fast`; `cargo nextest run -p taru-metadata --no-fail-fast`; `cargo nextest run -p taru-server metadata --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check` | Pass. Authoritative command evidence is recorded in `docs/workstreams/metadata-provider-breadth/EVIDENCE_AND_GATES.md`. |
| 2026-05-21 | PRPH-040 refresh | `docs/workstreams/nfo-link-authority` | Pass. Next mainline execution lane opened with LNA-020 as the first non-destructive VFS link planning slice. |
| 2026-05-21 | PRPH-040 progress | `docs/workstreams/nfo-link-authority/EVIDENCE_AND_GATES.md` | Pass. NFO/link authority completed VFS link dry-run diagnostics and Source Duplicate Relationship filesystem-link suggestions; next executable task is LNA-040 NFO authority preview. |
| 2026-05-21 | PRPH-040 progress | `docs/workstreams/nfo-link-authority/EVIDENCE_AND_GATES.md` | Pass. NFO/link authority completed non-mutating NFO authority preview; next executable task is LNA-050 link apply split decision. |
| 2026-05-21 | PRPH-050 child closeout | `docs/workstreams/nfo-link-authority` | Pass. NFO/link authority is complete. Actual hardlink/symlink apply is split to a follow-on after managed import staging; next recommended mainline is `managed-import-staging`. |
| 2026-05-21 | PRPH-060 lane open | `docs/workstreams/managed-import-staging` | Pass. Managed Import Staging opened with MIS-020 as first executable durable domain/schema slice; generic downloader protocols and promotion apply remain out of first slice. |
| 2026-05-21 | PRPH-070 child closeout and next lane | `docs/workstreams/managed-import-staging`; `docs/workstreams/link-apply-and-import-promotion` | Pass. Managed Import Staging is complete as a non-mutating staging/preview lane; `link-apply-and-import-promotion` is opened and has completed LAIP-020 durable acceptance/audit. Next mainline task is LAIP-030 app-service acceptance/replay. |
| 2026-05-21 | PRPH-070 refresh | `docs/workstreams/link-apply-and-import-promotion`; `docs/workstreams/nfo-sidecar-promotion-apply` | Pass. LAIP has progressed through accepted promotion apply, VFS-mediated target creation, catalog commit ordering, duplicate evidence, and cleanup-complete/cleanup-pending audit. LAIP-070 split NFO sidecar import/export mutation to a dedicated accepted Library File Write / metadata-authority lane. Next mainline task is LAIP-080 closeout, followed by umbrella re-scoring. |
| 2026-05-21 | PRPH-070 planning verification | `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Umbrella and child workstream JSON files are valid and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | PRPH-070 child closeout refresh | `docs/workstreams/link-apply-and-import-promotion/EVIDENCE_AND_GATES.md`; `docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json` | Pass. `link-apply-and-import-promotion` is complete. Next umbrella task is PRPH-080 lane scoring across NFO sidecar apply, playback/transcode ops, network, AI, addon runtime, and downloads/watch-folder. |
| 2026-05-21 | PRPH-080 lane scoring | `docs/workstreams/post-rpd-product-hardening/DESIGN.md`; `docs/workstreams/nfo-sidecar-promotion-apply/HANDOFF.md`; `docs/workstreams/link-apply-and-import-promotion/EVIDENCE_AND_GATES.md` | Pass. NFO sidecar apply is selected as the next mainline lane because it is the remaining high-risk local Library File Write and metadata-authority boundary. Playback/transcode ops remains the safest parallel sidecar; downloads/watch-folder, network, AI, and addon runtime remain downstream. |
| 2026-05-21 | PRPH-080 planning verification | `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `python -m json.tool docs/workstreams/link-apply-and-import-promotion/WORKSTREAM.json`; `git diff --check` | Pass. Umbrella and child workstream JSON files are valid and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-21 | PRPH-090 lane scoring | `docs/workstreams/post-rpd-product-hardening/DESIGN.md`; `docs/workstreams/nfo-sidecar-promotion-apply/EVIDENCE_AND_GATES.md`; `docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json` | Pass. NFO Sidecar Promotion Apply is complete. Playback/Transcode Ops Hardening is selected as the next mainline lane because local metadata, file-write, staged import, NFO sidecar, rollback, and repair boundaries are now proven. Downloads/watch-folder, network, AI, and addon runtime remain downstream or parallel only if they consume existing accepted boundaries. |
| 2026-05-21 | PRPH-090 planning verification | `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/nfo-sidecar-promotion-apply/WORKSTREAM.json`; `git diff --check` | Pass. Umbrella and NFO sidecar workstream JSON files are valid and diff hygiene passed. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PRPH-100 lane open | `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Playback/Transcode Ops Hardening is opened as the active mainline lane; PTOH-020 is the next executable task. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PRPH-110 lane scoring | `docs/workstreams/playback-transcode-ops-hardening/EVIDENCE_AND_GATES.md`; `docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `docs/workstreams/post-rpd-product-hardening/DESIGN.md` | Pass. Playback/Transcode Ops Hardening is complete. Downloads/watch-folder intake is selected as the next mainline lane, scoped to staged artifact acquisition and existing promotion/apply boundaries. Network remains the best sidecar; AI and Addon runtime remain downstream. |
| 2026-05-22 | PRPH-120 lane open | `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Downloads / Watch-Folder Intake is opened as the active mainline lane; DWI-020 is the next executable task. `git diff --check` emitted only repository CRLF conversion warnings. |

## Notes

Fresh verification is required before marking any child execution lane complete.
Do not use this umbrella as evidence that code behavior shipped.
