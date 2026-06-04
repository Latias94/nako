# Evidence

## 2026-06-05

- Task opened after `control-plane-observability-and-trace-context` remained a
  real follow-on and code inspection showed HTTP scan handlers still used the
  untraced enqueue path while the app layer already supported typed scan trace
  context.
- Verification:
  - `cargo fmt --all -- --check` ✓
  - `cargo check -p nako-server --tests` ✓
  - `cargo nextest run -p nako-server scan_route_queues_background_job --no-fail-fast` ✓
  - `cargo nextest run -p nako-server admin_scan_route_persists_safe_trace_context_without_exposing_input --no-fail-fast` ✓
  - `git diff --check` ✓
  - `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-06-05-http-library-scan-trace-context-first-slice` ✓
