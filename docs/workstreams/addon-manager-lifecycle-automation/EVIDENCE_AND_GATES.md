# Addon Manager Lifecycle Automation - Evidence And Gates

Status: Active
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

Addon smoke gate, once a managed lifecycle slot exists:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | AMG-010 | Lane opened after the official addon alpha loop was proven with published Nako, Addon Protocol crates, GHCR server image, and the first official companion addon. | Pass |

## Closeout Evidence

Closeout requires:

- a manager-owned lifecycle slot proof;
- fresh server and docs gates;
- explicit split/defer notes for marketplace and package signing.
