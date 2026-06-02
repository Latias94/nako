# Bootstrap Task: Fill Project Development Guidelines

**You (the AI) are running this task. The developer does not read this file.**

The developer just ran `trellis init` on this project for the first time.
`.trellis/` now exists with empty spec scaffolding, and this bootstrap task
exists under `.trellis/tasks/`. When they want to work on it, they should start
this task from a session that provides Trellis session identity.

**Your job**: help them populate `.trellis/spec/` with the team's real
coding conventions. Every future AI session — this project's
`trellis-implement` and `trellis-check` sub-agents — auto-loads spec files
listed in per-task jsonl manifests. Empty spec = sub-agents write generic
code. Real spec = sub-agents match the team's actual patterns.

Don't dump instructions. Open with a short greeting, figure out if the repo
has any existing convention docs (CLAUDE.md, .cursorrules, etc.), and drive
the rest conversationally.

---

## Status (update the checkboxes as you complete each item)

- [x] Fill guidelines for nako
- [x] Fill guidelines for nako-addon-client
- [x] Fill guidelines for nako-addon-protocol
- [x] Fill guidelines for nako-api
- [x] Fill guidelines for nako-automation
- [x] Fill guidelines for nako-catalog
- [x] Fill guidelines for nako-client
- [x] Fill guidelines for nako-client-cli
- [x] Fill guidelines for nako-client-core
- [x] Fill guidelines for nako-client-protocol
- [x] Fill guidelines for nako-client-uniffi
- [x] Fill guidelines for nako-core
- [x] Fill guidelines for nako-db
- [x] Fill guidelines for nako-events
- [x] Fill guidelines for nako-library
- [x] Fill guidelines for nako-media-probe
- [x] Fill guidelines for nako-metadata
- [x] Fill guidelines for nako-naming
- [x] Fill guidelines for nako-nfo
- [x] Fill guidelines for nako-official-addon-catalog
- [x] Fill guidelines for nako-playback
- [x] Fill guidelines for nako-reference-addon
- [x] Fill guidelines for nako-search
- [x] Fill guidelines for nako-server
- [x] Fill guidelines for nako-streaming
- [x] Fill guidelines for nako-transcode
- [x] Fill guidelines for nako-uniffi-bindgen
- [x] Fill guidelines for nako-vfs
- [x] Add code examples

Session progress on 2026-06-02:

- Seeded real, code-referenced specs for `nako-core`, `nako-db`,
  `nako-metadata`, `nako-server`, `nako-api`, `nako-vfs`, `nako-playback`,
  and `nako-transcode`.
- Extended real specs for `nako-library`, `nako-catalog`, `nako-search`,
  `nako-nfo`, `nako-streaming`, `nako-events`, and `nako-automation`.
- Extended real specs for `nako-addon-protocol`, `nako-addon-client`,
  `nako-official-addon-catalog`, and `nako-reference-addon`.
- Extended real specs for `nako-client-protocol`, `nako-client-core`,
  `nako-client`, `nako-client-cli`, and `nako-client-uniffi`.
- Completed the remaining real specs for `nako`, `nako-media-probe`,
  `nako-naming`, and `nako-uniffi-bindgen`, closing the bootstrap checklist.

---

## Spec files to populate

### Package: nako (`spec/nako/`)

- Backend guidelines: `.trellis/spec/nako/backend/`

### Package: nako-addon-client (`spec/nako-addon-client/`)

- Backend guidelines: `.trellis/spec/nako-addon-client/backend/`

### Package: nako-addon-protocol (`spec/nako-addon-protocol/`)

- Backend guidelines: `.trellis/spec/nako-addon-protocol/backend/`

### Package: nako-api (`spec/nako-api/`)

- Backend guidelines: `.trellis/spec/nako-api/backend/`

### Package: nako-automation (`spec/nako-automation/`)

- Backend guidelines: `.trellis/spec/nako-automation/backend/`

### Package: nako-catalog (`spec/nako-catalog/`)

- Backend guidelines: `.trellis/spec/nako-catalog/backend/`

### Package: nako-client (`spec/nako-client/`)

- Backend guidelines: `.trellis/spec/nako-client/backend/`

### Package: nako-client-cli (`spec/nako-client-cli/`)

- Backend guidelines: `.trellis/spec/nako-client-cli/backend/`

### Package: nako-client-core (`spec/nako-client-core/`)

- Backend guidelines: `.trellis/spec/nako-client-core/backend/`

### Package: nako-client-protocol (`spec/nako-client-protocol/`)

- Backend guidelines: `.trellis/spec/nako-client-protocol/backend/`

### Package: nako-client-uniffi (`spec/nako-client-uniffi/`)

- Backend guidelines: `.trellis/spec/nako-client-uniffi/backend/`

### Package: nako-core (`spec/nako-core/`)

- Backend guidelines: `.trellis/spec/nako-core/backend/`

### Package: nako-db (`spec/nako-db/`)

- Backend guidelines: `.trellis/spec/nako-db/backend/`

### Package: nako-events (`spec/nako-events/`)

