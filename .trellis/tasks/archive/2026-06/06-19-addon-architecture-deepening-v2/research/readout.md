# Addon architecture deepening research

Date: 2026-06-19

## Current readout

- `docs/workstreams/addon-architecture-deepening/README.md` and `HANDOFF.md` show the earlier lane is closed, but the open follow-on question is still depth, not direction.
- `crates/nako-server/src/app/addons.rs` is still the widest Addon server orchestrator in scope for the next pass.
- `crates/nako-server/src/http/addons.rs` is a large route translator with many Addon route families, but it is structurally thinner than the app orchestrator.
- `crates/nako-server/src/app/automation.rs` is broad too, but it is already tied to a different product lane and is not the best next move for the current Addon objective.
- The best next move is to deepen the Addon control-plane seam first, then use that structure to decide whether automation deserves its own follow-on later.

## Files used

- `docs/workstreams/addon-architecture-deepening/README.md`
- `docs/workstreams/addon-architecture-deepening/DESIGN.md`
- `docs/workstreams/addon-architecture-deepening/HANDOFF.md`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/addons/resource_search.rs`
- `crates/nako-server/src/app/addons/subtitles.rs`
- `crates/nako-server/src/app/addons/task_runtime.rs`
- `crates/nako-server/src/app/addons/runtime.rs`
