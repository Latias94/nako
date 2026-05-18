# Addon Protected Writes Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs
git diff --check
```

This proves the current protected-write inventory is fresh before concrete
metadata, artwork, subtitle, NFO, or Library File Write handlers are added.

## Gate Set

### Audit Gate

```powershell
rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs
git diff --check
```

Proves APW-020 has current file anchors for the existing intake and write
boundaries.

### Canonical Metadata Apply Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Add focused metadata/catalog tests selected by APW-020. The gate must prove
that a valid `metadata_write` side effect applies through Taru-owned metadata
and catalog seams, and that denied or replayed requests remain safe.

### Artwork And Artifact Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests
git diff --check
```

Add focused artwork/addon tests after APW-020 identifies the concrete model.
The gate must prove artwork/artifact outputs do not become raw provider hotlinks
or path leaks.

### Library File Write Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests
cargo fmt --all -- --check
git diff --check
```

Add focused NFO/storage/addon tests after APW-020 identifies the concrete
write path. The gate must prove Library File Write behavior remains Taru-owned,
redacted, and compatible with NFO Round Trip and backup policy.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to `cargo check --workspace --tests` and `cargo nextest run --workspace
--no-fail-fast` if protected-write changes affect shared repository, catalog,
metadata, storage, or API boundaries across the workspace.

### Review Gate

Run `review-workstream` before accepting APW-020, before accepting any concrete
protected-write apply task, and before lane closeout. Record blocking findings,
missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/addon-protected-writes/DESIGN.md`
- `docs/workstreams/addon-protected-writes/TODO.md`
- `docs/workstreams/addon-protected-writes/MILESTONES.md`
- `docs/workstreams/addon-token-grants-side-effects/EVIDENCE_AND_GATES.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `crates/taru-core/src/addon.rs`
- `crates/taru-core/src/repository/addon.rs`
- `crates/taru-db/src/addons.rs`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-api/src/extension.rs`
- code/test paths proving concrete protected-write behavior after
  implementation

## Fresh Evidence

2026-05-18, APW-010:

- Workstream opened from ATGSE-060 closeout as the follow-on for concrete
  Addon protected writes.
- Scope is intentionally after Addon Token, accepted grant, addon-principal,
  and Addon Side Effect intake proof.
- First executable task set to APW-020 protected write seam audit before
  changing metadata, artwork, subtitle, NFO, or Library File Write behavior.
- Workstream index and ATGSE handoff point to this lane.
- Validation is recorded in ATGSE-060 closeout evidence.

Fresh verification is required before marking any later task, Codex goal, or
lane complete.
