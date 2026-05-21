# Generated SDK Runtime Ownership — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Runtime Ownership Freeze

Status: Complete on 2026-05-21.

Exit when `SDKRT-010` records:

- current SDK/runtime/app responsibility inventory;
- frozen ownership matrix;
- selected option among Android-owned, Kotlin SDK/runtime, split runtime
  package, or early shared Rust client core / UniFFI target state;
- ADR impact;
- first implementation or closeout path.

Result: `SDKRT-010` selected early shared Rust client core / UniFFI target state
with app-supplied Android transport. ADR 0031's mobile-FFI sequencing must be
superseded or amended by `SDKRT-020` before implementation.

## M1 — Runtime Contract Decision

Status: Complete on 2026-05-21.

Exit when the chosen runtime boundary has an explicit API shape or an explicit
no-code closeout decision.

Acceptance criteria:

- generated DTO/request surfaces remain OpenAPI-backed, or Rust core consumes
  equivalent Public Client API contract authority through `taru-client` /
  `taru-client-protocol`;
- app/product policy remains outside SDK runtime;
- if Rust core is selected, FFI-safe data shapes, transport ownership, and
  Android build topology are named before code;
- compatibility and publishing implications are recorded;
- the first tracer is small enough to review independently.

Result: ADR 0032 now owns the target state. The first tracer is a no-socket
Rust client core with app-supplied Android transport for connection health plus
auth probe. `SDKRT-030` may implement the Rust-side core tracer only.

## M2 — Small Runtime/Core Tracer

Exit only if the lane chooses implementation. The tracer must prove
protocol-level runtime semantics without broad migration.

Acceptance criteria:

- public error-envelope decode uses generated SDK surfaces where practical;
- API-version header observation is consistent with ADR 0025/0031;
- request previews redact bearer tokens and provided secrets;
- transport remains supplied by the platform app or adapter;
- if Rust core is selected, strict Rust protocol enum decode does not regress
  the forward-compatibility tolerance already achieved for Kotlin;
- SDK tests cover success, HTTP error, invalid JSON, unsupported API version,
  and redaction behavior.

## M3 — Android Consumption Tracer

Exit when one Android flow consumes the runtime tracer without moving product
diagnostics into SDK code.

Acceptance criteria:

- Android connection/playback categories and user messages remain app-owned;
- cleartext policy and token storage remain Android-owned;
- focused Android tests prove existing diagnostics and safe request previews;
- no Compose, navigation, or Media3 ownership change appears in the diff.

## M4 — Broaden Or Split

Exit when the lane either broadens the proven runtime seam across repeated
families with fresh gates, or splits follow-ons and closes.

Acceptance criteria:

- each broadened route family has focused evidence;
- broadening removes duplication rather than hiding platform policy;
- publishing, KMP, full-platform Rust/UniFFI migration, and multi-SDK runtime
  work remain separate.

## M5 — Closeout

Exit when:

- final evidence gates are recorded;
- `TODO.md` task statuses match reality;
- `WORKSTREAM.json` status and current task are updated;
- closeout notes record residual risks and follow-ons;
- workstream review has no blocking findings.
