# Next Operations And Release Lanes

## Scope

This note audits the current release, deployment, remote-access, and
self-hosted operations surface to decide whether the next work should prioritize
operator readiness instead of more pure code refactoring.

Reviewed:

- `docs/deployment/`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `.github/workflows/*release*`, `docker-publish.yml`, `crates-publish.yml`
- `scripts/release-gate.*`, `scripts/package-release.*`
- `deploy/**`
- `docs/workstreams/self-hosted-release-readiness/`
- `docs/workstreams/release-packaging-and-distribution/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/mvp-release-shape/`
- `.trellis/tasks/06-05-remote-access-cookbook-config-gates/`
- parent audit research under
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/`

## Executive Answer

Yes. The next practical slice should prioritize operations/release readiness,
specifically the active `06-05-remote-access-cookbook-config-gates` task, before
starting another pure refactor campaign.

The repository already has meaningful release foundations: self-hosted docs,
backup/restore docs, package scripts, compose examples, container shape,
config-check, release gates, PostgreSQL harnesses, self-host smoke, and a
playback hardware report. The first operator-visible gap is not another
internal restructure. It is making remote access deployment concrete and
verifiable for self-hosted users without accidentally adding endpoint discovery
or a built-in tunnel runtime.

## Evidence Snapshot

### Release And Deployment Baseline

- `docs/deployment/SELF_HOSTED.md` covers SQLite/PostgreSQL setup, secrets,
  config-check, local-only default bind, reverse-proxy and tunnel config
  snippets, compose startup, auth, metadata providers, Addons/Webhooks,
  playback runtime, diagnostics, release gates, and the hardware report.
- `docs/deployment/RELEASE_CHECKLIST.md` covers artifact verification,
  crates.io publish readiness, Docker image publish shape, first start,
  compose start, diagnostics, playback release gate, official addon smoke,
  backup, upgrade, rollback, and support bundle boundaries.
- `docs/deployment/BACKUP_RESTORE_UPGRADE.md` correctly classifies durable
  state versus cache/rebuildable state and gives SQLite/PostgreSQL procedures.
- `docs/deployment/RELEASE_ARTIFACTS.md` defines archive, manifest, checksum,
  and operator verification expectations.
- `deploy/**` contains source and container configs plus compose stacks that
  bind to loopback at the host and run `config-check --create-dirs` before
  `serve`.

### Release Gates And Packaging

- `scripts/release-gate.*` support `docs`, `fast`, `db`, `api`, `playback`,
  `postgres`, `container`, `workspace`, and `all` modes.
- The `playback` gate checks FFmpeg/FFprobe, transcode/server tests, hardware
  tests, HLS tests, self-host smoke, and writes
  `target/release-gate/playback-hardware-report.json`.
- The `container` gate runs server config tests and `docker compose config` for
  SQLite/PostgreSQL Nako stacks.
- `.github/workflows/release-gate.yml` runs fast, PostgreSQL, self-host smoke,
  and API/SDK redaction jobs.
- `.github/workflows/release-package.yml` creates a Linux release archive and
  draft GitHub release from `v*` tags.
- `.github/workflows/docker-publish.yml` builds/smokes/publishes GHCR images
  from immutable release tags.
- `.github/workflows/crates-publish.yml` dry-runs and manually publishes only
  public permissive crates.

### Completed Workstream Evidence

- `self-hosted-release-readiness` is complete and proved local release gates,
  PostgreSQL contract harnesses, API/SDK/redaction gates, deployment examples,
  backup/restore docs, and self-host smoke.
- `release-packaging-and-distribution` is complete and proved package scripts,
  release manifests, checksums, container/compose shape, config preflight, and
  operator docs.
- `network-access-boundary` is complete and shipped policy/readiness foundation:
  exposure modes, trusted proxy sources, CORS/origin policy, tunnel provider
  declarations, request-time trusted header enforcement, and redacted Admin
  diagnostics. It explicitly deferred built-in NAT traversal, endpoint
  discovery, and tunnel runtime.
- `mvp-release-shape` closed with remote access marked as an included release
  capability through configuration/cookbook, excluding built-in tunnel
  ownership.

## Is `06-05-remote-access-cookbook-config-gates` Implementation-Ready?

Yes, with one small execution detail to choose at implementation time.

The task is ready because it has:

- clear goal: remote access cookbook plus config-check/release-gate fixtures;
- concrete requirements: Caddy, Nginx, DDNS, Tailscale Funnel, Cloudflare
  Tunnel, ngrok, generic external tunnel providers, playback ticket caveats,
  CORS/origin, HTTPS, trusted proxy, tunnel token caveats;
- explicit non-goals: no endpoint discovery, no LAN/remote client selection
  model, no built-in tunnel provider runtime, no Addon Manager;
- acceptance criteria for docs, at least one reverse-proxy fixture, at least
  one tunnel-provider fixture, redaction assertions, conservative defaults,
  and no new discovery/runtime routes;
- parent research links and spec files for cross-layer, server config, and
  Admin API checks;
- likely file ranges.

The implementation detail to choose:

- Option A: add static sample configs under `deploy/` and validate them through
  existing `nako-server config` tests plus release-gate docs/container mode.
- Option B: add a focused release-gate substep or fixture script that invokes
  `nako-server config-check --json` against reverse-proxy and tunnel-provider
  configs and asserts redaction.

Recommendation: choose Option B if the fixture is lightweight and
cross-platform; otherwise start with Option A and focused config tests. The
important contract is executable config-check evidence, not a new runtime.

## Highest-Value First Slice For Self-Hosted Operators

The first slice should be:

`remote-access-cookbook-config-gates`

Why this is the first operator slice:

- Most self-hosted users will try to access Nako outside `127.0.0.1` soon after
  first install.
- Nako already has policy/readiness enforcement, but the current deployment
  guide only has short snippets, not a real cookbook operators can follow.
- Remote exposure mistakes are security-sensitive: disabled auth, wildcard
  origins, untrusted forwarded headers, HTTP external URLs, leaked tunnel
  tokens, and proxy/tunnel URL leakage are higher-risk than many internal code
  cleanup items.
- The work is low-conflict if it stays in docs/config fixtures and does not
  alter Public Client contracts or tunnel runtime.
- It converts already-shipped network policy into release evidence operators
  can trust.

The next operations slice after that depends on the release goal:

- If the team wants actual alpha artifact publication, run a release-candidate
  gate wrapper/publication execution task.
- If the team wants hardware claims beyond CPU/default playback, run container
  device pass-through and optional one-frame GPU smoke evidence.
- If the team wants supportability polish, run read-only Admin network
  diagnostics drill-down after Admin/API contract scope is free.

## Alternatives Considered

### Option A: Remote Access Cookbook And Config Gates

Decision: recommended first.

Pros:

- Highest immediate self-hosted value.
- Low conflict when kept to docs, deploy fixtures, and release/config gates.
- Builds directly on shipped `network-access-boundary` work.
- Prevents unsafe remote exposure before endpoint discovery or tunnel runtime.

Cons:

- Does not add a new product runtime capability.
- Requires careful wording to avoid implying Nako owns cloudflared, Tailscale,
  ngrok, DNS, TLS certificates, or proxy lifecycle.

### Option B: Release-Candidate Wrapper And Publication Gate

Decision: second, only if an actual release execution is imminent.

Pros:

- Turns existing gates into a more repeatable release manager workflow.
- Useful before cutting/publishing an alpha artifact.

Cons:

- Less useful than remote-access guidance for daily operator success if no
  release is being published this week.
- Touches release scripts/workflows and should serialize with packaging work.

### Option C: Continue Pure Code Refactor

Decision: reject for the next operations slice.

Pros:

- Could reduce drift in duplicated network policy classification or release
  helper structure.

Cons:

- The current operator gap is not blocked by internal refactor.
- Network classifier extraction is only justified when new readiness states
  are added.
- Pure refactor gives self-hosted users no new install, exposure, or support
  confidence.

### Option D: Endpoint Discovery Or Built-In Tunnel Runtime

Decision: blocked pending architecture/product decision.

Pros:

- Could improve client convenience in the long run.

Cons:

- Crosses Public Client protocol, SDKs, auth, playback/cast ticket policy,
  Admin diagnostics, and possibly Addon Manager or sidecar packaging.
- Built-in tunnel runtime conflicts with the established decision that Nako
  should not own first-party NAT traversal in the first phase.

## Candidate Lane Classification

| Candidate | Classification | File Range | Validation Commands | Notes |
| --- | --- | --- | --- | --- |
| Remote access cookbook | Parallel | `docs/deployment/**`, maybe `deploy/**` examples | `git diff --check`; `scripts/release-gate.* --mode docs` | Safe if docs/examples only and no config structs or DTOs change. |
| Reverse-proxy/tunnel config fixtures | Serial-first inside operations-release | `deploy/**`, `scripts/release-gate.*`, maybe `crates/nako-server/src/config/**` tests | `cargo nextest run -p nako-server config --no-fail-fast`; `scripts/release-gate.* --mode docs`; `git diff --check` | Serialize with other release-gate or config-preflight work. |
| Release-candidate wrapper/publication execution | Serial-first | `scripts/package-release.*`, `.github/workflows/**`, `docs/deployment/**` | package `--dry-run`/`-WhatIf`; release gate `fast`, `container`, `postgres`, `playback`; `git diff --check` | Do only when a real candidate needs publishing. |
| Container hardware pass-through cookbook/matrix | Serial-first with playback/release | `docs/deployment/**`, `deploy/compose/**`, `scripts/release-gate.*`, maybe `nako-transcode` examples/tests | `scripts/release-gate.* --mode playback`; `scripts/release-gate.* --mode container`; optional host-specific GPU smoke | Not needed for default CPU release; needed before vendor GPU claims. |
| Admin network diagnostics drill-down | Serial with Admin/API | `crates/nako-api/src/admin/**`, `crates/nako-server/src/http/admin.rs`, generated Admin Web contract, `apps/admin-web/**` | `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `npm run generate:admin-api --prefix apps/admin-web`; `npm run check --prefix apps/admin-web`; server HTTP tests | Useful only if Admin contract scope is free. |
| Config hot-apply/restart-required model | Serial with Admin/settings/runtime | config structs, Admin settings routes, runtime orchestration docs | config tests, Admin contract tests, runtime smoke | Should not run concurrently with remote access config fixture edits if both touch `config.rs`/preflight. |
| Public Client endpoint discovery | Blocked | `nako-client-protocol`, `nako-client-core`, `nako-client`, `nako-api`, `nako-server`, SDKs, docs | architecture first; later OpenAPI/SDK/server/client gates | Needs product architecture before implementation. |
| Concrete tunnel runtime | Blocked | likely Addon Sidecar/Addon Manager/official catalog/control-plane docs | architecture/product decision first | Do not hide provider process supervision in `nako-server`. |

## Operations Work That Conflicts With API/Admin/Playback

### API And Public Client Conflicts

- Endpoint discovery implementation conflicts with Public Client route
  inventory, generated SDKs, `nako-client-core`, auth behavior, and remote
  playback/cast ticket semantics.
- Any route returning endpoint candidates can expose LAN names, private
  domains, or tunnel hosts unless a redaction/public-visibility contract is
  designed first.

### Admin Conflicts

- Admin network diagnostics drill-down touches Admin DTOs, generated TypeScript
  contracts, Admin Web settings, and redaction tests.
- Admin route/contract work should remain serialized. The completed Admin route
  inventory parity gate helps, but it does not remove the need for one owner
  when DTOs or generated contracts change.
- Config hot-apply/restart-required work conflicts with Admin settings API and
  runtime diagnostics because it changes operator-visible config semantics.

### Playback Conflicts

- Remote access cookbook must mention playback tickets and external base URL
  caveats, but should not change playback ticket validation or renderer/cast
  transport policy.
- Container GPU pass-through or one-frame hardware smoke conflicts with
  playback/transcode release work and should serialize with hardware/tone-map
  lanes.
- HEVC/AV1, tone mapping, subtitle burn-in, or output profile changes should
  not be mixed into operations-release docs/gates except as documented
  prerequisites or evidence.

### Release Script And Deployment Conflicts

- `scripts/release-gate.*`, `scripts/package-release.*`, `.github/workflows/*`,
  `deploy/**`, and `docs/deployment/**` should have a single operations owner
  when release-candidate execution or fixture gates are being changed.
- Remote access docs-only work can run in parallel with architecture audits,
  but config fixture gate changes should not overlap another release-gate edit.

## Success Metrics

| Metric | Current | Target For First Slice | Measurement |
| --- | --- | --- | --- |
| Remote access cookbook coverage | Short snippets in `SELF_HOSTED.md` | Dedicated cookbook sections for reverse proxy, DDNS, Tailscale Funnel, Cloudflare Tunnel, ngrok, and generic external tunnel | Docs review and topic inventory |
| Config fixture coverage | Unit/config tests exist; no release-gate fixture proving cookbook examples | At least one reverse-proxy and one tunnel-provider config-check fixture | Focused config tests or fixture script |
| Redaction confidence | Redaction tests and inventory exist | Fixture output proves no raw URLs, tokens, origins, forwarded headers, host details, or secrets leak | JSON assertion or grep-based negative checks |
| Conservative defaults | Existing examples bind local/private | Defaults remain loopback/private and auth enabled | TOML review and config-check |
| Runtime scope control | Tunnel runtime explicitly deferred | No endpoint discovery route and no built-in tunnel process supervision | Git diff scope review |

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Cookbook implies Nako manages third-party tunnel processes | High | Medium | State that providers are operator-run/external and Nako only validates declarations. |
| Fixture output leaks URLs, hostnames, origins, or tunnel tokens | High | Medium | Add negative assertions against raw sensitive values and use redacted diagnostics only. |
| Work expands into Public Client endpoint discovery | High | Medium | Keep endpoint discovery out of scope; open an architecture task first. |
| Release-gate edits conflict with packaging or playback gates | Medium | Medium | Serialize script edits under operations-release ownership. |
| Admin diagnostics drill-down conflicts with generated contracts | Medium | High | Defer until Admin/API owner scope is free. |
| GPU/container work makes false hardware guarantees | Medium | Medium | Keep default gate CPU-safe; document device pass-through as optional evidence. |

## Recommended Queue

1. Run `06-05-remote-access-cookbook-config-gates` now.
2. Keep its implementation limited to cookbook docs, deploy/config fixtures,
   and config-check/release-gate assertions.
3. Do not add endpoint discovery, LAN/remote client selection, or tunnel
   runtime.
4. After that, choose either:
   - release-candidate wrapper/publication execution, if an alpha artifact is
     being cut; or
   - container hardware pass-through evidence, if the release claims GPU
     acceleration beyond best-effort diagnostics; or
   - Admin network diagnostics drill-down, if Admin/API generated contract
     scope is free.

## Bottom Line

Nako's next self-hosted value is not another broad refactor. The release and
deployment foundation is already strong enough that the highest-return work is
operator-facing: a concrete remote access cookbook with executable, redaction-
safe config gates. Pure refactor should wait until a chosen operations/API/
playback lane creates real local pressure.
