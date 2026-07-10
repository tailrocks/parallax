# Plan 080: Fix the first-touch docs — dev setup path, ui/README, PROJECT_STRUCTURE staleness, CLI reference gaps

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- PROJECT_STRUCTURE.md CONTRIBUTING.md ui/README.md docs/guide/cli.md README.md`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live text before proceeding.

## Status

- **Priority**: P3
- **Effort**: S
- **Risk**: LOW (docs only)
- **Depends on**: none
- **Category**: docs / dx
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The repo is public and accepting contributions, but: no document tells a
contributor how to build/test/run the project (CONTRIBUTING.md is
licensing-only; README has no Development section); `ui/README.md` is
unmodified TanStack template boilerplate that instructs `npx shadcn@latest`
— exactly the toolchain path AGENTS.md bans (Bun-only, with a documented
2026-06-18 incident caused by a foreign package manager); the canonical repo
map `PROJECT_STRUCTURE.md` says "there is no release process or CI contract
yet" while the SAME file later describes the CI/release workflows — and it
omits two tracked top-level dirs (`plans/`, `advisor-plans/`), violating its
own update rule; the CLI reference omits two real flags of `run start`.

## Current state

- `CONTRIBUTING.md` (787B) — licensing/provenance only; zero build/dev
  content. Verified: no mention of mise/cargo/bun/test anywhere in it or in
  README.
- `ui/README.md` — starts `# TanStack Start + shadcn/ui` / "This is a
  template for a new TanStack Start project…"; line ~10 instructs
  `npx shadcn@latest add button`. Forbidden: root `AGENTS.md` §JS tooling
  mandates Bun-only (`bunx --bun shadcn@latest add <name>` per
  `ui/AGENTS.md:9`).
- `PROJECT_STRUCTURE.md`:
  - `:11-13`: "V1 implementation is underway under `crates/` (authorized
    2026-06-12); there is no release process or CI contract yet." — false;
    contradicted by its own `.github/workflows/` row (which accurately
    describes CI + preview/release + the Homebrew tap).
  - Directory table: has rows for `bench/`, `poc/`, `crates/`, `ui/`,
    `.github/workflows/`, `mise.toml`, `scripts/`, `prompts/` — NO rows for
    `plans/` (22 tracked files — the operator's feature-plan backlog; see
    `plans/README.md`) or `advisor-plans/` (this directory: improve-skill
    implementation plans; see `advisor-plans/README.md`).
  - Its own "Update Rule" (bottom of file, ~`:100-104`) and `AGENTS.md`
    require same-change updates when top-level dirs appear.
