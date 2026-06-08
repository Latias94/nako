# Evidence

## Implementation

- Added generated Admin route key `managedArtworkCandidateAccept` for
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- Removed only `artwork/candidates/{candidate_id}/accept` from the explicit
  Admin route exclusion list.
- Emitted `AcceptManagedArtworkCandidateResponse` and `JobResponse` in the
  generated Admin TypeScript contract body.
- Regenerated:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Added `AdminApiClient.acceptManagedArtworkCandidate(candidateId)` using the
  generated route key, encoded `candidate_id`, and `POST {}`.
- Added a focused Admin Web client test covering generated route usage, path
  parameter encoding, empty body, response typing, and unsafe fixture terms.

## Jellyfin Comparison

- Jellyfin exposes remote image download as an elevated item-scoped POST action.
- Nako keeps the same explicit operator selection shape but narrows the client
  input to an opaque candidate ID and queues Managed Artwork ingest instead of
  directly publishing public artwork.

## Verification

- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passed
  with 32 tests.
- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 8
  tests.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  passed with 1 test.
- `cargo nextest run -p nako-server admin_accept_artwork_candidate --no-fail-fast`
  passed with 1 test.

## Spec Updates

- Updated `.trellis/spec/nako-api/backend/quality-guidelines.md` with the
  generated Managed Artwork candidate accept Admin contract and the current
  remaining route exclusions.
- Updated `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` with the
  Managed Artwork candidate accept client command contract.
