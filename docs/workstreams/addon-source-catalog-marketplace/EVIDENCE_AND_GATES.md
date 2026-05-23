# Addon Source Catalog And Marketplace - Evidence And Gates

Status: Active
Last updated: 2026-05-23

## Required Gates

Baseline docs gate:

```bash
git diff --check
```

Future Rust gate, once catalog code exists:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-server addon --no-fail-fast
```

Future discovery smoke, once a catalog surface exists:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | ASCM-010 | Lane opened after the Addon Manager lifecycle lane closed with a proven manager-owned registry/plan slot. | Pass |

## Closeout Evidence

Closeout requires:

- a clear source catalog / marketplace boundary;
- fresh docs and runtime gates;
- explicit split/defer notes for package signing, provider breadth, and
  process supervision.