- Backend guidelines: `.trellis/spec/nako-events/backend/`

### Package: nako-library (`spec/nako-library/`)

- Backend guidelines: `.trellis/spec/nako-library/backend/`

### Package: nako-media-probe (`spec/nako-media-probe/`)

- Backend guidelines: `.trellis/spec/nako-media-probe/backend/`

### Package: nako-metadata (`spec/nako-metadata/`)

- Backend guidelines: `.trellis/spec/nako-metadata/backend/`

### Package: nako-naming (`spec/nako-naming/`)

- Backend guidelines: `.trellis/spec/nako-naming/backend/`

### Package: nako-nfo (`spec/nako-nfo/`)

- Backend guidelines: `.trellis/spec/nako-nfo/backend/`

### Package: nako-official-addon-catalog (`spec/nako-official-addon-catalog/`)

- Backend guidelines: `.trellis/spec/nako-official-addon-catalog/backend/`

### Package: nako-playback (`spec/nako-playback/`)

- Backend guidelines: `.trellis/spec/nako-playback/backend/`

### Package: nako-reference-addon (`spec/nako-reference-addon/`)

- Backend guidelines: `.trellis/spec/nako-reference-addon/backend/`

### Package: nako-search (`spec/nako-search/`)

- Backend guidelines: `.trellis/spec/nako-search/backend/`

### Package: nako-server (`spec/nako-server/`)

- Backend guidelines: `.trellis/spec/nako-server/backend/`

### Package: nako-streaming (`spec/nako-streaming/`)

- Backend guidelines: `.trellis/spec/nako-streaming/backend/`

### Package: nako-transcode (`spec/nako-transcode/`)

- Backend guidelines: `.trellis/spec/nako-transcode/backend/`

### Package: nako-uniffi-bindgen (`spec/nako-uniffi-bindgen/`)

- Backend guidelines: `.trellis/spec/nako-uniffi-bindgen/backend/`

### Package: nako-vfs (`spec/nako-vfs/`)

- Backend guidelines: `.trellis/spec/nako-vfs/backend/`


### Thinking guides (already populated)

`.trellis/spec/guides/` contains general thinking guides pre-filled with
best practices. Customize only if something clearly doesn't fit this project.

---

## How to fill the spec

### Step 1: Import from existing convention files first (preferred)

Search the repo for existing convention docs. If any exist, read them and
extract the relevant rules into the matching `.trellis/spec/` files —
usually much faster than documenting from scratch.

| File / Directory | Tool |
|------|------|
| `CLAUDE.md` / `CLAUDE.local.md` | Claude Code |
| `AGENTS.md` | Codex / Claude Code / agent-compatible tools |
| `.cursorrules` | Cursor |
| `.cursor/rules/*.mdc` | Cursor (rules directory) |
| `.windsurfrules` | Windsurf |
| `.clinerules` | Cline |
| `.roomodes` | Roo Code |
| `.github/copilot-instructions.md` | GitHub Copilot |
| `.vscode/settings.json` → `github.copilot.chat.codeGeneration.instructions` | VS Code Copilot |
| `CONVENTIONS.md` / `.aider.conf.yml` | aider |
| `CONTRIBUTING.md` | General project conventions |
| `.editorconfig` | Editor formatting rules |

### Step 2: Analyze the codebase for anything not covered by existing docs

Scan real code to discover patterns. Before writing each spec file:
- Find 2-3 real examples of each pattern in the codebase.
- Reference real file paths (not hypothetical ones).
- Document anti-patterns the team clearly avoids.

### Step 3: Document reality, not ideals

**Critical**: write what the code *actually does*, not what it should do.
Sub-agents match the spec, so aspirational patterns that don't exist in the
codebase will cause sub-agents to write code that looks out of place.

If the team has known tech debt, document the current state — improvement
is a separate conversation, not a bootstrap concern.

---

## Quick explainer of the runtime (share when they ask "why do we need spec at all")

- Every AI coding task spawns two sub-agents: `trellis-implement` (writes
  code) and `trellis-check` (verifies quality).
- Each task has `implement.jsonl` / `check.jsonl` manifests listing which
  spec files to load.
- The platform hook auto-injects those spec files + the task's `prd.md`
  into every sub-agent prompt, so the sub-agent codes/reviews per team
  conventions without anyone pasting them manually.
- Source of truth: `.trellis/spec/`. That's why filling it well now pays
  off forever.

---

## Completion

When the developer confirms the checklist items above are done with real
examples (not placeholders), guide them to run:

```bash
python ./.trellis/scripts/task.py finish
python ./.trellis/scripts/task.py archive 00-bootstrap-guidelines
```

After archive, every new developer who joins this project will get a
`00-join-<slug>` onboarding task instead of this bootstrap task.

---

## Suggested opening line

"Welcome to Trellis! Your init just set me up to help you fill the project
spec — a one-time setup so every future AI session follows the team's
conventions instead of writing generic code. Before we start, do you have
any existing convention docs (CLAUDE.md, .cursorrules, CONTRIBUTING.md,
etc.) I can pull from, or should I scan the codebase from scratch?"
