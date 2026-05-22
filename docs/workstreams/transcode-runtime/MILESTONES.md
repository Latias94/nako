# Transcode Runtime Milestones

## M25.0: Transcode Runtime Design Baseline

Status: completed in [Phase 25.1](PHASE25_1_RUNTIME_PRODUCTIZATION.md).

Outcome: the runtime productization target is documented before code is moved.

Deliverables:

- Audit current playback, HLS, remux, staging, hardware selection, and
  transcode session surfaces.
- Decide the module split between `nako-server::app`, `nako-transcode`, and
  `nako-streaming`.
- Define the client-facing playback session lifecycle and stable error
  taxonomy.
- Record non-goals for adaptive bitrate, clients, distributed queues, and
  direct remote FFmpeg inputs.

Exit criteria:

- `git diff --check`

## M25.1: Playback Service Decomposition

Status: completed in [Phase 25.1](PHASE25_1_RUNTIME_PRODUCTIZATION.md).

Outcome: playback orchestration is split into focused modules without changing
public behavior.

Deliverables:

- Move direct-play planning helpers out of the largest playback app module.
- Move remux orchestration and HLS orchestration into separate internal
  services.
- Keep staging and persisted session coordination explicit.
- Preserve existing HTTP routes and API response shapes.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server`
- `git diff --check`

## M25.2: FFmpeg Hardware Capability Probe

Status: completed in [Phase 25.1](PHASE25_1_RUNTIME_PRODUCTIZATION.md).

Outcome: hardware acceleration is selected from real FFmpeg capability evidence
when configured.

Deliverables:

- Add an FFmpeg-backed hardware detector boundary.
- Detect VAAPI, NVENC, and QuickSync availability without requiring real GPU
  hardware in CI tests.
- Preserve CPU fallback and fail-fast policy behavior.
- Expose safe diagnostics that do not leak local paths or command internals
  beyond what operators need.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p nako-transcode`
- focused `cargo nextest run -p nako-server`
- `git diff --check`

## M25.3: Runtime Contracts and Stabilization

Status: completed in [Phase 25.1](PHASE25_1_RUNTIME_PRODUCTIZATION.md).

Outcome: transcode sessions have a clean service contract for future clients
and adaptive streaming work.

Deliverables:

- Document playback session lifecycle and error categories.
- Add focused tests for selected acceleration, fallback, budget selection,
  cancellation, timeout, and startup stale-session recovery.
- Remove temporary helpers left by the decomposition.
- Record validation evidence and remaining post-M25 work.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

## M26.0: Playback API Contract and Client Readiness

Status: completed in [Phase 26.0](PHASE26_0_PLAYBACK_CLIENT_CONTRACT.md).

Outcome: playback/session HTTP routes expose a stable control and inspection
contract that future web or Flutter clients can depend on.

Deliverables:

- Add a public playback session cancellation route.
- Wire cancellation to the live remux/HLS FFmpeg runner token, not only to the
  persisted session row.
- Keep session inspection on the existing `TranscodeSessionResponse` envelope.
- Document active and terminal session states, cancellation conflicts, and
  playback error DTO behavior.
- Add route-level tests for active cancellation, terminal cancellation
  conflicts, process-local stale active-session conflicts, session inspection,
  and HLS segment readiness/error behavior.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check -p nako-server --tests`
- focused `cargo nextest run -p nako-server` playback route tests
- `git diff --check`
