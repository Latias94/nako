# Transcode Runtime Milestones

## M25.0: Transcode Runtime Design Baseline

Outcome: the runtime productization target is documented before code is moved.

Deliverables:

- Audit current playback, HLS, remux, staging, hardware selection, and
  transcode session surfaces.
- Decide the module split between `taru-server::app`, `taru-transcode`, and
  `taru-streaming`.
- Define the client-facing playback session lifecycle and stable error
  taxonomy.
- Record non-goals for adaptive bitrate, clients, distributed queues, and
  direct remote FFmpeg inputs.

Exit criteria:

- `git diff --check`

## M25.1: Playback Service Decomposition

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
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server`
- `git diff --check`

## M25.2: FFmpeg Hardware Capability Probe

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
- focused `cargo nextest run -p taru-transcode`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

## M25.3: Runtime Contracts and Stabilization

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