- `docs/guide/cli.md` `run start` rows (`:16-18`) document wrapper and bare
  mode only. Real flags in `crates/parallax-cli/src/main.rs:132-148`:
  `--otlp-forward <TARGET>` ("A URL, `rotel` (the configured hub), or `off`.
  Also settable ambiently via `PARALLAX_OTLP_FORWARD`") and `--print-env`
  ("Print the OTel env that would be injected, then exit (dry-run)").
- Toolchain facts for the Development section (verified): `mise.toml` pins
  actionlint/cosign/cargo-nextest/cargo-zigbuild/bun/shellcheck/syft/zig, and
  `[settings] idiomatic_version_file_enable_tools = ["rust"]` (Rust version
  comes from `rust-toolchain.toml` through mise). Dev proxy: `ui/vite.config.ts`
  proxies `/graphql` → `http://127.0.0.1:4000`, so `parallax serve` (or
  `cargo run -p parallax-cli -- serve`) must run alongside `bun run dev`.
  Verification commands: root `cargo build --workspace`, `cargo nextest run`,
  `cargo clippy --workspace --all-targets` (zero warnings), `cargo fmt --all`;
  ui/ `bun install`, `bun run dev|typecheck|lint|test|build`. Diagnostics:
  `parallax doctor`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Link check (manual) | verify each relative link target exists with `ls` | all exist |
| UI gates unaffected | none required (docs only) | — |

## Scope

**In scope** (the only files you should modify):
- `CONTRIBUTING.md` (add Development section) — or `README.md` if you find an
  existing "Using It" section is the better host; put the full dev loop in
  ONE place and link it from the other.
- `ui/README.md` (rewrite)
- `PROJECT_STRUCTURE.md` (Current Stage + two table rows)
- `docs/guide/cli.md` (two flags)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `AGENTS.md`, `ui/AGENTS.md` — agent rules are operator-owned; link to them,
  don't edit them.
- `docs/research/**` — research record.
- `docs/guide/quickstart.md` — verified accurate at planning (ports, brew
  path); leave unless you spot a factual error, then report it instead.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- One commit, Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `docs: add dev setup path and fix stale onboarding docs`.

## Steps

### Step 1: Development section

In `CONTRIBUTING.md`, append a `## Development` section covering, in order:

1. Prerequisites: `mise install` from the repo root (installs Bun + tools;
   Rust toolchain resolves via `rust-toolchain.toml`).
2. Backend: `cargo build --workspace`; test with `cargo nextest run`;
   lint gates `cargo clippy --workspace --all-targets` (zero warnings) and
   `cargo fmt --all -- --check`.
3. UI: `cd ui && bun install && bun run dev` (port 3000), noting the
   `/graphql` proxy expects `parallax serve` on `:4000`
   (`cargo run -p parallax-cli -- serve`). Bun-only warning with a link to
   the root `AGENTS.md` JS-tooling rule.
4. Full local gate list mirroring CI (fmt, clippy, nextest, ui
   typecheck/lint/test/build).
5. `parallax doctor` for install diagnostics.

Add one line under README's "Using It" linking to
`CONTRIBUTING.md#development` ("Developing Parallax itself: see …").

**Verify**: every command named exists in the repo's manifests
(`grep -n "typecheck\|lint\|test\|build" ui/package.json` shows the scripts;
`ls mise.toml rust-toolchain.toml` exist). Links resolve (`ls` each target).

### Step 2: Rewrite ui/README.md

Replace the template content with Parallax-specific notes (~20 lines):
what the app is (TanStack Start SPA over the GraphQL API), `bun install` /
`bun run dev` + the `:4000` proxy note, the Bun-only rule (never
npm/pnpm/yarn/npx — link `../AGENTS.md`), shadcn additions via
`bunx --bun shadcn@latest add <component>` (link `ui/AGENTS.md` for the full
rules), and the gate commands.

**Verify**: `grep -n "npx" ui/README.md` → 0 matches;
`grep -n "template" ui/README.md` → 0 matches.

### Step 3: PROJECT_STRUCTURE.md truth

1. Rewrite the "Current Stage" paragraph: V1 implemented under
   `crates/` + `ui/`; CI, stable-release, and preview-Homebrew automation
   exist under `.github/workflows/` (keep it to 3-4 sentences; the existing
   workflows row already carries the detail).
2. Add two table rows:
   - `plans/` — operator feature-plan audit backlog; active items + retire
     rules in `plans/README.md`.
   - `advisor-plans/` — implementation plans generated by codebase audits
     (improve skill); execution order + status in `advisor-plans/README.md`.

**Verify**: `grep -n "no release process" PROJECT_STRUCTURE.md` → 0;
`grep -n "advisor-plans" PROJECT_STRUCTURE.md` → ≥1;
`grep -n "| \`plans/\`" PROJECT_STRUCTURE.md` → 1.

### Step 4: cli.md flags

In the `run start` area of `docs/guide/cli.md`, add rows/notes for
`--print-env` (dry-run: print the injected OTel env and exit) and
`--otlp-forward <url|rotel|off>` (compare mode: forward child telemetry to a
collector instead of Parallax; ambient `PARALLAX_OTLP_FORWARD`). Copy the
semantics from `main.rs:132-148` — do not invent behavior.

**Verify**: `grep -n "print-env" docs/guide/cli.md` → ≥1;
`grep -n "otlp-forward" docs/guide/cli.md` → ≥1.

## Test plan

Docs-only: verification is the grep gates above plus manually resolving each
added relative link (`ls <target>`).

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "## Development" CONTRIBUTING.md` → 1
- [ ] `grep -n "mise install" CONTRIBUTING.md` → ≥1
- [ ] `grep -n "npx" ui/README.md` → 0
- [ ] `grep -n "no release process" PROJECT_STRUCTURE.md` → 0
- [ ] `grep -cn "plans/" PROJECT_STRUCTURE.md` → ≥2 (both dirs)
- [ ] `grep -n "print-env\|otlp-forward" docs/guide/cli.md` → ≥2
- [ ] All added relative links resolve to existing files
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Any command you're about to document doesn't work as described when
  spot-checked (e.g. `mise install` errors on this machine) — document only
  verified reality.
- `docs/guide/cli.md` has been restructured so the `run start` rows moved.
- You find quickstart.md factually wrong while cross-checking — report it,
  don't expand scope.

## Maintenance notes

- AGENTS.md's rule stands: `PROJECT_STRUCTURE.md` updates in the same change
  as structure changes — reviewers should hold the line now that it's
  accurate again.
- When V2/server profiles land, the Development section gains a config
  subsection; keep the single-source-of-truth discipline (dev loop lives in
  CONTRIBUTING.md, everything else links).
