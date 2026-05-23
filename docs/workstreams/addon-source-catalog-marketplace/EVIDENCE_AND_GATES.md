# Addon Source Catalog And Marketplace - Evidence And Gates

Status: Completed
Last updated: 2026-05-24

## Closeout Gates

Baseline docs gate:

```bash
git diff --check
```

Rust gates:

```bash
cargo fmt --all -- --check
cargo test -p nako-api admin_contract_includes_route_constants --no-default-features
cargo nextest run -p nako-server admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work --no-fail-fast
cargo nextest run -p nako-server addon --no-fail-fast
```

Related existing official-addon hosted smoke:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

The official smoke remains the hosted sidecar health/resource diagnostic gate
for the existing companion addon lane. It is not a closeout gate for this
catalog slice because catalog discovery is read-only and does not start the
external official addon. The catalog discovery behavior is covered by the
focused Admin route test above.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | ASCM-010 | Lane opened after the Addon Manager lifecycle lane closed with a proven manager-owned registry/plan slot. | Pass |
| 2026-05-23 | ASCM-010 | Reference study completed against Jellyfin plugin repositories, Home Assistant app repositories, Visual Studio Code extension manifests, and Obsidian plugin manifests/version gating. | Pass |
| 2026-05-24 | ASCM-010 | Frozen the first slice as read-only source listing, browse metadata, and descriptor resolution. Package signing, provider breadth, rollback/update execution, authenticated outbound task credentials, official-addon task-path smoke, and process supervision remain follow-ons. | Pass |
| 2026-05-24 | ASCM-020/030 | Added Admin DTOs and routes for catalog sources, entries, and resolution; reused `AddonInstallDescriptor` / `AddonInstallGuide` and proved no hidden addon registration, routing-plan, job, manager intent, package signing, or process supervision side effects. | Pass |
| 2026-05-24 | ASCM-020/030 | `cargo nextest run -p nako-server admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work --no-fail-fast`; `cargo test -p nako-api admin_contract_includes_route_constants --no-default-features`. | Pass |
| 2026-05-24 | ASCM-060 | `cargo fmt --all -- --check`; `git diff --check`; `cargo check -p nako-api -p nako-server`; `cargo nextest run -p nako-server addon --no-fail-fast`. | Pass, addon gate ran 62 tests |
| 2026-05-24 | ASCM-060 | Final closeout rerun after gate-scope correction: `cargo fmt --all -- --check`; `git diff --check`; WORKSTREAM.json parse; `cargo test -p nako-api admin_contract_includes_route_constants --no-default-features`; `cargo nextest run -p nako-server admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work --no-fail-fast`; `cargo nextest run -p nako-server addon --no-fail-fast`. | Pass, focused catalog test passed and addon gate ran 62 tests |
| 2026-05-24 | ASCM-060 | Closed the lane and split remaining addon ecosystem breadth into explicit follow-ons. | Pass |

## Closeout Evidence

Closeout completed with:

- a clear source catalog / marketplace boundary;
- fresh docs and runtime gates;
- explicit split/defer notes for package signing, provider breadth,
  rollback/update execution, authenticated outbound task-dispatch credentials,
  official-addon task-path smoke, and process supervision.
