# Research: playback-transcode next candidates

- Query: 为长期 Goal 选择 playback/transcode lane 的下一个高杠杆 bounded implementation task
- Scope: mixed
- Date: 2026-06-03

## Findings

### Files found

- `docs/ARCHITECTURE.md` - 总体系统图，确认 playback/transcode lane 仍是明确的能力域。
- `docs/architecture/PLAYBACK.md` - 播放/转码进度地图，列出当前已闭合与仍待拆分的 follow-on。
- `docs/architecture/LANES.md` - lane 级队列与当前候选动作。
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md` - playback planning / transcode policy seam。
- `docs/adr/0044-playback-capability-profile-planner.md` - profile-driven playback capability planner。
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md` - FFmpeg hardware pipeline planner。
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md` - FFmpeg CLI-first HLS boundary。
- `docs/adr/0053-application-control-plane-boundary.md` - control-plane baseline for admission / diagnostics / durable work.
- `.trellis/tasks/06-02-03b-playback-runtime-resource-admission/prd.md` - 资源 admission 前置任务的完成证据。
- `.trellis/tasks/06-02-03b-playback-runtime-resource-admission/resource-audit.md` - 资源 admission 当前行为审计与 follow-up。
- `.trellis/tasks/06-04-playback-hls-admission-policy-seam/prd.md` - 当前已开出的 HLS admission policy seam 任务。
- `crates/nako-playback/src/lib.rs` - playback decision / transcode requirement 产出。
- `crates/nako-playback/src/capability.rs` - client capability 归一化与兼容性评估。
- `crates/nako-playback/src/values.rs` - playback value types 与 subtitle/HLS policy 枚举。
- `crates/nako-transcode/src/pipeline.rs` - transcode pipeline planner。
- `crates/nako-transcode/src/profile.rs` - transcode profile identity / validation。
- `crates/nako-transcode/src/plan.rs` - HLS playback transcode plan validation。
- `crates/nako-transcode/src/hardware.rs` - hardware inventory / diagnostics / capability report。
- `crates/nako-transcode/src/ffmpeg/hls.rs` - HLS FFmpeg command builder。
- `crates/nako-transcode/src/ffmpeg/hls/{filters,encoders,sidecars}.rs` - HLS filter/encoder/sidecar argv seams。
- `crates/nako-server/src/app/playback/{resource,hls,hls_flow,selection,support}.rs` - playback runtime admission 与 HLS orchestration seam。

### Code patterns

- `crates/nako-server/src/app/playback/resource.rs:134-158` - HLS resource demand 把 `cpu_transcode` / `gpu_transcode` 与 `hls_artifact_io` 组合成 typed demand。
- `crates/nako-server/src/app/playback/resource.rs:298-355` - admission 目前是 `try_acquire` / `try_acquire_until` / capacity validation 的组合，没有 waitlist/queue 数据结构。
- `crates/nako-server/src/app/playback/resource.rs:365-510` - runtime pressure 与 admission decision 已 typed 化，适合继续抽 policy，不需要动 HTTP handler。
- `crates/nako-server/src/app/playback/hls_flow.rs:231-318` - HLS source/playlist start 负责 supersede、capacity check、admission、input release、background start。
- `crates/nako-server/src/app/playback/hls.rs:139-220,231-340` - HLS runner / reserve / reuse / supersede 仍持有 orchestration 责任。
- `crates/nako-transcode/src/pipeline.rs:259-311` - pipeline planner 负责 readiness、fallback、subtitle strategy 切换，不应回退到字符串拼接。
- `crates/nako-transcode/src/pipeline.rs:361-475` - HDR / source incompatibility / CPU fallback 已是 typed readiness 逻辑。
- `crates/nako-transcode/src/hardware.rs:10-27,602-645,711-751,1193-1238` - inventory 已覆盖 decoder/encoder/filter/tone-map/smoke probe，且可表达 VAAPI/NVENC/QSV/AMF/VideoToolbox。
- `crates/nako-transcode/src/ffmpeg/hls.rs:300-326` - HLS subtitle strategy 当前明确拒绝 burn-in / preserve-in-container。
- `crates/nako-transcode/src/ffmpeg/hls/sidecars.rs:37-75` - selected subtitle sidecar 已是 typed argv seam。
- `crates/nako-transcode/src/ffmpeg/hls/filters.rs:13-20,37-92` - HLS audio / HDR filter 仍是 typed graph，不是 ad hoc command string。
- `crates/nako-transcode/src/lib.rs:482,993,1043,1100,1364,2137,2187,2722` - 已有 exact-argv / pipeline / hardware diagnostics 测试覆盖。
- `crates/nako-playback/src/lib.rs:270-305,590-705` - client capability 与 transcode requirement 已经能表达 HLS variant/container / subtitle / HDR / audio facts。
- `crates/nako-transcode/src/plan.rs:55-85` 与 `crates/nako-transcode/src/profile.rs:343-365` - 当前 HLS 仍以 h264/aac 为执行上限。

### Candidate queue, sorted by value / risk

1. `06-04-playback-hls-admission-policy-seam`
   - Why: 这是最小、最明确、收益最高的 seam；它把 HLS supersede 的 bounded wait 语义集中到 typed resource-admission policy，直接改善 playback reliability，并且已经在 task 目录中开出。
   - Scope suggestion: `crates/nako-server/src/app/playback/resource.rs`, `crates/nako-server/src/app/playback/hls.rs`, `crates/nako-server/src/app/playback/hls_flow.rs`, 以及最少量 playback tests。
   - Verification: `cargo fmt --all -- --check`; `cargo check -p nako-server --tests`; `cargo nextest run -p nako-server playback_resource_admission --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`.
   - Parallel: 否，和任何 HLS / playback resource admission 任务不应并行；与纯 `nako-transcode` 任务理论上可并行，但需要避免共享 playback HLS 文件。

2. HLS subtitle burn-in planning slice
   - Why: 用户可见收益大，且架构图明确把 ASS/SSA、PGS、burn-in 列为 follow-on；当前 FFmpeg adapter 仍明确拒绝 burn-in/preserve-in-container。
   - Scope suggestion: `crates/nako-playback`, `crates/nako-transcode/src/pipeline.rs`, `crates/nako-transcode/src/ffmpeg/hls.rs`, 可能再加少量 `crates/nako-server/src/app/playback` glue。
   - Verification: `cargo fmt --all -- --check`; `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`; `cargo nextest run -p nako-transcode hls --no-fail-fast`; `cargo nextest run -p nako-server playback --no-fail-fast`.
   - Parallel: 可，与 06-04 admission seam 可以并行，只要不碰相同的 `crates/nako-server/src/app/playback/*` orchestration 文件。

3. HLS seek / restart command identity slice
   - Why: 风险 register 已明确指出 seek 不是字符串替换，适合做 bounded command identity / restart admission slice；对首帧体验和 session reuse 有直接价值。
   - Scope suggestion: `crates/nako-server/src/app/playback/hls*.rs`, `crates/nako-transcode/src/ffmpeg/hls/seek.rs`, 以及少量 request identity / playlist tests。
   - Verification: `cargo fmt --all -- --check`; `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`; `cargo nextest run -p nako-server hls_source --no-fail-fast`; `cargo nextest run -p nako-transcode hls --no-fail-fast`.
   - Parallel: 否，和 HLS orchestration / FFmpeg builder 变化容易撞文件与测试语义。

4. HEVC / AV1 output policy slice
   - Why: 这是明显的 Jellyfin-class follow-on，且 `hardware.rs` 已具备更宽的 inventory 形状；但它会向 planner、profile、validation、server policy 多层扩散，风险高于前 3 项。
   - Scope suggestion: `crates/nako-transcode/src/hardware.rs`, `crates/nako-transcode/src/pipeline.rs`, `crates/nako-transcode/src/profile.rs`, `crates/nako-playback/src/*`，以及可能的 HLS policy tests。
   - Verification: `cargo fmt --all -- --check`; `cargo check -p nako-playback -p nako-transcode --tests`; `cargo nextest run -p nako-transcode pipeline --no-fail-fast`; `cargo nextest run -p nako-transcode hardware --no-fail-fast`.
   - Parallel: 可与纯 server admission work 并行，但不建议与其他需要改 `nako-transcode` pipeline/profile 的任务并行。

5. Hardware smoke / admin diagnostics slice
   - Why: 架构图已把 hardware smoke 单列为 follow-on；它对 release / operator 可见性有价值，且比完整 LL-HLS/CMAF 小得多。
   - Scope suggestion: `crates/nako-transcode/src/hardware.rs`, `crates/nako-server/src/app/playback/support.rs`, 以及可能的 admin DTO / diagnostics tests。
   - Verification: `cargo fmt --all -- --check`; `cargo check -p nako-transcode -p nako-server --tests`; `cargo nextest run -p nako-transcode hardware --no-fail-fast`; `cargo nextest run -p nako-server playback --no-fail-fast`.
   - Parallel: 可，与 HLS admission / seek work 只要不改同一组 diagnostics DTO 即可并行。

### Recommended first actual task

继续 `06-04-playback-hls-admission-policy-seam`。

理由：它已经开成任务、风险最小、收益直接，且与现有代码边界最对齐。`resource.rs` 已经有 typed demand / pressure / permit 结构，`hls_flow.rs` 只是把 supersede wait policy open-coded 在 orchestration 里。把这层 seam 收口后，后续再做 subtitle burn-in、seek/restart、HEVC/AV1，会更容易把 admission、runtime 和 FFmpeg 责任分开。

### Required spec / ADR / architecture updates

- 资源 admission policy seam 如果抽成可复用 convention，建议补 `.trellis/spec/nako-server/backend/quality-guidelines.md` 和/或 `directory-structure.md`，把 typed policy seam 记录成可重用模式。
- HLS subtitle burn-in 或 HEVC/AV1 输出政策若真进入实现，`docs/architecture/PLAYBACK.md` 的对应 follow-on 行需要从“待拆分”更新为新的 shipped / in-progress 状态或新增更细的 follow-on。
- 目前没有看到必须新增 ADR 的证据；这些候选都能在 ADR 0053、ADR 0044、ADR 0045、ADR 0052 的边界内推进。

### Why not choose LL-HLS/CMAF now

`docs/architecture/PLAYBACK.md` 明确把 LL-HLS/CMAF 和 player-facing follow-ons 作为更大的后续项。它会把 transport、playlist semantics、player behavior 和 server runtime 一起拉进来，超出“下一个 bounded implementation task”的最佳粒度。

### Repo-ref guardrail

不复制 `repo-ref/` 里的实现、注释、测试、schema 或资产；只能借鉴行为压力与边界形状，不能做 line-by-line 迁移。

## Caveats / Not Found

- `git status` 显示当前 worktree 已有相关 playback 文件的本地修改，以及本任务目录和 `06-04-playback-hls-admission-policy-seam` 目录为 untracked；本研究未回退、未改动这些文件，只读取了现有内容。
- `.trellis/tasks/06-02-03b-playback-runtime-resource-admission/resource-audit.md` 记录了当前行为和 follow-up，但其 `research/` 子目录没有单独研究文件可读。
- 当前研究没有发现必须立即新增 ADR 的硬证据；若后续实现把 admission policy 扩展成更广泛的 queue/waitlist 语义，再重新评估 ADR / spec 更新。
