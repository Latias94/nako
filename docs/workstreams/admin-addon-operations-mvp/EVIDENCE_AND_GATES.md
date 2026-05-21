# Admin Addon Operations MVP — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Preferred Gates

```text
cargo fmt --all -- --check
cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p taru-core -p taru-db -p taru-server --tests
cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast
cargo nextest run -p taru-db addon --no-fail-fast
cargo nextest run -p taru-server addons --no-fail-fast
git diff --check
```

When `TARU_TEST_POSTGRES_URL` is available:

```text
cargo nextest run -p taru-db addon --run-ignored ignored-only --no-fail-fast
```

Closeout should add workspace gates when practical:

```text
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
```

## Evidence Ledger

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | AAO-000 workstream open | Created `docs/workstreams/admin-addon-operations-mvp/` after release packaging and Addon architecture deepening completed. | Pending validation |

## Redaction Gates

Admin Addon operations must not expose:

- raw Addon Token values or token hashes;
- administrator bearer tokens;
- provider credentials or resolved Secret Reference values;
- raw diagnostic request/response bodies;
- raw Addon Side Effect payload/provenance in Admin summaries;
- Source Locators;
- storage URIs;
- local filesystem paths;
- backup URIs;
- raw remote handles;
- raw network errors that include credentials.
