# Nako Rebrand Baseline — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Planned Gates

Run the strongest feasible gates after each major phase:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
npm run check
npm test
npm run build
git diff --check
old-name grep gate over tracked source, excluding build output
```

Android gates are desirable after the package namespace move:

```powershell
./gradlew testDebugUnitTest
./gradlew assembleDebug
```

Official addon gates after dependency update:

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo check --workspace
```

## Evidence Log

| Time | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-22 | Initial scan | PASS | Repository had widespread old-name residue across docs, crates, Android, deploy, and SDK paths. |
| 2026-05-22 | cargo fmt --all | PASS | Workspace formatted after crate/package rename. |
| 2026-05-22 | cargo check --workspace --tests | PASS | All renamed Rust crates compile under `nako-*` package names. |
| 2026-05-22 | cargo nextest run --workspace --no-fail-fast | PASS | 691 tests passed, 30 skipped. |
| 2026-05-22 | npm run generate --prefix sdk/typescript | PASS | TypeScript SDK regenerated from `nako-api`. |
| 2026-05-22 | npm run generate:admin-api --prefix apps/admin-web | PASS | Admin Web contract regenerated from `nako-api`. |
| 2026-05-22 | cargo run -q -p nako-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt | PASS | Kotlin SDK regenerated under `dev.nako.sdk`. |
| 2026-05-22 | npm run check; npm test; npm run build in apps/admin-web | PASS | 22 Vitest tests passed and Vite production build succeeded. |
| 2026-05-22 | npm run check --prefix sdk/typescript | PASS | Generated TypeScript SDK type-checks. |
| 2026-05-22 | Android Gradle unit-test gate | BLOCKED | `:app:testDebugUnitTest` and `:nako-public-client-sdk:test` fail during Gradle test task creation with `Type T not present` on local JDK/Gradle setup before tests execute. |
| 2026-05-22 | Official addon cargo fmt/test/check | PASS | `nako-official-addons` updated to `../nako/crates/nako-addon-protocol`; 4 tests passed. |
| 2026-05-22 | old-name grep gate | PASS | No active source-tree old-name matches outside Git history/build outputs. |
| 2026-05-22 | git diff --check | PASS_WITH_WARNINGS | No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings. |

## Residual Old-Name Policy

Allowed only when explicitly reviewed:

- immutable Git history outside tracked files,
- external URLs that cannot yet be renamed,
- third-party references unrelated to the product name.

Everything else should move to Nako in this lane.
