# brainstorm: implement next media-server roadmap slice

## Goal

Implement the next media-server maturity roadmap slice after U1: U2 Device Capability Profiles And Playback Reasons. The goal is to make playback decisions explainable from explicit client/device facts across browser, Android, renderer, and future TV clients, while preserving Nako's existing playback/transcode boundaries and redaction rules.

## What I already know

* U1 Product-Operator readiness is complete and committed as `c474bc16`.
* The roadmap plan names U2 as the next playback maturity slice: client capability profiles must carry codec, container, subtitle, HDR, audio, network, and renderer facts into Playback Source Selection.
* Existing playback architecture already has `ClientPlaybackCapabilities`, `PlaybackTargetProfile`, capability evaluations, Direct Play/Remux/Transcode/Denied decisions, and safe planner reasons.
* Browser routes currently map flat capability query/DTO fields into `ClientPlaybackCapabilities`.
* `docs/architecture/PLAYBACK.md` already identifies Lane A - Device Capability Profiles as the direct next lane.
* ADR 0038 and ADR 0044 are the primary playback planner and capability-profile constraints.

## Assumptions (temporary)

* The first U2 implementation slice should deepen the existing capability/reason contract rather than replace the planner.
* The MVP avoids Android and UI work so the server/public DTO shape can stabilize first.
* Compatibility reasons should be stable enum/string codes safe for Public Client and Admin surfaces, not raw FFmpeg command details, local paths, Source Locators, or provider payloads.
* Renderer-specific profile work can be modeled in the contract even if full Chromecast/DLNA/AirPlay adapters remain later.

## Decisions

* MVP scope is Server/API first: implement planner, public/admin contract, and playback route tests before Android or UI work.

## Open Questions

* None blocking for the first implementation slice.

## Requirements (evolving)

* Add or refine a capability-profile contract that carries enough client facts for source selection: container, video codec, audio codec, subtitle capability, HDR capability, bitrate/resolution/channel bounds, and HLS/remux output preferences.
* Make Direct Play, Remux, Transcode, and Denied outcomes explainable through redaction-safe reason vocabulary.
* Keep playback planning in `nako-playback`; keep FFmpeg command planning/runtime in `nako-transcode`; keep HTTP route mapping in `nako-server`.
* Preserve Public Client contract stability and generated SDK expectations.
* Update tests across planner, HTTP route contract, and any touched client adapter.

## Acceptance Criteria (evolving)

* [x] A browser/client capability profile that lacks the source video codec produces a Transcode decision with a stable safe reason.
* [x] A client that supports the source container but not the audio codec produces a Remux or audio Transcode decision according to current policy, with a stable safe reason.
* [x] A selected subtitle that the client cannot render produces an explicit sidecar/burn-in/denied reason according to current policy.
* [x] Public Client/Admin-facing responses expose reason codes but not FFmpeg command lines, host paths, Source Locators, bearer tokens, or raw device secrets.
* [x] Existing flat capability query compatibility is preserved or migrated with tests.

## Definition of Done (team quality bar)

* Tests added/updated for planner capability evaluation and HTTP route contract.
* Generated Public/Admin TypeScript contracts updated when DTOs change.
* `cargo fmt --all` and focused `cargo nextest run` gates pass.
* Frontend/client checks pass if generated contracts or UI are touched.
* Architecture docs or ADR references updated if public DTO or playback policy semantics change.

## Out of Scope (explicit)

* New FFmpeg runtime execution behavior.
* LL-HLS/CMAF, remote workers, or durable playback queueing.
* Broad TV client implementation.
* Android capability mapping; this remains a follow-up after the server/public DTO shape stabilizes.
* Admin Web or client playback explainability UI; this remains a follow-up after reason codes are available.
* Plex-style cloud relay, accounts, or premium gating.
* Copying Jellyfin source, schemas, comments, tests, assets, or generated code.

## Technical Notes

* Roadmap source: `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`, U2.
* Playback architecture source: `docs/architecture/PLAYBACK.md`, especially Lane A - Device Capability Profiles.
* Primary likely files: `crates/nako-playback/src/capability.rs`, `crates/nako-playback/src/lib.rs`, `crates/nako-api/src/public_client.rs`, `crates/nako-client-protocol/src`, `crates/nako-server/src/http/playback.rs`, `crates/nako-server/src/app/playback/selection.rs`, and playback HTTP tests.
* Existing public route capability mapping is in `crates/nako-server/src/http/playback.rs`.
* Existing planner capability model is in `crates/nako-playback/src/capability.rs`.

## Research References

* [`docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`](../../../docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md) - U2 scope and test scenarios.
* [`docs/architecture/PLAYBACK.md`](../../../docs/architecture/PLAYBACK.md) - playback capability progress matrix and Lane A.
* [`docs/adr/0038-playback-planning-and-transcode-policy-seams.md`](../../../docs/adr/0038-playback-planning-and-transcode-policy-seams.md) - planner and transcode policy boundaries.
* [`docs/adr/0044-playback-capability-profile-planner.md`](../../../docs/adr/0044-playback-capability-profile-planner.md) - capability profile planner baseline.
