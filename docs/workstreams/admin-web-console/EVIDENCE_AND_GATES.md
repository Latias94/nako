# Admin Web Console Evidence And Gates

Status: Proposed
Last updated: 2026-05-17

## Planning Gates

For documentation-only planning slices:

```bash
git diff --check
```

Review checks:

- `DESIGN.md` uses Taru domain language from `CONTEXT.md`.
- `V0_CONTEXT.md` does not choose a front-end framework.
- Admin console scope is not confused with the flagship playback client.
- Route families are described as product routes, not stable API guarantees.
- Public Client API and Admin API responsibilities remain separated.

## Future API Gates

When Admin API changes are introduced, expected validation should include
focused package checks and tests for touched crates, then broader gates if
route behavior or shared DTOs change.

Likely commands:

```bash
cargo fmt --all -- --check
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-api --no-fail-fast
cargo nextest run -p taru-server --no-fail-fast
git diff --check
```

Broaden to workspace gates when API ownership, route behavior, or shared
protocol crates are touched:

```bash
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
```

## Future Frontend Gates

The exact commands depend on the chosen web stack. The first real web
implementation should provide:

- type-check command;
- lint command if configured;
- unit/component test command if configured;
- browser smoke verification for primary routes;
- visual checks for desktop and mobile widths;
- redaction checks for secrets/tokens/local paths in mock fixtures and UI.

## Evidence Anchors

- `docs/workstreams/admin-web-console/DESIGN.md`
- `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
- `docs/workstreams/admin-web-console/V0_CONTEXT.md`
- future v0 prompt
- future web app scaffold and verification notes
