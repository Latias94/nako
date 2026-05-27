# External Casting Adapter Boundary Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Evidence Log

### ECAB-010

Completed on 2026-05-27.

Evidence:

- opened `docs/workstreams/external-casting-adapter-boundary/`;
- added ADR 0042 for sidecar renderer adapters;
- linked the lane from ADR and workstream indexes;
- identified `ECAB-020` as the first executable task.

Gates:

```powershell
python -m json.tool docs/workstreams/external-casting-adapter-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/external-casting-adapter-boundary docs/adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md docs/adr/README.md docs/workstreams/README.md
```

Results:

- JSON parsed.
- Diff check passed.

### ECAB-020

Completed on 2026-05-27.

Evidence:

- strengthened Public renderer registration characterization so Chromecast,
  DLNA, and AirPlay targets all stay outside Public Client renderer
  registration;
- changed the rejection message to name the external adapter boundary;
- strengthened Admin renderer diagnostics redaction checks for renderer ticket
  query values and ticket prefixes.

Gates:

```powershell
cargo nextest run -p nako-server -E 'test(public_renderer_registration_rejects_external_cast_protocol_targets) | test(admin_v1_playback_renderers_reports_safe_diagnostics_and_adapter_readiness)' --no-fail-fast
cargo nextest run -p nako-server -E 'test(renderer) | test(admin_v1_playback_renderers)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- targeted renderer/admin characterization: 2 passed, 368 skipped.
- focused renderer/admin renderer gate: 12 passed, 358 skipped.
- `nako-api admin_contract`: 5 passed, 56 skipped.
- format check passed.
- diff check passed. Git printed CRLF conversion warnings only.

## Gate Policy

Use synthetic adapter tests before real LAN protocol tests. Physical receiver
availability, multicast support, and platform-specific AirPlay behavior must
not be required for ordinary CI gates.

## Focused Gates

```powershell
python -m json.tool docs/workstreams/external-casting-adapter-boundary/WORKSTREAM.json
cargo nextest run -p nako-server -E 'test(renderer_adapter) | test(renderer) | test(playback)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Redaction Gates

External adapter diagnostics and command payloads must not expose:

- bearer tokens;
- renderer transport ticket values outside intended media URLs;
- raw Source Locators;
- local filesystem paths;
- Transcode Session IDs as credentials;
- raw command payload JSON;
- protocol-private network addresses unless explicitly redacted or
  fingerprinted.

## Closeout Gates

```powershell
python -m json.tool docs/workstreams/external-casting-adapter-boundary/WORKSTREAM.json
cargo nextest run -p nako-server -E 'test(renderer_adapter) | test(renderer) | test(playback)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
