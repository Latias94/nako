# Addon Manager Lifecycle Automation - Evidence And Gates

Status: Completed
Last updated: 2026-05-23

## Required Gates

Baseline docs gate:

```bash
git diff --check
```

Rust gate, once manager code exists:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-server addon --no-fail-fast
```

Addon smoke gate, once a managed registry/plan slot exists:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | AMG-010 | Lane opened after the official addon alpha loop was proven with published Nako, Addon Protocol crates, GHCR server image, and the first official companion addon. | Pass |
| 2026-05-23 | AMG-010 | First manager slice narrowed to addon source discovery, operator-confirmed install/update/remove intent, permissions, token rotation, Addon Health Check visibility, and Addon Install Guide behavior; direct process/container supervision split to follow-on. | Pass |
| 2026-05-23 | AMG-020 | Added the first manager-owned registry/plan snapshot surface as `GET /admin/v1/addons/{addon_id}/manager-plan`, combining registration, health, tokens, grants, and install guide without process/container supervision. | Pass |
| 2026-05-23 | AMG-030 | Added operator-confirmed `POST /admin/v1/addons/{addon_id}/manager-plan` lifecycle intents for `install`, `update`, and `remove`; focused server addon tests plus `scripts/official-addon-e2e-smoke.ps1 -NakoImage nako:amg030` passed. | Pass |
| 2026-05-23 | AMG-060 | `cargo fmt --all -- --check`; `cargo nextest run -p nako-server addon --no-fail-fast` (55 passed, 217 skipped); `cargo check -p nako-api --tests`; `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1 -NakoImage nako:amg030`. | Pass |

## Closeout Evidence

Closeout completed with:

- a manager-owned registry/plan slot proof;
- fresh server and docs gates;
- explicit split/defer notes for marketplace, package signing, provider
  breadth, and process supervision.

Follow-ons:

- Addon source catalog / marketplace hosting;
- package signing and trust roots;
- provider breadth beyond the first official companion addon;
- rollback and update-policy execution beyond the operator-confirmed plan slot;
- direct process/container supervision.
